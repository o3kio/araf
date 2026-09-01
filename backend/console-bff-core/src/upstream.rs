//! Upstream adapter boundary.
//!
//! The BFF never calls O3K directly from handlers. Instead handlers depend on
//! an `Upstream` trait that is implemented by the production adapter (M7+)
//! and by the deterministic fixture adapter (M3-M6).

use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    error::ApiError,
    model::{
        ActionRequest, CreateResourceRequest, Operation, PaginatedCollection, Resource,
        ServiceDescriptor, SessionContext, SortDirection,
    },
    request::RequestContext,
};

/// Parameters for listing resources.
///
/// Bundles pagination, common scope filters, and resource-type-specific
/// filters so the upstream trait method stays narrow.
#[derive(Clone, Debug, Default)]
pub struct ListResourcesParams {
    pub page: u32,
    pub page_size: u32,
    pub project_id: Option<String>,
    pub region_id: Option<String>,
    pub filters: HashMap<String, String>,
    pub sort_field: Option<String>,
    pub sort_direction: SortDirection,
}

/// Abstract upstream dependency for BFF handlers.
///
/// Production implementations call the O3K native API. Fixture implementations
/// return deterministic synthetic data. The trait is kept narrow: it contains
/// only the contracts the console runtime needs for the prototype/MVP.
#[async_trait]
pub trait Upstream: Send + Sync + 'static {
    /// Surface this adapter serves (e.g. `tenant-bff`, `operator-bff`).
    fn surface(&self) -> &'static str;

    /// Current session context and capabilities.
    async fn context(&self, ctx: &RequestContext) -> Result<SessionContext, ApiError>;

    /// Discover service/resource descriptors available to this session.
    async fn services(&self, ctx: &RequestContext) -> Result<Vec<ServiceDescriptor>, ApiError>;

    /// List resources with server-bounded pagination.
    async fn list_resources(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        params: ListResourcesParams,
    ) -> Result<PaginatedCollection<Resource>, ApiError>;

    /// Get a single resource by id.
    async fn get_resource(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        id: &str,
    ) -> Result<Resource, ApiError>;

    /// Submit an action and return the canonical Operation.
    async fn submit_action(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        id: &str,
        request: ActionRequest,
    ) -> Result<Operation, ApiError>;

    /// Create a new resource and return the canonical Operation.
    async fn create_resource(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        request: CreateResourceRequest,
    ) -> Result<Operation, ApiError>;

    /// List Operations.
    async fn list_operations(
        &self,
        ctx: &RequestContext,
        page: u32,
        page_size: u32,
    ) -> Result<PaginatedCollection<Operation>, ApiError>;

    /// Get a single Operation by id.
    async fn get_operation(&self, ctx: &RequestContext, id: &str) -> Result<Operation, ApiError>;
}
