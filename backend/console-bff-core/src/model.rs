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

/// JSON Schema document describing a create or action input shape.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonSchema(pub serde_json::Value);

impl JsonSchema {
    pub fn as_value(&self) -> &serde_json::Value {
        &self.0
    }
}

/// Risk classification for a resource action.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ActionRiskClass {
    Normal,
    Disruptive,
    Destructive,
    Privileged,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_schema: Option<JsonSchema>,
    pub create_capability: Capability,
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
    pub risk_class: ActionRiskClass,
    pub required_capability: Capability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<JsonSchema>,
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
    pub events: Vec<OperationEvent>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OperationState {
    Pending,
    Running,
    Succeeded,
    Failed,
    Retryable,
    UnknownOutcome,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationError {
    pub code: String,
    pub title: String,
    pub detail: String,
}

/// A single event in an Operation's lifecycle timeline.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationEvent {
    pub id: String,
    pub state: OperationState,
    pub occurred_at: OffsetDateTime,
    pub message: String,
    pub correlation_id: String,
}

impl OperationEvent {
    /// Build the initial `Pending` event for a newly created operation.
    pub fn initial(operation: &Operation) -> Self {
        Self {
            id: format!("{}-pending", operation.id),
            state: OperationState::Pending,
            occurred_at: operation.started_at.unwrap_or_else(OffsetDateTime::now_utc),
            message: "Operation created and pending".to_string(),
            correlation_id: operation.correlation_id.clone(),
        }
    }
}

/// Request to perform an action on a resource.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequest {
    pub action_id: String,
    pub payload: Option<serde_json::Value>,
}

/// Request to create a new resource.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResourceRequest {
    /// The create payload is the body itself, flattened so the frontend sends
    /// the resource fields directly while the handler receives them as a single
    /// `Value` for schema validation.
    #[serde(flatten)]
    pub payload: serde_json::Value,
}

impl SessionContext {
    /// Check whether the session has been granted a specific capability.
    pub fn has_capability(&self, resource_type: &str, action: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| c.resource_type == resource_type && c.action == action)
    }

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
                    action: "start".to_string(),
                },
                Capability {
                    resource_type: "compute.server".to_string(),
                    action: "delete".to_string(),
                },
            ],
        }
    }
}
