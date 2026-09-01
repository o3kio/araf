//! Deterministic fixture upstream adapter.
//!
//! This adapter intentionally does not call O3K. It exists only for prototype
//! development and must be explicitly selected by the BFF binary. It is
//! impossible to confuse with a production adapter because it implements
//! `Upstream` but has no O3K client, URLs, or credentials.

use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::{
    error::ApiError,
    model::{
        ActionDescriptor, ActionRequest, ActionRiskClass, AlertSeverity, ApiCredential, AuditEvent,
        AvailabilityZone, Capability, CapacitySummary, ColumnDescriptor,
        CreateApiCredentialRequest, CreateResourceRequest, CustomerAccount,
        DetailsSectionDescriptor, DiscoveredResourceType, FilterDescriptor, FilterKind, JsonSchema,
        ListAuditEventsParams, Operation, OperationError, OperationEvent, OperationState,
        OperatorAuditEvent, OperatorProject, PaginatedCollection, PlatformAlert, PlatformOverview,
        Project, ProjectMember, ProjectQuota, ProviderHealth, ProviderKind, QuotaEntry, Region,
        RegionStatus, RelationshipDescriptor, RelationshipDirection, Resource, ResourceStatus,
        ResourceTypeDescriptor, Role, ServiceCatalogEntry, ServiceDescriptor, ServiceHealth,
        SessionContext, SortDirection, StatusCount, UsageQuery, UsageRecord, UsageSummary, User,
    },
    request::RequestContext,
    upstream::{
        ListOperationsParams, ListOperatorAuditEventsParams, ListOperatorOperationsParams,
        ListResourcesParams, Upstream,
    },
};

/// Total number of synthetic resources in the fixture universe.
pub const FIXTURE_RESOURCE_TOTAL: u64 = 100_000;

/// Total number of synthetic VPC resources.
pub const FIXTURE_VPC_TOTAL: u64 = 1_000;

/// Total number of synthetic volume resources.
pub const FIXTURE_VOLUME_TOTAL: u64 = 5_000;

/// Total number of synthetic object storage bucket resources.
pub const FIXTURE_OBJECT_STORAGE_BUCKET_TOTAL: u64 = 2_000;

/// Total number of synthetic users in the fixture tenant.
const FIXTURE_USER_TOTAL: u64 = 50;

/// Total number of synthetic roles in the fixture tenant.
const FIXTURE_ROLE_TOTAL: u64 = 4;

/// Total number of synthetic audit events in the fixture tenant.
const FIXTURE_AUDIT_TOTAL: u64 = 200;

/// Valid API credential kinds.
const API_CREDENTIAL_KINDS: &[&str] = &["service-account", "application-credential"];

/// Seconds before a Pending fixture operation transitions to Running.
const PENDING_TO_RUNNING_SECS: i64 = 1;

/// Seconds before a Running fixture operation transitions to Succeeded or Failed.
const RUNNING_TO_TERMINAL_SECS: i64 = 3;

/// In-memory operation store for the fixture adapter.
///
/// Stored operations survive page reloads and can be retrieved by id. They are
/// deliberately isolated to this adapter instance; the fixture does not claim
/// to be a persistent cloud control plane.
#[derive(Debug, Default)]
struct OperationStore {
    operations: Vec<Operation>,
    api_credentials: Vec<ApiCredential>,
}

/// Fixture adapter configured for a specific BFF surface.
#[derive(Clone)]
pub struct FixtureAdapter {
    surface: &'static str,
    store: Arc<Mutex<OperationStore>>,
}

impl std::fmt::Debug for FixtureAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FixtureAdapter")
            .field("surface", &self.surface)
            .finish_non_exhaustive()
    }
}

impl FixtureAdapter {
    pub fn new(surface: &'static str) -> Self {
        Self {
            surface,
            store: Arc::new(Mutex::new(OperationStore::default())),
        }
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
            "object.storage.bucket" => Some(FIXTURE_OBJECT_STORAGE_BUCKET_TOTAL),
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

    fn object_storage_bucket_at(id: u64) -> Resource {
        let seed = Self::seed_from_id(id ^ 0xabcd_ef01_2345_6789);
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
        let size_mb = 100 + (seed % 900);

        let mut properties = HashMap::new();
        properties.insert(
            "sizeMb".to_string(),
            serde_json::Value::Number(serde_json::Number::from(size_mb)),
        );
        properties.insert(
            "publicAccess".to_string(),
            serde_json::Value::Bool(seed % 3 == 0),
        );

        Resource {
            id: format!("bucket-{id:08}"),
            name: format!("fixture-bucket-{id}"),
            resource_type: "object.storage.bucket".to_string(),
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
            "object.storage.bucket" => Some(Self::object_storage_bucket_at(id)),
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
            "object.storage.bucket" => id
                .strip_prefix("bucket-")
                .and_then(|s| s.parse::<u64>().ok()),
            _ => None,
        }
    }

    fn descriptor_for(resource_type: &str) -> Option<ResourceTypeDescriptor> {
        match resource_type {
            "compute.server" => Some(Self::compute_server_descriptor()),
            "network.vpc" => Some(Self::network_vpc_descriptor()),
            "storage.volume" => Some(Self::storage_volume_descriptor()),
            "object.storage.bucket" => Some(Self::object_storage_bucket_descriptor()),
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
            events: vec![],
        }
    }

    /// Deterministically decide whether an operation id should eventually
    /// succeed or fail. The result is stable for a given id so tests can rely
    /// on it without mocking a cloud control plane.
    fn is_success_id(id: &str) -> bool {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        hasher.finish() % 2 == 0
    }

    /// Advance a single stored operation through its deterministic lifecycle.
    ///
    /// - Pending -> Running after `PENDING_TO_RUNNING_SECS`.
    /// - Running -> Succeeded/Failed after `RUNNING_TO_TERMINAL_SECS`.
    fn advance_operation(op: &mut Operation, now: OffsetDateTime) {
        let started_at = op.started_at.unwrap_or(now);
        let elapsed = now - started_at;

        match op.state {
            OperationState::Pending => {
                if elapsed >= time::Duration::seconds(PENDING_TO_RUNNING_SECS) {
                    op.state = OperationState::Running;
                    op.updated_at = Some(now);
                    op.events.push(OperationEvent {
                        id: format!("{}-running", op.id),
                        state: OperationState::Running,
                        occurred_at: now,
                        message: "Operation started running".to_string(),
                        correlation_id: op.correlation_id.clone(),
                    });
                }
            }
            OperationState::Running => {
                if elapsed >= time::Duration::seconds(RUNNING_TO_TERMINAL_SECS) {
                    let success = Self::is_success_id(&op.id);
                    if success {
                        op.state = OperationState::Succeeded;
                        op.updated_at = Some(now);
                        op.events.push(OperationEvent {
                            id: format!("{}-succeeded", op.id),
                            state: OperationState::Succeeded,
                            occurred_at: now,
                            message: "Operation completed successfully".to_string(),
                            correlation_id: op.correlation_id.clone(),
                        });
                    } else {
                        op.state = OperationState::Failed;
                        op.updated_at = Some(now);
                        op.error = Some(OperationError {
                            code: "fixture-failure".to_string(),
                            title: "Fixture operation failed".to_string(),
                            detail: "Deterministic failure for this operation id.".to_string(),
                        });
                        op.events.push(OperationEvent {
                            id: format!("{}-failed", op.id),
                            state: OperationState::Failed,
                            occurred_at: now,
                            message: "Operation failed deterministically".to_string(),
                            correlation_id: op.correlation_id.clone(),
                        });
                    }
                }
            }
            OperationState::Succeeded | OperationState::Failed => {}
            OperationState::Retryable | OperationState::UnknownOutcome => {}
        }
    }

    /// Advance every stored operation to its current deterministic state.
    fn advance_operations(&self, now: OffsetDateTime) {
        let mut store = self.store.lock().expect("fixture operation store poisoned");
        for op in &mut store.operations {
            Self::advance_operation(op, now);
        }
    }

    fn hash_id(id: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        hasher.finish()
    }

    fn accessible_project_ids(session: &SessionContext) -> Vec<String> {
        let mut ids: Vec<String> = (1..=5).map(|i| format!("project-{i}")).collect();
        if let Some(pid) = &session.project_id {
            if !ids.contains(pid) {
                ids.push(pid.clone());
            }
        }
        ids
    }

    fn project_for(session: &SessionContext, id: &str) -> Option<Project> {
        let accessible = Self::accessible_project_ids(session);
        if !accessible.iter().any(|pid| pid == id) {
            return None;
        }
        let seed = Self::hash_id(id);
        let name = if id == "project-fixture" {
            "Fixture Project".to_owned()
        } else {
            format!("Project {}", id.strip_prefix("project-").unwrap_or(id))
        };
        let status = if seed % 10 == 0 {
            "suspended"
        } else {
            "active"
        };
        let created =
            OffsetDateTime::UNIX_EPOCH + time::Duration::seconds((seed % 1_000_000) as i64);
        Some(Project {
            id: id.to_owned(),
            name,
            organization_id: session
                .organization_id
                .clone()
                .unwrap_or_else(|| "org-fixture".to_owned()),
            status: status.to_owned(),
            created_at: created,
            updated_at: created + time::Duration::seconds(60),
        })
    }

    fn user_at(id: u64) -> User {
        let seed = Self::seed_from_id(id ^ 0xdead_beef);
        let status = if seed % 7 == 0 { "suspended" } else { "active" };
        let email = if seed % 3 == 0 {
            None
        } else {
            Some(format!("user-{id:03}@example.com"))
        };
        User {
            id: format!("user-{id:03}"),
            name: format!("Fixture User {id}"),
            email,
            status: status.to_owned(),
            created_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
        }
    }

    fn role_at(id: u64) -> Role {
        match id % FIXTURE_ROLE_TOTAL {
            0 => Role {
                id: "role-tenant-admin".to_owned(),
                name: "Tenant Admin".to_owned(),
                description: Some("Full tenant project administration".to_owned()),
            },
            1 => Role {
                id: "role-tenant-member".to_owned(),
                name: "Tenant Member".to_owned(),
                description: Some("Can manage resources within assigned projects".to_owned()),
            },
            2 => Role {
                id: "role-tenant-reader".to_owned(),
                name: "Tenant Reader".to_owned(),
                description: Some("Read-only access to assigned projects".to_owned()),
            },
            _ => Role {
                id: "role-developer".to_owned(),
                name: "Developer".to_owned(),
                description: Some("Can manage API credentials and developer resources".to_owned()),
            },
        }
    }

    fn quota_for(project_id: &str) -> ProjectQuota {
        let seed = Self::hash_id(project_id);
        // Aligned with O3K kernel known quota dimensions.
        let entries = [
            ("compute.server", 100_u64, "instances"),
            ("compute.vcpus", 200_u64, "cores"),
            ("compute.memory_mb", 524_288_u64, "mebibytes"),
            ("compute.disk_gb", 10_000_u64, "gibibytes"),
            ("storage.volume", 5000_u64, "gibibytes"),
            ("network.vpc", 20_u64, "networks"),
            ("network.subnets", 50_u64, "subnets"),
            ("network.ports", 200_u64, "ports"),
            ("image.images", 100_u64, "images"),
        ]
        .into_iter()
        .enumerate()
        .map(|(idx, (resource_type, limit, unit))| {
            let used = ((seed >> (idx * 4 + 3)) % (limit.max(1))).min(limit);
            QuotaEntry {
                resource_type: resource_type.to_owned(),
                limit,
                used,
                unit: unit.to_owned(),
            }
        })
        .collect();
        ProjectQuota {
            project_id: project_id.to_owned(),
            entries,
        }
    }

    fn audit_event_at(id: u64, session: &SessionContext) -> AuditEvent {
        let seed = Self::seed_from_id(id ^ 0xc0ff_ee00);
        let accessible = Self::accessible_project_ids(session);
        let project_id = if accessible.is_empty() {
            None
        } else {
            Some(accessible[(id as usize) % accessible.len()].clone())
        };
        let actor = format!("user-{:03}", id % FIXTURE_USER_TOTAL);
        let action = match id % 5 {
            0 => "create",
            1 => "delete",
            2 => "update",
            3 => "login",
            _ => "logout",
        };
        let resource_type = if id % 3 == 0 {
            None
        } else {
            Some(match id % 4 {
                0 => "compute.server",
                1 => "storage.volume",
                2 => "network.vpc",
                _ => "tenant.project",
            })
        };
        let resource_id = resource_type.as_ref().map(|rt| format!("{rt}-{id:010}"));
        let outcome = if seed % 8 == 0 { "failed" } else { "succeeded" };
        AuditEvent {
            id: format!("audit-{id:010}"),
            actor,
            action: action.to_owned(),
            resource_type: resource_type.map(|s| s.to_owned()),
            resource_id,
            project_id,
            outcome: outcome.to_owned(),
            recorded_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
            correlation_id: format!("corr-audit-{id}"),
        }
    }

    fn validate_api_credential_kind(kind: &str) -> Result<(), ApiError> {
        if API_CREDENTIAL_KINDS.contains(&kind) {
            Ok(())
        } else {
            Err(ApiError::BadRequest(format!(
                "invalid credential kind: {kind}"
            )))
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

    fn object_storage_bucket_descriptor() -> ResourceTypeDescriptor {
        ResourceTypeDescriptor {
            id: "object.storage.bucket".to_string(),
            name: "Object Storage Bucket".to_string(),
            plural_name: "Object Storage Buckets".to_string(),
            icon_token: "storage".to_string(),
            create_schema: Some(JsonSchema(serde_json::json!({
                "type": "object",
                "required": ["name", "regionId", "projectId"],
                "properties": {
                    "name": { "type": "string", "minLength": 1 },
                    "regionId": { "enum": ["eu-west", "us-east", "ap-south"] },
                    "projectId": { "enum": ["project-1", "project-2", "project-3", "project-4", "project-5"] },
                    "publicAccess": { "type": "boolean" }
                }
            }))),
            create_capability: Capability {
                resource_type: "object.storage.bucket".to_string(),
                action: "create".to_string(),
            },
            supported_actions: vec![ActionDescriptor {
                id: "delete".to_string(),
                name: "Delete".to_string(),
                requires_confirmation: true,
                risk_class: ActionRiskClass::Destructive,
                required_capability: Capability {
                    resource_type: "object.storage.bucket".to_string(),
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
                    id: "size".to_string(),
                    header: "Size (MB)".to_string(),
                    field: "properties.sizeMb".to_string(),
                    width: Some("120px".to_string()),
                },
                ColumnDescriptor {
                    id: "publicAccess".to_string(),
                    header: "Public Access".to_string(),
                    field: "properties.publicAccess".to_string(),
                    width: Some("140px".to_string()),
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
                    "properties.sizeMb".to_string(),
                    "properties.publicAccess".to_string(),
                    "createdAt".to_string(),
                    "updatedAt".to_string(),
                ],
            }],
            relationships: vec![],
        }
    }

    // Operator platform fixture data

    /// Total number of synthetic customer accounts in the fixture operator universe.
    const FIXTURE_ACCOUNT_TOTAL: u64 = 12;

    /// Total number of synthetic operator audit events.
    const FIXTURE_OPERATOR_AUDIT_TOTAL: u64 = 150;

    fn region_at(id: u64) -> Region {
        let seed = Self::seed_from_id(id ^ 0x15a0_7e70);
        let statuses = [
            RegionStatus::Healthy,
            RegionStatus::Degraded,
            RegionStatus::Unavailable,
            RegionStatus::Maintenance,
        ];
        let status = statuses[(seed as usize) % statuses.len()];
        let base_id = match id % 4 {
            0 => "eu-west",
            1 => "us-east",
            2 => "ap-south",
            _ => "sa-east",
        };
        let region_id = base_id.to_string();
        let name = match base_id {
            "eu-west" => "EU West",
            "us-east" => "US East",
            "ap-south" => "AP South",
            _ => "SA East",
        }
        .to_string();

        let az_count = 2 + (seed % 3) as usize;
        let azs: Vec<AvailabilityZone> = (0..az_count)
            .map(|az_idx| {
                let az_seed = Self::seed_from_id(id ^ (az_idx as u64));
                let az_status = statuses[(az_seed as usize) % statuses.len()];
                AvailabilityZone {
                    id: format!("{}-az-{}", region_id, az_idx + 1),
                    name: format!("{} Availability Zone {}", name, az_idx + 1),
                    region_id: region_id.clone(),
                    status: az_status,
                }
            })
            .collect();

        Region {
            id: region_id.clone(),
            name,
            status,
            azs,
            updated_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
        }
    }

    fn provider_at(id: u64) -> ProviderHealth {
        let seed = Self::seed_from_id(id ^ 0x000b_adc0_ffee);
        let kinds = [
            ProviderKind::Compute,
            ProviderKind::Network,
            ProviderKind::Storage,
        ];
        let kind = kinds[(id as usize) % kinds.len()];
        let statuses = [
            RegionStatus::Healthy,
            RegionStatus::Degraded,
            RegionStatus::Unavailable,
            RegionStatus::Maintenance,
        ];
        let status = statuses[(seed as usize) % statuses.len()];
        let regions = ["eu-west", "us-east", "ap-south", "sa-east"];
        let region_id = regions[(id as usize) % regions.len()].to_string();
        let name = format!("{:?} provider {}", kind, id % 6 + 1);
        let message = match status {
            RegionStatus::Healthy => "All health checks passing",
            RegionStatus::Degraded => "Elevated latency on some control-plane calls",
            RegionStatus::Unavailable => "Control-plane heartbeats missed",
            RegionStatus::Maintenance => "Scheduled maintenance in progress",
        }
        .to_string();

        ProviderHealth {
            id: format!("provider-{id:04}"),
            kind,
            name,
            status,
            region_id,
            last_seen_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
            message,
        }
    }

    fn service_health_at(id: u64) -> ServiceHealth {
        let seed = Self::seed_from_id(id ^ 0xfeed_face);
        let services = ["compute", "network", "storage", "identity", "catalog"];
        let name = services[(id as usize) % services.len()].to_string();
        let states = ["ready", "starting", "degraded", "unavailable"];
        let lifecycle_state = states[(seed as usize) % states.len()].to_string();
        let ready_since = if lifecycle_state == "ready" {
            Some(OffsetDateTime::UNIX_EPOCH + time::Duration::seconds((seed % 1_000_000) as i64))
        } else {
            None
        };

        ServiceHealth {
            id: format!("service-{id:04}"),
            name,
            lifecycle_state,
            ready_since,
        }
    }

    fn capacity_summary(resource_class: &str) -> CapacitySummary {
        let seed = Self::hash_id(resource_class);
        let (total, used, unit) = match resource_class {
            "vcpu" => (10_000_u64, ((seed % 6_000) + 1).min(10_000), "cores"),
            "memory" => (40_000_u64, ((seed % 24_000) + 1).min(40_000), "gibibytes"),
            "disk" => (
                500_000_u64,
                ((seed % 300_000) + 1).min(500_000),
                "gibibytes",
            ),
            _ => (1_000_u64, ((seed % 600) + 1).min(1_000), "units"),
        };
        let available = total.saturating_sub(used);

        CapacitySummary {
            resource_class: resource_class.to_owned(),
            total,
            used,
            available,
            unit: unit.to_owned(),
            updated_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
        }
    }

    fn customer_account_at(id: u64) -> CustomerAccount {
        let seed = Self::seed_from_id(id ^ 0xcafe_babe);
        let status = if seed % 7 == 0 { "suspended" } else { "active" };
        CustomerAccount {
            id: format!("account-{id:03}"),
            name: format!("Customer Account {id}"),
            status: status.to_owned(),
            created_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
        }
    }

    fn operator_project_at(id: u64) -> OperatorProject {
        let seed = Self::seed_from_id(id ^ 0xdeca_fb00);
        let account_id = format!("account-{:03}", id % Self::FIXTURE_ACCOUNT_TOTAL);
        let regions = ["eu-west", "us-east", "ap-south", "sa-east"];
        let region_id = regions[(id as usize) % regions.len()].to_string();
        let status = if seed % 9 == 0 { "suspended" } else { "active" };
        OperatorProject {
            id: format!("project-{id:03}"),
            name: format!("Operator Project {id}"),
            account_id,
            region_id,
            status: status.to_owned(),
            created_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
        }
    }

    fn operator_operation_at(id: u64) -> Operation {
        let seed = Self::seed_from_id(id ^ 0x0bad_f00d);
        let states = [
            OperationState::Pending,
            OperationState::Running,
            OperationState::Succeeded,
            OperationState::Failed,
        ];
        let state = states[(seed as usize) % states.len()];
        let error = if state == OperationState::Failed {
            Some(OperationError {
                code: "fixture-operator-failure".to_string(),
                title: "Fixture operator operation failed".to_string(),
                detail: "This is a deterministic operator-scope failure for prototype testing."
                    .to_string(),
            })
        } else {
            None
        };

        Operation {
            id: format!("op-operator-{id:010}"),
            action: if seed % 2 == 0 { "create" } else { "delete" }.to_string(),
            state,
            resource_id: Some(format!("resource-{}", seed % 1_000)),
            resource_type: Some("compute.server".to_string()),
            project_id: Some(format!("project-{}", seed % 50)),
            region_id: Some(["eu-west", "us-east", "ap-south"][(seed as usize) % 3].to_string()),
            initiated_by: Some("fixture-operator".to_string()),
            started_at: Some(
                OffsetDateTime::UNIX_EPOCH + time::Duration::seconds((seed % 1_000_000) as i64),
            ),
            updated_at: Some(
                OffsetDateTime::UNIX_EPOCH
                    + time::Duration::seconds((seed % 1_000_000) as i64 + 60),
            ),
            correlation_id: format!("corr-operator-{id}"),
            error,
            events: vec![],
        }
    }

    fn operator_audit_event_at(id: u64) -> OperatorAuditEvent {
        let seed = Self::seed_from_id(id ^ 0xabad_1dea);
        let actor = format!("operator-{:03}", id % 20);
        let action = match id % 5 {
            0 => "create",
            1 => "delete",
            2 => "update",
            3 => "login",
            _ => "logout",
        };
        let resource_type = if id % 3 == 0 {
            None
        } else {
            Some(match id % 4 {
                0 => "compute.server",
                1 => "storage.volume",
                2 => "network.vpc",
                _ => "operator.region",
            })
        };
        let resource_id = resource_type.as_ref().map(|rt| format!("{rt}-{id:010}"));
        let outcome = if seed % 8 == 0 { "failed" } else { "succeeded" };
        OperatorAuditEvent {
            id: format!("audit-operator-{id:010}"),
            actor,
            action: action.to_owned(),
            resource_type: resource_type.map(|s| s.to_owned()),
            resource_id,
            account_id: Some(format!("account-{:03}", id % Self::FIXTURE_ACCOUNT_TOTAL)),
            project_id: if id % 2 == 0 {
                Some(format!("project-{}", id % 50))
            } else {
                None
            },
            outcome: outcome.to_owned(),
            recorded_at: OffsetDateTime::UNIX_EPOCH
                + time::Duration::seconds((seed % 1_000_000) as i64),
            correlation_id: format!("corr-operator-audit-{id}"),
        }
    }

    fn platform_overview() -> PlatformOverview {
        let regions: Vec<Region> = (0..4).map(Self::region_at).collect();
        let providers: Vec<ProviderHealth> = (0..12).map(Self::provider_at).collect();

        let mut region_counts: std::collections::HashMap<RegionStatus, u64> =
            std::collections::HashMap::new();
        for r in &regions {
            *region_counts.entry(r.status).or_insert(0) += 1;
            for az in &r.azs {
                *region_counts.entry(az.status).or_insert(0) += 1;
            }
        }

        let mut provider_counts: std::collections::HashMap<RegionStatus, u64> =
            std::collections::HashMap::new();
        for p in &providers {
            *provider_counts.entry(p.status).or_insert(0) += 1;
        }

        let region_status_summary = region_counts
            .into_iter()
            .map(|(status, count)| StatusCount { status, count })
            .collect();
        let provider_status_summary = provider_counts
            .into_iter()
            .map(|(status, count)| StatusCount { status, count })
            .collect();

        let active_operations_count = (0..100)
            .map(Self::operator_operation_at)
            .filter(|o| o.state == OperationState::Running)
            .count() as u64;

        let recent_alerts = vec![
            PlatformAlert {
                id: "alert-001".to_string(),
                severity: AlertSeverity::Warning,
                message: "Elevated compute control-plane latency in us-east".to_string(),
                occurred_at: OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(1_700_000_000),
            },
            PlatformAlert {
                id: "alert-002".to_string(),
                severity: AlertSeverity::Info,
                message: "Capacity rebalance completed in eu-west".to_string(),
                occurred_at: OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(1_700_000_300),
            },
        ];

        PlatformOverview {
            region_status_summary,
            provider_status_summary,
            active_operations_count,
            recent_alerts,
            data_freshness_at: OffsetDateTime::now_utc(),
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
            ServiceDescriptor {
                id: "object.storage".to_string(),
                name: "Object Storage".to_string(),
                category: "Services".to_string(),
                resource_types: vec![Self::object_storage_bucket_descriptor()],
            },
        ])
    }

    async fn list_discovered_services(
        &self,
        ctx: &RequestContext,
    ) -> Result<Vec<ServiceCatalogEntry>, ApiError> {
        let session = self.context(ctx).await?;
        let required_capability = if self.surface == "operator-bff" {
            ("operator.service", "list")
        } else {
            ("tenant.service-catalog", "list")
        };
        if !session.has_capability(required_capability.0, required_capability.1) {
            return Err(ApiError::Forbidden);
        }

        let services = [
            ("identity", "Identity", "ready", vec!["global"]),
            (
                "image",
                "Image",
                "ready",
                vec!["eu-west", "us-east", "ap-south"],
            ),
            (
                "network",
                "Networking",
                "ready",
                vec!["eu-west", "us-east", "ap-south"],
            ),
            (
                "compute",
                "Compute",
                "ready",
                vec!["eu-west", "us-east", "ap-south"],
            ),
            (
                "placement",
                "Placement",
                "ready",
                vec!["eu-west", "us-east", "ap-south"],
            ),
            (
                "object.storage",
                "Object Storage",
                "ready",
                vec!["eu-west", "us-east", "ap-south"],
            ),
        ];

        Ok(services
            .into_iter()
            .map(|(id, name, lifecycle_state, regions)| {
                let mut capability_resource_type = id.to_string();
                capability_resource_type.push_str(".*");
                ServiceCatalogEntry {
                    id: id.to_string(),
                    namespace: id.to_string(),
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    ownership: Some("platform".to_string()),
                    lifecycle_state: lifecycle_state.to_string(),
                    capabilities: vec![Capability {
                        resource_type: capability_resource_type,
                        action: "list".to_string(),
                    }],
                    regions: regions.into_iter().map(String::from).collect(),
                    description: Some(format!("Fixture {name} service")),
                    documentation_url: Some(format!("https://docs.araf.o3k.io/services/{id}")),
                }
            })
            .collect())
    }

    async fn list_discovered_resource_types(
        &self,
        ctx: &RequestContext,
    ) -> Result<Vec<DiscoveredResourceType>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("operator.service", "read") {
            return Err(ApiError::Forbidden);
        }

        let types = [
            ("identity", "project", "projects", "project", true),
            ("identity", "user", "users", "project", true),
            ("image", "image", "images", "project", true),
            ("network", "vpc", "vpcs", "project", true),
            ("compute", "server", "servers", "project", true),
            (
                "placement",
                "server_group",
                "server-groups",
                "project",
                true,
            ),
            ("storage", "volume", "volumes", "project", true),
            ("object.storage", "bucket", "buckets", "project", true),
        ];

        Ok(types
            .into_iter()
            .map(|(namespace, name, collection, scope, ready)| {
                let mut lifecycle_actions = HashMap::new();
                lifecycle_actions.insert(
                    "list".to_string(),
                    format!("/o3k/v1/{namespace}/{collection}"),
                );
                DiscoveredResourceType {
                    namespace: namespace.to_string(),
                    name: name.to_string(),
                    service_id: namespace.to_string(),
                    schema_version: "1.0".to_string(),
                    collection: collection.to_string(),
                    scope: scope.to_string(),
                    ready,
                    lifecycle_actions,
                }
            })
            .collect())
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
        let now = OffsetDateTime::now_utc();
        op.action = request.action_id;
        op.resource_id = Some(id.to_string());
        op.resource_type = Some(resource_type.to_string());
        op.project_id = Some(resource.project_id);
        op.region_id = Some(resource.region_id);
        op.state = OperationState::Pending;
        op.started_at = Some(now);
        op.updated_at = Some(now);
        op.correlation_id = ctx.correlation_id().to_string();
        op.events = vec![OperationEvent::initial(&op)];

        self.store
            .lock()
            .expect("fixture operation store poisoned")
            .operations
            .push(op.clone());
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

        let mut op = Operation {
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
            events: vec![],
        };
        op.events = vec![OperationEvent::initial(&op)];

        self.store
            .lock()
            .expect("fixture operation store poisoned")
            .operations
            .push(op.clone());
        Ok(op)
    }

    async fn list_operations(
        &self,
        _ctx: &RequestContext,
        params: ListOperationsParams,
    ) -> Result<PaginatedCollection<Operation>, ApiError> {
        let now = OffsetDateTime::now_utc();
        self.advance_operations(now);

        let page_size = params.page_size.clamp(1, 100);

        // Merge stored operations with deterministic synthetic ones. Stored
        // operations take precedence for the same id; synthetic ids already in
        // the store are skipped so the total stays predictable when the store
        // is empty (the existing contract tests assert total == 1_000).
        let mut all_operations: Vec<Operation> = Vec::new();
        let stored_ids: HashSet<String> = {
            let store = self.store.lock().expect("fixture operation store poisoned");
            all_operations.extend(store.operations.clone());
            store.operations.iter().map(|o| o.id.clone()).collect()
        };
        for id in 0..1_000_u64 {
            let op_id = format!("op-{id:010}");
            if !stored_ids.contains(&op_id) {
                all_operations.push(Self::operation_at(id));
            }
        }

        if let Some(state) = params.state {
            all_operations.retain(|o| o.state == state);
        }
        if let Some(action) = params.action {
            all_operations.retain(|o| o.action == action);
        }
        if let Some(resource_type) = params.resource_type {
            all_operations.retain(|o| o.resource_type.as_ref() == Some(&resource_type));
        }
        if let Some(resource_id) = params.resource_id {
            all_operations.retain(|o| o.resource_id.as_ref() == Some(&resource_id));
        }
        if let Some(project_id) = params.project_id {
            all_operations.retain(|o| o.project_id.as_ref() == Some(&project_id));
        }
        if let Some(region_id) = params.region_id {
            all_operations.retain(|o| o.region_id.as_ref() == Some(&region_id));
        }
        if let Some(since) = params.since {
            all_operations.retain(|o| {
                o.started_at
                    .map(|started| started >= since)
                    .unwrap_or(false)
            });
        }
        if let Some(until) = params.until {
            all_operations.retain(|o| {
                o.started_at
                    .map(|started| started <= until)
                    .unwrap_or(false)
            });
        }

        let total = all_operations.len() as u64;
        let offset = (params.page as u64).saturating_mul(page_size as u64);
        let items = if offset >= total {
            vec![]
        } else {
            let end = (offset + page_size as u64).min(total) as usize;
            all_operations[offset as usize..end].to_vec()
        };

        Ok(PaginatedCollection {
            items,
            total,
            page: params.page,
            page_size,
            has_more: offset + (page_size as u64) < total,
        })
    }

    async fn get_operation(&self, _ctx: &RequestContext, id: &str) -> Result<Operation, ApiError> {
        let now = OffsetDateTime::now_utc();
        self.advance_operations(now);

        {
            let store = self.store.lock().expect("fixture operation store poisoned");
            if let Some(op) = store.operations.iter().find(|o| o.id == id).cloned() {
                return Ok(op);
            }
        }

        // Fallback to deterministic synthetic operation for backward-compatible
        // tests that request ids in the fixture universe.
        if let Some(numeric_id) = id
            .strip_prefix("op-operator-")
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|n| *n < 1_000)
        {
            return Ok(Self::operator_operation_at(numeric_id));
        }

        let numeric_id = id
            .strip_prefix("op-")
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|n| *n < 1_000)
            .ok_or(ApiError::NotFound)?;

        Ok(Self::operation_at(numeric_id))
    }

    async fn list_projects(
        &self,
        ctx: &RequestContext,
    ) -> Result<PaginatedCollection<Project>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.project", "list") {
            return Err(ApiError::Forbidden);
        }

        let items: Vec<Project> = Self::accessible_project_ids(&session)
            .into_iter()
            .filter_map(|id| Self::project_for(&session, &id))
            .collect();

        Ok(PaginatedCollection {
            total: items.len() as u64,
            has_more: false,
            page: 0,
            page_size: items.len() as u32,
            items,
        })
    }

    async fn get_project(&self, ctx: &RequestContext, id: &str) -> Result<Project, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.project", "read") {
            return Err(ApiError::Forbidden);
        }

        Self::project_for(&session, id).ok_or(ApiError::NotFound)
    }

    async fn list_project_members(
        &self,
        ctx: &RequestContext,
        id: &str,
    ) -> Result<Vec<ProjectMember>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.project", "read") {
            return Err(ApiError::Forbidden);
        }

        let accessible = Self::accessible_project_ids(&session);
        if !accessible.iter().any(|pid| pid == id) {
            return Err(ApiError::NotFound);
        }

        let seed = Self::hash_id(id);
        let member_count = ((seed % 5) + 1) as usize;
        let members: Vec<ProjectMember> = (0..member_count)
            .map(|idx| {
                let user_id = (seed + idx as u64) % FIXTURE_USER_TOTAL;
                let role = Self::role_at((seed + idx as u64) % FIXTURE_ROLE_TOTAL);
                ProjectMember {
                    user: Self::user_at(user_id),
                    roles: vec![role.name],
                }
            })
            .collect();
        Ok(members)
    }

    async fn list_users(
        &self,
        ctx: &RequestContext,
    ) -> Result<PaginatedCollection<User>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.user", "list") {
            return Err(ApiError::Forbidden);
        }

        let all: Vec<User> = (0..FIXTURE_USER_TOTAL).map(Self::user_at).collect();
        let page_size = 25_u32;
        let items = all.iter().take(page_size as usize).cloned().collect();
        Ok(PaginatedCollection {
            total: all.len() as u64,
            has_more: all.len() > page_size as usize,
            page: 0,
            page_size,
            items,
        })
    }

    async fn get_user(&self, ctx: &RequestContext, id: &str) -> Result<User, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.user", "read") {
            return Err(ApiError::Forbidden);
        }

        let numeric_id = id
            .strip_prefix("user-")
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|n| *n < FIXTURE_USER_TOTAL)
            .ok_or(ApiError::NotFound)?;
        Ok(Self::user_at(numeric_id))
    }

    async fn list_roles(
        &self,
        ctx: &RequestContext,
    ) -> Result<PaginatedCollection<Role>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.role", "list") {
            return Err(ApiError::Forbidden);
        }

        let items: Vec<Role> = (0..FIXTURE_ROLE_TOTAL).map(Self::role_at).collect();
        Ok(PaginatedCollection {
            total: items.len() as u64,
            has_more: false,
            page: 0,
            page_size: items.len() as u32,
            items,
        })
    }

    async fn list_quotas(
        &self,
        ctx: &RequestContext,
        project_id: Option<&str>,
    ) -> Result<PaginatedCollection<ProjectQuota>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.quota", "read") {
            return Err(ApiError::Forbidden);
        }

        let accessible = Self::accessible_project_ids(&session);
        let ids: Vec<String> = match project_id {
            Some(pid) => {
                if !accessible.iter().any(|id| id == pid) {
                    return Err(ApiError::NotFound);
                }
                vec![pid.to_owned()]
            }
            None => accessible,
        };

        let items: Vec<ProjectQuota> = ids.iter().map(|id| Self::quota_for(id)).collect();
        Ok(PaginatedCollection {
            total: items.len() as u64,
            has_more: false,
            page: 0,
            page_size: items.len() as u32,
            items,
        })
    }

    async fn list_usage(
        &self,
        ctx: &RequestContext,
        query: UsageQuery,
    ) -> Result<UsageSummary, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.quota", "read") {
            return Err(ApiError::Forbidden);
        }

        let (since, until) = query.bounded_range()?;

        let accessible = Self::accessible_project_ids(&session);
        let pid = query.project_id.as_deref().unwrap_or_else(|| {
            accessible
                .first()
                .map(|s| s.as_str())
                .unwrap_or("project-1")
        });
        if !accessible.iter().any(|id| id.as_str() == pid) {
            return Err(ApiError::NotFound);
        }

        let seed = Self::hash_id(pid);
        let range_hours = (until - since).whole_hours().max(1) as u64;

        // Generate hourly usage data points over the window, with deterministic
        // values derived from the project seed so each project is distinct and
        // stable across requests.
        let dimensions = [
            ("compute.server", "instances"),
            ("compute.vcpus", "cores"),
            ("network.vpc", "networks"),
            ("storage.volume", "gibibytes"),
        ];

        let mut records = Vec::new();
        let now = OffsetDateTime::now_utc();

        for hour_offset in 0..range_hours.min(72) {
            let ts = since + time::Duration::hours(hour_offset as i64);
            if ts > now || ts > until {
                break;
            }

            for (dim_idx, (resource_type, unit)) in dimensions.iter().enumerate() {
                let base = (seed >> (dim_idx * 8)) % 100;
                // Deterministic hourly variation so each data point is
                // distinguishable but pattern remains stable.
                let variation =
                    (seed.wrapping_mul(hour_offset.wrapping_add(1)) >> (dim_idx * 4)) % 20;
                let value = base.saturating_add(variation).max(1);
                records.push(UsageRecord {
                    resource_type: resource_type.to_string(),
                    value: value.min(1000),
                    unit: unit.to_string(),
                    timestamp: ts,
                });
            }
        }

        // If the requested range is empty or too short, return at least one point.
        if records.is_empty() {
            let ts = since;
            for (dim_idx, (resource_type, unit)) in dimensions.iter().enumerate() {
                let base = (seed >> (dim_idx * 8)) % 100;
                records.push(UsageRecord {
                    resource_type: resource_type.to_string(),
                    value: base.max(1),
                    unit: unit.to_string(),
                    timestamp: ts,
                });
            }
        }

        Ok(UsageSummary {
            project_id: pid.to_string(),
            records,
            since,
            until,
        })
    }

    async fn list_audit_events(
        &self,
        ctx: &RequestContext,
        params: ListAuditEventsParams,
    ) -> Result<PaginatedCollection<AuditEvent>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.audit", "read") {
            return Err(ApiError::Forbidden);
        }

        let accessible = Self::accessible_project_ids(&session);
        if let Some(pid) = &params.project_id {
            if !accessible.iter().any(|id| id == pid) {
                return Err(ApiError::NotFound);
            }
        }

        let mut events: Vec<AuditEvent> = (0..FIXTURE_AUDIT_TOTAL)
            .map(|id| Self::audit_event_at(id, &session))
            .filter(|e| {
                params
                    .project_id
                    .as_ref()
                    .map(|pid| e.project_id.as_ref() == Some(pid))
                    .unwrap_or(true)
            })
            .filter(|e| {
                params
                    .action
                    .as_ref()
                    .map(|a| &e.action == a)
                    .unwrap_or(true)
            })
            .filter(|e| params.actor.as_ref().map(|a| &e.actor == a).unwrap_or(true))
            .filter(|e| {
                params
                    .since
                    .map(|since| e.recorded_at >= since)
                    .unwrap_or(true)
            })
            .filter(|e| {
                params
                    .until
                    .map(|until| e.recorded_at <= until)
                    .unwrap_or(true)
            })
            .collect();

        events.sort_by_key(|b| std::cmp::Reverse(b.recorded_at));

        let page_size = params.page_size.clamp(1, 100);
        let total = events.len() as u64;
        let offset = (params.page as u64).saturating_mul(page_size as u64);
        let items = if offset >= total {
            vec![]
        } else {
            let end = (offset + page_size as u64).min(total) as usize;
            events[offset as usize..end].to_vec()
        };

        Ok(PaginatedCollection {
            items,
            total,
            page: params.page,
            page_size,
            has_more: offset + (page_size as u64) < total,
        })
    }

    async fn list_api_credentials(
        &self,
        ctx: &RequestContext,
    ) -> Result<PaginatedCollection<ApiCredential>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.api-credential", "list") {
            return Err(ApiError::Forbidden);
        }

        let store = self.store.lock().expect("fixture operation store poisoned");
        let items = store.api_credentials.clone();
        let total = items.len() as u64;
        Ok(PaginatedCollection {
            items,
            total,
            page: 0,
            page_size: total.max(1) as u32,
            has_more: false,
        })
    }

    async fn create_api_credential(
        &self,
        ctx: &RequestContext,
        request: CreateApiCredentialRequest,
    ) -> Result<ApiCredential, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.api-credential", "create") {
            return Err(ApiError::Forbidden);
        }

        Self::validate_api_credential_kind(&request.kind)?;

        let accessible = Self::accessible_project_ids(&session);
        if !accessible.iter().any(|id| id == &request.project_id) {
            return Err(ApiError::NotFound);
        }

        let id = format!("cred-{}", uuid::Uuid::new_v4());
        let secret = format!("secret-{}", uuid::Uuid::new_v4());
        let now = OffsetDateTime::now_utc();
        let metadata = ApiCredential {
            id: id.clone(),
            name: request.name,
            kind: request.kind,
            project_id: request.project_id,
            created_at: now,
            expires_at: request.expires_at,
            secret: None,
        };

        self.store
            .lock()
            .expect("fixture operation store poisoned")
            .api_credentials
            .push(metadata.clone());

        Ok(ApiCredential {
            secret: Some(secret),
            ..metadata
        })
    }

    async fn delete_api_credential(&self, ctx: &RequestContext, id: &str) -> Result<(), ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("tenant.api-credential", "delete") {
            return Err(ApiError::Forbidden);
        }

        let mut store = self.store.lock().expect("fixture operation store poisoned");
        let pos = store
            .api_credentials
            .iter()
            .position(|c| c.id == id)
            .ok_or(ApiError::NotFound)?;
        store.api_credentials.remove(pos);
        Ok(())
    }

    async fn list_regions(&self, ctx: &RequestContext) -> Result<Vec<Region>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("platform.region", "list") {
            return Err(ApiError::Forbidden);
        }
        Ok((0..4).map(Self::region_at).collect())
    }

    async fn list_availability_zones(
        &self,
        ctx: &RequestContext,
        region_id: &str,
    ) -> Result<Vec<AvailabilityZone>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("platform.region", "read") {
            return Err(ApiError::Forbidden);
        }

        let region = (0..4)
            .map(Self::region_at)
            .find(|r| r.id == region_id)
            .ok_or(ApiError::NotFound)?;
        Ok(region.azs)
    }

    async fn list_provider_health(
        &self,
        ctx: &RequestContext,
    ) -> Result<Vec<ProviderHealth>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("platform.health", "read") {
            return Err(ApiError::Forbidden);
        }
        Ok((0..12).map(Self::provider_at).collect())
    }

    async fn list_service_health(
        &self,
        ctx: &RequestContext,
    ) -> Result<Vec<ServiceHealth>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("platform.health", "read") {
            return Err(ApiError::Forbidden);
        }
        Ok((0..10).map(Self::service_health_at).collect())
    }

    async fn get_capacity_summary(
        &self,
        ctx: &RequestContext,
    ) -> Result<Vec<CapacitySummary>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("platform.capacity", "read") {
            return Err(ApiError::Forbidden);
        }
        Ok(["vcpu", "memory", "disk"]
            .into_iter()
            .map(Self::capacity_summary)
            .collect())
    }

    async fn list_customer_accounts(
        &self,
        ctx: &RequestContext,
    ) -> Result<PaginatedCollection<CustomerAccount>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("operator.account", "list") {
            return Err(ApiError::Forbidden);
        }

        let items: Vec<CustomerAccount> = (0..Self::FIXTURE_ACCOUNT_TOTAL)
            .map(Self::customer_account_at)
            .collect();
        Ok(PaginatedCollection {
            total: items.len() as u64,
            has_more: false,
            page: 0,
            page_size: items.len() as u32,
            items,
        })
    }

    async fn list_operator_projects(
        &self,
        ctx: &RequestContext,
        account_id: Option<&str>,
    ) -> Result<PaginatedCollection<OperatorProject>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("operator.project", "list") {
            return Err(ApiError::Forbidden);
        }

        let mut items: Vec<OperatorProject> = (0..50).map(Self::operator_project_at).collect();
        if let Some(account) = account_id {
            items.retain(|p| p.account_id == account);
        }

        Ok(PaginatedCollection {
            total: items.len() as u64,
            has_more: false,
            page: 0,
            page_size: items.len() as u32,
            items,
        })
    }

    async fn list_operator_operations(
        &self,
        ctx: &RequestContext,
        params: ListOperatorOperationsParams,
    ) -> Result<PaginatedCollection<Operation>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("operator.operation", "list") {
            return Err(ApiError::Forbidden);
        }

        let page_size = params.page_size.clamp(1, 100);
        let mut items: Vec<Operation> = (0..100).map(Self::operator_operation_at).collect();

        if let Some(state) = params.state {
            items.retain(|o| o.state == state);
        }
        if let Some(action) = params.action {
            items.retain(|o| o.action == action);
        }
        if let Some(resource_type) = params.resource_type {
            items.retain(|o| o.resource_type.as_ref() == Some(&resource_type));
        }
        if let Some(region_id) = params.region_id {
            items.retain(|o| o.region_id.as_ref() == Some(&region_id));
        }
        if let Some(account_id) = params.account_id {
            items.retain(|o| {
                o.project_id
                    .as_ref()
                    .and_then(|pid| pid.strip_prefix("project-"))
                    .and_then(|n| n.parse::<u64>().ok())
                    .map(|n| format!("account-{:03}", n % Self::FIXTURE_ACCOUNT_TOTAL))
                    == Some(account_id.to_owned())
            });
        }
        if let Some(since) = params.since {
            items.retain(|o| {
                o.started_at
                    .map(|started| started >= since)
                    .unwrap_or(false)
            });
        }
        if let Some(until) = params.until {
            items.retain(|o| {
                o.started_at
                    .map(|started| started <= until)
                    .unwrap_or(false)
            });
        }

        let total = items.len() as u64;
        let offset = (params.page as u64).saturating_mul(page_size as u64);
        let page_items = if offset >= total {
            vec![]
        } else {
            let end = (offset + page_size as u64).min(total) as usize;
            items[offset as usize..end].to_vec()
        };

        Ok(PaginatedCollection {
            items: page_items,
            total,
            page: params.page,
            page_size,
            has_more: offset + (page_size as u64) < total,
        })
    }

    async fn list_operator_audit_events(
        &self,
        ctx: &RequestContext,
        params: ListOperatorAuditEventsParams,
    ) -> Result<PaginatedCollection<OperatorAuditEvent>, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("operator.audit", "read") {
            return Err(ApiError::Forbidden);
        }

        let mut events: Vec<OperatorAuditEvent> = (0..Self::FIXTURE_OPERATOR_AUDIT_TOTAL)
            .map(Self::operator_audit_event_at)
            .filter(|e| {
                params
                    .action
                    .as_ref()
                    .map(|a| &e.action == a)
                    .unwrap_or(true)
            })
            .filter(|e| params.actor.as_ref().map(|a| &e.actor == a).unwrap_or(true))
            .filter(|e| {
                params
                    .account_id
                    .as_ref()
                    .map(|a| e.account_id.as_ref() == Some(a))
                    .unwrap_or(true)
            })
            .filter(|e| {
                params
                    .since
                    .map(|since| e.recorded_at >= since)
                    .unwrap_or(true)
            })
            .filter(|e| {
                params
                    .until
                    .map(|until| e.recorded_at <= until)
                    .unwrap_or(true)
            })
            .collect();

        events.sort_by_key(|b| std::cmp::Reverse(b.recorded_at));

        let page_size = params.page_size.clamp(1, 100);
        let total = events.len() as u64;
        let offset = (params.page as u64).saturating_mul(page_size as u64);
        let items = if offset >= total {
            vec![]
        } else {
            let end = (offset + page_size as u64).min(total) as usize;
            events[offset as usize..end].to_vec()
        };

        Ok(PaginatedCollection {
            items,
            total,
            page: params.page,
            page_size,
            has_more: offset + (page_size as u64) < total,
        })
    }

    async fn get_platform_overview(
        &self,
        ctx: &RequestContext,
    ) -> Result<PlatformOverview, ApiError> {
        let session = self.context(ctx).await?;
        if !session.has_capability("platform.overview", "read") {
            return Err(ApiError::Forbidden);
        }
        Ok(Self::platform_overview())
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

        if surface == "tenant-bff" {
            capabilities.extend([
                Capability {
                    resource_type: "tenant.project".to_string(),
                    action: "list".to_string(),
                },
                Capability {
                    resource_type: "tenant.project".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "tenant.user".to_string(),
                    action: "list".to_string(),
                },
                Capability {
                    resource_type: "tenant.user".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "tenant.role".to_string(),
                    action: "list".to_string(),
                },
                Capability {
                    resource_type: "tenant.quota".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "tenant.audit".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "tenant.api-credential".to_string(),
                    action: "list".to_string(),
                },
                Capability {
                    resource_type: "tenant.api-credential".to_string(),
                    action: "create".to_string(),
                },
                Capability {
                    resource_type: "tenant.api-credential".to_string(),
                    action: "delete".to_string(),
                },
                Capability {
                    resource_type: "tenant.service-catalog".to_string(),
                    action: "list".to_string(),
                },
            ]);
        }

        if surface == "operator-bff" {
            capabilities.extend([
                Capability {
                    resource_type: "platform.overview".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "platform.region".to_string(),
                    action: "list".to_string(),
                },
                Capability {
                    resource_type: "platform.region".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "platform.health".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "platform.capacity".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "operator.account".to_string(),
                    action: "list".to_string(),
                },
                Capability {
                    resource_type: "operator.project".to_string(),
                    action: "list".to_string(),
                },
                Capability {
                    resource_type: "operator.operation".to_string(),
                    action: "list".to_string(),
                },
                Capability {
                    resource_type: "operator.audit".to_string(),
                    action: "read".to_string(),
                },
                Capability {
                    resource_type: "operator.service".to_string(),
                    action: "list".to_string(),
                },
                Capability {
                    resource_type: "operator.service".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn pending_operation(id: &str, started_at: OffsetDateTime) -> Operation {
        let mut op = Operation {
            id: id.to_string(),
            action: "create".to_string(),
            state: OperationState::Pending,
            resource_id: None,
            resource_type: None,
            project_id: None,
            region_id: None,
            initiated_by: None,
            started_at: Some(started_at),
            updated_at: Some(started_at),
            correlation_id: "corr-test".to_string(),
            error: None,
            events: vec![],
        };
        op.events = vec![OperationEvent::initial(&op)];
        op
    }

    #[test]
    fn operation_transitions_pending_to_running_to_succeeded() {
        let success_id = (0..100)
            .map(|i| format!("op-{i:010}"))
            .find(|id| FixtureAdapter::is_success_id(id))
            .expect("at least one id should hash to success");

        let started_at = OffsetDateTime::now_utc();
        let mut op = pending_operation(&success_id, started_at);

        FixtureAdapter::advance_operation(&mut op, started_at);
        assert_eq!(op.state, OperationState::Pending);

        FixtureAdapter::advance_operation(
            &mut op,
            started_at + time::Duration::seconds(PENDING_TO_RUNNING_SECS),
        );
        assert_eq!(op.state, OperationState::Running);
        assert!(op.events.iter().any(|e| e.state == OperationState::Running));

        FixtureAdapter::advance_operation(
            &mut op,
            started_at + time::Duration::seconds(RUNNING_TO_TERMINAL_SECS),
        );
        assert_eq!(op.state, OperationState::Succeeded);
        assert!(op
            .events
            .iter()
            .any(|e| e.state == OperationState::Succeeded));
    }

    #[test]
    fn operation_transitions_pending_to_running_to_failed() {
        let failure_id = (0..100)
            .map(|i| format!("op-{i:010}"))
            .find(|id| !FixtureAdapter::is_success_id(id))
            .expect("at least one id should hash to failure");

        let started_at = OffsetDateTime::now_utc();
        let mut op = pending_operation(&failure_id, started_at);

        FixtureAdapter::advance_operation(
            &mut op,
            started_at + time::Duration::seconds(PENDING_TO_RUNNING_SECS),
        );
        assert_eq!(op.state, OperationState::Running);

        FixtureAdapter::advance_operation(
            &mut op,
            started_at + time::Duration::seconds(RUNNING_TO_TERMINAL_SECS),
        );
        assert_eq!(op.state, OperationState::Failed);
        assert!(op.error.is_some());
        assert!(op.events.iter().any(|e| e.state == OperationState::Failed));
    }
}
