//! Request context: correlation/request IDs and session-scoped metadata.

use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, HeaderName},
};
use std::sync::Arc;
use uuid::Uuid;

const CORRELATION_ID_HEADER: &str = "x-correlation-id";
const REQUEST_ID_HEADER: &str = "x-request-id";

/// Per-request context shared across handlers and middleware.
#[derive(Clone, Debug)]
pub struct RequestContext {
    pub request_id: String,
    pub correlation_id: String,
    pub session: Arc<SessionState>,
    /// Client-supplied idempotency identity for one logical mutation.
    pub idempotency_key: Option<String>,
    /// O3K generation precondition for update/delete mutations.
    pub if_match: Option<String>,
}

impl RequestContext {
    pub fn new(request_id: String, correlation_id: String, session: Arc<SessionState>) -> Self {
        Self {
            request_id,
            correlation_id,
            session,
            idempotency_key: None,
            if_match: None,
        }
    }

    pub fn correlation_id(&self) -> &str {
        &self.correlation_id
    }
}

/// Lightweight session state placeholder.
///
/// M3 uses a fixture-aware session. M12 replaces this with real OIDC/session
/// validation while keeping the same handler shape.
#[derive(Clone, Debug, Default)]
pub struct SessionState {
    pub surface: &'static str,
    pub authenticated: bool,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub o3k_token: Option<String>,
    pub oidc_access_token: Option<String>,
    pub session_token: Option<String>,
}

impl SessionState {
    pub fn fixture(surface: &'static str) -> Self {
        Self {
            surface,
            authenticated: true,
            user_id: Some("fixture-user".to_string()),
            user_name: Some("Fixture User".to_owned()),
            o3k_token: None,
            oidc_access_token: None,
            session_token: None,
        }
    }
}

impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let request_id = parts
            .headers
            .get(REQUEST_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map_or_else(|| Uuid::new_v4().to_string(), ToString::to_string);

        let correlation_id = parts
            .headers
            .get(CORRELATION_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map_or_else(|| request_id.clone(), ToString::to_string);

        let idempotency_key = parts
            .headers
            .get("idempotency-key")
            .and_then(|h| h.to_str().ok())
            .map(str::to_owned);
        let if_match = parts
            .headers
            .get("if-match")
            .and_then(|h| h.to_str().ok())
            .map(str::to_owned);

        let session = parts
            .extensions
            .get::<Arc<SessionState>>()
            .cloned()
            .unwrap_or_default();

        let mut context = Self::new(request_id, correlation_id, session);
        context.idempotency_key = idempotency_key;
        context.if_match = if_match;
        Ok(context)
    }
}

pub fn correlation_id_header() -> HeaderName {
    HeaderName::from_static(CORRELATION_ID_HEADER)
}

pub fn request_id_header() -> HeaderName {
    HeaderName::from_static(REQUEST_ID_HEADER)
}

/// Extract the request id from a request for middleware use.
pub fn request_id_from_request(req: &Request) -> String {
    req.headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|h| h.to_str().ok())
        .map_or_else(|| Uuid::new_v4().to_string(), ToString::to_string)
}

/// Extract the correlation id from a request for middleware use.
pub fn correlation_id_from_request(req: &Request) -> String {
    req.headers()
        .get(CORRELATION_ID_HEADER)
        .and_then(|h| h.to_str().ok())
        .map_or_else(|| request_id_from_request(req), ToString::to_string)
}
