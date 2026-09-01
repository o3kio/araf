//! Shared BFF HTTP handlers.

use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{rejection::JsonRejection, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::info;

use crate::{
    error::{ApiError, BffError},
    model::{
        ActionRequest, ApiCredential, AuditEvent, CapacitySummary, CreateApiCredentialRequest,
        CreateResourceRequest, CustomerAccount, ListAuditEventsParams, Operation, OperationState,
        OperatorAuditEvent, OperatorProject, PaginatedCollection, PlatformOverview, Project,
        ProjectMember, ProjectQuota, ProviderHealth, Region, Resource, Role, ServiceDescriptor,
        ServiceHealth, SessionContext, SortDirection, User,
    },
    request::RequestContext,
    upstream::{
        ListOperationsParams, ListOperatorAuditEventsParams, ListOperatorOperationsParams,
        ListResourcesParams, Upstream,
    },
};

/// Attach the request's correlation id to an upstream error so Problem Details
/// responses can be traced end-to-end.
fn with_ctx<E: Into<ApiError>>(err: E, ctx: &RequestContext) -> BffError {
    BffError::new(err.into(), ctx.correlation_id())
}

/// Application state shared by handlers.
#[derive(Clone)]
pub struct AppState {
    pub upstream: Arc<dyn Upstream>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesQuery {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub project_id: Option<String>,
    pub region_id: Option<String>,
    pub attached_server_id: Option<String>,
    pub sort_field: Option<String>,
    #[serde(default)]
    pub sort_direction: SortDirection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOperationsQuery {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub state: Option<OperationState>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub project_id: Option<String>,
    pub region_id: Option<String>,
    pub since: Option<time::OffsetDateTime>,
    pub until: Option<time::OffsetDateTime>,
}

fn default_page_size() -> u32 {
    25
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListGovernanceQuery {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuotasQuery {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditEventsQuery {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub project_id: Option<String>,
    pub action: Option<String>,
    pub actor: Option<String>,
    pub since: Option<time::OffsetDateTime>,
    pub until: Option<time::OffsetDateTime>,
}

pub async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "ok", "service": state.upstream.surface() })),
    )
}

pub async fn get_context(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> Result<Json<SessionContext>, BffError> {
    let context = state
        .upstream
        .context(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(context))
}

pub async fn list_services(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> Result<Json<Vec<ServiceDescriptor>>, BffError> {
    let services = state
        .upstream
        .services(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(services))
}

pub async fn list_resources(
    State(state): State<AppState>,
    Path(resource_type): Path<String>,
    Query(query): Query<ListResourcesQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<Resource>>, BffError> {
    let mut filters = HashMap::new();
    if let Some(server_id) = query.attached_server_id {
        filters.insert("attached_server_id".to_string(), server_id);
    }

    let params = ListResourcesParams {
        page: query.page,
        page_size: query.page_size,
        project_id: query.project_id,
        region_id: query.region_id,
        filters,
        sort_field: query.sort_field,
        sort_direction: query.sort_direction,
    };

    let collection = state
        .upstream
        .list_resources(&ctx, &resource_type, params)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(collection))
}

pub async fn create_resource(
    State(state): State<AppState>,
    Path(resource_type): Path<String>,
    ctx: RequestContext,
    request: Result<Json<CreateResourceRequest>, JsonRejection>,
) -> Result<Json<Operation>, BffError> {
    let Json(request) = request.map_err(|e| with_ctx(e, &ctx))?;

    info!(
        correlation_id = %ctx.correlation_id(),
        resource_type = %resource_type,
        "creating resource"
    );
    let operation = state
        .upstream
        .create_resource(&ctx, &resource_type, request)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(operation))
}

pub async fn get_resource(
    State(state): State<AppState>,
    Path((resource_type, id)): Path<(String, String)>,
    ctx: RequestContext,
) -> Result<Json<Resource>, BffError> {
    let resource = state
        .upstream
        .get_resource(&ctx, &resource_type, &id)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(resource))
}

pub async fn submit_action(
    State(state): State<AppState>,
    Path((resource_type, id)): Path<(String, String)>,
    ctx: RequestContext,
    request: Result<Json<ActionRequest>, JsonRejection>,
) -> Result<Json<Operation>, BffError> {
    let Json(request) = request.map_err(|e| with_ctx(e, &ctx))?;

    info!(
        correlation_id = %ctx.correlation_id(),
        resource_type = %resource_type,
        resource_id = %id,
        action = %request.action_id,
        "submitting action"
    );
    let operation = state
        .upstream
        .submit_action(&ctx, &resource_type, &id, request)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(operation))
}

pub async fn list_operations(
    State(state): State<AppState>,
    Query(query): Query<ListOperationsQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<Operation>>, BffError> {
    let params = ListOperationsParams {
        page: query.page,
        page_size: query.page_size,
        state: query.state,
        action: query.action,
        resource_type: query.resource_type,
        resource_id: query.resource_id,
        project_id: query.project_id,
        region_id: query.region_id,
        since: query.since,
        until: query.until,
    };
    let operations = state
        .upstream
        .list_operations(&ctx, params)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(operations))
}

pub async fn get_operation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ctx: RequestContext,
) -> Result<Json<Operation>, BffError> {
    let operation = state
        .upstream
        .get_operation(&ctx, &id)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(operation))
}

pub async fn list_projects(
    State(state): State<AppState>,
    Query(_query): Query<ListGovernanceQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<Project>>, BffError> {
    let projects = state
        .upstream
        .list_projects(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(projects))
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ctx: RequestContext,
) -> Result<Json<Project>, BffError> {
    let project = state
        .upstream
        .get_project(&ctx, &id)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(project))
}

pub async fn list_project_members(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ctx: RequestContext,
) -> Result<Json<Vec<ProjectMember>>, BffError> {
    let members = state
        .upstream
        .list_project_members(&ctx, &id)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(members))
}

pub async fn list_users(
    State(state): State<AppState>,
    Query(_query): Query<ListGovernanceQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<User>>, BffError> {
    let users = state
        .upstream
        .list_users(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(users))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ctx: RequestContext,
) -> Result<Json<User>, BffError> {
    let user = state
        .upstream
        .get_user(&ctx, &id)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(user))
}

pub async fn list_roles(
    State(state): State<AppState>,
    Query(_query): Query<ListGovernanceQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<Role>>, BffError> {
    let roles = state
        .upstream
        .list_roles(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(roles))
}

pub async fn list_quotas(
    State(state): State<AppState>,
    Query(query): Query<ListQuotasQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<ProjectQuota>>, BffError> {
    let quotas = state
        .upstream
        .list_quotas(&ctx, query.project_id.as_deref())
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(quotas))
}

pub async fn list_audit_events(
    State(state): State<AppState>,
    Query(query): Query<ListAuditEventsQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<AuditEvent>>, BffError> {
    let params = ListAuditEventsParams {
        page: query.page,
        page_size: query.page_size,
        project_id: query.project_id,
        action: query.action,
        actor: query.actor,
        since: query.since,
        until: query.until,
    };
    let events = state
        .upstream
        .list_audit_events(&ctx, params)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(events))
}

pub async fn list_api_credentials(
    State(state): State<AppState>,
    Query(_query): Query<ListGovernanceQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<ApiCredential>>, BffError> {
    let credentials = state
        .upstream
        .list_api_credentials(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(credentials))
}

pub async fn create_api_credential(
    State(state): State<AppState>,
    ctx: RequestContext,
    request: Result<Json<CreateApiCredentialRequest>, JsonRejection>,
) -> Result<Json<ApiCredential>, BffError> {
    let Json(request) = request.map_err(|e| with_ctx(e, &ctx))?;
    let credential = state
        .upstream
        .create_api_credential(&ctx, request)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(credential))
}

pub async fn delete_api_credential(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ctx: RequestContext,
) -> Result<StatusCode, BffError> {
    state
        .upstream
        .delete_api_credential(&ctx, &id)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOperatorOperationsQuery {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub state: Option<OperationState>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub region_id: Option<String>,
    pub account_id: Option<String>,
    pub since: Option<time::OffsetDateTime>,
    pub until: Option<time::OffsetDateTime>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOperatorAuditEventsQuery {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub action: Option<String>,
    pub actor: Option<String>,
    pub account_id: Option<String>,
    pub since: Option<time::OffsetDateTime>,
    pub until: Option<time::OffsetDateTime>,
}

pub async fn get_platform_overview(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> Result<Json<PlatformOverview>, BffError> {
    let overview = state
        .upstream
        .get_platform_overview(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(overview))
}

pub async fn list_regions(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> Result<Json<Vec<Region>>, BffError> {
    let regions = state
        .upstream
        .list_regions(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(regions))
}

pub async fn list_availability_zones(
    State(state): State<AppState>,
    Path(region_id): Path<String>,
    ctx: RequestContext,
) -> Result<Json<Vec<crate::model::AvailabilityZone>>, BffError> {
    let zones = state
        .upstream
        .list_availability_zones(&ctx, &region_id)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(zones))
}

pub async fn list_provider_health(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> Result<Json<Vec<ProviderHealth>>, BffError> {
    let health = state
        .upstream
        .list_provider_health(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(health))
}

pub async fn list_service_health(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> Result<Json<Vec<ServiceHealth>>, BffError> {
    let health = state
        .upstream
        .list_service_health(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(health))
}

pub async fn get_capacity_summary(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> Result<Json<Vec<CapacitySummary>>, BffError> {
    let summary = state
        .upstream
        .get_capacity_summary(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(summary))
}

pub async fn list_customer_accounts(
    State(state): State<AppState>,
    Query(_query): Query<ListGovernanceQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<CustomerAccount>>, BffError> {
    let accounts = state
        .upstream
        .list_customer_accounts(&ctx)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(accounts))
}

pub async fn list_operator_projects(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<OperatorProject>>, BffError> {
    let projects = state
        .upstream
        .list_operator_projects(&ctx, Some(&account_id))
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(projects))
}

pub async fn list_operator_operations(
    State(state): State<AppState>,
    Query(query): Query<ListOperatorOperationsQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<Operation>>, BffError> {
    let params = ListOperatorOperationsParams {
        page: query.page,
        page_size: query.page_size,
        state: query.state,
        action: query.action,
        resource_type: query.resource_type,
        region_id: query.region_id,
        account_id: query.account_id,
        since: query.since,
        until: query.until,
    };
    let operations = state
        .upstream
        .list_operator_operations(&ctx, params)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(operations))
}

pub async fn list_operator_audit_events(
    State(state): State<AppState>,
    Query(query): Query<ListOperatorAuditEventsQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<OperatorAuditEvent>>, BffError> {
    let params = ListOperatorAuditEventsParams {
        page: query.page,
        page_size: query.page_size,
        action: query.action,
        actor: query.actor,
        account_id: query.account_id,
        since: query.since,
        until: query.until,
    };
    let events = state
        .upstream
        .list_operator_audit_events(&ctx, params)
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(events))
}
