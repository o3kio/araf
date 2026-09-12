//! Real-process operator diagnostics smoke gate.
//!
//! Run with `O3K_URL` pointing at a converged O3K process and `O3K_TOKEN`
//! containing a server-issued system/operator token. The test deliberately
//! uses the production O3K adapter and operator BFF router; it never enables
//! the fixture adapter.

use std::sync::Arc;

use axum::{body::Body, http::Request};
use console_bff_core::{
    middleware::apply_default_layers, operator_api_router, O3kAdapter, O3kClientConfig,
};
use tower::ServiceExt;

async fn json(response: axum::response::Response) -> (axum::http::StatusCode, serde_json::Value) {
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .expect("response body");
    (status, serde_json::from_slice(&body).unwrap_or_default())
}

#[tokio::test]
#[ignore = "requires a running converged O3K process and system/operator token; use tests/p2-6-real-operator-process.sh"]
async fn operator_bff_reads_real_o3k_diagnostics_without_fixture_fallback() {
    let base_url = std::env::var("O3K_URL").expect("O3K_URL is required");
    let token = std::env::var("O3K_TOKEN").expect("O3K_TOKEN is required");
    let adapter = O3kAdapter::new("operator-bff", O3kClientConfig { base_url, token });
    let app = apply_default_layers(operator_api_router(Arc::new(adapter)), "operator-bff");

    for (path, expected_key) in [
        ("/api/v1/operator/platform/overview", "dataFreshnessAt"),
        ("/api/v1/operator/services/health", "items"),
        ("/api/v1/operator/providers/health", "items"),
        ("/api/v1/operator/capacity", "items"),
        ("/api/v1/operator/regions", "0"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        let (status, body) = json(response).await;
        assert!(status.is_success(), "{path} failed with {status}: {body}");
        if expected_key == "0" {
            assert!(body.is_array(), "regions response must be an array: {body}");
        } else {
            assert!(
                body.get(expected_key).is_some(),
                "{path} missing {expected_key}: {body}"
            );
        }
        let rendered = body.to_string().to_ascii_lowercase();
        assert!(!rendered.contains("access_token"));
        assert!(!rendered.contains("refresh_token"));
        assert!(!rendered.contains("provider_password"));
    }
}
