//! Real-process Operations Center smoke gate.
//!
//! Requires `O3K_URL` and `O3K_TOKEN` and is run by
//! `tests/p2-4-real-operations-process.sh`. The test deliberately uses the
//! production O3K adapter/router path and never enables the fixture adapter.

use std::sync::Arc;

use axum::{body::Body, http::Request};
use console_bff_core::{
    middleware::apply_default_layers, tenant_api_router, O3kAdapter, O3kClientConfig,
};
use tower::ServiceExt;

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .expect("response body");
    assert!(
        status.is_success(),
        "request failed: {status}: {}",
        String::from_utf8_lossy(&body)
    );
    serde_json::from_slice(&body).expect("response json")
}

#[tokio::test]
#[ignore = "requires a running converged O3K process; use tests/p2-4-real-operations-process.sh"]
async fn tenant_bff_reads_and_reconstructs_real_o3k_operations() {
    let base_url = std::env::var("O3K_URL").expect("O3K_URL is required");
    let token = std::env::var("O3K_TOKEN").expect("O3K_TOKEN is required");
    let adapter = O3kAdapter::new("tenant-bff", O3kClientConfig { base_url, token });
    let app = apply_default_layers(tenant_api_router(Arc::new(adapter)), "tenant-bff");

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/network.network")
                .header("Idempotency-Key", "p2-4-real-operation-create")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"name":"p2-4-real-operation-network"}"#))
                .expect("create request"),
        )
        .await
        .expect("create response");
    let created = body_json(create).await;
    let resource_id = created["resourceId"]
        .as_str()
        .expect("created resource id")
        .to_owned();

    // Network CRUD is synchronous in the accepted fake-provider profile. A
    // compute create exercises the durable canonical-operation path used by
    // the Operations Center while still going through the Araf BFF.
    let compute_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server")
                .header("Idempotency-Key", "p2-4-real-operation-compute-create")
                .header("Content-Type", "application/json")
                .body(Body::from(format!(
                    r#"{{"name":"p2-4-real-operation-server","image_id":"image-a","flavor_id":"00000000-0000-0000-0000-000000000001","network_ids":["{resource_id}"]}}"#
                )))
                .expect("compute create request"),
        )
        .await
        .expect("compute create response");
    let compute = body_json(compute_create).await;
    let operation_id = compute["id"]
        .as_str()
        .expect("canonical compute operation id")
        .to_owned();
    let compute_id = compute["resourceId"]
        .as_str()
        .expect("created compute resource id")
        .to_owned();

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/operations?page=0&pageSize=10")
                .body(Body::empty())
                .expect("operation list request"),
        )
        .await
        .expect("operation list response");
    let listed = body_json(list).await;
    let items = listed["items"].as_array().expect("operation items");
    assert!(
        items.iter().any(|item| item["id"] == operation_id),
        "create operation must be discoverable through the canonical list: {listed}"
    );

    let show = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/operations/{operation_id}"))
                .body(Body::empty())
                .expect("operation show request"),
        )
        .await
        .expect("operation show response");
    let shown = body_json(show).await;
    assert_eq!(shown["id"], operation_id);
    assert_eq!(shown["resourceId"], compute_id);

    // A second read is the reload/reconstruction proof: no frontend or BFF
    // process-local operation state is consulted.
    let reload = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/operations/{operation_id}"))
                .body(Body::empty())
                .expect("operation reload request"),
        )
        .await
        .expect("operation reload response");
    let reloaded = body_json(reload).await;
    assert_eq!(reloaded["id"], operation_id);
    assert_eq!(reloaded["resourceId"], compute_id);

    let delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/resources/network.network/{resource_id}"))
                .header("Idempotency-Key", "p2-4-real-operation-delete")
                .body(Body::empty())
                .expect("delete request"),
        )
        .await
        .expect("delete response");
    let deleted = body_json(delete).await;
    assert_eq!(deleted["action"], "delete");
    assert!(
        deleted["id"].as_str().is_some(),
        "delete has operation identity"
    );

    let compute_delete = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/resources/compute.server/{compute_id}"))
                .header("Idempotency-Key", "p2-4-real-operation-compute-delete")
                .body(Body::empty())
                .expect("compute delete request"),
        )
        .await
        .expect("compute delete response");
    let _ = body_json(compute_delete).await;
}
