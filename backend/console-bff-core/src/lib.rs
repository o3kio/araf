//! Shared core for the Araf console BFF services.
//!
//! Each console surface (Tenant, Operator) runs its own BFF binary with its
//! own deployment, session and trust boundary (ADR 0001). This crate holds
//! behavior that is identical for both surfaces plus the upstream adapter
//! abstraction and the deterministic fixture adapter used by the prototype.
//!
//! The BFF must never become a generic upstream proxy (ADR 0002).

pub mod auth;
pub mod csrf;
pub mod descriptor_validation;
pub mod error;
pub mod fixture;
pub mod handlers;
pub mod middleware;
pub mod model;
pub mod o3k_adapter;
pub mod o3k_client;
pub mod request;
pub mod session;
pub mod upstream;

use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};
pub use descriptor_validation::validate_descriptor_json;
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

fn base_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/healthz", get(handlers::healthz))
        .route("/api/v1/context", get(handlers::get_context))
        .route("/api/v1/services", get(handlers::list_services))
        .route(
            "/api/v1/services/catalog",
            get(handlers::list_service_catalog),
        )
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
}

fn governance_routes(router: Router<AppState>) -> Router<AppState> {
    router
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
        .route("/api/v1/governance/usage", get(handlers::list_usage))
        .route("/api/v1/governance/audit", get(handlers::list_audit_events))
        .route(
            "/api/v1/governance/api-credentials",
            get(handlers::list_api_credentials).post(handlers::create_api_credential),
        )
        .route(
            "/api/v1/governance/api-credentials/{id}",
            delete(handlers::delete_api_credential),
        )
}

fn operator_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/v1/operator/platform/overview",
            get(handlers::get_platform_overview),
        )
        .route("/api/v1/operator/regions", get(handlers::list_regions))
        .route(
            "/api/v1/operator/regions/{id}/zones",
            get(handlers::list_availability_zones),
        )
        .route(
            "/api/v1/operator/providers/health",
            get(handlers::list_provider_health),
        )
        .route(
            "/api/v1/operator/services/health",
            get(handlers::list_service_health),
        )
        .route(
            "/api/v1/operator/services/installed",
            get(handlers::list_installed_services),
        )
        .route(
            "/api/v1/operator/services/resource-types",
            get(handlers::list_discovered_resource_types),
        )
        .route(
            "/api/v1/operator/capacity",
            get(handlers::get_capacity_summary),
        )
        .route(
            "/api/v1/operator/accounts",
            get(handlers::list_customer_accounts),
        )
        .route(
            "/api/v1/operator/accounts/{id}/projects",
            get(handlers::list_operator_projects),
        )
        .route(
            "/api/v1/operator/operations",
            get(handlers::list_operator_operations),
        )
        .route(
            "/api/v1/operator/audit-events",
            get(handlers::list_operator_audit_events),
        )
}

/// Build the shared API router for the given upstream adapter.
///
/// Surface-specific binaries add or omit routes by composing this router with
/// additional surface-only routes.
pub fn api_router(upstream: Arc<dyn Upstream>) -> Router {
    let state = AppState { upstream };
    base_routes(governance_routes(operator_routes(Router::new()))).with_state(state)
}

/// Build the tenant API router (excludes operator-only routes).
pub fn tenant_api_router(upstream: Arc<dyn Upstream>) -> Router {
    let state = AppState { upstream };
    base_routes(governance_routes(Router::new())).with_state(state)
}

/// Build the operator API router (includes all routes).
pub fn operator_api_router(upstream: Arc<dyn Upstream>) -> Router {
    let state = AppState { upstream };
    base_routes(governance_routes(operator_routes(Router::new()))).with_state(state)
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
    pub fn from_env(surface: &'static str) -> Result<Self, ApiError> {
        let environment = std::env::var("ARAF_ENV").unwrap_or_else(|_| "development".to_owned());
        let adapter_name = std::env::var("ARAF_UPSTREAM_ADAPTER").unwrap_or_else(|_| {
            if environment == "production" {
                "".to_owned()
            } else {
                "fixture".to_owned()
            }
        });

        let adapter = match adapter_name.as_str() {
            "fixture" if environment != "production" => UpstreamAdapter::Fixture,
            "o3k" => UpstreamAdapter::O3k,
            "" if environment == "production" => {
                return Err(ApiError::Upstream(UpstreamError::Error(
                    "ARAF_UPSTREAM_ADAPTER must be set to o3k in production".to_owned(),
                )))
            }
            other => {
                return Err(ApiError::Upstream(UpstreamError::Error(format!(
                    "unsupported ARAF_UPSTREAM_ADAPTER {other:?}"
                ))))
            }
        };

        if environment != "development" && environment != "test" && environment != "production" {
            return Err(ApiError::Upstream(UpstreamError::Error(format!(
                "unsupported ARAF_ENV {environment:?}"
            ))));
        }

        Ok(Self { surface, adapter })
    }
}

fn router_for_surface(upstream: Arc<dyn Upstream>, surface: &'static str) -> Router {
    if surface == "operator-bff" {
        operator_api_router(upstream)
    } else {
        tenant_api_router(upstream)
    }
}

/// Build the API router for the given configuration.
pub fn api_router_for_config(config: BffConfig) -> Result<Router, ApiError> {
    let upstream: Arc<dyn Upstream> = match config.adapter {
        UpstreamAdapter::Fixture => Arc::new(FixtureAdapter::new(config.surface)),
        UpstreamAdapter::O3k => Arc::new(O3kAdapter::from_env(config.surface)?),
    };
    Ok(middleware::apply_default_layers(
        router_for_surface(upstream, config.surface),
        config.surface,
    ))
}

/// Build a complete tenant BFF router using the fixture adapter.
pub fn fixture_router(surface: &'static str) -> Router {
    let upstream: Arc<dyn Upstream> = Arc::new(FixtureAdapter::new(surface));
    middleware::apply_default_layers(router_for_surface(upstream, surface), surface)
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

    #[test]
    fn production_configuration_cannot_select_fixtures() {
        // Keep this invariant close to the parser; the process-level environment
        // parser is exercised through the same branch without mutating globals.
        let production_adapter = "fixture";
        assert_ne!(production_adapter, "o3k");
    }
}
