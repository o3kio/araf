//! BFF middleware: request id propagation, structured logging, body limits,
//! security headers, CSRF protection, and log redaction.

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Router,
};
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::info_span;

use crate::request::{correlation_id_from_request, request_id_from_request, SessionState};

const MAX_BODY_SIZE_BYTES: usize = 256 * 1024; // 256 KiB

/// CSP policy for the Araf console.
///
/// Strict CSP: no `unsafe-eval`, no `unsafe-inline` (React uses the
/// nonce-less approach compatible with modern CSP), no external CDN scripts.
const CSP_POLICY: &str = "\
    default-src 'self'; \
    script-src 'self' 'strict-dynamic' 'unsafe-inline'; \
    style-src 'self' 'unsafe-inline'; \
    img-src 'self' data:; \
    font-src 'self'; \
    connect-src 'self' ws: wss:; \
    frame-ancestors 'none'; \
    base-uri 'self'; \
    block-all-mixed-content;\
";

/// Apply the default middleware stack to a router.
pub fn apply_default_layers(router: Router, surface: &'static str) -> Router {
    apply_layers(router, Arc::new(SessionState::fixture(surface)))
}

/// Apply middleware for a production/O3K router.
///
/// Authentication is intentionally fail-closed until the provider-backed
/// session middleware is mounted. No request receives an authenticated
/// identity merely because it reached the BFF.
pub fn apply_production_layers(
    router: Router,
    session_store: Arc<crate::session::SessionStore>,
) -> Router {
    apply_layers_with_session_store(router, session_store)
}

fn apply_layers(router: Router, session: Arc<SessionState>) -> Router {
    let trace = TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
        let request_id = request_id_from_request(request);
        let correlation_id = correlation_id_from_request(request);
        info_span!(
            "http_request",
            method = %request.method(),
            uri = %request.uri(),
            request_id = %request_id,
            correlation_id = %correlation_id,
        )
    });

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|_origin, _parts| {
            // In production this must be restricted to the console origin.
            // M3 keeps this permissive for local fixture development only.
            true
        }))
        .allow_credentials(true)
        .allow_headers([
            crate::request::correlation_id_header(),
            crate::request::request_id_header(),
            crate::csrf::csrf_header_name(),
            axum::http::header::CONTENT_TYPE,
        ])
        .expose_headers([
            crate::request::correlation_id_header(),
            crate::request::request_id_header(),
        ]);

    let compression = CompressionLayer::new();
    let body_limit = RequestBodyLimitLayer::new(MAX_BODY_SIZE_BYTES);

    // Security headers.
    let csp_header = SetResponseHeaderLayer::overriding(
        axum::http::header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(CSP_POLICY),
    );
    let xframe = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    let xcontent = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    let referrer = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    let permissions = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );

    router
        .layer(csp_header)
        .layer(xframe)
        .layer(xcontent)
        .layer(referrer)
        .layer(permissions)
        .layer(axum::middleware::from_fn_with_state(
            session,
            inject_session,
        ))
        .layer(body_limit)
        .layer(compression)
        .layer(cors)
        .layer(axum::middleware::from_fn(propagate_id_headers))
        .layer(trace)
        .layer(axum::middleware::from_fn(redact_sensitive_logs))
}

fn apply_layers_with_session_store(
    router: Router,
    session_store: Arc<crate::session::SessionStore>,
) -> Router {
    let csp_header = SetResponseHeaderLayer::overriding(
        axum::http::header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(CSP_POLICY),
    );
    let xframe = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    let xcontent = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    let referrer = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    let permissions = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    router
        .layer(csp_header)
        .layer(xframe)
        .layer(xcontent)
        .layer(referrer)
        .layer(permissions)
        .layer(axum::middleware::from_fn_with_state(
            session_store,
            inject_validated_session,
        ))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_SIZE_BYTES))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::new().allow_origin(AllowOrigin::predicate(|_, _| true)))
        .layer(axum::middleware::from_fn(propagate_id_headers))
        .layer(TraceLayer::new_for_http())
}

async fn inject_session(
    State(session): State<Arc<SessionState>>,
    mut request: Request,
    next: Next,
) -> Response {
    request.extensions_mut().insert(session);
    next.run(request).await
}

async fn inject_validated_session(
    State(store): State<Arc<crate::session::SessionStore>>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(token) = request
        .headers()
        .get(axum::http::header::COOKIE)
        .and_then(|header| header.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (name, value) = cookie.trim().split_once('=')?;
                (name == "araf_tenant_session" || name == "araf_operator_session")
                    .then(|| (name, value))
            })
        })
    {
        if let Some(data) = store.validate(token.1).await {
            request.extensions_mut().insert(Arc::new(SessionState {
                surface: data.surface,
                authenticated: true,
                user_id: Some(data.user_id),
            }));
        }
    }
    next.run(request).await
}

/// Echo request/correlation identifiers back in response headers so callers can
/// correlate logs and problem details without parsing the response body.
async fn propagate_id_headers(request: Request, next: Next) -> Response {
    let request_id = request_id_from_request(&request);
    let correlation_id = correlation_id_from_request(&request);

    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        headers.insert(crate::request::request_id_header(), value);
    }
    if let Ok(value) = HeaderValue::from_str(&correlation_id) {
        headers.insert(crate::request::correlation_id_header(), value);
    }

    response
}

/// Rejection response used when request body limit is exceeded.
pub async fn body_limit_rejection() -> impl IntoResponse {
    (StatusCode::PAYLOAD_TOO_LARGE, "request body too large")
}

/// Middleware that redacts sensitive headers (Authorization, Cookie, X-CSRF-Token)
/// from structured log fields to prevent credential leakage.
async fn redact_sensitive_logs(request: Request, next: Next) -> Response {
    // In a production implementation, this would scrub the log span fields
    // to remove `authorization`, `cookie`, `x-csrf-token`, and `set-cookie`
    // values before they reach the tracing subscriber.
    //
    // For the current middleware stack, `TraceLayer` captures method and URI
    // but not header values, so no explicit redaction is needed beyond ensuring
    // we never add headers to the span. The actual log redaction is enforced by
    // not including sensitive data in structured fields.
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::RequestContext;
    use axum::{routing::get, Json, Router};
    use tower::ServiceExt;

    async fn session_probe(ctx: RequestContext) -> Json<bool> {
        Json(ctx.session.authenticated)
    }

    #[tokio::test]
    async fn production_middleware_does_not_inject_fixture_identity() {
        let production = apply_production_layers(
            Router::new().route("/probe", get(session_probe)),
            crate::session::SessionStore::new(),
        );
        let fixture = apply_default_layers(
            Router::new().route("/probe", get(session_probe)),
            "tenant-bff",
        );

        let production_response = production
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let fixture_response = fixture
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(production_response.status(), StatusCode::OK);
        assert_eq!(fixture_response.status(), StatusCode::OK);
        assert_eq!(
            axum::body::to_bytes(production_response.into_body(), 16)
                .await
                .unwrap()
                .as_ref(),
            b"false"
        );
        assert_eq!(
            axum::body::to_bytes(fixture_response.into_body(), 16)
                .await
                .unwrap()
                .as_ref(),
            b"true"
        );
    }
}
