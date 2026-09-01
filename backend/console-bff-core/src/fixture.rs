//! Deterministic fixture upstream adapter.
//!
//! This adapter intentionally does not call O3K. It exists only for prototype
//! development and must be explicitly selected by the BFF binary. It is
//! impossible to confuse with a production adapter because it implements
//! `Upstream` but has no O3K client, URLs, or credentials.

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::{
    error::ApiError,
    model::{
        ActionDescriptor, ActionRequest, ActionRiskClass, Capability, ColumnDescriptor,
        CreateResourceRequest, DetailsSectionDescriptor, FilterDescriptor, FilterKind, JsonSchema,
        Operation, OperationError, OperationState, PaginatedCollection, RelationshipDescriptor,
        RelationshipDirection, Resource, ResourceStatus, ResourceTypeDescriptor, ServiceDescriptor,
        SessionContext, SortDirection,
    },
    request::RequestContext,
    upstream::{ListResourcesParams, Upstream},
};

/// Total number of synthetic resources in the fixture universe.
pub const FIXTURE_RESOURCE_TOTAL: u64 = 100_000;

/// Total number of synthetic VPC resources.
pub const FIXTURE_VPC_TOTAL: u64 = 1_000;

/// Total number of synthetic volume resources.
pub const FIXTURE_VOLUME_TOTAL: u64 = 5_000;

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

    /// Total number of fixtures for a supported resource type.
    fn total_for(resource_type: &str) -> Option<u64> {
        match resource_type {
            "compute.server" => Some(FIXTURE_RESOURCE_TOTAL),
            "network.vpc" => Some(FIXTURE_VPC_TOTAL),
            "storage.volume" => Some(FIXTURE_VOLUME_TOTAL),
            _ => None,
        }
    }

    fn compute_server_at(id: u64) -> Resource {
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
            properties: None,
        }
    }

    fn network_vpc_at(id: u64) -> Resource {
        let seed = Self::seed_from_id(id ^ 0x1234_5678_9abc_def0);
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
        let third = (seed % 254) + 1;
        let fourth = (seed >> 8) % 256;

        let mut properties = HashMap::new();
        properties.insert(
            "cidrBlock".to_string(),
            serde_json::Value::String(format!("10.0.{third}.{fourth}/24")),
        );

        Resource {
            id: format!("vpc-{id:07}"),
            name: format!("fixture-vpc-{id}"),
            resource_type: "network.vpc".to_string(),
            project_id,
            region_id: region.to_string(),
            status,
            created_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
            updated_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64 + 60),
            properties: Some(properties),
        }
    }

    fn storage_volume_at(id: u64) -> Resource {
        // Use the same seed as compute.server so that the same id maps to the
        // same fixture project/region. This keeps the prototype fixture universe
        // consistent for end-to-end tests that rely on a shared scope.
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
        let size_gb = 10 + (seed % 990);
        // Deterministically reference a valid compute.server id.
        let attached_server_id = format!("resource-{:010}", id % FIXTURE_RESOURCE_TOTAL);

        let mut properties = HashMap::new();
        properties.insert(
            "sizeGb".to_string(),
            serde_json::Value::Number(serde_json::Number::from(size_gb)),
        );
        properties.insert(
            "attachedServerId".to_string(),
            serde_json::Value::String(attached_server_id.clone()),
        );

        Resource {
            id: format!("volume-{id:08}"),
            name: format!("fixture-volume-{id}"),
            resource_type: "storage.volume".to_string(),
            project_id,
            region_id: region.to_string(),
            status,
            created_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
            updated_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64 + 60),
            properties: Some(properties),
        }
    }

    fn resource_at(resource_type: &str, id: u64) -> Option<Resource> {
        match resource_type {
            "compute.server" => Some(Self::compute_server_at(id)),
            "network.vpc" => Some(Self::network_vpc_at(id)),
            "storage.volume" => Some(Self::storage_volume_at(id)),
            _ => None,
        }
    }

    fn field_value(resource: &Resource, field: &str) -> String {
        match field {
            "id" => resource.id.clone(),
            "name" => resource.name.clone(),
            "status" => format!("{:?}", resource.status),
            "projectId" => resource.project_id.clone(),
            "regionId" => resource.region_id.clone(),
            "createdAt" => resource.created_at.to_string(),
            "updatedAt" => resource.updated_at.to_string(),
            _ if field.starts_with("properties.") => {
                let key = field.strip_prefix("properties.").unwrap_or(field);
                resource
                    .properties
                    .as_ref()
                    .and_then(|p| p.get(key).map(|v| v.to_string()))
                    .unwrap_or_default()
            }
            _ => String::new(),
        }
    }

    fn sort_resources(items: &mut [Resource], field: &str, direction: SortDirection) {
        items.sort_by(|a, b| {
            let a_val = Self::field_value(a, field);
            let b_val = Self::field_value(b, field);
            let cmp = a_val.cmp(&b_val);
            if direction == SortDirection::Desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    fn parse_id(resource_type: &str, id: &str) -> Option<u64> {
        match resource_type {
            "compute.server" => id
                .strip_prefix("resource-")
                .and_then(|s| s.parse::<u64>().ok()),
            "network.vpc" => id.strip_prefix("vpc-").and_then(|s| s.parse::<u64>().ok()),
            "storage.volume" => id
                .strip_prefix("volume-")
                .and_then(|s| s.parse::<u64>().ok()),
            _ => None,
        }
    }

    fn descriptor_for(resource_type: &str) -> Option<ResourceTypeDescriptor> {
        match resource_type {
            "compute.server" => Some(Self::compute_server_descriptor()),
            "network.vpc" => Some(Self::network_vpc_descriptor()),
            "storage.volume" => Some(Self::storage_volume_descriptor()),
            _ => None,
        }
    }

    fn validate_payload(schema: &JsonSchema, payload: &serde_json::Value) -> Result<(), ApiError> {
        let validator = jsonschema::validator_for(schema.as_value())
            .map_err(|e| ApiError::BadRequest(format!("invalid schema: {e}")))?;
        let errors: Vec<String> = validator
            .iter_errors(payload)
            .map(|e| e.to_string())
            .collect();
        if !errors.is_empty() {
            return Err(ApiError::BadRequest(format!(
                "validation failed: {}",
                errors.join("; ")
            )));
        }
        Ok(())
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

    fn compute_server_descriptor() -> ResourceTypeDescriptor {
        ResourceTypeDescriptor {
            id: "compute.server".to_string(),
            name: "Server".to_string(),
            plural_name: "Servers".to_string(),
            icon_token: "server".to_string(),
            create_schema: Some(JsonSchema(serde_json::json!({
                "type": "object",
                "required": ["name", "regionId", "projectId"],
                "properties": {
                    "name": { "type": "string", "minLength": 1 },
                    "regionId": { "enum": ["eu-west", "us-east", "ap-south"] },
                    "projectId": { "enum": ["project-1", "project-2", "project-3", "project-4", "project-5"] },
                    "bootVolumeSizeGb": { "type": "number", "minimum": 10 }
                }
            }))),
            create_capability: Capability {
                resource_type: "compute.server".to_string(),
                action: "create".to_string(),
            },
            supported_actions: vec![
                ActionDescriptor {
                    id: "start".to_string(),
                    name: "Start".to_string(),
                    requires_confirmation: false,
                    risk_class: ActionRiskClass::Normal,
                    required_capability: Capability {
                        resource_type: "compute.server".to_string(),
                        action: "start".to_string(),
                    },
                    input_schema: None,
                },
                ActionDescriptor {
                    id: "stop".to_string(),
                    name: "Stop".to_string(),
                    requires_confirmation: true,
                    risk_class: ActionRiskClass::Disruptive,
                    required_capability: Capability {
                        resource_type: "compute.server".to_string(),
                        action: "stop".to_string(),
                    },
                    input_schema: None,
                },
                ActionDescriptor {
                    id: "delete".to_string(),
                    name: "Delete".to_string(),
                    requires_confirmation: true,
                    risk_class: ActionRiskClass::Destructive,
                    required_capability: Capability {
                        resource_type: "compute.server".to_string(),
                        action: "delete".to_string(),
                    },
                    input_schema: None,
                },
            ],
            columns: vec![
                ColumnDescriptor {
                    id: "name".to_string(),
                    header: "Name".to_string(),
                    field: "name".to_string(),
                    width: None,
                },
                ColumnDescriptor {
                    id: "status".to_string(),
                    header: "Status".to_string(),
                    field: "status".to_string(),
                    width: Some("120px".to_string()),
                },
                ColumnDescriptor {
                    id: "region".to_string(),
                    header: "Region".to_string(),
                    field: "regionId".to_string(),
                    width: Some("140px".to_string()),
                },
                ColumnDescriptor {
                    id: "project".to_string(),
                    header: "Project".to_string(),
                    field: "projectId".to_string(),
                    width: Some("140px".to_string()),
                },
            ],
            filters: vec![
                FilterDescriptor {
                    id: "project".to_string(),
                    label: "Project".to_string(),
                    field: "projectId".to_string(),
                    kind: FilterKind::Select,
                },
                FilterDescriptor {
                    id: "region".to_string(),
                    label: "Region".to_string(),
                    field: "regionId".to_string(),
                    kind: FilterKind::Select,
                },
            ],
            sortable_fields: vec![
                "name".to_string(),
                "status".to_string(),
                "createdAt".to_string(),
                "updatedAt".to_string(),
            ],
            details_sections: vec![DetailsSectionDescriptor {
                id: "overview".to_string(),
                label: "Overview".to_string(),
                fields: vec![
                    "id".to_string(),
                    "name".to_string(),
                    "status".to_string(),
                    "projectId".to_string(),
                    "regionId".to_string(),
                    "createdAt".to_string(),
                    "updatedAt".to_string(),
                ],
            }],
            relationships: vec![],
        }
    }

    fn network_vpc_descriptor() -> ResourceTypeDescriptor {
        ResourceTypeDescriptor {
            id: "network.vpc".to_string(),
            name: "VPC".to_string(),
            plural_name: "VPCs".to_string(),
            icon_token: "network".to_string(),
            create_schema: Some(JsonSchema(serde_json::json!({
                "type": "object",
                "required": ["name", "regionId", "projectId", "cidrBlock"],
                "properties": {
                    "name": { "type": "string", "minLength": 1 },
                    "regionId": { "enum": ["eu-west", "us-east", "ap-south"] },
                    "projectId": { "enum": ["project-1", "project-2", "project-3", "project-4", "project-5"] },
                    "cidrBlock": { "type": "string", "minLength": 1 }
                }
            }))),
            create_capability: Capability {
                resource_type: "network.vpc".to_string(),
                action: "create".to_string(),
            },
            supported_actions: vec![ActionDescriptor {
                id: "delete".to_string(),
                name: "Delete".to_string(),
                requires_confirmation: true,
                risk_class: ActionRiskClass::Destructive,
                required_capability: Capability {
                    resource_type: "network.vpc".to_string(),
                    action: "delete".to_string(),
                },
                input_schema: None,
            }],
            columns: vec![
                ColumnDescriptor {
                    id: "name".to_string(),
                    header: "Name".to_string(),
                    field: "name".to_string(),
                    width: None,
                },
                ColumnDescriptor {
                    id: "cidr".to_string(),
                    header: "CIDR".to_string(),
                    field: "properties.cidrBlock".to_string(),
                    width: Some("160px".to_string()),
                },
                ColumnDescriptor {
                    id: "status".to_string(),
                    header: "Status".to_string(),
                    field: "status".to_string(),
                    width: Some("120px".to_string()),
                },
                ColumnDescriptor {
                    id: "region".to_string(),
                    header: "Region".to_string(),
                    field: "regionId".to_string(),
                    width: Some("140px".to_string()),
                },
            ],
            filters: vec![
                FilterDescriptor {
                    id: "project".to_string(),
                    label: "Project".to_string(),
                    field: "projectId".to_string(),
                    kind: FilterKind::Select,
                },
                FilterDescriptor {
                    id: "region".to_string(),
                    label: "Region".to_string(),
                    field: "regionId".to_string(),
                    kind: FilterKind::Select,
                },
            ],
            sortable_fields: vec![
                "name".to_string(),
                "status".to_string(),
                "createdAt".to_string(),
            ],
            details_sections: vec![DetailsSectionDescriptor {
                id: "overview".to_string(),
                label: "Overview".to_string(),
                fields: vec![
                    "id".to_string(),
                    "name".to_string(),
                    "status".to_string(),
                    "projectId".to_string(),
                    "regionId".to_string(),
                    "properties.cidrBlock".to_string(),
                    "createdAt".to_string(),
                    "updatedAt".to_string(),
                ],
            }],
            relationships: vec![],
        }
    }

    fn storage_volume_descriptor() -> ResourceTypeDescriptor {
        ResourceTypeDescriptor {
            id: "storage.volume".to_string(),
            name: "Volume".to_string(),
            plural_name: "Volumes".to_string(),
            icon_token: "storage".to_string(),
            create_schema: Some(JsonSchema(serde_json::json!({
                "type": "object",
                "required": ["name", "regionId", "projectId", "sizeGb"],
                "properties": {
                    "name": { "type": "string", "minLength": 1 },
                    "regionId": { "enum": ["eu-west", "us-east", "ap-south"] },
                    "projectId": { "enum": ["project-1", "project-2", "project-3", "project-4", "project-5"] },
                    "sizeGb": { "type": "number", "minimum": 10 }
                }
            }))),
            create_capability: Capability {
                resource_type: "storage.volume".to_string(),
                action: "create".to_string(),
            },
            supported_actions: vec![
                ActionDescriptor {
                    id: "attach".to_string(),
                    name: "Attach".to_string(),
                    requires_confirmation: false,
                    risk_class: ActionRiskClass::Normal,
                    required_capability: Capability {
                        resource_type: "storage.volume".to_string(),
                        action: "attach".to_string(),
                    },
                    input_schema: Some(JsonSchema(serde_json::json!({
                        "type": "object",
                        "required": ["serverId"],
                        "properties": {
                            "serverId": { "type": "string", "minLength": 1 }
                        }
                    }))),
                },
                ActionDescriptor {
                    id: "detach".to_string(),
                    name: "Detach".to_string(),
                    requires_confirmation: true,
                    risk_class: ActionRiskClass::Disruptive,
                    required_capability: Capability {
                        resource_type: "storage.volume".to_string(),
                        action: "detach".to_string(),
                    },
                    input_schema: None,
                },
                ActionDescriptor {
                    id: "delete".to_string(),
                    name: "Delete".to_string(),
                    requires_confirmation: true,
                    risk_class: ActionRiskClass::Destructive,
                    required_capability: Capability {
                        resource_type: "storage.volume".to_string(),
                        action: "delete".to_string(),
                    },
                    input_schema: None,
                },
            ],
            columns: vec![
                ColumnDescriptor {
                    id: "name".to_string(),
                    header: "Name".to_string(),
                    field: "name".to_string(),
                    width: None,
                },
                ColumnDescriptor {
                    id: "size".to_string(),
                    header: "Size (GB)".to_string(),
                    field: "properties.sizeGb".to_string(),
                    width: Some("120px".to_string()),
                },
                ColumnDescriptor {
                    id: "attachedServer".to_string(),
                    header: "Attached Server".to_string(),
                    field: "properties.attachedServerId".to_string(),
                    width: Some("200px".to_string()),
                },
                ColumnDescriptor {
                    id: "status".to_string(),
                    header: "Status".to_string(),
                    field: "status".to_string(),
                    width: Some("120px".to_string()),
                },
                ColumnDescriptor {
                    id: "region".to_string(),
                    header: "Region".to_string(),
                    field: "regionId".to_string(),
                    width: Some("140px".to_string()),
                },
            ],
            filters: vec![
                FilterDescriptor {
                    id: "project".to_string(),
                    label: "Project".to_string(),
                    field: "projectId".to_string(),
                    kind: FilterKind::Select,
                },
                FilterDescriptor {
                    id: "region".to_string(),
                    label: "Region".to_string(),
                    field: "regionId".to_string(),
                    kind: FilterKind::Select,
                },
                FilterDescriptor {
                    id: "attachedServer".to_string(),
                    label: "Attached Server".to_string(),
                    field: "attachedServerId".to_string(),
                    kind: FilterKind::Select,
                },
            ],
            sortable_fields: vec![
                "name".to_string(),
                "status".to_string(),
                "createdAt".to_string(),
            ],
            details_sections: vec![DetailsSectionDescriptor {
                id: "overview".to_string(),
                label: "Overview".to_string(),
                fields: vec![
                    "id".to_string(),
                    "name".to_string(),
                    "status".to_string(),
                    "projectId".to_string(),
                    "regionId".to_string(),
                    "properties.sizeGb".to_string(),
                    "properties.attachedServerId".to_string(),
                    "createdAt".to_string(),
                    "updatedAt".to_string(),
                ],
            }],
            relationships: vec![RelationshipDescriptor {
                id: "attachedServer".to_string(),
                target_resource_type: "compute.server".to_string(),
                label: "Attached Server".to_string(),
                source_property_key: "attachedServerId".to_string(),
                direction: RelationshipDirection::ToOne,
            }],
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
        Ok(vec![
            ServiceDescriptor {
                id: "compute".to_string(),
                name: "Compute".to_string(),
                category: "Services".to_string(),
                resource_types: vec![Self::compute_server_descriptor()],
            },
            ServiceDescriptor {
                id: "network".to_string(),
                name: "Networking".to_string(),
                category: "Services".to_string(),
                resource_types: vec![Self::network_vpc_descriptor()],
            },
            ServiceDescriptor {
                id: "storage".to_string(),
                name: "Storage".to_string(),
                category: "Services".to_string(),
                resource_types: vec![Self::storage_volume_descriptor()],
            },
        ])
    }

    async fn list_resources(
        &self,
        _ctx: &RequestContext,
        resource_type: &str,
        params: ListResourcesParams,
    ) -> Result<PaginatedCollection<Resource>, ApiError> {
        let total = Self::total_for(resource_type).ok_or(ApiError::NotFound)?;
        let page_size = params.page_size.clamp(1, 100);
        let offset = (params.page as u64).saturating_mul(page_size as u64);

        if offset >= total {
            return Ok(PaginatedCollection {
                items: vec![],
                total,
                page: params.page,
                page_size,
                has_more: false,
            });
        }

        let end = (offset + page_size as u64).min(total);
        let mut items: Vec<Resource> = (offset..end)
            .filter_map(|id| Self::resource_at(resource_type, id))
            .collect();

        // Server-side filtering on project/region. These are not indexes; they
        // are deterministic scans over the bounded page to prove the concept.
        if let Some(project) = params.project_id {
            items.retain(|r| r.project_id == project);
        }
        if let Some(region) = params.region_id {
            items.retain(|r| r.region_id == region);
        }

        // Resource-type-specific filters.
        if resource_type == "storage.volume" {
            if let Some(server_id) = params.filters.get("attached_server_id") {
                items.retain(|r| {
                    r.properties
                        .as_ref()
                        .and_then(|p| p.get("attachedServerId"))
                        .and_then(|v| v.as_str())
                        == Some(server_id)
                });
            }
        }

        if let Some(sort_field) = params.sort_field {
            Self::sort_resources(&mut items, &sort_field, params.sort_direction);
        }

        Ok(PaginatedCollection {
            items,
            total,
            page: params.page,
            page_size,
            has_more: end < total,
        })
    }

    async fn get_resource(
        &self,
        _ctx: &RequestContext,
        resource_type: &str,
        id: &str,
    ) -> Result<Resource, ApiError> {
        let total = Self::total_for(resource_type).ok_or(ApiError::NotFound)?;
        let numeric_id = Self::parse_id(resource_type, id)
            .filter(|n| *n < total)
            .ok_or(ApiError::NotFound)?;

        Self::resource_at(resource_type, numeric_id).ok_or(ApiError::NotFound)
    }

    async fn submit_action(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        id: &str,
        request: ActionRequest,
    ) -> Result<Operation, ApiError> {
        // Ensure the resource exists before accepting an action.
        let resource = self.get_resource(ctx, resource_type, id).await?;
        let session = self.context(ctx).await?;

        let descriptor = Self::descriptor_for(resource_type)
            .ok_or(ApiError::BadRequest("invalid action id".to_string()))?;
        let action = descriptor
            .supported_actions
            .into_iter()
            .find(|a| a.id == request.action_id)
            .ok_or(ApiError::BadRequest("invalid action id".to_string()))?;

        if !session.has_capability(
            &action.required_capability.resource_type,
            &action.required_capability.action,
        ) {
            return Err(ApiError::Forbidden);
        }

        if let Some(schema) = &action.input_schema {
            let payload = request
                .payload
                .as_ref()
                .ok_or(ApiError::BadRequest("action requires payload".to_string()))?;
            Self::validate_payload(schema, payload)?;
        }

        let seed = Self::seed_from_id(
            Self::parse_id(resource_type, id)
                .map(|n| n % FIXTURE_RESOURCE_TOTAL)
                .unwrap_or(0),
        );
        let mut op = Self::operation_at(seed % FIXTURE_RESOURCE_TOTAL);
        op.action = request.action_id;
        op.resource_id = Some(id.to_string());
        op.resource_type = Some(resource_type.to_string());
        op.project_id = Some(resource.project_id);
        op.region_id = Some(resource.region_id);
        op.state = OperationState::Pending;
        op.correlation_id = ctx.correlation_id().to_string();
        Ok(op)
    }

    async fn create_resource(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        request: CreateResourceRequest,
    ) -> Result<Operation, ApiError> {
        let session = self.context(ctx).await?;
        let descriptor = Self::descriptor_for(resource_type).ok_or(ApiError::NotFound)?;

        let schema = descriptor
            .create_schema
            .as_ref()
            .ok_or(ApiError::BadRequest(
                "create not supported for resource type".to_string(),
            ))?;

        if !session.has_capability(
            &descriptor.create_capability.resource_type,
            &descriptor.create_capability.action,
        ) {
            return Err(ApiError::Forbidden);
        }

        Self::validate_payload(schema, &request.payload)?;

        let payload_str =
            serde_json::to_string(&request.payload).map_err(|_| ApiError::Internal)?;
        let mut hasher = DefaultHasher::new();
        payload_str.hash(&mut hasher);
        let numeric_id = hasher.finish();
        let resource_id = format!("resource-{numeric_id:010}");
        let now = OffsetDateTime::now_utc();

        Ok(Operation {
            id: format!("op-{resource_id}"),
            action: "create".to_string(),
            state: OperationState::Pending,
            resource_id: Some(resource_id),
            resource_type: Some(resource_type.to_string()),
            project_id: request
                .payload
                .get("projectId")
                .and_then(|v| v.as_str())
                .map(String::from),
            region_id: request
                .payload
                .get("regionId")
                .and_then(|v| v.as_str())
                .map(String::from),
            initiated_by: Some(session.user_id),
            started_at: Some(now),
            updated_at: Some(now),
            correlation_id: ctx.correlation_id().to_string(),
            error: None,
        })
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
        let mut capabilities = base.capabilities.clone();
        capabilities.extend([
            Capability {
                resource_type: "network.vpc".to_string(),
                action: "list".to_string(),
            },
            Capability {
                resource_type: "storage.volume".to_string(),
                action: "list".to_string(),
            },
            Capability {
                resource_type: "storage.volume".to_string(),
                action: "attach".to_string(),
            },
            Capability {
                resource_type: "storage.volume".to_string(),
                action: "delete".to_string(),
            },
        ]);

        if surface == "operator-bff" {
            capabilities.extend([
                Capability {
                    resource_type: "platform.overview".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "platform.health".to_string(),
                    action: "read".to_string(),
                },
            ]);
        }

        Self {
            surface,
            capabilities,
            ..base
        }
    }
}
