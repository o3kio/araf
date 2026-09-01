//! Deterministic fixture upstream adapter.
//!
//! This adapter intentionally does not call O3K. It exists only for prototype
//! development and must be explicitly selected by the BFF binary. It is
//! impossible to confuse with a production adapter because it implements
//! `Upstream` but has no O3K client, URLs, or credentials.

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::{
    error::ApiError,
    model::{
        ActionDescriptor, ActionRequest, Capability, Operation, OperationError, OperationState,
        PaginatedCollection, Resource, ResourceStatus, ResourceTypeDescriptor, ServiceDescriptor,
        SessionContext,
    },
    request::RequestContext,
    upstream::Upstream,
};

/// Total number of synthetic resources in the fixture universe.
pub const FIXTURE_RESOURCE_TOTAL: u64 = 100_000;

/// Fixture adapter configured for a specific BFF surface.
#[derive(Clone, Debug)]
pub struct FixtureAdapter {
    surface: &'static str,
}

impl FixtureAdapter {
    pub fn new(surface: &'static str) -> Self {
        Self { surface }
    }

    fn seed_from_id(id: u64) -> u64 {
        // Deterministic hash-like mixing for stable synthetic data.
        id.wrapping_mul(0x9e37_79b9_7f4a_7c15)
    }

    fn resource_at(id: u64) -> Resource {
        let seed = Self::seed_from_id(id);
        let regions = ["eu-west", "us-east", "ap-south"];
        let region = regions[(seed as usize) % regions.len()];
        let statuses = [
            ResourceStatus::Ready,
            ResourceStatus::Busy,
            ResourceStatus::Error,
            ResourceStatus::Unknown,
        ];
        let status = statuses[(seed as usize >> 8) % statuses.len()];
        let project_id = format!("project-{}", (seed % 5) + 1);

        Resource {
            id: format!("resource-{id:010}"),
            name: format!("fixture-server-{id}"),
            resource_type: "compute.server".to_string(),
            project_id,
            region_id: region.to_string(),
            status,
            created_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
            updated_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64 + 60),
        }
    }

    fn operation_at(id: u64) -> Operation {
        let seed = Self::seed_from_id(id);
        let states = [
            OperationState::Pending,
            OperationState::Running,
            OperationState::Succeeded,
            OperationState::Failed,
        ];
        let state = states[(seed as usize) % states.len()];
        let error = if state == OperationState::Failed {
            Some(OperationError {
                code: "fixture-failure".to_string(),
                title: "Fixture operation failed".to_string(),
                detail: "This is a deterministic failure for prototype testing.".to_string(),
            })
        } else {
            None
        };

        Operation {
            id: format!("op-{id:010}"),
            action: if seed % 2 == 0 { "create" } else { "delete" }.to_string(),
            state,
            resource_id: Some(format!("resource-{}", seed % FIXTURE_RESOURCE_TOTAL)),
            resource_type: Some("compute.server".to_string()),
            project_id: Some(format!("project-{}", (seed % 5) + 1)),
            region_id: Some("eu-west".to_string()),
            initiated_by: Some("fixture-user".to_string()),
            started_at: Some(
                OffsetDateTime::UNIX_EPOCH + time::Duration::seconds((seed % 1_000_000) as i64),
            ),
            updated_at: Some(
                OffsetDateTime::UNIX_EPOCH
                    + time::Duration::seconds((seed % 1_000_000) as i64 + 60),
            ),
            correlation_id: format!("corr-{id}"),
            error,
        }
    }
}

#[async_trait]
impl Upstream for FixtureAdapter {
    fn surface(&self) -> &'static str {
        self.surface
    }

    async fn context(&self, _ctx: &RequestContext) -> Result<SessionContext, ApiError> {
        Ok(SessionContext::fixture_for_surface(self.surface))
    }

    async fn services(&self, _ctx: &RequestContext) -> Result<Vec<ServiceDescriptor>, ApiError> {
        Ok(vec![ServiceDescriptor {
            id: "compute".to_string(),
            name: "Compute".to_string(),
            category: "Services".to_string(),
            resource_types: vec![ResourceTypeDescriptor {
                id: "compute.server".to_string(),
                name: "Server".to_string(),
                plural_name: "Servers".to_string(),
                supported_actions: vec![
                    ActionDescriptor {
                        id: "start".to_string(),
                        name: "Start".to_string(),
                        requires_confirmation: false,
                    },
                    ActionDescriptor {
                        id: "stop".to_string(),
                        name: "Stop".to_string(),
                        requires_confirmation: true,
                    },
                    ActionDescriptor {
                        id: "delete".to_string(),
                        name: "Delete".to_string(),
                        requires_confirmation: true,
                    },
                ],
            }],
        }])
    }

    async fn list_resources(
        &self,
        _ctx: &RequestContext,
        resource_type: &str,
        page: u32,
        page_size: u32,
        project_id: Option<&str>,
        region_id: Option<&str>,
    ) -> Result<PaginatedCollection<Resource>, ApiError> {
        if resource_type != "compute.server" {
            return Err(ApiError::NotFound);
        }

        let page_size = page_size.clamp(1, 100);
        let offset = (page as u64).saturating_mul(page_size as u64);

        if offset >= FIXTURE_RESOURCE_TOTAL {
            return Ok(PaginatedCollection {
                items: vec![],
                total: FIXTURE_RESOURCE_TOTAL,
                page,
                page_size,
                has_more: false,
            });
        }

        let end = (offset + page_size as u64).min(FIXTURE_RESOURCE_TOTAL);
        let mut items: Vec<Resource> = (offset..end).map(Self::resource_at).collect();

        // Server-side filtering on project/region. These are not indexes; they
        // are deterministic scans over the bounded page to prove the concept.
        if let Some(project) = project_id {
            items.retain(|r| r.project_id == project);
        }
        if let Some(region) = region_id {
            items.retain(|r| r.region_id == region);
        }

        Ok(PaginatedCollection {
            items,
            total: FIXTURE_RESOURCE_TOTAL,
            page,
            page_size,
            has_more: end < FIXTURE_RESOURCE_TOTAL,
        })
    }

    async fn get_resource(
        &self,
        _ctx: &RequestContext,
        resource_type: &str,
        id: &str,
    ) -> Result<Resource, ApiError> {
        if resource_type != "compute.server" {
            return Err(ApiError::NotFound);
        }

        let numeric_id = id
            .strip_prefix("resource-")
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|n| *n < FIXTURE_RESOURCE_TOTAL)
            .ok_or(ApiError::NotFound)?;

        Ok(Self::resource_at(numeric_id))
    }

    async fn submit_action(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        id: &str,
        request: ActionRequest,
    ) -> Result<Operation, ApiError> {
        if resource_type != "compute.server" {
            return Err(ApiError::NotFound);
        }

        // Ensure the resource exists before accepting an action.
        let _resource = self.get_resource(ctx, resource_type, id).await?;

        let seed = Self::seed_from_id(
            id.strip_prefix("resource-")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0),
        );
        let mut op = Self::operation_at(seed % FIXTURE_RESOURCE_TOTAL);
        op.action = request.action_id;
        op.resource_id = Some(id.to_string());
        op.state = OperationState::Pending;
        op.correlation_id = ctx.correlation_id().to_string();
        Ok(op)
    }

    async fn list_operations(
        &self,
        _ctx: &RequestContext,
        page: u32,
        page_size: u32,
    ) -> Result<PaginatedCollection<Operation>, ApiError> {
        let page_size = page_size.clamp(1, 100);
        let total = 1_000_u64;
        let offset = (page as u64).saturating_mul(page_size as u64);

        if offset >= total {
            return Ok(PaginatedCollection {
                items: vec![],
                total,
                page,
                page_size,
                has_more: false,
            });
        }

        let end = (offset + page_size as u64).min(total);
        let items: Vec<Operation> = (offset..end).map(Self::operation_at).collect();

        Ok(PaginatedCollection {
            items,
            total,
            page,
            page_size,
            has_more: end < total,
        })
    }

    async fn get_operation(&self, _ctx: &RequestContext, id: &str) -> Result<Operation, ApiError> {
        let numeric_id = id
            .strip_prefix("op-")
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|n| *n < 1_000)
            .ok_or(ApiError::NotFound)?;

        Ok(Self::operation_at(numeric_id))
    }
}

impl SessionContext {
    /// Fixture context with capabilities appropriate to the surface.
    pub fn fixture_for_surface(surface: &'static str) -> Self {
        let base = Self::fixture(surface);
        let capabilities = if surface == "operator-bff" {
            let mut caps = base.capabilities.clone();
            caps.extend([
                Capability {
                    resource_type: "platform.overview".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "platform.health".to_string(),
                    action: "read".to_string(),
                },
            ]);
            caps
        } else {
            base.capabilities
        };

        Self {
            surface,
            capabilities,
            ..base
        }
    }
}
