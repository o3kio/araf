use std::sync::Arc;

use axum::http::StatusCode;
use console_bff_core::{
    model::ResourceStatus,
    openstack::{OpenStackAdapter, OpenStackClientConfig},
    request::{RequestContext, SessionState},
    upstream::{ListResourcesParams, Upstream},
};
use wiremock::{
    matchers::{header, method, path, query_param},
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
