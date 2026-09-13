use std::sync::Arc;

use axum::http::StatusCode;
use console_bff_core::{
    model::{CreateResourceRequest, ResourceStatus},
    openstack::{OpenStackAdapter, OpenStackClientConfig},
    request::{RequestContext, SessionState},
    upstream::{ListResourcesParams, Upstream},
};

#[tokio::test]
async fn scopes_cinder_v3_catalog_endpoint_to_project() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v3/project-1/volumes"))
        .and(header("X-Auth-Token", "keystone-token"))
        .respond_with(ResponseTemplate::new(StatusCode::ACCEPTED).set_body_json(
            serde_json::json!({"volume": {"id": "vol-1", "name": "test", "size": 1, "status": "creating", "project_id": "project-1"}}),
        ))
        .mount(&server)
        .await;
    let adapter = OpenStackAdapter::new(
        "tenant-bff",
        OpenStackClientConfig {
            auth_url: server.uri(),
            token: None,
            username: None,
            password: None,
            user_domain: "Default".into(),
            project_name: None,
            project_id: Some("project-1".into()),
            region: None,
            compute_url: None,
            image_url: None,
            network_url: None,
            volume_url: Some(format!("{}/v3", server.uri())),
            object_storage_url: None,
        },
    )
    .expect("adapter");

    let operation = adapter
        .create_resource(
            &context(),
            "block.volume",
            CreateResourceRequest {
                payload: serde_json::json!({"name": "test", "size": 1}),
            },
        )
        .await
        .expect("create");
    assert_eq!(operation.resource_id.as_deref(), Some("vol-1"));
}
use wiremock::{
    matchers::{header, method, path, query_param, query_param_is_missing},
    Mock, MockServer, ResponseTemplate,
};

fn context() -> RequestContext {
    RequestContext::new(
        "request".into(),
        "correlation".into(),
        Arc::new(SessionState {
            authenticated: true,
            openstack_token: Some("keystone-token".into()),
            ..SessionState::fixture("tenant-bff")
        }),
    )
}

#[tokio::test]
async fn lists_nova_servers_with_server_side_scope_and_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/servers/detail"))
        .and(query_param("project_id", "project-1"))
        .and(header("X-Auth-Token", "keystone-token"))
        .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_json(serde_json::json!({"servers":[{"id":"srv-1","name":"web","status":"ACTIVE","tenant_id":"project-1"}]})))
        .mount(&server)
        .await;
    let adapter = OpenStackAdapter::new(
        "tenant-bff",
        OpenStackClientConfig {
            auth_url: server.uri(),
            token: None,
            username: None,
            password: None,
            user_domain: "Default".into(),
            project_name: None,
            project_id: Some("project-1".into()),
            region: None,
            compute_url: Some(server.uri()),
            image_url: None,
            network_url: None,
            volume_url: None,
            object_storage_url: None,
        },
    )
    .expect("adapter");
    let page = adapter
        .list_resources(
            &context(),
            "compute.server",
            ListResourcesParams {
                page_size: 25,
                ..Default::default()
            },
        )
        .await
        .expect("list");
    assert_eq!(page.items[0].status, ResourceStatus::Ready);
    assert_eq!(page.items[0].project_id, "project-1");
}

#[tokio::test]
async fn rejects_direct_id_reads_outside_selected_project() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/servers/srv-2"))
        .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_json(
            serde_json::json!({"server":{"id":"srv-2","name":"other","status":"ACTIVE","tenant_id":"project-2"}}),
        ))
        .mount(&server)
        .await;
    let adapter = OpenStackAdapter::new(
        "tenant-bff",
        OpenStackClientConfig {
            auth_url: server.uri(),
            token: None,
            username: None,
            password: None,
            user_domain: "Default".into(),
            project_name: None,
            project_id: Some("project-1".into()),
            region: None,
            compute_url: Some(server.uri()),
            image_url: None,
            network_url: None,
            volume_url: None,
            object_storage_url: None,
        },
    )
    .expect("adapter");
    let error = adapter
        .get_resource(&context(), "compute.server", "srv-2")
        .await
        .expect_err("cross-project read must be hidden");
    assert!(matches!(error, console_bff_core::error::ApiError::NotFound));
}

#[tokio::test]
async fn rejects_collection_project_override_for_tenant() {
    let adapter = OpenStackAdapter::new(
        "tenant-bff",
        OpenStackClientConfig {
            auth_url: "https://keystone.example/v3".into(),
            token: Some("keystone-token".into()),
            username: None,
            password: None,
            user_domain: "Default".into(),
            project_name: None,
            project_id: Some("project-1".into()),
            region: None,
            compute_url: Some("https://compute.example".into()),
            image_url: None,
            network_url: None,
            volume_url: None,
            object_storage_url: None,
        },
    )
    .expect("adapter");
    let error = adapter
        .list_resources(
            &context(),
            "compute.server",
            ListResourcesParams {
                project_id: Some("project-2".into()),
                page_size: 25,
                ..Default::default()
            },
        )
        .await
        .expect_err("tenant project override must be rejected");
    assert!(matches!(
        error,
        console_bff_core::error::ApiError::Forbidden
    ));
}

#[tokio::test]
async fn uses_server_sorting_and_neutron_marker_pagination() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2.0/networks"))
        .and(query_param("limit", "2"))
        .and(query_param_is_missing("marker"))
        .and(query_param("sort_key", "name"))
        .and(query_param("sort_dir", "desc"))
        .respond_with(
            ResponseTemplate::new(StatusCode::OK).set_body_json(serde_json::json!({
                "networks": [
                    {"id":"net-1","name":"zeta","status":"ACTIVE","project_id":"project-1"},
                    {"id":"net-2","name":"beta","status":"ACTIVE","project_id":"project-1"}
                ]
            })),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2.0/networks"))
        .and(query_param("limit", "2"))
        .and(query_param("marker", "net-2"))
        .and(query_param("sort_key", "name"))
        .and(query_param("sort_dir", "desc"))
        .respond_with(
            ResponseTemplate::new(StatusCode::OK).set_body_json(serde_json::json!({
                "networks": [
                    {"id":"net-3","name":"alpha","status":"ACTIVE","project_id":"project-1"}
                ]
            })),
        )
        .mount(&server)
        .await;
    let adapter = OpenStackAdapter::new(
        "tenant-bff",
        OpenStackClientConfig {
            auth_url: server.uri(),
            token: Some("keystone-token".into()),
            username: None,
            password: None,
            user_domain: "Default".into(),
            project_name: None,
            project_id: Some("project-1".into()),
            region: None,
            compute_url: None,
            image_url: None,
            network_url: Some(server.uri()),
            volume_url: None,
            object_storage_url: None,
        },
    )
    .expect("adapter");
    let page = adapter
        .list_resources(
            &context(),
            "network.network",
            ListResourcesParams {
                page: 1,
                page_size: 2,
                sort_field: Some("name".into()),
                sort_direction: console_bff_core::model::SortDirection::Desc,
                ..Default::default()
            },
        )
        .await
        .expect("list");
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, "net-3");
}

#[tokio::test]
async fn neutron_short_page_returns_empty_for_later_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2.0/networks"))
        .and(query_param("limit", "2"))
        .and(query_param("project_id", "project-1"))
        .and(query_param_is_missing("marker"))
        .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_json(
            serde_json::json!({
                "networks": [{"id":"net-1","name":"only","status":"ACTIVE","project_id":"project-1"}]
            }),
        ))
        .mount(&server)
        .await;

    let adapter = OpenStackAdapter::new(
        "tenant-bff",
        OpenStackClientConfig {
            auth_url: server.uri(),
            token: Some("keystone-token".into()),
            username: None,
            password: None,
            user_domain: "Default".into(),
            project_name: None,
            project_id: Some("project-1".into()),
            region: None,
            compute_url: None,
            image_url: None,
            network_url: Some(server.uri()),
            volume_url: None,
            object_storage_url: None,
        },
    )
    .expect("adapter");

    let page = adapter
        .list_resources(
            &context(),
            "network.network",
            ListResourcesParams {
                page: 1,
                page_size: 2,
                ..Default::default()
            },
        )
        .await
        .expect("list");
    assert!(page.items.is_empty());
    assert!(!page.has_more);
}
