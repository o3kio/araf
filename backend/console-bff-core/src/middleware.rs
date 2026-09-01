//! BFF middleware: request id propagation, structured logging, body limits.

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
    trace::TraceLayer,
};
use tracing::info_span;

use crate::request::{correlation_id_from_request, request_id_from_request, SessionState};

const MAX_BODY_SIZE_BYTES: usize = 256 * 1024; // 256 KiB

/// Apply the default middleware stack to a router.
pub fn apply_default_layers(router: Router, surface: &'static str) -> Router {
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
        .allow_headers([
            crate::request::correlation_id_header(),
            crate::request::request_id_header(),
        ])
        .expose_headers([
            crate::request::correlation_id_header(),
            crate::request::request_id_header(),
        ]);

    let compression = CompressionLayer::new();
    let body_limit = RequestBodyLimitLayer::new(MAX_BODY_SIZE_BYTES);

    router
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(SessionState::fixture(surface)),
            inject_session,
        ))
        .layer(body_limit)
        .layer(compression)
        .layer(cors)
        .layer(axum::middleware::from_fn(propagate_id_headers))
        .layer(trace)
}

async fn inject_session(
    State(session): State<Arc<SessionState>>,
    mut request: Request,
    next: Next,
) -> Response {
    request.extensions_mut().insert(session);
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
