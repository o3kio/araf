//! Araf BFF domain models.
//!
//! These are presentation-oriented shapes exposed to the frontend. They are
//! deliberately not O3K wire types; the upstream adapter is responsible for
//! translating authoritative O3K representations into these UI-facing shapes.

use std::collections::HashMap;

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
    pub icon_token: String,
    pub supported_actions: Vec<ActionDescriptor>,
    pub columns: Vec<ColumnDescriptor>,
    pub filters: Vec<FilterDescriptor>,
    pub sortable_fields: Vec<String>,
    pub details_sections: Vec<DetailsSectionDescriptor>,
    pub relationships: Vec<RelationshipDescriptor>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionDescriptor {
    pub id: String,
    pub name: String,
    pub requires_confirmation: bool,
}

/// Presentation column for a resource collection table.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnDescriptor {
    pub id: String,
    pub header: String,
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
}

/// Presentation filter for a resource collection.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterDescriptor {
    pub id: String,
    pub label: String,
    pub field: String,
    pub kind: FilterKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FilterKind {
    Text,
    Select,
}

/// Section of fields shown on a resource detail page.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailsSectionDescriptor {
    pub id: String,
    pub label: String,
    pub fields: Vec<String>,
}

/// Relationship from this resource type to another resource type.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipDescriptor {
    pub id: String,
    pub target_resource_type: String,
    pub label: String,
    pub source_property_key: String,
    pub direction: RelationshipDirection,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RelationshipDirection {
    ToOne,
    ToMany,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceStatus {
    Ready,
    Busy,
    Error,
    Unknown,
}

/// Sort direction for server-bounded collection requests.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
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
