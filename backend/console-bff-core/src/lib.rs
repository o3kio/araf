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
        .route("/api/v1/auth/login", get(auth::login))
        .route("/api/v1/auth/callback", get(auth::auth_callback))
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/session", get(auth::session_status))
        .route("/api/v1/auth/scopes", get(auth::discover_scopes))
        .route("/api/v1/auth/scope", post(auth::select_scope))
        .route("/api/v1/context", get(handlers::get_context))
        .route("/api/v1/services", get(handlers::list_services))
        .route(
            "/api/v1/services/catalog",
            get(handlers::list_service_catalog),
        )
        // Region/AZ identity is discovered through the same authenticated
        // upstream boundary for both consoles. Operator-prefixed aliases are
        // retained for the operator platform surface below.
        .route("/api/v1/regions", get(handlers::list_regions))
        .route(
            "/api/v1/regions/{region_id}/zones",
            get(handlers::list_availability_zones),
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
        .route(
            "/api/v1/operator/profile",
            get(handlers::get_operator_profile),
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
    let state = AppState {
        upstream,
        oidc: auth::OidcConfig::fixture("tenant-bff"),
        sessions: session::SessionStore::new(),
    };
    base_routes(governance_routes(operator_routes(Router::new()))).with_state(state)
}

/// Build the tenant API router (excludes operator-only routes).
pub fn tenant_api_router(upstream: Arc<dyn Upstream>) -> Router {
    let state = AppState {
        upstream,
        oidc: auth::OidcConfig::fixture("tenant-bff"),
        sessions: session::SessionStore::new(),
    };
    base_routes(governance_routes(Router::new())).with_state(state)
}

/// Build the operator API router (includes all routes).
pub fn operator_api_router(upstream: Arc<dyn Upstream>) -> Router {
    let state = AppState {
        upstream,
        oidc: auth::OidcConfig::fixture("operator-bff"),
        sessions: session::SessionStore::new(),
    };
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeProfile {
    Development,
    Test,
    Production,
}

impl RuntimeProfile {
    fn from_env() -> Result<Self, ApiError> {
        match std::env::var("ARAF_RUNTIME_PROFILE")
            .unwrap_or_else(|_| "production".into())
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "development" | "dev" => Ok(Self::Development),
            "test" => Ok(Self::Test),
            "production" | "prod" => Ok(Self::Production),
            value => Err(config_error(format!(
                "ARAF_RUNTIME_PROFILE must be development, test, or production (got {value:?})"
            ))),
        }
    }
}

fn config_error(message: impl Into<String>) -> ApiError {
    ApiError::Upstream(crate::error::UpstreamError::Error(message.into()))
}

/// Configuration for building a BFF router.
#[derive(Clone, Debug)]
pub struct BffConfig {
    pub surface: &'static str,
    pub adapter: UpstreamAdapter,
    pub profile: RuntimeProfile,
    pub public_url: Option<String>,
    pub trusted_origins: Vec<String>,
}

impl BffConfig {
    /// Read configuration from environment variables.
    ///
    /// Production is the safe default. Fixture mode requires an explicit
    /// development/test profile and an explicit `fixture` adapter.
    pub fn from_env(surface: &'static str) -> Result<Self, ApiError> {
        let profile = RuntimeProfile::from_env()?;
        let raw_adapter = std::env::var("ARAF_UPSTREAM_ADAPTER").map_err(|_| {
            config_error("ARAF_UPSTREAM_ADAPTER is required; select o3k or explicitly select fixture in development/test")
        })?;
        let adapter = match raw_adapter.trim().to_ascii_lowercase().as_str() {
            "o3k" => UpstreamAdapter::O3k,
            "fixture" if profile != RuntimeProfile::Production => UpstreamAdapter::Fixture,
            "fixture" => return Err(config_error("fixture adapter is forbidden in production")),
            value => {
                return Err(config_error(format!(
                    "unsupported ARAF_UPSTREAM_ADAPTER value {value:?}"
                )))
            }
        };
        if profile == RuntimeProfile::Production && adapter != UpstreamAdapter::O3k {
            return Err(config_error("production requires the o3k upstream adapter"));
        }
        if profile == RuntimeProfile::Production && adapter == UpstreamAdapter::O3k {
            validate_production_o3k_url()?;
        }
        let public_url = std::env::var("ARAF_PUBLIC_URL").ok();
        let trusted_origins: Vec<String> = std::env::var("ARAF_TRUSTED_ORIGINS")
            .ok()
            .map(|value| {
                value
                    .split(',')
                    .map(|origin| origin.trim().to_owned())
                    .filter(|origin| !origin.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        if profile == RuntimeProfile::Production {
            let public_url = public_url
                .as_deref()
                .ok_or_else(|| config_error("ARAF_PUBLIC_URL is required in production"))?;
            validate_public_url(public_url)?;
            if trusted_origins.is_empty() {
                return Err(config_error(
                    "ARAF_TRUSTED_ORIGINS is required in production",
                ));
            }
            for origin in &trusted_origins {
                validate_trusted_origin(origin)?;
            }
        }
        Ok(Self {
            surface,
            adapter,
            profile,
            public_url,
            trusted_origins,
        })
    }
}

fn validate_production_o3k_url() -> Result<(), ApiError> {
    let value = std::env::var("O3K_URL")
        .map_err(|_| config_error("O3K_URL is required for the production o3k adapter"))?;
    let url = reqwest::Url::parse(value.trim())
        .map_err(|_| config_error("production O3K_URL must be a valid absolute URL"))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(config_error(
            "production O3K_URL must be HTTPS, host-qualified, and contain no credentials, query, or fragment",
        ));
    }
    Ok(())
}

fn validate_public_url(value: &str) -> Result<(), ApiError> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| config_error("public URL must be a valid absolute URL"))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || url.username() != ""
        || url.password().is_some()
    {
        return Err(config_error(
            "production public URL/origin must be HTTPS and contain no credentials",
        ));
    }
    Ok(())
}

fn validate_trusted_origin(value: &str) -> Result<(), ApiError> {
    validate_public_url(value)?;
    let url = reqwest::Url::parse(value).expect("validated URL");
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err(config_error(
            "trusted origins must be HTTPS origins without a path, query, or fragment",
        ));
    }
    Ok(())
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
    let sessions = session::SessionStore::new();
    let oidc = if config.adapter == UpstreamAdapter::Fixture {
        auth::OidcConfig::fixture(config.surface)
    } else {
        auth::OidcConfig::from_env(config.surface, config.profile)?
    };
    let state = handlers::AppState {
        upstream,
        oidc,
        sessions: sessions.clone(),
    };
    let router = if config.surface == "operator-bff" {
        operator_api_router_with_state(state)
    } else {
        tenant_api_router_with_state(state)
    };
    Ok(if config.adapter == UpstreamAdapter::Fixture {
        middleware::apply_default_layers(router, config.surface)
    } else {
        middleware::apply_production_layers(
            router,
            config.surface,
            sessions,
            config.trusted_origins,
        )
    })
}

fn tenant_api_router_with_state(state: handlers::AppState) -> Router {
    base_routes(governance_routes(Router::new())).with_state(state)
}

fn operator_api_router_with_state(state: handlers::AppState) -> Router {
    base_routes(governance_routes(operator_routes(Router::new()))).with_state(state)
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
}
