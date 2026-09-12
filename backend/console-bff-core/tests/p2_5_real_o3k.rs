//! Real-process governance smoke gate for the frozen native quota/audit API.
use std::sync::Arc;

use axum::{body::Body, http::Request};
use console_bff_core::{
    middleware::apply_default_layers, tenant_api_router, O3kAdapter, O3kClientConfig,
};
use tower::ServiceExt;

async fn json(response: axum::response::Response) -> (axum::http::StatusCode, serde_json::Value) {
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .expect("body");
    (status, serde_json::from_slice(&body).unwrap_or_default())
}

#[tokio::test]
#[ignore = "requires a running converged O3K process; use tests/p2-5-real-governance-process.sh"]
async fn tenant_bff_reads_real_o3k_quota_and_audit() {
    let base_url = std::env::var("O3K_URL").expect("O3K_URL is required");
    let token = std::env::var("O3K_TOKEN").expect("O3K_TOKEN is required");
    let adapter = O3kAdapter::new("tenant-bff", O3kClientConfig { base_url, token });
    let app = apply_default_layers(tenant_api_router(Arc::new(adapter)), "tenant-bff");

    let context = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/context")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, context) = json(context).await;
    assert!(status.is_success(), "context status {status}: {context}");
    let caps = context["capabilities"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(caps.iter().any(|c| c["resourceType"] == "tenant.quota"));
    assert!(caps.iter().any(|c| c["resourceType"] == "tenant.audit"));

    let quota = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/governance/quotas")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, quota) = json(quota).await;
    assert!(status.is_success(), "quota status {status}: {quota}");
    assert!(quota["items"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));

    let audit = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/governance/audit?pageSize=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, audit) = json(audit).await;
    assert!(status.is_success(), "audit status {status}: {audit}");

    let foreign = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/governance/quotas?projectId=foreign-project")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(foreign.status(), axum::http::StatusCode::FORBIDDEN);
}
