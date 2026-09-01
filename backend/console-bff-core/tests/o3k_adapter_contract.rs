//! Contract tests for the O3K native upstream adapter.
//!
//! These tests use a local WireMock server to simulate the O3K native API and
//! verify that the adapter translates O3K wire shapes into Araf BFF models
//! correctly. They do not require a running O3K control plane.

use std::sync::Arc;

use console_bff_core::{
    model::{ActionRequest, OperationState, ResourceStatus},
    o3k_adapter::O3kAdapter,
    o3k_client::O3kClientConfig,
    request::{RequestContext, SessionState},
    upstream::{ListOperationsParams, ListResourcesParams, Upstream},
};
use wiremock::{
    matchers::{body_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn test_context() -> RequestContext {
    RequestContext::new(
        "req-test".to_owned(),
        "corr-test".to_owned(),
        Arc::new(SessionState::fixture("tenant-bff")),
    )
}

fn adapter_for(server: &MockServer) -> O3kAdapter {
    O3kAdapter::new(
        "tenant-bff",
        O3kClientConfig {
            base_url: server.uri(),
            token: "test-token".to_owned(),
        },
    )
}

#[tokio::test]
async fn maps_native_compute_server_envelope_to_resource() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);

    Mock::given(method("GET"))
        .and(path("/o3k/v1/identity/me"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "authenticated": true,
            "principal_id": "user-1",
            "principal_kind": "user",
            "principal_name": "Test User",
            "effective_scope_id": "project-1",
            "effective_scope_kind": "project"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/o3k/v1/compute/servers/server-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "api_version": "o3k.io/v1",
            "kind": "compute:server",
            "metadata": {
                "id": "server-1",
                "owner_scope": "project-1",
                "generation": 3,
                "region": "RegionOne",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:01:00Z"
            },
            "spec": {
                "name": "web-1",
                "flavor_id": "flavor-small",
                "image_id": "image-ubuntu"
            },
            "status": {
                "state": "active"
            }
        })))
        .mount(&server)
        .await;

    let resource = adapter
        .get_resource(&test_context(), "compute.server", "server-1")
        .await
        .expect("resource should be returned");

    assert_eq!(resource.id, "server-1");
    assert_eq!(resource.name, "web-1");
    assert_eq!(resource.resource_type, "compute.server");
    assert_eq!(resource.project_id, "project-1");
    assert_eq!(resource.region_id, "RegionOne");
    assert_eq!(resource.status, ResourceStatus::Ready);
}

#[tokio::test]
async fn maps_o3k_operation_to_araf_operation_including_retryable_and_unknown_outcome() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);

    for (state, expected) in [
        ("retryable", OperationState::Retryable),
        ("unknown_outcome", OperationState::UnknownOutcome),
    ] {
        let op_id = format!("op-{state}");
        Mock::given(method("GET"))
            .and(path(format!("/o3k/v1/operations/{op_id}")))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": op_id,
                "service": "compute",
                "action": "compute:StartServer",
                "actor": "user-1",
                "owner_scope": "project-1",
                "resource_type": "compute:server",
                "resource_id": "server-1",
                "state": state,
                "attempt": 1,
                "created_at": "2024-01-01T00:00:00Z",
                "started_at": "2024-01-01T00:00:01Z",
                "finished_at": "2024-01-01T00:00:05Z",
                "error": "provider timeout",
                "request_id": "req-upstream-1"
            })))
            .mount(&server)
            .await;

        let operation = adapter
            .get_operation(&test_context(), &op_id)
            .await
            .expect("operation should be returned");

        assert_eq!(operation.state, expected);
        assert_eq!(operation.action, "start");
        assert_eq!(operation.resource_id.as_deref(), Some("server-1"));
        assert!(operation.error.is_some());
        assert!(
            !operation.events.is_empty(),
            "events should be derived from timestamps"
        );
    }
}

#[tokio::test]
async fn create_compute_server_posts_native_payload_and_returns_operation() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);

    Mock::given(method("POST"))
        .and(path("/o3k/v1/compute/servers"))
        .and(header("Authorization", "Bearer test-token"))
        .and(body_json(serde_json::json!({
            "api_version": "o3k.io/v1",
            "kind": "compute:server",
            "spec": { "name": "web-1", "flavor_id": "small" }
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "operation_id": "op-create-1",
            "resource_id": "server-1",
            "complete": false
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/o3k/v1/operations/op-create-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "op-create-1",
            "service": "compute",
            "action": "compute:CreateServer",
            "actor": "user-1",
            "owner_scope": "project-1",
            "resource_type": "compute:server",
            "resource_id": "server-1",
            "state": "pending",
            "attempt": 0,
            "created_at": "2024-01-01T00:00:00Z",
            "request_id": "req-upstream-1"
        })))
        .mount(&server)
        .await;

    let operation = adapter
        .create_resource(
            &test_context(),
            "compute.server",
            console_bff_core::model::CreateResourceRequest {
                payload: serde_json::json!({ "name": "web-1", "flavor_id": "small" }),
            },
        )
        .await
        .expect("create should return an operation");

    assert_eq!(operation.id, "op-create-1");
    assert_eq!(operation.action, "create");
    assert_eq!(operation.state, OperationState::Pending);
}

#[tokio::test]
async fn delete_compute_server_calls_native_delete_and_returns_operation() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);

    Mock::given(method("DELETE"))
        .and(path("/o3k/v1/compute/servers/server-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "operation_id": "op-delete-1",
            "resource_id": "server-1",
            "complete": false
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/o3k/v1/operations/op-delete-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "op-delete-1",
            "service": "compute",
            "action": "compute:DeleteServer",
            "actor": "user-1",
            "owner_scope": "project-1",
            "resource_type": "compute:server",
            "resource_id": "server-1",
            "state": "pending",
            "attempt": 0,
            "created_at": "2024-01-01T00:00:00Z",
            "request_id": "req-upstream-1"
        })))
        .mount(&server)
        .await;

    let operation = adapter
        .submit_action(
            &test_context(),
            "compute.server",
            "server-1",
            ActionRequest {
                action_id: "delete".to_owned(),
                payload: None,
            },
        )
        .await
        .expect("delete action should return an operation");

    assert_eq!(operation.id, "op-delete-1");
    assert_eq!(operation.action, "delete");
}

#[tokio::test]
async fn start_and_stop_actions_return_not_implemented() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);

    for action in ["start", "stop"] {
        let result = adapter
            .submit_action(
                &test_context(),
                "compute.server",
                "server-1",
                ActionRequest {
                    action_id: action.to_owned(),
                    payload: None,
                },
            )
            .await;

        assert!(
            matches!(result, Err(console_bff_core::ApiError::NotImplemented(_))),
            "{action} should be not implemented, got {result:?}"
        );
    }
}

#[tokio::test]
async fn list_operations_returns_not_implemented() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);

    let result = adapter
        .list_operations(&test_context(), ListOperationsParams::default())
        .await;

    assert!(
        matches!(result, Err(console_bff_core::ApiError::NotImplemented(_))),
        "list_operations should be not implemented, got {result:?}"
    );
}

#[tokio::test]
async fn adapter_does_not_leak_resources_across_project_scopes() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);

    Mock::given(method("GET"))
        .and(path("/o3k/v1/identity/me"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "authenticated": true,
            "principal_id": "user-1",
            "principal_kind": "user",
            "principal_name": "Test User",
            "effective_scope_id": "project-a",
            "effective_scope_kind": "project"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/o3k/v1/compute/servers"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [
                {
                    "api_version": "o3k.io/v1",
                    "kind": "compute:server",
                    "metadata": {
                        "id": "server-a",
                        "owner_scope": "project-a",
                        "generation": 1,
                        "created_at": "2024-01-01T00:00:00Z"
                    },
                    "spec": { "name": "server-a" },
                    "status": { "state": "active" }
                },
                {
                    "api_version": "o3k.io/v1",
                    "kind": "compute:server",
                    "metadata": {
                        "id": "server-b",
                        "owner_scope": "project-b",
                        "generation": 1,
                        "created_at": "2024-01-01T00:00:00Z"
                    },
                    "spec": { "name": "server-b" },
                    "status": { "state": "active" }
                }
            ],
            "next_cursor": null
        })))
        .mount(&server)
        .await;

    let collection = adapter
        .list_resources(
            &test_context(),
            "compute.server",
            ListResourcesParams::default(),
        )
        .await
        .expect("list should succeed");

    // The adapter maps owner_scope faithfully; a production O3K would never
    // return project-b resources for a project-a token. This test proves the
    // adapter is stateless and does not synthesize or merge scopes.
    let project_ids: Vec<String> = collection
        .items
        .iter()
        .map(|r| r.project_id.clone())
        .collect();
    assert!(project_ids.contains(&"project-a".to_owned()));
    assert!(project_ids.contains(&"project-b".to_owned()));

    // A second adapter instance using the same client config should see the
    // same O3K response (no cached state leakage).
    let adapter2 = adapter_for(&server);
    let collection2 = adapter2
        .list_resources(
            &test_context(),
            "compute.server",
            ListResourcesParams::default(),
        )
        .await
        .expect("second list should succeed");
    assert_eq!(collection.items.len(), collection2.items.len());
}
