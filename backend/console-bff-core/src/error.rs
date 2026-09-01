//! BFF error handling and Problem Details passthrough.
//!
//! All BFF errors are converted into a JSON Problem Details response that
//! preserves a safe, structured shape for the frontend. Internal details are
//! logged server-side and never leaked to the client.

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

/// Public Problem Details shape returned to browsers.
///
/// Follows RFC 7807 plus Araf-specific extensions for correlation and
/// operation linkage. Values that may be internal are deliberately omitted.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub correlation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
}

impl ProblemDetails {
    pub fn new(
        correlation_id: impl Into<String>,
        status: StatusCode,
        title: impl Into<String>,
    ) -> Self {
        Self {
            problem_type: format!("https://araf.o3k.io/errors/{}", status.as_u16()),
            title: title.into(),
            status: status.as_u16(),
            detail: status.canonical_reason().unwrap_or("Error").to_string(),
            correlation_id: correlation_id.into(),
            operation_id: None,
            resource_id: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    pub fn with_operation_id(mut self, operation_id: impl Into<String>) -> Self {
        self.operation_id = Some(operation_id.into());
        self
    }

    pub fn with_resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }
}

/// Internal BFF error taxonomy.
#[derive(Clone, Debug, thiserror::Error)]
pub enum ApiError {
    #[error("upstream request failed: {0}")]
    Upstream(#[source] UpstreamError),
    #[error("request body too large")]
    BodyTooLarge,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found")]
    NotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("unauthorized")]
    Unauthorized,
    #[error("internal error")]
    Internal,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum UpstreamError {
    #[error("upstream unavailable")]
    Unavailable,
    #[error("upstream returned error: {0}")]
    Error(String),
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonSyntaxError(_) | JsonRejection::JsonDataError(_) => {
                ApiError::BadRequest("invalid json".to_string())
            }
            JsonRejection::BytesRejection(_) => ApiError::BodyTooLarge,
            _ => ApiError::BadRequest("invalid request body".to_string()),
        }
    }
}

impl ApiError {
    pub fn status(&self) -> StatusCode {
        match self {
            ApiError::Upstream(UpstreamError::Unavailable) => StatusCode::BAD_GATEWAY,
            ApiError::Upstream(UpstreamError::Error(_)) => StatusCode::BAD_GATEWAY,
            ApiError::BodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn into_problem(self, correlation_id: &str) -> ProblemDetails {
        let status = self.status();
        let title = match &self {
            ApiError::Upstream(UpstreamError::Unavailable) => "Upstream unavailable",
            ApiError::Upstream(UpstreamError::Error(_)) => "Upstream error",
            ApiError::BodyTooLarge => "Request body too large",
            ApiError::BadRequest(_) => "Bad request",
            ApiError::NotFound => "Not found",
            ApiError::Forbidden => "Forbidden",
            ApiError::Unauthorized => "Unauthorized",
            ApiError::Internal => "Internal server error",
        };

        let detail = self.to_string();
        ProblemDetails::new(correlation_id, status, title).with_detail(detail)
    }
}

/// Request-scoped error that carries the correlation id needed for the public
/// Problem Details response.
#[derive(Debug)]
pub struct BffError {
    error: ApiError,
    correlation_id: String,
}

impl BffError {
    pub fn new(error: ApiError, correlation_id: impl Into<String>) -> Self {
        Self {
            error,
            correlation_id: correlation_id.into(),
        }
    }
}

impl IntoResponse for BffError {
    fn into_response(self) -> Response {
        let problem = self.error.into_problem(&self.correlation_id);

        if problem.status >= 500 {
            error!(correlation_id = %problem.correlation_id, status = problem.status, "server error response");
        } else {
            debug!(correlation_id = %problem.correlation_id, status = problem.status, "client error response");
        }

        let status =
            StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(problem)).into_response()
    }
}
