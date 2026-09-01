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
use console_bff_core::{
    fixture_router, O3kAdapter, O3kClientConfig, RequestContext, SessionState, Upstream,
};
use serde_json::json;
use std::sync::Arc;
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

fn capability_present(ctx: &serde_json::Value, resource_type: &str, action: &str) -> bool {
    ctx["capabilities"]
        .as_array()
        .expect("capabilities array")
        .iter()
        .any(|c| c["resourceType"] == resource_type && c["action"] == action)
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

#[tokio::test]
async fn services_include_three_resource_types_with_presentation_metadata() {
    let app = fixture_router(TENANT);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/services")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    let services = json.as_array().expect("services array");
    let ids: Vec<_> = services.iter().map(|s| s["id"].as_str().unwrap()).collect();
    assert!(ids.contains(&"compute"));
    assert!(ids.contains(&"network"));
    assert!(ids.contains(&"storage"));

    let find_type = |type_id: &str| {
        services
            .iter()
            .flat_map(|s| s["resourceTypes"].as_array().unwrap())
            .find(|rt| rt["id"] == type_id)
            .cloned()
            .unwrap_or_else(|| panic!("missing resource type {type_id}"))
    };

    let server = find_type("compute.server");
    assert_eq!(server["name"], "Server");
    assert!(!server["columns"].as_array().unwrap().is_empty());
    assert!(!server["filters"].as_array().unwrap().is_empty());
    assert!(!server["sortableFields"].as_array().unwrap().is_empty());
    assert!(!server["detailsSections"].as_array().unwrap().is_empty());
    assert_eq!(server["iconToken"], "server");

    let vpc = find_type("network.vpc");
    assert_eq!(vpc["name"], "VPC");
    assert!(vpc["columns"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["id"] == "cidr"));

    let volume = find_type("storage.volume");
    assert_eq!(volume["name"], "Volume");
    assert!(!volume["relationships"].as_array().unwrap().is_empty());
    let rel = &volume["relationships"][0];
    assert_eq!(rel["targetResourceType"], "compute.server");
    assert_eq!(rel["sourcePropertyKey"], "attachedServerId");
    assert_eq!(rel["direction"], "to-one");
}

#[tokio::test]
async fn list_network_vpc_is_bounded_and_reports_total() {
    let app = fixture_router(TENANT);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/resources/network.vpc")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["total"], 1_000);
    assert_eq!(json["items"][0]["resourceType"], "network.vpc");
    assert!(json["items"][0]["id"].as_str().unwrap().starts_with("vpc-"));
    assert_eq!(json["hasMore"], true);
}

#[tokio::test]
async fn list_storage_volume_is_bounded_and_reports_total() {
    let app = fixture_router(TENANT);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/resources/storage.volume")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["total"], 5_000);
    assert_eq!(json["items"][0]["resourceType"], "storage.volume");
    assert!(json["items"][0]["id"]
        .as_str()
        .unwrap()
        .starts_with("volume-"));
    assert_eq!(json["hasMore"], true);
}

#[tokio::test]
async fn storage_volume_detail_exposes_attached_server_relationship() {
    let app = fixture_router(TENANT);

    let volume_detail = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/resources/storage.volume/volume-00000001")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert_eq!(volume_detail["resourceType"], "storage.volume");
    assert!(volume_detail["properties"].is_object());
    let attached_server_id = volume_detail["properties"]["attachedServerId"]
        .as_str()
        .expect("attachedServerId");
    assert!(attached_server_id.starts_with("resource-"));

    let server_detail = body_json(
        app.oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/resources/compute.server/{attached_server_id}"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert_eq!(server_detail["id"], attached_server_id);
    assert_eq!(server_detail["resourceType"], "compute.server");
}

#[tokio::test]
async fn storage_volume_filters_by_attached_server_id() {
    let app = fixture_router(TENANT);

    let volume_detail = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/resources/storage.volume/volume-00000001")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    let attached_server_id = volume_detail["properties"]["attachedServerId"]
        .as_str()
        .unwrap();

    let filtered = body_json(
        app.oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/resources/storage.volume?pageSize=100&attachedServerId={attached_server_id}"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert!(!filtered["items"].as_array().unwrap().is_empty());
    for item in filtered["items"].as_array().unwrap() {
        assert_eq!(
            item["properties"]["attachedServerId"].as_str().unwrap(),
            attached_server_id
        );
    }
}

#[tokio::test]
async fn list_resources_honors_sorting() {
    let app = fixture_router(TENANT);

    let asc = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/resources/compute.server?pageSize=5&sortField=name&sortDirection=asc")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    let desc = body_json(
        app.oneshot(
            Request::builder()
                .uri(
                    "/api/v1/resources/compute.server?pageSize=5&sortField=name&sortDirection=desc",
                )
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    let asc_names: Vec<String> = asc["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap().to_string())
        .collect();
    let mut sorted = asc_names.clone();
    sorted.sort();
    assert_eq!(asc_names, sorted);

    let desc_names: Vec<String> = desc["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(desc_names, sorted.into_iter().rev().collect::<Vec<_>>());
}

#[tokio::test]
async fn network_vpc_detail_exposes_cidr_property() {
    let app = fixture_router(TENANT);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/resources/network.vpc/vpc-0000001")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["resourceType"], "network.vpc");
    assert!(json["properties"]["cidrBlock"].is_string());
}

#[tokio::test]
async fn create_compute_server_valid_returns_pending_operation() {
    let app = fixture_router(TENANT);
    let correlation = "corr-create-ok";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from(
                    json!({
                        "name": "test-server",
                        "regionId": "eu-west",
                        "projectId": "project-1",
                        "bootVolumeSizeGb": 50
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["state"], "pending");
    assert_eq!(json["action"], "create");
    assert_eq!(json["resourceType"], "compute.server");
    assert!(json["resourceId"]
        .as_str()
        .unwrap()
        .starts_with("resource-"));
    assert_eq!(json["correlationId"], correlation);
}

#[tokio::test]
async fn create_compute_server_missing_required_returns_400() {
    let app = fixture_router(TENANT);
    let correlation = "corr-create-missing";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from(
                    json!({"regionId": "eu-west", "projectId": "project-1"}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::BAD_REQUEST, correlation).await;
}

#[tokio::test]
async fn create_compute_server_wrong_type_returns_400() {
    let app = fixture_router(TENANT);
    let correlation = "corr-create-type";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from(
                    json!({
                        "name": 123,
                        "regionId": "eu-west",
                        "projectId": "project-1"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::BAD_REQUEST, correlation).await;
}

#[tokio::test]
async fn create_compute_server_out_of_range_returns_400() {
    let app = fixture_router(TENANT);
    let correlation = "corr-create-range";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from(
                    json!({
                        "name": "test-server",
                        "regionId": "eu-west",
                        "projectId": "project-1",
                        "bootVolumeSizeGb": 5
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::BAD_REQUEST, correlation).await;
}

#[tokio::test]
async fn create_without_capability_returns_403() {
    let app = fixture_router(TENANT);
    let correlation = "corr-create-forbidden";

    // The fixture does not grant network.vpc/create.
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/network.vpc")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from(
                    json!({
                        "name": "test-vpc",
                        "regionId": "eu-west",
                        "projectId": "project-1",
                        "cidrBlock": "10.0.0.0/24"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::FORBIDDEN, correlation).await;
}

#[tokio::test]
async fn submit_delete_action_returns_pending_operation() {
    let app = fixture_router(TENANT);
    let correlation = "corr-delete-ok";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server/resource-0000000001/actions")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from(json!({"actionId": "delete"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["state"], "pending");
    assert_eq!(json["action"], "delete");
    assert_eq!(json["resourceId"], "resource-0000000001");
    assert_eq!(json["correlationId"], correlation);
}

#[tokio::test]
async fn submit_invalid_action_id_returns_400() {
    let app = fixture_router(TENANT);
    let correlation = "corr-action-invalid";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server/resource-0000000001/actions")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from(json!({"actionId": "reboot"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::BAD_REQUEST, correlation).await;
}

#[tokio::test]
async fn submit_attach_action_invalid_input_returns_400() {
    let app = fixture_router(TENANT);
    let correlation = "corr-attach-invalid";

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/storage.volume/volume-00000001/actions")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from(
                    json!({"actionId": "attach", "payload": {"serverId": ""}}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::BAD_REQUEST, correlation).await;
}

#[tokio::test]
async fn submit_action_without_capability_returns_403() {
    let app = fixture_router(TENANT);
    let correlation = "corr-action-forbidden";

    // The fixture descriptor defines stop but the fixture session is not granted compute.server/stop.
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/resources/compute.server/resource-0000000001/actions")
                .header("content-type", "application/json")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::from(json!({"actionId": "stop"}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::FORBIDDEN, correlation).await;
}

#[tokio::test]
async fn services_expose_create_schema_and_action_metadata() {
    let app = fixture_router(TENANT);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/services")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    let server = json
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "compute")
        .unwrap()["resourceTypes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|rt| rt["id"] == "compute.server")
        .cloned()
        .unwrap();

    assert!(server["createSchema"].is_object());
    assert_eq!(server["createCapability"]["resourceType"], "compute.server");
    assert_eq!(server["createCapability"]["action"], "create");

    let delete = server["supportedActions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == "delete")
        .cloned()
        .unwrap();
    assert_eq!(delete["riskClass"], "destructive");
    assert_eq!(delete["requiredCapability"]["action"], "delete");

    let attach = json
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "storage")
        .unwrap()["resourceTypes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|rt| rt["id"] == "storage.volume")
        .cloned()
        .unwrap()["supportedActions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == "attach")
        .cloned()
        .unwrap();
    assert!(attach["inputSchema"].is_object());
    assert_eq!(attach["riskClass"], "normal");
}

#[tokio::test]
async fn create_operation_is_retrievable_by_id_after_reload() {
    let app = fixture_router(TENANT);
    let correlation = "corr-create-reload";

    let create = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/resources/compute.server")
                    .header("content-type", "application/json")
                    .header(CORRELATION_HEADER, correlation)
                    .body(Body::from(
                        json!({
                            "name": "reload-server",
                            "regionId": "eu-west",
                            "projectId": "project-1",
                            "bootVolumeSizeGb": 50
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert_eq!(create["state"], "pending");
    assert_eq!(create["correlationId"], correlation);
    let op_id = create["id"].as_str().expect("operation id");

    // Simulate a page reload by fetching the same operation again.
    let detail = body_json(
        app.oneshot(
            Request::builder()
                .uri(format!("/api/v1/operations/{op_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert_eq!(detail["id"], op_id);
    assert_eq!(detail["state"], "pending");
    assert!(detail["events"].is_array());
    assert!(!detail["events"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn operation_state_transitions_through_lifecycle() {
    let app = fixture_router(TENANT);
    let correlation = "corr-transition";

    let create = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/resources/compute.server")
                    .header("content-type", "application/json")
                    .header(CORRELATION_HEADER, correlation)
                    .body(Body::from(
                        json!({
                            "name": "transition-server",
                            "regionId": "eu-west",
                            "projectId": "project-1",
                            "bootVolumeSizeGb": 50
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    let op_id = create["id"].as_str().expect("operation id");
    assert_eq!(create["state"], "pending");

    // Wait for Pending -> Running transition.
    tokio::time::sleep(std::time::Duration::from_millis(1_100)).await;
    let running = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/operations/{op_id}"))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;
    assert_eq!(running["state"], "running");

    // Wait for Running -> terminal transition.
    tokio::time::sleep(std::time::Duration::from_millis(2_100)).await;
    let terminal = body_json(
        app.oneshot(
            Request::builder()
                .uri(format!("/api/v1/operations/{op_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;
    assert!(
        terminal["state"] == "succeeded" || terminal["state"] == "failed",
        "operation should reach a terminal state, got {}",
        terminal["state"]
    );
}

#[tokio::test]
async fn list_operations_filters_by_state() {
    let app = fixture_router(TENANT);

    let pending = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/operations?state=pending&pageSize=5")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert!(!pending["items"].as_array().unwrap().is_empty());
    for item in pending["items"].as_array().unwrap() {
        assert_eq!(item["state"], "pending");
    }

    let succeeded = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/operations?state=succeeded&pageSize=5")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert!(!succeeded["items"].as_array().unwrap().is_empty());
    for item in succeeded["items"].as_array().unwrap() {
        assert_eq!(item["state"], "succeeded");
    }
}

#[tokio::test]
async fn operation_detail_returns_event_timeline() {
    let app = fixture_router(TENANT);
    let correlation = "corr-events";

    let create = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/resources/compute.server")
                    .header("content-type", "application/json")
                    .header(CORRELATION_HEADER, correlation)
                    .body(Body::from(
                        json!({
                            "name": "events-server",
                            "regionId": "eu-west",
                            "projectId": "project-1",
                            "bootVolumeSizeGb": 50
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    let op_id = create["id"].as_str().expect("operation id");

    // Advance the operation so the timeline contains multiple events.
    tokio::time::sleep(std::time::Duration::from_millis(3_300)).await;
    let detail = body_json(
        app.oneshot(
            Request::builder()
                .uri(format!("/api/v1/operations/{op_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    let events = detail["events"].as_array().expect("events array");
    assert!(
        events.len() >= 2,
        "expected multiple lifecycle events, got {}",
        events.len()
    );

    let first = &events[0];
    assert_eq!(first["state"], "pending");
    assert!(first["occurredAt"].is_string());
    assert!(first["message"].is_string());
    assert_eq!(first["correlationId"], correlation);
}

#[tokio::test]
async fn tenant_context_includes_governance_capabilities() {
    let ctx = body_json(
        fixture_router(TENANT)
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

    assert!(capability_present(&ctx, "tenant.project", "list"));
    assert!(capability_present(&ctx, "tenant.project", "read"));
    assert!(capability_present(&ctx, "tenant.user", "list"));
    assert!(capability_present(&ctx, "tenant.user", "read"));
    assert!(capability_present(&ctx, "tenant.role", "list"));
    assert!(capability_present(&ctx, "tenant.quota", "read"));
    assert!(capability_present(&ctx, "tenant.audit", "read"));
    assert!(capability_present(&ctx, "tenant.api-credential", "list"));
    assert!(capability_present(&ctx, "tenant.api-credential", "create"));
    assert!(capability_present(&ctx, "tenant.api-credential", "delete"));
}

#[tokio::test]
async fn governance_lists_are_bounded_and_paginated() {
    let app = fixture_router(TENANT);

    let projects = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/governance/projects")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;
    assert!(projects["items"].as_array().unwrap().len() <= 6);
    assert_eq!(projects["hasMore"], false);

    let users = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/governance/users?pageSize=10")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;
    assert!(
        users["items"].as_array().unwrap().len() <= 25,
        "user list must remain server-bounded"
    );
    assert_eq!(users["total"], 50);
    assert_eq!(users["hasMore"], true);

    let roles = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/governance/roles")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;
    assert_eq!(roles["total"], 4);

    let audit = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/governance/audit?pageSize=5")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;
    assert_eq!(audit["items"].as_array().unwrap().len(), 5);
    assert_eq!(audit["total"], 200);
    assert_eq!(audit["hasMore"], true);

    let quotas = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/governance/quotas")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;
    assert!(!quotas["items"].as_array().unwrap().is_empty());
    for quota in quotas["items"].as_array().unwrap() {
        assert!(!quota["entries"].as_array().unwrap().is_empty());
    }
}

#[tokio::test]
async fn get_project_returns_requested_project() {
    let app = fixture_router(TENANT);

    let project = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/governance/projects/project-1")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert_eq!(project["id"], "project-1");
    assert_eq!(project["status"], "active");
    assert!(project["name"].is_string());
    assert!(project["organizationId"].is_string());
}

#[tokio::test]
async fn cross_project_access_is_rejected() {
    let app = fixture_router(TENANT);
    let correlation = "corr-cross-project";

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/governance/projects/project-99")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_problem_details(response, StatusCode::NOT_FOUND, correlation).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/governance/quotas?projectId=project-99")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_problem_details(response, StatusCode::NOT_FOUND, correlation).await;
}

#[tokio::test]
async fn governance_missing_capability_returns_403() {
    let app = fixture_router(OPERATOR);
    let correlation = "corr-governance-forbidden";

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/governance/projects")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::FORBIDDEN, correlation).await;
}

#[tokio::test]
async fn audit_events_are_distinct_from_operations() {
    let app = fixture_router(TENANT);

    let audit = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/governance/audit?pageSize=5")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert!(!audit["items"].as_array().unwrap().is_empty());
    for event in audit["items"].as_array().unwrap() {
        assert!(event["outcome"].is_string());
        assert!(event["recordedAt"].is_string());
        assert!(event["actor"].is_string());
        assert!(event["state"].is_null());
    }

    let operations = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/operations?pageSize=5")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    for op in operations["items"].as_array().unwrap() {
        assert!(op["state"].is_string());
        assert!(op["outcome"].is_null());
    }
}

#[tokio::test]
async fn api_credential_creation_returns_secret_and_list_hides_it() {
    let app = fixture_router(TENANT);
    let correlation = "corr-credential";

    let create = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/governance/api-credentials")
                    .header("content-type", "application/json")
                    .header(CORRELATION_HEADER, correlation)
                    .body(Body::from(
                        json!({
                            "name": "test-credential",
                            "kind": "service-account",
                            "projectId": "project-1"
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert_eq!(create["name"], "test-credential");
    assert_eq!(create["kind"], "service-account");
    assert_eq!(create["projectId"], "project-1");
    let secret = create["secret"].as_str().expect("secret present on create");
    assert!(secret.starts_with("secret-"));
    let credential_id = create["id"].as_str().expect("credential id");

    let list = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/governance/api-credentials")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert_eq!(list["total"], 1);
    let item = &list["items"][0];
    assert_eq!(item["id"], credential_id);
    assert!(item["secret"].is_null());

    let delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/governance/api-credentials/{credential_id}"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(delete.status(), StatusCode::NO_CONTENT);

    let list = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/governance/api-credentials")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;
    assert_eq!(list["total"], 0);
}

#[tokio::test]
async fn o3k_adapter_governance_methods_return_501() {
    let adapter = O3kAdapter::new(
        "tenant-bff",
        O3kClientConfig {
            base_url: "http://localhost:9999".to_owned(),
            token: "unused-token".to_owned(),
        },
    );
    let ctx = RequestContext::new(
        "req-test".to_owned(),
        "corr-test".to_owned(),
        Arc::new(SessionState::default()),
    );

    macro_rules! assert_not_implemented {
        ($expr:expr) => {
            let err = $expr.expect_err("expected NotImplemented");
            assert_eq!(err.status(), StatusCode::NOT_IMPLEMENTED);
        };
    }

    assert_not_implemented!(adapter.list_projects(&ctx).await);
    assert_not_implemented!(adapter.get_project(&ctx, "project-1").await);
    assert_not_implemented!(adapter.list_users(&ctx).await);
    assert_not_implemented!(adapter.get_user(&ctx, "user-001").await);
    assert_not_implemented!(adapter.list_roles(&ctx).await);
    assert_not_implemented!(adapter.list_quotas(&ctx, None).await);
    assert_not_implemented!(adapter.list_audit_events(&ctx, Default::default()).await);
    assert_not_implemented!(adapter.list_api_credentials(&ctx).await);
    assert_not_implemented!(
        adapter
            .create_api_credential(
                &ctx,
                console_bff_core::model::CreateApiCredentialRequest {
                    name: "x".to_owned(),
                    kind: "service-account".to_owned(),
                    project_id: "project-1".to_owned(),
                    expires_at: None,
                },
            )
            .await
    );
    assert_not_implemented!(adapter.delete_api_credential(&ctx, "cred-1").await);
}

#[tokio::test]
async fn operator_can_list_platform_overview() {
    let app = fixture_router(OPERATOR);

    let overview = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/operator/platform/overview")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert!(overview["regionStatusSummary"].is_array());
    assert!(overview["providerStatusSummary"].is_array());
    assert!(overview["activeOperationsCount"].is_number());
    assert!(overview["recentAlerts"].is_array());
    assert!(overview["dataFreshnessAt"].is_string());
}

#[tokio::test]
async fn operator_can_list_regions_and_zones() {
    let app = fixture_router(OPERATOR);

    let regions = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/operator/regions")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert_eq!(regions.as_array().unwrap().len(), 4);
    for region in regions.as_array().unwrap() {
        assert!(region["id"].is_string());
        assert!(region["azs"].is_array());
    }

    let zones = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/operator/regions/eu-west/zones")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert!(!zones.as_array().unwrap().is_empty());
    for zone in zones.as_array().unwrap() {
        assert_eq!(zone["regionId"], "eu-west");
    }
}

#[tokio::test]
async fn operator_can_list_provider_health_and_capacity() {
    let app = fixture_router(OPERATOR);

    let providers = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/operator/providers/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert_eq!(providers.as_array().unwrap().len(), 12);
    for provider in providers.as_array().unwrap() {
        assert!(provider["kind"].is_string());
        assert!(provider["status"].is_string());
    }

    let capacity = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/operator/capacity")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert_eq!(capacity.as_array().unwrap().len(), 3);
    for entry in capacity.as_array().unwrap() {
        assert!(entry["resourceClass"].is_string());
        assert!(entry["available"].is_number());
    }
}

#[tokio::test]
async fn operator_can_list_accounts_and_account_projects() {
    let app = fixture_router(OPERATOR);

    let accounts = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/operator/accounts")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert!(!accounts["items"].as_array().unwrap().is_empty());

    let account_id = accounts["items"][0]["id"].as_str().unwrap();
    let projects = body_json(
        app.oneshot(
            Request::builder()
                .uri(format!("/api/v1/operator/accounts/{account_id}/projects"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    for project in projects["items"].as_array().unwrap() {
        assert_eq!(project["accountId"], account_id);
    }
}

#[tokio::test]
async fn operator_can_list_operations_and_audit_events() {
    let app = fixture_router(OPERATOR);

    let operations = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/operator/operations?pageSize=10")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response"),
    )
    .await;

    assert_eq!(operations["items"].as_array().unwrap().len(), 10);
    assert!(operations["total"].is_number());

    let audit = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/operator/audit-events?pageSize=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    assert_eq!(audit["items"].as_array().unwrap().len(), 10);
    assert!(audit["total"].is_number());
    for event in audit["items"].as_array().unwrap() {
        assert!(event["actor"].is_string());
        assert!(event["accountId"].is_string());
    }
}

#[tokio::test]
async fn tenant_bff_does_not_expose_operator_endpoints() {
    let app = fixture_router(TENANT);
    let correlation = "corr-tenant-operator-denied";

    let paths = [
        "/api/v1/operator/platform/overview",
        "/api/v1/operator/regions",
        "/api/v1/operator/providers/health",
        "/api/v1/operator/capacity",
        "/api/v1/operator/accounts",
        "/api/v1/operator/operations",
        "/api/v1/operator/audit-events",
    ];

    for path in paths {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header(CORRELATION_HEADER, correlation)
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        // The tenant BFF process does not mount operator routes (ADR 0001).
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn operator_endpoints_reject_missing_capabilities_with_403() {
    let app = fixture_router(OPERATOR);
    let correlation = "corr-operator-capability-denied";

    // Tenant governance endpoints are forbidden to operator fixture session.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/governance/projects")
                .header(CORRELATION_HEADER, correlation)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_problem_details(response, StatusCode::FORBIDDEN, correlation).await;
}

#[tokio::test]
async fn operator_accounts_do_not_leak_cross_tenant_data() {
    let app = fixture_router(OPERATOR);

    let accounts = body_json(
        app.oneshot(
            Request::builder()
                .uri("/api/v1/operator/accounts")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response"),
    )
    .await;

    let ids: std::collections::HashSet<String> = accounts["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["id"].as_str().unwrap().to_string())
        .collect();

    // Operator fixture universe uses account-* identifiers distinct from tenant projects.
    for id in ids {
        assert!(
            id.starts_with("account-"),
            "operator account id should be account-scoped, got {id}"
        );
    }
}

#[tokio::test]
async fn o3k_adapter_operator_methods_return_501() {
    let adapter = O3kAdapter::new(
        "operator-bff",
        O3kClientConfig {
            base_url: "http://localhost:9999".to_owned(),
            token: "unused-token".to_owned(),
        },
    );
    let ctx = RequestContext::new(
        "req-test".to_owned(),
        "corr-test".to_owned(),
        Arc::new(SessionState::default()),
    );

    macro_rules! assert_not_implemented {
        ($expr:expr) => {
            let err = $expr.expect_err("expected NotImplemented");
            assert_eq!(err.status(), StatusCode::NOT_IMPLEMENTED);
        };
    }

    assert_not_implemented!(adapter.list_regions(&ctx).await);
    assert_not_implemented!(adapter.list_availability_zones(&ctx, "eu-west").await);
    assert_not_implemented!(adapter.list_provider_health(&ctx).await);
    assert_not_implemented!(adapter.list_service_health(&ctx).await);
    assert_not_implemented!(adapter.get_capacity_summary(&ctx).await);
    assert_not_implemented!(adapter.list_customer_accounts(&ctx).await);
    assert_not_implemented!(adapter.list_operator_projects(&ctx, None).await);
    assert_not_implemented!(
        adapter
            .list_operator_operations(&ctx, Default::default())
            .await
    );
    assert_not_implemented!(
        adapter
            .list_operator_audit_events(&ctx, Default::default())
            .await
    );
    assert_not_implemented!(adapter.get_platform_overview(&ctx).await);
}
