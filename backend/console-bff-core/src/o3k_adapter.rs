//! Production O3K native upstream adapter.
//!
//! Translates between the authoritative O3K `/o3k/v1` native API and Araf's
//! presentation-oriented BFF models. It is stateless: every call goes to O3K,
//! and the adapter does not retain resources or operations between requests.
//!
//! Known upstream boundaries handled explicitly:
//! - O3K does not expose `GET /o3k/v1/operations`; `list_operations` remains
//!   rejected rather than fabricating an inventory.
//! - O3K Operation has no event array; events are derived from its canonical
//!   timestamps and error fields.
//! - Native list endpoints do not filter/sort or return totals; Araf keeps
//!   requests bounded and reports only an explicitly synthetic page total.
//! - Resource mutations and domain actions are executable only when their
//!   discovered lifecycle/action contract is advertised by O3K.
//! - `/identity/me` returns identity context, not evaluated capabilities;
//!   capability truth therefore comes from service/resource discovery.

use std::collections::HashMap;

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::{
    error::{ApiError, UpstreamError},
    model::{
        ActionDescriptor, ActionRequest, ActionRiskClass, ActionSchemaMetadata, Capability,
        CapacitySummary, ColumnDescriptor, CreateResourceRequest, CustomerAccount,
        DetailsSectionDescriptor, DiscoveredResourceType, FilterDescriptor, FilterKind, JsonSchema,
        Operation, OperationError, OperationEvent, OperationState, OperatorAuditEvent,
        OperatorProfile, OperatorProject, PaginatedCollection, PlatformOverview, ProviderHealth,
        Region, RegionStatus, Resource, ResourceStatus, ResourceTypeDescriptor, SchemaReference,
        ServiceCatalogEntry, ServiceDescriptor, ServiceHealth, SessionContext,
    },
    o3k_client::{
        MutationResult, NativeOperation, NativeResourceEnvelope, O3kClient, O3kClientConfig,
        O3kClientError,
    },
    request::RequestContext,
    upstream::{
        ListOperationsParams, ListOperatorAuditEventsParams, ListOperatorOperationsParams,
        ListResourcesParams, Upstream,
    },
};

/// Adapter that calls the O3K native API.
#[derive(Clone, Debug)]
pub struct O3kAdapter {
    surface: &'static str,
    client: O3kClient,
}

impl O3kAdapter {
    const MAX_SCHEMA_DEPTH: usize = 32;
    const MAX_SCHEMA_CHILDREN: usize = 256;

    /// Build an adapter for the given surface using configuration from the
    /// environment.
    pub fn from_env(surface: &'static str) -> Result<Self, ApiError> {
        let config = O3kClientConfig::from_env()
            .map_err(|e| ApiError::Upstream(UpstreamError::Error(e.to_string())))?;
        Ok(Self::new(surface, config))
    }

    /// Build an adapter with an explicit client configuration.
    pub fn new(surface: &'static str, config: O3kClientConfig) -> Self {
        Self {
            surface,
            client: O3kClient::new(config),
        }
    }

    fn map_client_error(err: O3kClientError) -> ApiError {
        match err {
            O3kClientError::NotImplemented(msg) => ApiError::NotImplemented(msg),
            O3kClientError::Upstream { status: 404, .. } => ApiError::NotFound,
            O3kClientError::Upstream { status: 403, .. } => ApiError::Forbidden,
            O3kClientError::Upstream { status: 401, .. } => ApiError::Unauthorized,
            other => ApiError::Upstream(UpstreamError::Error(other.to_string())),
        }
    }

    fn parse_timestamp(value: Option<&str>) -> Option<OffsetDateTime> {
        value.and_then(|s| {
            time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok()
        })
    }

    fn map_status_state(state: &str) -> ResourceStatus {
        match state.to_lowercase().as_str() {
            "active" | "available" | "ready" | "running" => ResourceStatus::Ready,
            "busy" | "pending" | "stopped" | "stopping" | "starting" | "building" => {
                ResourceStatus::Busy
            }
            "error" | "failed" => ResourceStatus::Error,
            _ => ResourceStatus::Unknown,
        }
    }

    fn map_operation_state(state: &str) -> OperationState {
        match state {
            "pending" => OperationState::Pending,
            "running" => OperationState::Running,
            "succeeded" => OperationState::Succeeded,
            "failed" => OperationState::Failed,
            "retryable" => OperationState::Retryable,
            "unknown_outcome" => OperationState::UnknownOutcome,
            _ => OperationState::UnknownOutcome,
        }
    }

    fn kind_to_resource_type(kind: &str) -> Option<String> {
        let (namespace, name) = kind.split_once(':')?;
        if namespace.is_empty() || name.is_empty() {
            return None;
        }
        Some(format!("{namespace}.{name}"))
    }

    fn map_operation_action(action: &str) -> String {
        // O3K actions are PascalCase verbs with optional resource suffixes
        // (e.g. "compute:CreateServer", "volume:Delete"). Map to Araf's
        // lowercase action ids.
        let verb = action.split(':').next_back().unwrap_or(action);
        if verb == "CreateServer" || verb == "Create" {
            "create".to_owned()
        } else if verb == "DeleteServer" || verb == "Delete" {
            "delete".to_owned()
        } else if verb == "StartServer" || verb == "Start" {
            "start".to_owned()
        } else if verb == "StopServer" || verb == "Stop" {
            "stop".to_owned()
        } else {
            verb.to_lowercase()
        }
    }

    fn map_native_resource(envelope: NativeResourceEnvelope) -> Result<Resource, ApiError> {
        let resource_type = Self::kind_to_resource_type(&envelope.kind).ok_or_else(|| {
            ApiError::Upstream(UpstreamError::Error(format!(
                "unsupported native kind {}",
                envelope.kind
            )))
        })?;

        let name = envelope
            .spec
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&envelope.metadata.id)
            .to_owned();

        let status = envelope
            .status
            .get("state")
            .and_then(|v| v.as_str())
            .map(Self::map_status_state)
            .unwrap_or(ResourceStatus::Unknown);

        let mut properties = HashMap::new();
        if let serde_json::Value::Object(map) = envelope.spec {
            for (k, v) in map {
                if k != "name" {
                    properties.insert(k, v);
                }
            }
        }

        let project_id = envelope.metadata.owner_scope.ok_or_else(|| {
            ApiError::Upstream(UpstreamError::Error(
                "O3K resource omitted its authoritative owner scope".to_owned(),
            ))
        })?;
        let region_id = envelope.metadata.region.ok_or_else(|| {
            ApiError::Upstream(UpstreamError::Error(
                "O3K resource omitted its authoritative region".to_owned(),
            ))
        })?;
        let created_at = envelope
            .metadata
            .created_at
            .as_deref()
            .and_then(|value| Self::parse_timestamp(Some(value)))
            .ok_or_else(|| {
                ApiError::Upstream(UpstreamError::Error(
                    "O3K resource omitted a valid created timestamp".to_owned(),
                ))
            })?;
        let updated_at = envelope
            .metadata
            .updated_at
            .as_deref()
            .and_then(|value| Self::parse_timestamp(Some(value)))
            .ok_or_else(|| {
                ApiError::Upstream(UpstreamError::Error(
                    "O3K resource omitted a valid updated timestamp".to_owned(),
                ))
            })?;

        Ok(Resource {
            id: envelope.metadata.id,
            name,
            resource_type,
            project_id,
            region_id,
            status,
            created_at,
            updated_at,
            properties: if properties.is_empty() {
                None
            } else {
                Some(properties)
            },
        })
    }

    fn derive_events(op: &NativeOperation, correlation_id: &str) -> Vec<OperationEvent> {
        let mut events = Vec::new();

        if let Some(created_at) = Self::parse_timestamp(Some(&op.created_at)) {
            events.push(OperationEvent {
                id: format!("{}-pending", op.id),
                state: OperationState::Pending,
                occurred_at: created_at,
                message: "Operation created and pending".to_owned(),
                correlation_id: correlation_id.to_owned(),
            });
        }

        if let Some(started_at) = op
            .started_at
            .as_deref()
            .and_then(|s| Self::parse_timestamp(Some(s)))
        {
            events.push(OperationEvent {
                id: format!("{}-running", op.id),
                state: OperationState::Running,
                occurred_at: started_at,
                message: "Operation started running".to_owned(),
                correlation_id: correlation_id.to_owned(),
            });
        }

        if let Some(finished_at) = op
            .finished_at
            .as_deref()
            .and_then(|s| Self::parse_timestamp(Some(s)))
        {
            let state = Self::map_operation_state(&op.state);
            let message = match state {
                OperationState::Succeeded => "Operation completed successfully".to_owned(),
                OperationState::Failed => format!(
                    "Operation failed: {}",
                    op.error.as_deref().unwrap_or("unknown error")
                ),
                OperationState::Retryable => format!(
                    "Operation retryable: {}",
                    op.error.as_deref().unwrap_or("transient failure")
                ),
                OperationState::UnknownOutcome => format!(
                    "Operation outcome unknown: {}",
                    op.error.as_deref().unwrap_or("no response from provider")
                ),
                _ => "Operation reached terminal state".to_owned(),
            };
            events.push(OperationEvent {
                id: format!(
                    "{}-{}",
                    op.id,
                    serde_json::to_string(&state)
                        .unwrap_or_default()
                        .replace('"', "")
                ),
                state,
                occurred_at: finished_at,
                message,
                correlation_id: correlation_id.to_owned(),
            });
        }

        events
    }

    fn map_native_operation(op: NativeOperation) -> Operation {
        let correlation_id = op.request_id.clone().unwrap_or_else(|| op.id.clone());
        let state = Self::map_operation_state(&op.state);
        let error = op.error.as_ref().and_then(|e| {
            if e.is_empty() {
                None
            } else {
                Some(OperationError {
                    code: "upstream-error".to_owned(),
                    title: "Upstream operation error".to_owned(),
                    detail: e.clone(),
                })
            }
        });
        let events = Self::derive_events(&op, &correlation_id);

        Operation {
            id: op.id.clone(),
            action: Self::map_operation_action(&op.action),
            state,
            resource_id: op.resource_id,
            resource_type: Some(op.resource_type.replace(':', ".")),
            project_id: Some(op.owner_scope),
            region_id: None,
            initiated_by: Some(op.actor),
            started_at: Self::parse_timestamp(op.started_at.as_deref()),
            updated_at: Self::parse_timestamp(op.finished_at.as_deref()),
            correlation_id,
            error,
            events,
        }
    }

    async fn context_from_o3k(&self, ctx: &RequestContext) -> Result<SessionContext, ApiError> {
        let me = self
            .client_for(ctx)
            .get_identity_me()
            .await
            .map_err(Self::map_client_error)?;

        Ok(SessionContext {
            surface: self.surface,
            user_id: me.principal_id,
            user_name: me.principal_name,
            organization_id: None,
            project_id: Some(me.effective_scope_id),
            // O3K `/identity/me` does not currently return a region or
            // evaluated capabilities. Do not turn missing upstream truth into
            // a synthetic `global` region or a fixed permission set.
            region_id: None,
            capabilities: Vec::new(),
        })
    }

    fn client_for(&self, ctx: &RequestContext) -> O3kClient {
        ctx.session
            .o3k_token
            .as_deref()
            .map(|token| self.client.with_token(token))
            .unwrap_or_else(|| self.client.clone())
    }

    async fn collection_for(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
    ) -> Result<(String, String), ApiError> {
        let discovered = self
            .client_for(ctx)
            .list_resource_types()
            .await
            .map_err(Self::map_client_error)?;
        discovered
            .into_iter()
            .find(|rt| format!("{}.{}", rt.namespace, rt.name) == resource_type)
            .map(|rt| (rt.namespace, rt.collection))
            .ok_or(ApiError::NotFound)
    }

    async fn discovered_type_for(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
    ) -> Result<crate::o3k_client::DiscoveredResourceType, ApiError> {
        self.client_for(ctx)
            .list_resource_types()
            .await
            .map_err(Self::map_client_error)?
            .into_iter()
            .find(|rt| format!("{}.{}", rt.namespace, rt.name) == resource_type)
            .ok_or(ApiError::NotFound)
    }

    fn valid_discovery_identifier(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    }

    /// O3K action identifiers are namespaced (`service:Action`) while schema
    /// and contract references are absolute HTTPS URLs.  Keep both forms
    /// bounded and structural; discovery metadata is data, never executable
    /// content.
    fn valid_action_identifier(value: &str) -> bool {
        let Some((namespace, action)) = value.split_once(':') else {
            return Self::valid_discovery_identifier(value);
        };
        Self::valid_discovery_identifier(namespace) && Self::valid_discovery_identifier(action)
    }

    fn valid_contract_reference(value: &str) -> bool {
        if value.len() > 512 {
            return false;
        }
        let Ok(url) = reqwest::Url::parse(value) else {
            return false;
        };
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none_or(|fragment| {
                fragment.len() <= 256
                    && fragment.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric()
                            || matches!(byte, b'/' | b'.' | b'-' | b'_' | b'~')
                    })
            })
    }

    fn validate_discovered_service(
        service: &crate::o3k_client::DiscoveredService,
    ) -> Result<(), ApiError> {
        let valid = Self::valid_discovery_identifier(&service.id)
            && Self::valid_discovery_identifier(&service.namespace)
            && Self::valid_discovery_identifier(&service.service_version)
            && service
                .ownership
                .as_deref()
                .is_none_or(|value| value.len() <= 128 && !value.chars().any(char::is_control))
            && service
                .lifecycle_state
                .as_deref()
                .is_none_or(|value| value.len() <= 64 && !value.chars().any(char::is_control));
        if !valid {
            return Err(ApiError::BadRequest(
                "O3K returned an invalid service descriptor".to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_discovered_resource_type(
        resource_type: &crate::o3k_client::DiscoveredResourceType,
    ) -> Result<(), ApiError> {
        let valid_fields = [
            resource_type.namespace.as_str(),
            resource_type.name.as_str(),
            resource_type.service.as_str(),
            resource_type.schema_version.as_str(),
            resource_type.collection.as_str(),
            resource_type.scope.as_str(),
        ]
        .into_iter()
        .all(Self::valid_discovery_identifier);
        let valid_actions = resource_type
            .lifecycle_actions
            .iter()
            .all(|(action, action_id)| {
                Self::valid_discovery_identifier(action)
                    && action.len() <= 64
                    && Self::valid_action_identifier(action_id)
            });
        let valid_regions = resource_type
            .regions
            .iter()
            .all(|region| Self::valid_discovery_identifier(region));
        let valid_schema = resource_type.schema.as_ref().is_none_or(|schema| {
            Self::valid_contract_reference(&schema.id)
                && Self::valid_discovery_identifier(&schema.version)
                && schema.representation.len() <= 128
        });
        let valid_metadata = resource_type.actions.len() <= 128
            && resource_type.actions.iter().all(|action| {
                Self::valid_discovery_identifier(&action.name)
                    && Self::valid_action_identifier(&action.action_id)
                    && action.target.len() <= 32
                    && action
                        .input
                        .as_deref()
                        .is_none_or(Self::valid_contract_reference)
                    && action
                        .output
                        .as_deref()
                        .is_none_or(Self::valid_contract_reference)
            });
        if !valid_fields || !valid_actions || !valid_regions || !valid_schema || !valid_metadata {
            return Err(ApiError::BadRequest(
                "O3K returned an invalid resource descriptor".to_owned(),
            ));
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn compute_server_descriptor() -> ResourceTypeDescriptor {
        ResourceTypeDescriptor {
            id: "compute.server".to_owned(),
            name: "Server".to_owned(),
            plural_name: "Servers".to_owned(),
            icon_token: "server".to_owned(),
            // Kept only for legacy fixture-unit coverage. Production
            // discovery never calls this helper or relies on its static
            // presentation metadata; the live descriptor path below derives
            // schema and capabilities from O3K.
            create_schema: None,
            create_capability: Capability {
                resource_type: "compute.server".to_owned(),
                action: "create".to_owned(),
            },
            supported_actions: vec![
                ActionDescriptor {
                    id: "start".to_owned(),
                    name: "Start".to_owned(),
                    requires_confirmation: false,
                    risk_class: ActionRiskClass::Normal,
                    required_capability: Capability {
                        resource_type: "compute.server".to_owned(),
                        action: "start".to_owned(),
                    },
                    input_schema: None,
                },
                ActionDescriptor {
                    id: "stop".to_owned(),
                    name: "Stop".to_owned(),
                    requires_confirmation: true,
                    risk_class: ActionRiskClass::Disruptive,
                    required_capability: Capability {
                        resource_type: "compute.server".to_owned(),
                        action: "stop".to_owned(),
                    },
                    input_schema: None,
                },
                ActionDescriptor {
                    id: "delete".to_owned(),
                    name: "Delete".to_owned(),
                    requires_confirmation: true,
                    risk_class: ActionRiskClass::Destructive,
                    required_capability: Capability {
                        resource_type: "compute.server".to_owned(),
                        action: "delete".to_owned(),
                    },
                    input_schema: None,
                },
            ],
            columns: vec![
                ColumnDescriptor {
                    id: "name".to_owned(),
                    header: "Name".to_owned(),
                    field: "name".to_owned(),
                    width: None,
                },
                ColumnDescriptor {
                    id: "status".to_owned(),
                    header: "Status".to_owned(),
                    field: "status".to_owned(),
                    width: Some("120px".to_owned()),
                },
                ColumnDescriptor {
                    id: "region".to_owned(),
                    header: "Region".to_owned(),
                    field: "regionId".to_owned(),
                    width: Some("140px".to_owned()),
                },
                ColumnDescriptor {
                    id: "project".to_owned(),
                    header: "Project".to_owned(),
                    field: "projectId".to_owned(),
                    width: Some("140px".to_owned()),
                },
            ],
            filters: vec![
                FilterDescriptor {
                    id: "project".to_owned(),
                    label: "Project".to_owned(),
                    field: "projectId".to_owned(),
                    kind: FilterKind::Select,
                },
                FilterDescriptor {
                    id: "region".to_owned(),
                    label: "Region".to_owned(),
                    field: "regionId".to_owned(),
                    kind: FilterKind::Select,
                },
            ],
            sortable_fields: vec![
                "name".to_owned(),
                "status".to_owned(),
                "createdAt".to_owned(),
                "updatedAt".to_owned(),
            ],
            details_sections: vec![DetailsSectionDescriptor {
                id: "overview".to_owned(),
                label: "Overview".to_owned(),
                fields: vec![
                    "id".to_owned(),
                    "name".to_owned(),
                    "status".to_owned(),
                    "projectId".to_owned(),
                    "regionId".to_owned(),
                    "createdAt".to_owned(),
                    "updatedAt".to_owned(),
                ],
            }],
            relationships: vec![],
        }
    }

    #[allow(dead_code)]
    fn network_vpc_descriptor() -> ResourceTypeDescriptor {
        ResourceTypeDescriptor {
            id: "network.vpc".to_owned(),
            name: "VPC".to_owned(),
            plural_name: "VPCs".to_owned(),
            icon_token: "network".to_owned(),
            create_schema: None,
            create_capability: Capability {
                resource_type: "network.vpc".to_owned(),
                action: "create".to_owned(),
            },
            supported_actions: vec![],
            columns: vec![
                ColumnDescriptor {
                    id: "name".to_owned(),
                    header: "Name".to_owned(),
                    field: "name".to_owned(),
                    width: None,
                },
                ColumnDescriptor {
                    id: "prefix".to_owned(),
                    header: "Prefix".to_owned(),
                    field: "properties.prefix".to_owned(),
                    width: Some("160px".to_owned()),
                },
                ColumnDescriptor {
                    id: "status".to_owned(),
                    header: "Status".to_owned(),
                    field: "status".to_owned(),
                    width: Some("120px".to_owned()),
                },
                ColumnDescriptor {
                    id: "region".to_owned(),
                    header: "Region".to_owned(),
                    field: "regionId".to_owned(),
                    width: Some("140px".to_owned()),
                },
            ],
            filters: vec![
                FilterDescriptor {
                    id: "project".to_owned(),
                    label: "Project".to_owned(),
                    field: "projectId".to_owned(),
                    kind: FilterKind::Select,
                },
                FilterDescriptor {
                    id: "region".to_owned(),
                    label: "Region".to_owned(),
                    field: "regionId".to_owned(),
                    kind: FilterKind::Select,
                },
            ],
            sortable_fields: vec![
                "name".to_owned(),
                "status".to_owned(),
                "createdAt".to_owned(),
            ],
            details_sections: vec![DetailsSectionDescriptor {
                id: "overview".to_owned(),
                label: "Overview".to_owned(),
                fields: vec![
                    "id".to_owned(),
                    "name".to_owned(),
                    "status".to_owned(),
                    "projectId".to_owned(),
                    "regionId".to_owned(),
                    "properties.prefix".to_owned(),
                    "createdAt".to_owned(),
                    "updatedAt".to_owned(),
                ],
            }],
            relationships: vec![],
        }
    }

    #[allow(dead_code)]
    fn storage_volume_descriptor() -> ResourceTypeDescriptor {
        ResourceTypeDescriptor {
            id: "storage.volume".to_owned(),
            name: "Volume".to_owned(),
            plural_name: "Volumes".to_owned(),
            icon_token: "storage".to_owned(),
            create_schema: None,
            create_capability: Capability {
                resource_type: "storage.volume".to_owned(),
                action: "create".to_owned(),
            },
            supported_actions: vec![],
            columns: vec![
                ColumnDescriptor {
                    id: "name".to_owned(),
                    header: "Name".to_owned(),
                    field: "name".to_owned(),
                    width: None,
                },
                ColumnDescriptor {
                    id: "size".to_owned(),
                    header: "Size (bytes)".to_owned(),
                    field: "properties.size_bytes".to_owned(),
                    width: Some("140px".to_owned()),
                },
                ColumnDescriptor {
                    id: "status".to_owned(),
                    header: "Status".to_owned(),
                    field: "status".to_owned(),
                    width: Some("120px".to_owned()),
                },
                ColumnDescriptor {
                    id: "region".to_owned(),
                    header: "Region".to_owned(),
                    field: "regionId".to_owned(),
                    width: Some("140px".to_owned()),
                },
            ],
            filters: vec![
                FilterDescriptor {
                    id: "project".to_owned(),
                    label: "Project".to_owned(),
                    field: "projectId".to_owned(),
                    kind: FilterKind::Select,
                },
                FilterDescriptor {
                    id: "region".to_owned(),
                    label: "Region".to_owned(),
                    field: "regionId".to_owned(),
                    kind: FilterKind::Select,
                },
            ],
            sortable_fields: vec![
                "name".to_owned(),
                "status".to_owned(),
                "createdAt".to_owned(),
            ],
            details_sections: vec![DetailsSectionDescriptor {
                id: "overview".to_owned(),
                label: "Overview".to_owned(),
                fields: vec![
                    "id".to_owned(),
                    "name".to_owned(),
                    "status".to_owned(),
                    "projectId".to_owned(),
                    "regionId".to_owned(),
                    "properties.size_bytes".to_owned(),
                    "createdAt".to_owned(),
                    "updatedAt".to_owned(),
                ],
            }],
            relationships: vec![],
        }
    }

    fn descriptor_for(
        rt: &crate::o3k_client::DiscoveredResourceType,
        create_schema: Option<JsonSchema>,
    ) -> ResourceTypeDescriptor {
        let id = format!("{}.{}", rt.namespace, rt.name);
        let display_name = rt
            .name
            .split(['_', '-'])
            .map(|part| {
                let mut chars = part.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join(" ");
        let mut action_names: Vec<String> = rt
            .lifecycle_actions
            .keys()
            .filter(|action| Self::is_supported_action(action))
            .cloned()
            .collect();
        action_names.extend(rt.actions.iter().filter_map(|action| {
            Self::is_supported_action(&action.name).then_some(action.name.clone())
        }));
        action_names.sort();
        action_names.dedup();
        let supported_actions = action_names
            .into_iter()
            .map(|action| {
                let verb = Self::action_verb(&action);
                ActionDescriptor {
                    id: action.clone(),
                    name: action.clone(),
                    requires_confirmation: matches!(verb.as_str(), "delete" | "destroy"),
                    risk_class: if matches!(verb.as_str(), "delete" | "destroy") {
                        ActionRiskClass::Destructive
                    } else {
                        ActionRiskClass::Normal
                    },
                    required_capability: Capability {
                        resource_type: id.clone(),
                        action,
                    },
                    input_schema: None,
                }
            })
            .collect();

        ResourceTypeDescriptor {
            id: id.clone(),
            name: if display_name.is_empty() {
                rt.name.clone()
            } else {
                display_name
            },
            plural_name: format!("{}s", rt.name),
            icon_token: "resource".to_owned(),
            create_schema,
            create_capability: Capability {
                resource_type: id,
                action: "create".to_owned(),
            },
            supported_actions,
            columns: vec![
                ColumnDescriptor {
                    id: "id".to_owned(),
                    header: "ID".to_owned(),
                    field: "id".to_owned(),
                    width: None,
                },
                ColumnDescriptor {
                    id: "name".to_owned(),
                    header: "Name".to_owned(),
                    field: "name".to_owned(),
                    width: None,
                },
                ColumnDescriptor {
                    id: "status".to_owned(),
                    header: "Status".to_owned(),
                    field: "status".to_owned(),
                    width: None,
                },
            ],
            filters: vec![FilterDescriptor {
                id: "name".to_owned(),
                label: "Name".to_owned(),
                field: "name".to_owned(),
                kind: FilterKind::Text,
            }],
            sortable_fields: vec!["name".to_owned(), "status".to_owned()],
            details_sections: vec![DetailsSectionDescriptor {
                id: "summary".to_owned(),
                label: "Summary".to_owned(),
                fields: vec!["id".to_owned(), "name".to_owned(), "status".to_owned()],
            }],
            relationships: vec![],
        }
    }

    fn action_verb(action: &str) -> String {
        action
            .rsplit_once(':')
            .map(|(_, verb)| verb)
            .unwrap_or(action)
            .trim_end_matches("Server")
            .trim_end_matches("Instance")
            .trim_end_matches("Resource")
            .trim_end_matches("s")
            .to_ascii_lowercase()
    }

    fn is_supported_action(action: &str) -> bool {
        matches!(
            Self::action_verb(action).as_str(),
            "start" | "stop" | "reboot" | "delete" | "destroy"
        )
    }

    fn service_name_from_id(id: &str) -> String {
        id.split('.')
            .next()
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => s.to_owned(),
                }
            })
            .unwrap_or_else(|| id.to_owned())
    }

    fn map_discovered_service(s: crate::o3k_client::DiscoveredService) -> ServiceCatalogEntry {
        ServiceCatalogEntry {
            id: s.id.clone(),
            namespace: s.namespace.clone(),
            name: Self::service_name_from_id(&s.id),
            version: s.service_version.clone(),
            ownership: s.ownership.clone(),
            lifecycle_state: s
                .lifecycle_state
                .clone()
                .unwrap_or_else(|| "unknown".to_owned()),
            capabilities: vec![],
            regions: vec![],
            description: None,
            documentation_url: None,
        }
    }

    fn map_discovered_resource_type(
        rt: crate::o3k_client::DiscoveredResourceType,
    ) -> DiscoveredResourceType {
        DiscoveredResourceType {
            namespace: rt.namespace,
            name: rt.name,
            service_id: rt.service,
            schema_version: rt.schema_version,
            collection: rt.collection,
            scope: rt.scope,
            ready: rt.ready,
            lifecycle_actions: rt.lifecycle_actions,
            placement: rt.placement,
            regions: rt.regions,
            availability_domain_selection: rt.availability_domain_selection,
            schema: rt.schema.map(|schema| SchemaReference {
                id: schema.id,
                version: schema.version,
                representation: schema.representation,
            }),
            actions: rt
                .actions
                .into_iter()
                .map(|action| ActionSchemaMetadata {
                    name: action.name,
                    action_id: action.action_id,
                    target: action.target,
                    input: action.input,
                    output: action.output,
                    asynchronous: action.asynchronous,
                })
                .collect(),
        }
    }

    fn extract_create_schema(value: serde_json::Value) -> Option<JsonSchema> {
        // The O3K schema projection is an envelope document. The generic
        // Araf form runtime consumes only the authoritative `spec` schema;
        // retain it as data and never execute schema-provided content.
        let schema = value
            .get("allOf")
            .and_then(serde_json::Value::as_array)
            .and_then(|parts| parts.get(1))
            .and_then(|part| part.get("properties"))
            .and_then(|properties| properties.get("spec"))
            .cloned()?;
        Self::schema_within_bounds(&schema, 0).then_some(JsonSchema(schema))
    }

    fn schema_within_bounds(value: &serde_json::Value, depth: usize) -> bool {
        if depth > Self::MAX_SCHEMA_DEPTH {
            return false;
        }
        match value {
            serde_json::Value::Array(items) => {
                items.len() <= Self::MAX_SCHEMA_CHILDREN
                    && items
                        .iter()
                        .all(|item| Self::schema_within_bounds(item, depth + 1))
            }
            serde_json::Value::Object(properties) => {
                properties.len() <= Self::MAX_SCHEMA_CHILDREN
                    && properties
                        .values()
                        .all(|item| Self::schema_within_bounds(item, depth + 1))
            }
            _ => true,
        }
    }
}

#[async_trait]
impl Upstream for O3kAdapter {
    fn surface(&self) -> &'static str {
        self.surface
    }

    async fn context(&self, ctx: &RequestContext) -> Result<SessionContext, ApiError> {
        self.context_from_o3k(ctx).await
    }

    async fn services(&self, ctx: &RequestContext) -> Result<Vec<ServiceDescriptor>, ApiError> {
        let discovered = self
            .client_for(ctx)
            .list_services()
            .await
            .map_err(Self::map_client_error)?;
        for service in &discovered {
            Self::validate_discovered_service(service)?;
        }
        let resource_types = self
            .client_for(ctx)
            .list_resource_types()
            .await
            .map_err(Self::map_client_error)?;

        let mut descriptors_by_service: HashMap<String, Vec<ResourceTypeDescriptor>> =
            HashMap::new();

        // Map only resource types actually advertised by O3K. Araf may know how
        // to render a descriptor, but that knowledge is not evidence that the
        // capability exists in the connected cloud. Keep not-ready or
        // read-incomplete types in the raw discovery projection for truthful
        // diagnostics, but do not turn them into executable tenant routes.
        for rt in resource_types {
            Self::validate_discovered_resource_type(&rt)?;
            if !rt.ready
                || !rt.lifecycle_actions.contains_key("list")
                || !rt.lifecycle_actions.contains_key("show")
            {
                continue;
            }
            let create_schema = if rt.lifecycle_actions.contains_key("create") {
                self.client_for(ctx)
                    .get_resource_schema(&rt.namespace, &rt.collection, &rt.schema_version)
                    .await
                    .ok()
                    .and_then(Self::extract_create_schema)
            } else {
                None
            };
            descriptors_by_service
                .entry(rt.service.clone())
                .or_default()
                .push(Self::descriptor_for(&rt, create_schema));
        }

        let services: Vec<ServiceDescriptor> = discovered
            .into_iter()
            .map(|s| {
                let service_id = s.id;
                let (name, category) = match service_id.as_str() {
                    "compute" => ("Compute", "Services"),
                    "network" => ("Networking", "Services"),
                    "volume" => ("Storage", "Services"),
                    _ => ("Other", "Services"),
                };
                ServiceDescriptor {
                    id: service_id.clone(),
                    name: name.to_owned(),
                    category: category.to_owned(),
                    resource_types: descriptors_by_service
                        .remove(&service_id)
                        .unwrap_or_default(),
                }
            })
            .collect();

        Ok(services)
    }

    async fn list_discovered_services(
        &self,
        ctx: &RequestContext,
    ) -> Result<Vec<ServiceCatalogEntry>, ApiError> {
        // Discovery authorization is authoritative in O3K.  `/identity/me`
        // intentionally does not claim to be an evaluated capability
        // document, so a local capability check here would reject valid
        // production sessions before O3K can authorize the request.
        let discovered = self
            .client_for(ctx)
            .list_services()
            .await
            .map_err(Self::map_client_error)?;
        for service in &discovered {
            Self::validate_discovered_service(service)?;
        }
        let resource_types = self
            .client_for(ctx)
            .list_resource_types()
            .await
            .map_err(Self::map_client_error)?;
        for resource_type in &resource_types {
            Self::validate_discovered_resource_type(resource_type)?;
        }
        let mut regions_by_service: HashMap<String, Vec<String>> = HashMap::new();
        for resource_type in resource_types {
            let regions = regions_by_service.entry(resource_type.service).or_default();
            regions.extend(resource_type.regions);
            regions.sort();
            regions.dedup();
        }
        Ok(discovered
            .into_iter()
            .map(|service| {
                let mut entry = Self::map_discovered_service(service.clone());
                entry.regions = regions_by_service.remove(&service.id).unwrap_or_default();
                entry
            })
            .collect())
    }

    async fn list_discovered_resource_types(
        &self,
        ctx: &RequestContext,
    ) -> Result<Vec<DiscoveredResourceType>, ApiError> {
        // As with service discovery, O3K authorizes this route.  Do not
        // substitute an incomplete `/identity/me` capability projection for
        // that decision.
        let resource_types = self
            .client_for(ctx)
            .list_resource_types()
            .await
            .map_err(Self::map_client_error)?;
        for resource_type in &resource_types {
            Self::validate_discovered_resource_type(resource_type)?;
        }
        Ok(resource_types
            .into_iter()
            .map(Self::map_discovered_resource_type)
            .collect())
    }

    async fn list_resources(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        params: ListResourcesParams,
    ) -> Result<PaginatedCollection<Resource>, ApiError> {
        let page_size = params.page_size.clamp(1, 100);

        let (namespace, collection) = self.collection_for(ctx, resource_type).await?;
        let mut cursor: Option<String> = None;
        let mut response = None;
        let target_page = params.page.min(100);
        for page_index in 0..=target_page {
            let next = self
                .client_for(ctx)
                .list_generic_resources(&namespace, &collection, Some(page_size), cursor.as_deref())
                .await
                .map_err(Self::map_client_error)?;
            let next_cursor = next.next_cursor.clone();
            response = Some(next);
            if response.as_ref().is_some_and(|r| r.next_cursor.is_none())
                && page_index < target_page
            {
                return Ok(PaginatedCollection {
                    items: vec![],
                    total: 0,
                    page: params.page,
                    page_size,
                    has_more: false,
                });
            }
            if page_index < target_page {
                if next_cursor.as_deref() == cursor.as_deref() {
                    return Err(ApiError::Upstream(UpstreamError::Error(
                        "O3K returned a repeated pagination cursor".to_owned(),
                    )));
                }
                cursor = next_cursor;
            }
        }
        let response = response.expect("bounded discovery loop always executes");
        let items: Vec<Resource> = response
            .items
            .into_iter()
            .map(|value| {
                serde_json::from_value::<NativeResourceEnvelope>(value)
                    .map_err(|e| ApiError::Upstream(UpstreamError::Error(e.to_string())))
                    .and_then(Self::map_native_resource)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PaginatedCollection {
            total: items.len() as u64 + u64::from(response.next_cursor.is_some()),
            has_more: response.next_cursor.is_some(),
            items,
            page: params.page,
            page_size,
        })
    }

    async fn get_resource(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        id: &str,
    ) -> Result<Resource, ApiError> {
        let (namespace, collection) = self.collection_for(ctx, resource_type).await?;
        let envelope = self
            .client_for(ctx)
            .get_generic_resource(&namespace, &collection, id)
            .await
            .map_err(Self::map_client_error)?;
        Self::map_native_resource(envelope)
    }

    async fn submit_action(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        id: &str,
        request: ActionRequest,
    ) -> Result<Operation, ApiError> {
        let discovered = self.discovered_type_for(ctx, resource_type).await?;
        let action_advertised = discovered
            .lifecycle_actions
            .contains_key(&request.action_id)
            || discovered.actions.iter().any(|action| {
                action.name == request.action_id || action.action_id == request.action_id
            });
        if !action_advertised {
            return Err(ApiError::BadRequest(format!(
                "action {} is not advertised for {}",
                request.action_id, resource_type
            )));
        }
        let namespace = discovered.namespace;
        let collection = discovered.collection;
        let result = if request.action_id == "delete" {
            self.client_for(ctx)
                .delete_generic_resource(&namespace, &collection, id)
                .await
        } else {
            self.client_for(ctx)
                .invoke_generic_action(
                    &namespace,
                    &collection,
                    id,
                    &request.action_id,
                    request
                        .payload
                        .unwrap_or(serde_json::Value::Object(Default::default())),
                )
                .await
        }
        .map_err(Self::map_client_error)?;

        self.fetch_or_build_operation(result, ctx).await
    }

    async fn create_resource(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        request: CreateResourceRequest,
    ) -> Result<Operation, ApiError> {
        let discovered = self.discovered_type_for(ctx, resource_type).await?;
        if !discovered.lifecycle_actions.contains_key("create") {
            return Err(ApiError::BadRequest(format!(
                "create is not advertised for {resource_type}"
            )));
        }
        let namespace = discovered.namespace;
        let collection = discovered.collection;
        let kind = format!(
            "{}:{}",
            namespace,
            resource_type.split('.').next_back().unwrap_or(&collection)
        );
        let result = self
            .client_for(ctx)
            .create_generic_resource(&namespace, &collection, &kind, request.payload)
            .await
            .map_err(Self::map_client_error)?;

        self.fetch_or_build_operation(result, ctx).await
    }

    async fn list_operations(
        &self,
        _ctx: &RequestContext,
        _params: ListOperationsParams,
    ) -> Result<PaginatedCollection<Operation>, ApiError> {
        // M7-O3K-002: O3K has no `GET /o3k/v1/operations` endpoint.
        Err(ApiError::NotImplemented(
            "O3K does not expose a list operations endpoint in M7".to_owned(),
        ))
    }

    async fn get_operation(&self, ctx: &RequestContext, id: &str) -> Result<Operation, ApiError> {
        let op = self
            .client_for(ctx)
            .get_operation(id)
            .await
            .map_err(Self::map_client_error)?;
        Ok(Self::map_native_operation(op))
    }

    async fn list_regions(&self, ctx: &RequestContext) -> Result<Vec<Region>, ApiError> {
        let regions = self
            .client_for(ctx)
            .list_regions()
            .await
            .map_err(Self::map_client_error)?;
        Ok(regions
            .into_iter()
            .map(|region| Region {
                id: region.id.clone(),
                name: region.id.clone(),
                status: RegionStatus::Healthy,
                azs: region
                    .availability_domains
                    .into_iter()
                    .map(|az| crate::model::AvailabilityZone {
                        id: az.id.clone(),
                        name: az.id,
                        region_id: region.id.clone(),
                        status: RegionStatus::Healthy,
                    })
                    .collect(),
                updated_at: OffsetDateTime::UNIX_EPOCH,
            })
            .collect())
    }

    async fn list_availability_zones(
        &self,
        ctx: &RequestContext,
        region_id: &str,
    ) -> Result<Vec<crate::model::AvailabilityZone>, ApiError> {
        Ok(self
            .list_regions(ctx)
            .await?
            .into_iter()
            .find(|region| region.id == region_id)
            .map(|region| region.azs)
            .unwrap_or_default())
    }

    async fn list_provider_health(
        &self,
        _ctx: &RequestContext,
    ) -> Result<Vec<ProviderHealth>, ApiError> {
        Err(ApiError::NotImplemented(
            "O3K does not expose a provider health endpoint".to_owned(),
        ))
    }

    async fn list_service_health(
        &self,
        _ctx: &RequestContext,
    ) -> Result<Vec<ServiceHealth>, ApiError> {
        Err(ApiError::NotImplemented(
            "O3K does not expose a service health endpoint".to_owned(),
        ))
    }

    async fn get_capacity_summary(
        &self,
        _ctx: &RequestContext,
    ) -> Result<Vec<CapacitySummary>, ApiError> {
        Err(ApiError::NotImplemented(
            "O3K does not expose a normalized capacity summary endpoint".to_owned(),
        ))
    }

    async fn list_customer_accounts(
        &self,
        _ctx: &RequestContext,
    ) -> Result<PaginatedCollection<CustomerAccount>, ApiError> {
        Err(ApiError::NotImplemented(
            "O3K does not expose a customer account enumeration endpoint".to_owned(),
        ))
    }

    async fn list_operator_projects(
        &self,
        _ctx: &RequestContext,
        _account_id: Option<&str>,
    ) -> Result<PaginatedCollection<OperatorProject>, ApiError> {
        Err(ApiError::NotImplemented(
            "O3K does not expose an operator-scope project enumeration endpoint".to_owned(),
        ))
    }

    async fn list_operator_operations(
        &self,
        _ctx: &RequestContext,
        _params: ListOperatorOperationsParams,
    ) -> Result<PaginatedCollection<Operation>, ApiError> {
        Err(ApiError::NotImplemented(
            "O3K does not expose a global operator operations list endpoint".to_owned(),
        ))
    }

    async fn list_operator_audit_events(
        &self,
        _ctx: &RequestContext,
        _params: ListOperatorAuditEventsParams,
    ) -> Result<PaginatedCollection<OperatorAuditEvent>, ApiError> {
        Err(ApiError::NotImplemented(
            "O3K does not expose an operator audit event query endpoint".to_owned(),
        ))
    }

    async fn get_platform_overview(
        &self,
        _ctx: &RequestContext,
    ) -> Result<PlatformOverview, ApiError> {
        Err(ApiError::NotImplemented(
            "O3K does not expose a platform overview endpoint".to_owned(),
        ))
    }

    async fn get_operator_profile(
        &self,
        ctx: &RequestContext,
    ) -> Result<OperatorProfile, ApiError> {
        let profile = self
            .client_for(ctx)
            .get_operator_profile()
            .await
            .map_err(Self::map_client_error)?;
        Ok(OperatorProfile {
            profile: profile.profile,
            scope: profile.scope,
            principal_id: profile.principal_id,
            audit_id: profile.audit_id,
        })
    }
}

impl O3kAdapter {
    async fn fetch_or_build_operation(
        &self,
        result: MutationResult,
        ctx: &RequestContext,
    ) -> Result<Operation, ApiError> {
        // O3K is authoritative for operation state. Never turn an unavailable
        // operation lookup into a synthetic pending operation: that would make
        // an upstream failure look like a real cloud operation.
        let op = self
            .client_for(ctx)
            .get_operation(&result.operation_id)
            .await
            .map_err(Self::map_client_error)?;
        let mut mapped = Self::map_native_operation(op);
        mapped.correlation_id = ctx.correlation_id().to_owned();
        Ok(mapped)
    }
}

#[cfg(test)]
mod resource_mapping_tests {
    use super::*;

    fn envelope(metadata: serde_json::Value) -> NativeResourceEnvelope {
        serde_json::from_value(serde_json::json!({
            "api_version": "o3k.io/v1",
            "kind": "compute:server",
            "metadata": metadata,
            "spec": {"name": "server-1"},
            "status": {"state": "active"}
        }))
        .expect("test envelope is valid")
    }

    #[test]
    fn resource_mapping_rejects_missing_authoritative_metadata() {
        let metadata = serde_json::json!({
            "id": "server-1",
            "owner_scope": "project-1",
            "generation": 1,
            "created_at": "2026-09-07T00:00:00Z",
            "updated_at": "2026-09-07T00:00:00Z"
        });

        let error = O3kAdapter::map_native_resource(envelope(metadata))
            .expect_err("missing region must not become a fabricated global resource");
        assert!(matches!(error, ApiError::Upstream(_)));
    }

    #[test]
    fn resource_mapping_rejects_missing_timestamps() {
        let metadata = serde_json::json!({
            "id": "server-1",
            "owner_scope": "project-1",
            "generation": 1,
            "region": "eu-west"
        });

        let error = O3kAdapter::map_native_resource(envelope(metadata))
            .expect_err("missing timestamps must not become current-time metadata");
        assert!(matches!(error, ApiError::Upstream(_)));
    }
}

#[cfg(test)]
mod discovery_validation_tests {
    use super::O3kAdapter;
    use crate::{error::ApiError, o3k_client::O3kClientError};

    #[test]
    fn accepts_converged_o3k_action_and_schema_references() {
        assert!(O3kAdapter::valid_action_identifier("compute:CreateServer"));
        assert!(O3kAdapter::valid_contract_reference(
            "https://o3k.io/schemas/compute/servers/v1/resource#/allOf/1/properties/spec"
        ));
    }

    #[test]
    fn rejects_unsafe_discovery_references() {
        assert!(!O3kAdapter::valid_action_identifier(
            "compute:Create Server"
        ));
        assert!(!O3kAdapter::valid_contract_reference(
            "http://o3k.io/contracts/native-resource-envelope-v1.schema.json"
        ));
        assert!(!O3kAdapter::valid_contract_reference(
            "https://o3k.io/schemas/resource?redirect=https://evil.example"
        ));
    }

    #[test]
    fn preserves_forbidden_discovery_errors() {
        let error = O3kAdapter::map_client_error(O3kClientError::Upstream {
            status: 403,
            title: "Forbidden".to_owned(),
            detail: "not authorized".to_owned(),
        });
        assert!(matches!(error, ApiError::Forbidden));
    }

    #[test]
    fn rejects_pathological_schema_depth() {
        let mut nested = serde_json::json!({"type": "string"});
        for _ in 0..=O3kAdapter::MAX_SCHEMA_DEPTH {
            nested = serde_json::json!({"properties": {"nested": nested}});
        }
        let envelope = serde_json::json!({
            "allOf": [{}, {"properties": {"spec": nested}}]
        });
        assert!(O3kAdapter::extract_create_schema(envelope).is_none());
    }
}
