//! Real-process tenant smoke gate. Requires O3K_URL and O3K_TOKEN.

use std::sync::Arc;

use axum::{body::Body, http::Request};
use console_bff_core::{
    middleware::apply_default_layers, tenant_api_router, O3kAdapter, O3kClientConfig,
};
use tower::ServiceExt;

#[tokio::test]
#[ignore = "requires a running converged O3K process; use tests/p2-3-real-tenant-process.sh"]
async fn tenant_bff_reads_real_o3k_discovery_and_bounded_collection() {
    let base_url = std::env::var("O3K_URL").expect("O3K_URL is required");
    let token = std::env::var("O3K_TOKEN").expect("O3K_TOKEN is required");
    let adapter = O3kAdapter::new("tenant-bff", O3kClientConfig { base_url, token });
    let app = apply_default_layers(tenant_api_router(Arc::new(adapter)), "tenant-bff");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/services")
                .body(Body::empty())
                .expect("services request"),
        )
        .await
        .expect("services response");
    if !response.status().is_success() {
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), 128 * 1024)
            .await
            .expect("error body");
        panic!(
            "services request failed: {status}: {}",
            String::from_utf8_lossy(&body)
        );
    }
    let body = axum::body::to_bytes(response.into_body(), 128 * 1024)
        .await
        .expect("services body");
    let services: serde_json::Value = serde_json::from_slice(&body).expect("services json");
    let ids: Vec<&str> = services
        .as_array()
        .expect("service array")
        .iter()
        .filter_map(|service| service.get("id").and_then(serde_json::Value::as_str))
        .collect();
    assert!(ids.contains(&"compute"));
    assert!(ids.contains(&"network"));
    assert!(ids.contains(&"image"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/resources/compute.server?page=0&pageSize=10")
                .body(Body::empty())
                .expect("resources request"),
        )
        .await
        .expect("resources response");
    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 128 * 1024)
        .await
        .expect("resources body");
    let resources: serde_json::Value = serde_json::from_slice(&body).expect("resources json");
    assert!(resources
        .get("items")
        .and_then(serde_json::Value::as_array)
        .is_some());
    assert_eq!(resources["pageSize"], 10);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/resources/network.network?page=0&pageSize=10")
                .body(Body::empty())
                .expect("network preflight list request"),
        )
        .await
        .expect("network preflight list response");
    assert!(
        response.status().is_success(),
        "network list status: {}",
        response.status()
    );
    let _ = response;

    // Exercise a real native tenant mutation through the BFF.  The fake O3K
    // provider advertises Network CRUD, so this is a deterministic proof that
    // the generic Araf runtime is not merely reading discovery metadata.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/network.network")
                .header("Idempotency-Key", "p2-3-real-network-create")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"name":"p2-3-real-network"}"#))
                .expect("network create request"),
        )
        .await
        .expect("network create response");
    if !response.status().is_success() {
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), 128 * 1024)
            .await
            .expect("network create error body");
        panic!(
            "network create status: {status}: {}",
            String::from_utf8_lossy(&body)
        );
    }
    let body = axum::body::to_bytes(response.into_body(), 128 * 1024)
        .await
        .expect("network create body");
    let operation: serde_json::Value = serde_json::from_slice(&body).expect("operation json");
    assert_eq!(operation["action"], "create");
    assert!(operation["id"].as_str().is_some());
    let resource_id = operation["resourceId"]
        .as_str()
        .expect("network create resource id")
        .to_owned();

    // The converged native network DELETE contract is a synchronous 204 with
    // no operation body.  Deletion is covered by the contract adapter tests;
    // this process gate intentionally verifies the operation-bearing create
    // path without inventing an Araf-local operation for a bodyless 204.
    assert!(!resource_id.is_empty());
}
