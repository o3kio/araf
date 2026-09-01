//! Shared BFF HTTP handlers.

use std::sync::Arc;

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
        ActionRequest, Operation, PaginatedCollection, Resource, ServiceDescriptor, SessionContext,
    },
    request::RequestContext,
    upstream::Upstream,
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
}

fn default_page_size() -> u32 {
    25
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
    let collection = state
        .upstream
        .list_resources(
            &ctx,
            &resource_type,
            query.page,
            query.page_size,
            query.project_id.as_deref(),
            query.region_id.as_deref(),
        )
        .await
        .map_err(|e| with_ctx(e, &ctx))?;
    Ok(Json(collection))
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
    Query(query): Query<ListResourcesQuery>,
    ctx: RequestContext,
) -> Result<Json<PaginatedCollection<Operation>>, BffError> {
    let operations = state
        .upstream
        .list_operations(&ctx, query.page, query.page_size)
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
