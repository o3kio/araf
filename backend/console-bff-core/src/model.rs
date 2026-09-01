//! Araf BFF domain models.
//!
//! These are presentation-oriented shapes exposed to the frontend. They are
//! deliberately not O3K wire types; the upstream adapter is responsible for
//! translating authoritative O3K representations into these UI-facing shapes.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Identity and capabilities for the current console session.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContext {
    pub surface: &'static str,
    pub user_id: String,
    pub user_name: String,
    pub organization_id: Option<String>,
    pub project_id: Option<String>,
    pub region_id: Option<String>,
    pub capabilities: Vec<Capability>,
}

/// A capability granted to the current session.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub resource_type: String,
    pub action: String,
}

/// Service/catalog descriptor exposed to the console.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDescriptor {
    pub id: String,
    pub name: String,
    pub category: String,
    pub resource_types: Vec<ResourceTypeDescriptor>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTypeDescriptor {
    pub id: String,
    pub name: String,
    pub plural_name: String,
    pub supported_actions: Vec<ActionDescriptor>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionDescriptor {
    pub id: String,
    pub name: String,
    pub requires_confirmation: bool,
}

/// A cloud resource in a collection or detail view.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub resource_type: String,
    pub project_id: String,
    pub region_id: String,
    pub status: ResourceStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceStatus {
    Ready,
    Busy,
    Error,
    Unknown,
}

/// Server-bounded paginated collection.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedCollection<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
}

/// Canonical asynchronous Operation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub id: String,
    pub action: String,
    pub state: OperationState,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub project_id: Option<String>,
    pub region_id: Option<String>,
    pub initiated_by: Option<String>,
    pub started_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
    pub correlation_id: String,
    pub error: Option<OperationError>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OperationState {
    Pending,
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationError {
    pub code: String,
    pub title: String,
    pub detail: String,
}

/// Request to perform an action on a resource.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequest {
    pub action_id: String,
    pub payload: Option<serde_json::Value>,
}

impl SessionContext {
    pub fn fixture(surface: &'static str) -> Self {
        Self {
            surface,
            user_id: Uuid::new_v4().to_string(),
            user_name: "Fixture User".to_string(),
            organization_id: Some("org-fixture".to_string()),
            project_id: Some("project-fixture".to_string()),
            region_id: Some("global".to_string()),
            capabilities: vec![
                Capability {
                    resource_type: "compute.server".to_string(),
                    action: "list".to_string(),
                },
                Capability {
                    resource_type: "compute.server".to_string(),
                    action: "create".to_string(),
                },
                Capability {
                    resource_type: "compute.server".to_string(),
                    action: "delete".to_string(),
                },
            ],
        }
    }
}
