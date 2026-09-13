use std::sync::Arc;

use axum::http::StatusCode;
use console_bff_core::{
    model::ResourceStatus,
    openstack::{OpenStackAdapter, OpenStackClientConfig},
    request::{RequestContext, SessionState},
    upstream::{ListResourcesParams, Upstream},
};
use wiremock::{
    matchers::{header, method, path},
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
