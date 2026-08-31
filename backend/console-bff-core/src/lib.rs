//! Shared core for the Araf console BFF services.
//!
//! Each console surface (Tenant, Operator) runs its own BFF binary with its
//! own deployment, session and trust boundary (ADR 0001). This crate holds
//! only behavior that is identical for both surfaces. Anything surface
//! specific belongs in the surface binary, not here.
//!
//! M0 scope: liveness/readiness health endpoints only. No authentication, no
//! O3K upstream routing and no session handling exist yet — those arrive with
//! M3/M12 and must never turn this crate into a generic upstream proxy
//! (ADR 0002).

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::Serialize;

/// Identity of the console surface a BFF instance serves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BffSurface {
    /// Stable machine name of the surface, e.g. `tenant-bff`.
    pub service: &'static str,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn healthz(State(surface): State<BffSurface>) -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok",
            service: surface.service,
        }),
    )
}

/// Build the BFF router for the given console surface.
pub fn router(surface: BffSurface) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .with_state(surface)
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
        let app = router(BffSurface {
            service: "tenant-bff",
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024).await.expect("body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "tenant-bff");
    }

    #[tokio::test]
    async fn unknown_routes_are_not_served() {
        let app = router(BffSurface {
            service: "operator-bff",
        });

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
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
