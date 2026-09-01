//! Upstream adapter boundary.
//!
//! The BFF never calls O3K directly from handlers. Instead handlers depend on
//! an `Upstream` trait that is implemented by the production adapter (M7+)
//! and by the deterministic fixture adapter (M3-M6).

use std::collections::HashMap;

use async_trait::async_trait;

use time::OffsetDateTime;

use crate::{
    error::ApiError,
    model::{
        ActionRequest, ApiCredential, AuditEvent, CreateApiCredentialRequest,
        CreateResourceRequest, ListAuditEventsParams, Operation, OperationState,
        PaginatedCollection, Project, ProjectMember, ProjectQuota, Resource, Role,
        ServiceDescriptor, SessionContext, SortDirection, User,
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

/// Parameters for listing operations.
///
/// Operation collections have their own filter surface (state, action, scope,
/// time bounds) so they are modelled separately from `ListResourcesParams`.
#[derive(Clone, Debug, Default)]
pub struct ListOperationsParams {
    pub page: u32,
    pub page_size: u32,
    pub state: Option<OperationState>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub project_id: Option<String>,
    pub region_id: Option<String>,
    pub since: Option<OffsetDateTime>,
    pub until: Option<OffsetDateTime>,
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
        params: ListOperationsParams,
    ) -> Result<PaginatedCollection<Operation>, ApiError>;

    /// Get a single Operation by id.
    async fn get_operation(&self, ctx: &RequestContext, id: &str) -> Result<Operation, ApiError>;

    /// List projects visible to the caller.
    async fn list_projects(
        &self,
        _ctx: &RequestContext,
    ) -> Result<PaginatedCollection<Project>, ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance project endpoints are not implemented by upstream O3K".to_owned(),
        ))
    }

    /// Get a single project by id.
    async fn get_project(&self, _ctx: &RequestContext, _id: &str) -> Result<Project, ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance project endpoints are not implemented by upstream O3K".to_owned(),
        ))
    }

    /// List members of a project.
    async fn list_project_members(
        &self,
        _ctx: &RequestContext,
        _id: &str,
    ) -> Result<Vec<ProjectMember>, ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance project membership endpoints are not implemented by upstream O3K"
                .to_owned(),
        ))
    }

    /// List users visible to the caller.
    async fn list_users(
        &self,
        _ctx: &RequestContext,
    ) -> Result<PaginatedCollection<User>, ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance user endpoints are not implemented by upstream O3K".to_owned(),
        ))
    }

    /// Get a single user by id.
    async fn get_user(&self, _ctx: &RequestContext, _id: &str) -> Result<User, ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance user endpoints are not implemented by upstream O3K".to_owned(),
        ))
    }

    /// List roles available for project membership.
    async fn list_roles(
        &self,
        _ctx: &RequestContext,
    ) -> Result<PaginatedCollection<Role>, ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance role endpoints are not implemented by upstream O3K".to_owned(),
        ))
    }

    /// List quota/usage entries for the authorized scope.
    async fn list_quotas(
        &self,
        _ctx: &RequestContext,
        _project_id: Option<&str>,
    ) -> Result<PaginatedCollection<ProjectQuota>, ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance quota endpoints are not implemented by upstream O3K".to_owned(),
        ))
    }

    /// List audit events for the authorized scope.
    async fn list_audit_events(
        &self,
        _ctx: &RequestContext,
        _params: ListAuditEventsParams,
    ) -> Result<PaginatedCollection<AuditEvent>, ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance audit endpoints are not implemented by upstream O3K".to_owned(),
        ))
    }

    /// List API credentials visible to the caller.
    async fn list_api_credentials(
        &self,
        _ctx: &RequestContext,
    ) -> Result<PaginatedCollection<ApiCredential>, ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance API credential endpoints are not implemented by upstream O3K"
                .to_owned(),
        ))
    }

    /// Create a new API credential.
    async fn create_api_credential(
        &self,
        _ctx: &RequestContext,
        _request: CreateApiCredentialRequest,
    ) -> Result<ApiCredential, ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance API credential endpoints are not implemented by upstream O3K"
                .to_owned(),
        ))
    }

    /// Delete an API credential by id.
    async fn delete_api_credential(
        &self,
        _ctx: &RequestContext,
        _id: &str,
    ) -> Result<(), ApiError> {
        Err(ApiError::NotImplemented(
            "tenant governance API credential endpoints are not implemented by upstream O3K"
                .to_owned(),
        ))
    }
}
