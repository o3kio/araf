//! Real-process metering/quota smoke gate for the frozen native contracts.
//! This test is ignored by default and must use a real O3K tenant token.
use std::sync::Arc;

use axum::{body::Body, http::Request};
use console_bff_core::{
    middleware::apply_default_layers, tenant_api_router, O3kAdapter, O3kClientConfig,
};
use tower::ServiceExt;

async fn json(response: axum::response::Response) -> (axum::http::StatusCode, serde_json::Value) {
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 512 * 1024)
        .await
        .expect("body");
    (status, serde_json::from_slice(&body).unwrap_or_default())
}

#[tokio::test]
#[ignore = "requires a running converged O3K process; use tests/p2-7-real-metering-process.sh"]
async fn tenant_bff_reads_real_o3k_metering_without_fixture_fallback() {
    let base_url = std::env::var("O3K_URL").expect("O3K_URL is required");
    let token = std::env::var("O3K_TOKEN").expect("O3K_TOKEN is required");
    let adapter = O3kAdapter::new("tenant-bff", O3kClientConfig { base_url, token });
    let app = apply_default_layers(tenant_api_router(Arc::new(adapter)), "tenant-bff");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/governance/usage?since=2024-01-01T00:00:00Z&until=2024-01-01T02:00:00Z")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let (status, body) = json(response).await;
    assert!(status.is_success(), "usage status {status}: {body}");
    assert!(body["projectId"].as_str().is_some());
    assert!(body["meters"].is_array());
    assert!(body["definitions"].is_array());
    let rendered = body.to_string().to_ascii_lowercase();
    for secret in [
        "access_token",
        "refresh_token",
        "client_secret",
        "authorization",
    ] {
        assert!(!rendered.contains(secret), "response leaked {secret}");
    }

    let quota = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/governance/quotas")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let (status, body) = json(quota).await;
    assert!(status.is_success(), "quota status {status}: {body}");
    assert!(body["items"].is_array());
}
