//! Contract/integration tests for the shared Araf BFF core.
//!
//! These tests drive `fixture_router` end-to-end through axum's `oneshot`
//! helper. They prove the frontend/BFF contract for both console surfaces and
//! verify that the fixture adapter is bounded, deterministic, and not a generic
//! proxy.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    response::Response,
};
use console_bff_core::fixture_router;
use serde_json::json;
use tower::ServiceExt;

const TENANT: &str = "tenant-bff";
const OPERATOR: &str = "operator-bff";
const CORRELATION_HEADER: &str = "x-correlation-id";
const REQUEST_ID_HEADER: &str = "x-request-id";

async fn body_json(response: Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("failed to read body");
    serde_json::from_slice(&body).expect("body is not valid json")
}

async fn assert_problem_details(
    response: Response,
    expected_status: StatusCode,
    expected_correlation: &str,
) -> serde_json::Value {
    assert_eq!(response.status(), expected_status);
    let json = body_json(response).await;
    assert_eq!(
        json["status"].as_u64(),
        Some(expected_status.as_u16() as u64)
    );
    assert_eq!(json["correlationId"].as_str(), Some(expected_correlation));
    assert!(json["type"].is_string());
    assert!(json["title"].is_string());
    json
}

#[tokio::test]
async fn healthz_returns_ok_and_service_name_for_both_surfaces() {
    for surface in [TENANT, OPERATOR] {
        let app = fixture_router(surface);
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
        let json = body_json(response).await;
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], surface);
    }
}

#[tokio::test]
async fn context_reflects_surface_and_operator_capabilities() {
    let tenant = fixture_router(TENANT);
    let operator = fixture_router(OPERATOR);

    let tenant_ctx = body_json(
        tenant
            .oneshot(
                Request::builder()
                    .uri("/api/v1/context")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    let operator_ctx = body_json(
        operator
            .oneshot(
                Request::builder()
                    .uri("/api/v1/context")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert_eq!(tenant_ctx["surface"], TENANT);
    assert_eq!(operator_ctx["surface"], OPERATOR);

    let has_capability = |ctx: &serde_json::Value, resource_type: &str, action: &str| {
        ctx["capabilities"]
            .as_array()
            .expect("capabilities array")
            .iter()
            .any(|c| c["resourceType"] == resource_type && c["action"] == action)
    };

    // Operator fixture includes platform-level capabilities not granted to tenants.
    assert!(
        has_capability(&operator_ctx, "platform.overview", "read"),
        "operator should have platform.overview/read"
    );
    assert!(
        !has_capability(&tenant_ctx, "platform.overview", "read"),
        "tenant should not have platform.overview/read"
    );

    // Both surfaces keep the base compute.server capabilities.
    assert!(has_capability(&tenant_ctx, "compute.server", "list"));
    assert!(has_capability(&operator_ctx, "compute.server", "list"));
}

#[tokio::test]
async fn list_resources_is_bounded_and_reports_total() {
    let app = fixture_router(TENANT);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/resources/compute.server")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["total"], 100_000);
    assert_eq!(json["page"], 0);
    assert!(json["pageSize"].as_u64().unwrap() <= 100);
    assert_eq!(
        json["items"].as_array().unwrap().len() as u64,
        json["pageSize"].as_u64().unwrap()
    );
    assert_eq!(json["hasMore"], true);
}

#[tokio::test]
async fn list_resources_honors_page_size_bound() {
    let app = fixture_router(TENANT);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/resources/compute.server?pageSize=200")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["pageSize"], 100);
    assert_eq!(json["items"].as_array().unwrap().len(), 100);
}

#[tokio::test]
async fn list_resources_honors_pagination() {
    let app = fixture_router(TENANT);

    let first_page = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/resources/compute.server?page=0&pageSize=10")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    let second_page = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/resources/compute.server?page=1&pageSize=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert_eq!(first_page["items"][0]["id"], "resource-0000000000");
    assert_eq!(second_page["items"][0]["id"], "resource-0000000010");
    assert_eq!(first_page["items"].as_array().unwrap().len(), 10);
}

#[tokio::test]
async fn list_resources_honors_project_and_region_filters() {
    let app = fixture_router(TENANT);

    // Use a known resource to obtain a project/region that must survive filtering.
    let detail = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/resources/compute.server/resource-0000000001")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    let project_id = detail["projectId"].as_str().unwrap();
    let region_id = detail["regionId"].as_str().unwrap();

    let by_project = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/resources/compute.server?pageSize=100&projectId={}",
                        project_id
                    ))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    let by_region = body_json(
        app.oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/resources/compute.server?pageSize=100&regionId={}",
                    region_id
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert!(!by_project["items"].as_array().unwrap().is_empty());
    for item in by_project["items"].as_array().unwrap() {
        assert_eq!(item["projectId"], project_id);
    }
    assert!(by_project["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["id"] == "resource-0000000001"));

    assert!(!by_region["items"].as_array().unwrap().is_empty());
    for item in by_region["items"].as_array().unwrap() {
        assert_eq!(item["regionId"], region_id);
    }
}

#[tokio::test]
async fn get_resource_returns_expected_resource() {
    let app = fixture_router(TENANT);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/resources/compute.server/resource-0000000001")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["id"], "resource-0000000001");
    assert_eq!(json["resourceType"], "compute.server");
    assert_eq!(json["name"], "fixture-server-1");
    assert!(json["status"].is_string());
}

#[tokio::test]
async fn submit_action_returns_pending_operation_with_correlation_id() {
    let app = fixture_router(TENANT);
    let correlation = "corr-test-123";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server/resource-0000000001/actions")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from(json!({"actionId": "start"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["state"], "pending");
    assert_eq!(json["action"], "start");
    assert_eq!(json["resourceId"], "resource-0000000001");
    assert_eq!(json["correlationId"], correlation);
}

#[tokio::test]
async fn list_operations_is_bounded_and_detail_returns_operation() {
    let app = fixture_router(TENANT);

    let list = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/operations?pageSize=50")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert_eq!(list["total"], 1_000);
    assert_eq!(list["pageSize"], 50);
    assert_eq!(list["items"].as_array().unwrap().len(), 50);
    assert_eq!(list["hasMore"], true);

    let detail = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/operations/op-0000000001")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert_eq!(detail["id"], "op-0000000001");
    assert!(detail["state"].is_string());
}

#[tokio::test]
async fn unknown_resource_type_returns_404_problem_details_with_correlation_id() {
    let app = fixture_router(TENANT);
    let correlation = "corr-404-test";

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/resources/network.foo")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::NOT_FOUND, correlation).await;
}

#[tokio::test]
async fn unregistered_paths_are_not_proxied() {
    let app = fixture_router(OPERATOR);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/arbitrary-upstream-path")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn request_and_correlation_ids_propagate_to_response_headers() {
    let app = fixture_router(TENANT);
    let request_id = "req-abc";
    let correlation_id = "corr-xyz";

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .header(REQUEST_ID_HEADER, request_id)
                .header(CORRELATION_HEADER, correlation_id)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(REQUEST_ID_HEADER)
            .expect("request id header missing"),
        request_id
    );
    assert_eq!(
        response
            .headers()
            .get(CORRELATION_HEADER)
            .expect("correlation id header missing"),
        correlation_id
    );
}

#[tokio::test]
async fn body_size_limit_rejects_large_post_with_413() {
    let app = fixture_router(TENANT);
    let big_payload = "x".repeat(300_000);
    let body = json!({
        "actionId": "start",
        "payload": { "data": big_payload }
    })
    .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server/resource-0000000001/actions")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn invalid_json_body_returns_400_problem_details() {
    let app = fixture_router(TENANT);
    let correlation = "corr-bad-json";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server/resource-0000000001/actions")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from("not valid json"))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::BAD_REQUEST, correlation).await;
}
