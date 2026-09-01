//! CSRF protection for cookie-authenticated state-changing routes.
//!
//! Uses a double-submit cookie pattern: the server generates a CSRF token stored
//! in the session, and the client must send it back via the `X-CSRF-Token` header
//! on state-changing requests (POST, PUT, DELETE, PATCH).
//!
//! SameSite=Lax cookies provide defense-in-depth, not sole protection.

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::session::SessionStore;

/// Header name for CSRF tokens.
pub const CSRF_HEADER: &str = "x-csrf-token";

pub fn csrf_header_name() -> HeaderName {
    HeaderName::from_static(CSRF_HEADER)
}

/// List of state-changing HTTP methods that require CSRF validation.
const STATE_CHANGING_METHODS: &[Method] =
    &[Method::POST, Method::PUT, Method::DELETE, Method::PATCH];

fn is_state_changing(method: &Method) -> bool {
    STATE_CHANGING_METHODS.contains(method)
}

/// CSRF middleware.
///
/// Requires a `SessionStore` in the router state and a valid session token cookie
/// in the request. For state-changing methods, validates the `X-CSRF-Token` header
/// against the stored session CSRF token.
pub async fn csrf_middleware(
    store: axum::extract::State<Arc<SessionStore>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !is_state_changing(request.method()) {
        return Ok(next.run(request).await);
    }

    // Extract session token from cookie.
    let session_token = request
        .headers()
        .get("cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|c| {
            c.split(';')
                .map(|s| s.trim())
                .find_map(|s| s.strip_prefix("araf_"))
                .and_then(|s| s.split('=').nth(1))
                .map(|s| s.to_owned())
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let session = store
        .validate(&session_token)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let csrf_token = request
        .headers()
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::FORBIDDEN)?;

    if csrf_token != session.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// Set the CSRF token cookie so the frontend can read it and include it in
/// state-changing requests. Called after authentication succeeds.
pub fn set_csrf_cookie(
    response: &mut Response,
    csrf_token: &str,
) -> Result<(), axum::http::header::InvalidHeaderValue> {
    let cookie = format!(
        "araf_csrf={}; Path=/; SameSite=Lax; HttpOnly; Max-Age=86400",
        csrf_token
    );
    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str(&cookie)?,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SessionStore;
    use axum::{body::Body, http::Request, middleware, routing::post, Router};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn csrf_valid_token_passes() {
        let store = Arc::new(SessionStore::default());
        let token = store
            .create(
                "user-1".into(),
                "Test".into(),
                "tenant-bff",
                None,
                None,
                None,
            )
            .await;

        // Retrieve the CSRF token from the session.
        let session = store.get(&token).await.expect("session");
        let csrf = session.csrf_token.clone();

        let app = Router::new().route("/test", post(|| async { "ok" })).layer(
            middleware::from_fn_with_state(store.clone(), csrf_middleware),
        );

        let response = app
            .oneshot(
                Request::post("/test")
                    .header("cookie", format!("araf_session={}", token))
                    .header(CSRF_HEADER, csrf)
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn csrf_missing_token_is_forbidden() {
        let store = Arc::new(SessionStore::default());
        let token = store
            .create(
                "user-1".into(),
                "Test".into(),
                "tenant-bff",
                None,
                None,
                None,
            )
            .await;

        let app = Router::new().route("/test", post(|| async { "ok" })).layer(
            middleware::from_fn_with_state(store.clone(), csrf_middleware),
        );

        let response = app
            .oneshot(
                Request::post("/test")
                    .header("cookie", format!("araf_session={}", token))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn csrf_wrong_token_is_forbidden() {
        let store = Arc::new(SessionStore::default());
        let token = store
            .create(
                "user-1".into(),
                "Test".into(),
                "tenant-bff",
                None,
                None,
                None,
            )
            .await;

        let app = Router::new().route("/test", post(|| async { "ok" })).layer(
            middleware::from_fn_with_state(store.clone(), csrf_middleware),
        );

        let response = app
            .oneshot(
                Request::post("/test")
                    .header("cookie", format!("araf_session={}", token))
                    .header(CSRF_HEADER, "wrong-token")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
