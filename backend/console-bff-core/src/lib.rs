//! Shared core for the Araf console BFF services.
//!
//! Each console surface (Tenant, Operator) runs its own BFF binary with its
//! own deployment, session and trust boundary (ADR 0001). This crate holds
//! behavior that is identical for both surfaces plus the upstream adapter
//! abstraction and the deterministic fixture adapter used by the prototype.
//!
//! The BFF must never become a generic upstream proxy (ADR 0002).

pub mod error;
pub mod fixture;
pub mod handlers;
pub mod middleware;
pub mod model;
pub mod o3k_adapter;
pub mod o3k_client;
pub mod request;
pub mod upstream;

use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};
pub use error::{ApiError, BffError, ProblemDetails, UpstreamError};
pub use fixture::{FixtureAdapter, FIXTURE_RESOURCE_TOTAL};
pub use handlers::AppState;
pub use o3k_adapter::O3kAdapter;
pub use o3k_client::{O3kClient, O3kClientConfig};
pub use request::{RequestContext, SessionState};
pub use upstream::Upstream;

/// Identity of the console surface a BFF instance serves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BffSurface {
    /// Stable machine name of the surface, e.g. `tenant-bff`.
    pub service: &'static str,
}

/// Build the shared API router for the given upstream adapter.
///
/// Surface-specific binaries add or omit routes by composing this router with
/// additional surface-only routes.
pub fn api_router(upstream: Arc<dyn Upstream>) -> Router {
    let state = AppState { upstream };

    Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/api/v1/context", get(handlers::get_context))
        .route("/api/v1/services", get(handlers::list_services))
        .route(
            "/api/v1/resources/{resource_type}",
            get(handlers::list_resources).post(handlers::create_resource),
        )
        .route(
            "/api/v1/resources/{resource_type}/{id}",
            get(handlers::get_resource),
        )
        .route(
            "/api/v1/resources/{resource_type}/{id}/actions",
            post(handlers::submit_action),
        )
        .route("/api/v1/operations", get(handlers::list_operations))
        .route("/api/v1/operations/{id}", get(handlers::get_operation))
        .route("/api/v1/governance/projects", get(handlers::list_projects))
        .route(
            "/api/v1/governance/projects/{id}",
            get(handlers::get_project),
        )
        .route(
            "/api/v1/governance/projects/{id}/members",
            get(handlers::list_project_members),
        )
        .route("/api/v1/governance/users", get(handlers::list_users))
        .route("/api/v1/governance/users/{id}", get(handlers::get_user))
        .route("/api/v1/governance/roles", get(handlers::list_roles))
        .route("/api/v1/governance/quotas", get(handlers::list_quotas))
        .route("/api/v1/governance/audit", get(handlers::list_audit_events))
        .route(
            "/api/v1/governance/api-credentials",
            get(handlers::list_api_credentials).post(handlers::create_api_credential),
        )
        .route(
            "/api/v1/governance/api-credentials/{id}",
            delete(handlers::delete_api_credential),
        )
        .with_state(state)
}

/// Which upstream adapter the BFF should use.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UpstreamAdapter {
    /// Deterministic fixture adapter used for prototype development.
    #[default]
    Fixture,
    /// Real O3K native API adapter.
    O3k,
}

/// Configuration for building a BFF router.
#[derive(Clone, Debug)]
pub struct BffConfig {
    pub surface: &'static str,
    pub adapter: UpstreamAdapter,
}

impl BffConfig {
    /// Read configuration from environment variables.
    ///
    /// - `ARAF_UPSTREAM_ADAPTER`: `fixture` (default) or `o3k`.
    pub fn from_env(surface: &'static str) -> Self {
        let adapter = match std::env::var("ARAF_UPSTREAM_ADAPTER").as_deref() {
            Ok("o3k") => UpstreamAdapter::O3k,
            _ => UpstreamAdapter::Fixture,
        };
        Self { surface, adapter }
    }
}

/// Build the API router for the given configuration.
pub fn api_router_for_config(config: BffConfig) -> Result<Router, ApiError> {
    let upstream: Arc<dyn Upstream> = match config.adapter {
        UpstreamAdapter::Fixture => Arc::new(FixtureAdapter::new(config.surface)),
        UpstreamAdapter::O3k => Arc::new(O3kAdapter::from_env(config.surface)?),
    };
    Ok(middleware::apply_default_layers(
        api_router(upstream),
        config.surface,
    ))
}

/// Build a complete BFF router for the given surface using the fixture adapter.
pub fn fixture_router(surface: &'static str) -> Router {
    let upstream: Arc<dyn Upstream> = Arc::new(FixtureAdapter::new(surface));
    middleware::apply_default_layers(api_router(upstream), surface)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn healthz_reports_ok_and_service_name() {
        let app = fixture_router("tenant-bff");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024).await.expect("body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "tenant-bff");
    }

    #[tokio::test]
    async fn unknown_routes_are_not_served() {
        let app = fixture_router("operator-bff");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/operator/anything")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        // The BFF is not a generic upstream proxy: unregistered paths 404.
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }
}
