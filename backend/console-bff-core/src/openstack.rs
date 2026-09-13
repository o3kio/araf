//! OpenStack compatibility backend.
//!
//! This adapter deliberately translates Keystone/Nova/Glance/Neutron/Cinder
//! wire shapes at the BFF boundary.  The browser only sees Araf resource
//! descriptors and resources.  OpenStack is authoritative for resource state;
//! the operation values returned here are correlation artifacts and are never
//! used to overwrite a subsequent resource read.

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use reqwest::{Client, Method, StatusCode, Url};
use serde_json::{json, Value};
use time::OffsetDateTime;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    cloud_backend::{BackendKind, CloudBackend},
    compatibility::{CompatibilityJournal, CompatibilityRecord},
    error::{ApiError, UpstreamError},
    model::{
        ActionDescriptor, ActionRequest, ActionRiskClass, Capability, ColumnDescriptor,
        CreateResourceRequest, DetailsSectionDescriptor, FilterDescriptor, FilterKind, JsonSchema,
        Operation, OperationError, OperationEvent, OperationState, PaginatedCollection, Project,
        ProjectQuota, QuotaEntry, RelationshipDescriptor, RelationshipDirection, Resource,
        ResourceStatus, ResourceTypeDescriptor, ServiceDescriptor, SessionContext,
    },
    request::RequestContext,
    upstream::{ListOperationsParams, ListResourcesParams, Upstream},
};

const MAX_PAGE_SIZE: u32 = 100;
const HTTP_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Debug)]
pub struct OpenStackClientConfig {
    pub auth_url: String,
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub user_domain: String,
    pub project_name: Option<String>,
    pub project_id: Option<String>,
    pub region: Option<String>,
    pub compute_url: Option<String>,
    pub image_url: Option<String>,
    pub network_url: Option<String>,
    pub volume_url: Option<String>,
    pub object_storage_url: Option<String>,
}

impl OpenStackClientConfig {
    pub fn from_env() -> Result<Self, ApiError> {
        let auth_url = std::env::var("OPENSTACK_AUTH_URL")
            .map_err(|_| config_error("OPENSTACK_AUTH_URL is required"))?;
        let parsed = Url::parse(auth_url.trim())
            .map_err(|_| config_error("OPENSTACK_AUTH_URL must be an absolute URL"))?;
        if parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return Err(config_error(
                "OPENSTACK_AUTH_URL must be credential-free and host-qualified",
            ));
        }
        Ok(Self {
            auth_url: auth_url.trim().trim_end_matches('/').to_owned(),
            token: std::env::var("OPENSTACK_TOKEN")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            username: std::env::var("OPENSTACK_USERNAME")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            password: std::env::var("OPENSTACK_PASSWORD")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            user_domain: std::env::var("OPENSTACK_USER_DOMAIN_NAME")
                .unwrap_or_else(|_| "Default".into()),
            project_name: std::env::var("OPENSTACK_PROJECT_NAME")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            project_id: std::env::var("OPENSTACK_PROJECT_ID")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            region: std::env::var("OPENSTACK_REGION")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            compute_url: env_url("OPENSTACK_COMPUTE_URL")?,
            image_url: env_url("OPENSTACK_IMAGE_URL")?,
            network_url: env_url("OPENSTACK_NETWORK_URL")?,
            volume_url: env_url("OPENSTACK_VOLUME_URL")?,
            object_storage_url: env_url("OPENSTACK_OBJECT_STORAGE_URL")?,
        })
    }
}

fn env_url(name: &str) -> Result<Option<String>, ApiError> {
    let Some(value) = std::env::var(name).ok().filter(|v| !v.trim().is_empty()) else {
        return Ok(None);
    };
    let parsed = Url::parse(value.trim())
        .map_err(|_| config_error(format!("{name} must be an absolute URL")))?;
    if parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(config_error(format!(
            "{name} must be credential-free and host-qualified"
        )));
    }
    Ok(Some(value.trim().trim_end_matches('/').to_owned()))
}

fn config_error(message: impl Into<String>) -> ApiError {
    ApiError::Upstream(UpstreamError::Error(message.into()))
}

#[derive(Clone, Debug)]
pub struct OpenStackAdapter {
    surface: &'static str,
    config: OpenStackClientConfig,
    client: Client,
    journal: Option<Arc<RwLock<CompatibilityJournal>>>,
    catalog: Arc<std::sync::RwLock<HashMap<String, String>>>,
}

impl OpenStackAdapter {
    pub fn from_env(surface: &'static str) -> Result<Self, ApiError> {
        let config = OpenStackClientConfig::from_env()?;
        let journal = std::env::var("ARAF_OPENSTACK_COMPATIBILITY_JOURNAL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|path| {
                CompatibilityJournal::open(path)
                    .map(|journal| Arc::new(RwLock::new(journal)))
                    .map_err(|error| {
                        config_error(format!("cannot open compatibility journal: {error}"))
                    })
            })
            .transpose()?;
        Self::new_with_journal(surface, config, journal)
    }

    pub fn new(surface: &'static str, config: OpenStackClientConfig) -> Result<Self, ApiError> {
        Self::new_with_journal(surface, config, None)
    }

    fn new_with_journal(
        surface: &'static str,
        config: OpenStackClientConfig,
        journal: Option<Arc<RwLock<CompatibilityJournal>>>,
    ) -> Result<Self, ApiError> {
        let client = Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .map_err(|e| config_error(e.to_string()))?;
        Ok(Self {
            surface,
            config,
            client,
            journal,
            catalog: Arc::new(std::sync::RwLock::new(HashMap::new())),
        })
    }

    fn token<'a>(&'a self, ctx: &'a RequestContext) -> Option<&'a str> {
        ctx.session
            .openstack_token
            .as_deref()
            .or(self.config.token.as_deref())
    }

    fn keystone_url(&self, resource: &str) -> Result<Url, ApiError> {
        let base = self.config.auth_url.trim_end_matches('/');
        let path = if base.ends_with("/v3") {
            format!("{base}/auth/{resource}")
        } else {
            format!("{base}/v3/auth/{resource}")
        };
        Url::parse(&path).map_err(|error| config_error(error.to_string()))
    }

    async fn server_token(&self, ctx: &RequestContext) -> Result<String, ApiError> {
        if let Some(token) = self.token(ctx) {
            return Ok(token.to_owned());
        }
        let (Some(username), Some(password)) = (&self.config.username, &self.config.password)
        else {
            return Err(ApiError::Unauthorized);
        };
        let url = self.keystone_url("tokens")?;
        let body = json!({"auth":{"identity":{"methods":["password"],"password":{"user":{"name":username,"domain":{"name":self.config.user_domain},"password":password}}},"scope": self.config.project_id.as_ref().map(|id| json!({"project":{"id":id}})).or_else(|| self.config.project_name.as_ref().map(|name| json!({"project":{"name":name,"domain":{"name":self.config.user_domain}}})))}});
        let response = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| config_error(e.to_string()))?;
        if !response.status().is_success() {
            return Err(map_status(response.status(), &Value::Null));
        }
        response
            .headers()
            .get("X-Subject-Token")
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned)
            .ok_or(ApiError::Unauthorized)
    }

    fn project<'a>(
        &'a self,
        ctx: &'a RequestContext,
        requested: Option<&'a str>,
    ) -> Option<&'a str> {
        requested
            .or(ctx.session.openstack_project_id.as_deref())
            .or(self.config.project_id.as_deref())
    }

    fn endpoint(&self, resource_type: &str) -> Option<(String, &'static str)> {
        let catalog_url = |service: &str| {
            self.catalog
                .read()
                .ok()
                .and_then(|catalog| catalog.get(service).cloned())
        };
        match resource_type {
            "compute.server" => self
                .config
                .compute_url
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| catalog_url("compute"))
                .map(|u| (u, "servers/detail")),
            "compute.flavor" => self
                .config
                .compute_url
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| catalog_url("compute"))
                .map(|u| (u, "flavors/detail")),
            "image.image" => self
                .config
                .image_url
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| catalog_url("image"))
                .map(|u| (u, "v2/images")),
            "network.network" => self
                .config
                .network_url
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| catalog_url("network"))
                .map(|u| (u, "v2.0/networks")),
            "network.subnet" => self
                .config
                .network_url
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| catalog_url("network"))
                .map(|u| (u, "v2.0/subnets")),
            "network.port" => self
                .config
                .network_url
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| catalog_url("network"))
                .map(|u| (u, "v2.0/ports")),
            "network.security-group" => self
                .config
                .network_url
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| catalog_url("network"))
                .map(|u| (u, "v2.0/security-groups")),
            "block.volume" => self
                .config
                .volume_url
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| catalog_url("volume"))
                .map(|u| (u, "volumes/detail")),
            "object.storage.bucket" => self
                .config
                .object_storage_url
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| catalog_url("object-store"))
                .map(|u| (u, "containers")),
            _ => None,
        }
    }

    fn update_catalog(&self, value: &Value) {
        let mut discovered = HashMap::new();
        if let Some(entries) = value.get("catalog").and_then(Value::as_array) {
            for entry in entries {
                let Some(kind) = entry.get("type").and_then(Value::as_str) else {
                    continue;
                };
                let Some(endpoints) = entry.get("endpoints").and_then(Value::as_array) else {
                    continue;
                };
                let selected = endpoints
                    .iter()
                    .find(|endpoint| {
                        endpoint.get("interface").and_then(Value::as_str) == Some("public")
                            && self.config.region.as_deref().is_none_or(|region| {
                                endpoint.get("region").and_then(Value::as_str) == Some(region)
                            })
                    })
                    .or_else(|| endpoints.first());
                if let Some(url) = selected
                    .and_then(|endpoint| endpoint.get("url"))
                    .and_then(Value::as_str)
                {
                    let key = match kind {
                        "compute" => "compute",
                        "image" => "image",
                        "network" => "network",
                        "volumev3" | "volume" => "volume",
                        "object-store" => "object-store",
                        _ => continue,
                    };
                    discovered.insert(key.to_owned(), url.trim_end_matches('/').to_owned());
                }
            }
        }
        if let Ok(mut catalog) = self.catalog.write() {
            catalog.extend(discovered);
        }
    }

    fn scoped_base(&self, base: &str, project: Option<&str>) -> String {
        project
            .map(|id| base.replace("%(tenant_id)s", id))
            .unwrap_or_else(|| base.to_owned())
    }

    async fn discover_catalog(&self, ctx: &RequestContext) -> Result<(), ApiError> {
        let url = self.keystone_url("catalog")?;
        let (_, value) = self.request_json(ctx, Method::GET, url, None).await?;
        self.update_catalog(&value);
        Ok(())
    }

    async fn request_json(
        &self,
        ctx: &RequestContext,
        method: Method,
        url: Url,
        body: Option<Value>,
    ) -> Result<(StatusCode, Value), ApiError> {
        let mut request = self.client.request(method, url);
        let token = self.server_token(ctx).await?;
        request = request.header("X-Auth-Token", token);
        if let Some(region) = &self.config.region {
            request = request.header("X-OpenStack-Request-ID", format!("araf-region-{region}"));
        }
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request
            .send()
            .await
            .map_err(|e| ApiError::Upstream(UpstreamError::Error(e.to_string())))?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|e| ApiError::Upstream(UpstreamError::Error(e.to_string())))?;
        let value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| json!({"detail": String::from_utf8_lossy(&bytes)}))
        };
        if !status.is_success() {
            return Err(map_status(status, &value));
        }
        Ok((status, value))
    }

    fn operation(
        &self,
        ctx: &RequestContext,
        action: &str,
        resource_type: &str,
        resource_id: Option<String>,
        state: OperationState,
        error: Option<OperationError>,
    ) -> Operation {
        let id = format!("openstack-compat-{}", Uuid::new_v4());
        let now = OffsetDateTime::now_utc();
        let event = OperationEvent {
            id: format!("{id}-state"),
            state,
            occurred_at: now,
            message: "Derived from authoritative OpenStack response".into(),
            correlation_id: ctx.correlation_id().into(),
        };
        Operation {
            id,
            action: action.into(),
            state,
            resource_id,
            resource_type: Some(resource_type.into()),
            project_id: self.project(ctx, None).map(ToOwned::to_owned),
            region_id: self.config.region.clone(),
            initiated_by: ctx.session.user_id.clone(),
            started_at: Some(now),
            updated_at: Some(now),
            correlation_id: ctx.correlation_id().into(),
            error,
            events: vec![event],
        }
    }

    async fn persist_operation(&self, operation: &Operation) -> Result<(), ApiError> {
        let Some(journal) = &self.journal else {
            return Ok(());
        };
        let record = CompatibilityRecord {
            operation_id: operation.id.clone(),
            action: operation.action.clone(),
            resource_type: operation.resource_type.clone().unwrap_or_default(),
            resource_id: operation.resource_id.clone(),
            project_id: operation.project_id.clone(),
            correlation_id: operation.correlation_id.clone(),
            state: operation.state,
            observed_status: None,
            updated_at: operation.updated_at.unwrap_or_else(OffsetDateTime::now_utc),
        };
        journal.write().await.insert(record).map_err(|error| {
            config_error(format!("cannot persist compatibility operation: {error}"))
        })
    }

    fn operation_from_record(record: &CompatibilityRecord) -> Operation {
        let occurred_at = record.updated_at;
        Operation {
            id: record.operation_id.clone(),
            action: record.action.clone(),
            state: record.state,
            resource_id: record.resource_id.clone(),
            resource_type: Some(record.resource_type.clone()),
            project_id: record.project_id.clone(),
            region_id: None,
            initiated_by: None,
            started_at: Some(occurred_at),
            updated_at: Some(occurred_at),
            correlation_id: record.correlation_id.clone(),
            error: None,
            events: vec![OperationEvent {
                id: format!("{}-state", record.operation_id),
                state: record.state,
                occurred_at,
                message: record
                    .observed_status
                    .as_deref()
                    .map(|status| format!("Observed OpenStack status: {status}"))
                    .unwrap_or_else(|| "Awaiting authoritative OpenStack state".into()),
                correlation_id: record.correlation_id.clone(),
            }],
        }
    }

    /// Reconcile a persisted compatibility operation against an authoritative
    /// OpenStack resource observation. This is safe to call after a BFF restart.
    pub async fn reconcile_operation(
        &self,
        ctx: &RequestContext,
        operation_id: &str,
    ) -> Result<Option<Operation>, ApiError> {
        let Some(journal) = &self.journal else {
            return Ok(None);
        };
        let record = journal.read().await.get(operation_id).cloned();
        let Some(record) = record else {
            return Ok(None);
        };
        let Some(resource_id) = record.resource_id.as_deref() else {
            return Ok(Some(Self::operation_from_record(&record)));
        };
        let (state, status) = match self
            .get_resource(ctx, &record.resource_type, resource_id)
            .await
        {
            Ok(resource) => {
                let status = resource
                    .properties
                    .as_ref()
                    .and_then(|properties| properties.get("status"))
                    .and_then(Value::as_str)
                    .unwrap_or("UNKNOWN");
                let state = match record.action.as_str() {
                    "delete" => OperationState::Running,
                    "start" | "reboot" if resource.status == ResourceStatus::Ready => {
                        OperationState::Succeeded
                    }
                    "stop"
                        if matches!(
                            resource.status,
                            ResourceStatus::Busy | ResourceStatus::Ready
                        ) =>
                    {
                        OperationState::Succeeded
                    }
                    "create" if resource.status == ResourceStatus::Error => OperationState::Failed,
                    "create" if resource.status == ResourceStatus::Ready => {
                        OperationState::Succeeded
                    }
                    _ if resource.status == ResourceStatus::Error => OperationState::Failed,
                    _ => OperationState::Running,
                };
                (state, status.to_owned())
            }
            Err(ApiError::NotFound) if record.action == "delete" => {
                (OperationState::Succeeded, "DELETED".into())
            }
            Err(_error) => {
                journal
                    .write()
                    .await
                    .update_authoritative_state(
                        operation_id,
                        "UNKNOWN",
                        OperationState::UnknownOutcome,
                    )
                    .map_err(|error| {
                        config_error(format!("cannot update compatibility operation: {error}"))
                    })?;
                return Ok(journal
                    .read()
                    .await
                    .get(operation_id)
                    .map(Self::operation_from_record));
            }
        };
        journal
            .write()
            .await
            .update_authoritative_state(operation_id, &status, state)
            .map_err(|error| {
                config_error(format!("cannot update compatibility operation: {error}"))
            })?;
        Ok(journal
            .read()
            .await
            .get(operation_id)
            .map(Self::operation_from_record))
    }

    fn descriptor(resource_type: &str) -> Option<ResourceTypeDescriptor> {
        let (name, plural, capability, columns) = match resource_type {
            "compute.server" => (
                "Virtual Machine",
                "Virtual Machines",
                "compute.server",
                vec![("name", "Name"), ("status", "Status")],
            ),
            "compute.flavor" => (
                "Compute Size",
                "Compute Sizes",
                "compute.flavor",
                vec![("name", "Name"), ("vcpus", "vCPUs"), ("ram", "RAM")],
            ),
            "image.image" => (
                "Image",
                "Images",
                "image.image",
                vec![("name", "Name"), ("status", "Status")],
            ),
            "network.network" => (
                "Network",
                "Networks",
                "network.network",
                vec![("name", "Name"), ("status", "Status")],
            ),
            "network.subnet" => (
                "Subnet",
                "Subnets",
                "network.subnet",
                vec![("name", "Name"), ("status", "Status")],
            ),
            "network.port" => (
                "Port",
                "Ports",
                "network.port",
                vec![("name", "Name"), ("status", "Status")],
            ),
            "network.security-group" => (
                "Security Group",
                "Security Groups",
                "network.security-group",
                vec![("name", "Name")],
            ),
            "block.volume" => (
                "Volume",
                "Volumes",
                "block.volume",
                vec![("name", "Name"), ("status", "Status")],
            ),
            "object.storage.bucket" => (
                "Object Storage Bucket",
                "Object Storage Buckets",
                "object.storage.bucket",
                vec![("name", "Name")],
            ),
            _ => return None,
        };
        let action = |id: &str, risk: ActionRiskClass| ActionDescriptor {
            id: id.into(),
            name: id.into(),
            requires_confirmation: matches!(risk, ActionRiskClass::Destructive),
            risk_class: risk,
            required_capability: Capability {
                resource_type: capability.into(),
                action: id.into(),
            },
            input_schema: None,
        };
        let supported_action_ids: &[(&str, ActionRiskClass)] = match resource_type {
            "compute.server" => &[
                ("delete", ActionRiskClass::Destructive),
                ("start", ActionRiskClass::Disruptive),
                ("stop", ActionRiskClass::Disruptive),
                ("reboot", ActionRiskClass::Disruptive),
            ],
            "compute.flavor" => &[],
            "image.image"
            | "network.network"
            | "network.subnet"
            | "network.port"
            | "network.security-group"
            | "block.volume" => &[("delete", ActionRiskClass::Destructive)],
            "object.storage.bucket" => &[("delete", ActionRiskClass::Destructive)],
            _ => &[],
        };
        let supports_create = Self::supports_create(resource_type);
        Some(ResourceTypeDescriptor {
            id: resource_type.into(),
            name: name.into(),
            plural_name: plural.into(),
            icon_token: "resource".into(),
            create_schema: supports_create.then(|| JsonSchema(json!({"type":"object"}))),
            create_capability: Capability {
                resource_type: capability.into(),
                action: if supports_create {
                    "create"
                } else {
                    "unavailable"
                }
                .into(),
            },
            supported_actions: supported_action_ids
                .iter()
                .map(|(id, risk)| action(id, *risk))
                .collect(),
            columns: columns
                .into_iter()
                .map(|(field, header)| ColumnDescriptor {
                    id: field.into(),
                    header: header.into(),
                    field: field.into(),
                    width: None,
                })
                .collect(),
            filters: vec![FilterDescriptor {
                id: "name".into(),
                label: "Name".into(),
                field: "name".into(),
                kind: FilterKind::Text,
            }],
            sortable_fields: vec!["name".into(), "status".into()],
            details_sections: vec![DetailsSectionDescriptor {
                id: "summary".into(),
                label: "Summary".into(),
                fields: vec!["id".into(), "name".into(), "status".into()],
            }],
            relationships: vec![RelationshipDescriptor {
                id: "project".into(),
                target_resource_type: "tenant.project".into(),
                label: "Project".into(),
                source_property_key: "projectId".into(),
                direction: RelationshipDirection::ToOne,
            }],
        })
    }

    fn supports_create(resource_type: &str) -> bool {
        matches!(
            resource_type,
            "compute.server"
                | "network.network"
                | "network.subnet"
                | "network.port"
                | "network.security-group"
                | "block.volume"
        )
    }

    fn map_resource(
        resource_type: &str,
        value: &Value,
        project: Option<&str>,
    ) -> Result<Resource, ApiError> {
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| config_error("OpenStack resource response omitted id"))?
            .to_owned();
        let name = value
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(&id)
            .to_owned();
        let status_string = value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN");
        let status = match status_string.to_ascii_uppercase().as_str() {
            "ACTIVE" | "AVAILABLE" | "UP" => ResourceStatus::Ready,
            "BUILD" | "CREATING" | "DELETING" | "SHUTOFF" | "DOWN" => ResourceStatus::Busy,
            "ERROR" | "FAILED" => ResourceStatus::Error,
            _ => ResourceStatus::Unknown,
        };
        Ok(Resource {
            id,
            name,
            resource_type: resource_type.into(),
            project_id: value
                .get("tenant_id")
                .or_else(|| value.get("project_id"))
                .or_else(|| value.get("owner"))
                .and_then(Value::as_str)
                .or(project)
                .unwrap_or("unknown")
                .into(),
            region_id: None,
            status,
            created_at: None,
            updated_at: None,
            generation: 1,
            properties: Some(
                value
                    .as_object()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
            ),
        })
    }
}

impl CloudBackend for OpenStackAdapter {}

#[async_trait]
impl Upstream for OpenStackAdapter {
    fn surface(&self) -> &'static str {
        self.surface
    }
    fn backend_kind(&self) -> BackendKind {
        BackendKind::OpenStack
    }

    async fn discover_scopes(
        &self,
        ctx: &RequestContext,
    ) -> Result<Vec<crate::upstream::BackendScope>, ApiError> {
        let _ = self.server_token(ctx).await?;
        let url = self.keystone_url("projects")?;
        let (_, body) = self.request_json(ctx, Method::GET, url, None).await?;
        Ok(body
            .get("projects")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|project| {
                Some(crate::upstream::BackendScope {
                    id: project.get("id")?.as_str()?.to_owned(),
                    kind: "project".into(),
                    name: project
                        .get("name")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    domain_id: project
                        .get("domain_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    can_request_token: false,
                })
            })
            .collect())
    }

    async fn select_scope(&self, ctx: &RequestContext, project_id: &str) -> Result<(), ApiError> {
        let projects = self.discover_scopes(ctx).await?;
        if projects.iter().any(|project| project.id == project_id) {
            Ok(())
        } else {
            Err(ApiError::Forbidden)
        }
    }

    async fn context(&self, ctx: &RequestContext) -> Result<SessionContext, ApiError> {
        let mut capabilities = Vec::new();
        // Validate the token against Keystone.  We intentionally do not expose
        // the token or catalog payload to the browser.
        let url = self.keystone_url("projects")?;
        let (_, value) = self.request_json(ctx, Method::GET, url, None).await?;
        // Keystone's service catalog is authoritative for endpoint discovery.
        // Explicit endpoint configuration remains an allowed operator override
        // for deployments that deliberately pin a public interface/region.
        if let Err(error) = self.discover_catalog(ctx).await {
            let any_explicit_endpoint = self.config.compute_url.is_some()
                || self.config.image_url.is_some()
                || self.config.network_url.is_some()
                || self.config.volume_url.is_some()
                || self.config.object_storage_url.is_some();
            if !any_explicit_endpoint {
                return Err(error);
            }
        }
        for resource_type in [
            "compute.server",
            "compute.flavor",
            "image.image",
            "network.network",
            "network.subnet",
            "network.port",
            "network.security-group",
            "block.volume",
        ] {
            if self.endpoint(resource_type).is_some() {
                let actions = if Self::supports_create(resource_type) {
                    ["list", "read", "create", "delete"].as_slice()
                } else {
                    ["list", "read", "delete"].as_slice()
                };
                for action in actions {
                    capabilities.push(Capability {
                        resource_type: resource_type.into(),
                        action: (*action).into(),
                    });
                }
            }
        }
        if self.endpoint("object.storage.bucket").is_some() {
            for action in ["list", "read", "delete"] {
                capabilities.push(Capability {
                    resource_type: "object.storage.bucket".into(),
                    action: action.into(),
                });
            }
        }
        let project_id = ctx
            .session
            .openstack_project_id
            .clone()
            .or_else(|| self.config.project_id.clone())
            .or_else(|| {
                value
                    .get("projects")
                    .and_then(Value::as_array)
                    .and_then(|p| p.first())
                    .and_then(|p| p.get("id"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            });
        Ok(SessionContext {
            surface: self.surface,
            user_id: ctx
                .session
                .user_id
                .clone()
                .unwrap_or_else(|| "openstack-user".into()),
            user_name: ctx
                .session
                .user_name
                .clone()
                .unwrap_or_else(|| "OpenStack User".into()),
            organization_id: None,
            project_id,
            region_id: self.config.region.clone(),
            capabilities,
        })
    }

    async fn services(&self, ctx: &RequestContext) -> Result<Vec<ServiceDescriptor>, ApiError> {
        let _ = self.discover_catalog(ctx).await;
        let mut services = Vec::new();
        for (id, name, resource_types) in [
            (
                "compute",
                "Compute",
                vec!["compute.server", "compute.flavor"],
            ),
            ("image", "Images", vec!["image.image"]),
            (
                "network",
                "Networking",
                vec![
                    "network.network",
                    "network.subnet",
                    "network.port",
                    "network.security-group",
                ],
            ),
            ("block", "Block Storage", vec!["block.volume"]),
        ] {
            let descriptors = resource_types
                .into_iter()
                .filter_map(Self::descriptor)
                .collect::<Vec<_>>();
            if !descriptors.is_empty() {
                services.push(ServiceDescriptor {
                    id: id.into(),
                    name: name.into(),
                    category: "Services".into(),
                    resource_types: descriptors,
                });
            }
        }
        if self.endpoint("object.storage.bucket").is_some() {
            if let Some(descriptor) = Self::descriptor("object.storage.bucket") {
                services.push(ServiceDescriptor {
                    id: "object-storage".into(),
                    name: "Object Storage".into(),
                    category: "Services".into(),
                    resource_types: vec![descriptor],
                });
            }
        }
        Ok(services)
    }

    async fn list_projects(
        &self,
        ctx: &RequestContext,
    ) -> Result<PaginatedCollection<Project>, ApiError> {
        let scopes = self.discover_scopes(ctx).await?;
        let now = OffsetDateTime::now_utc();
        let items = scopes
            .into_iter()
            .map(|scope| Project {
                id: scope.id,
                name: scope.name.unwrap_or_else(|| "Project".into()),
                organization_id: scope.domain_id.unwrap_or_else(|| "default".into()),
                status: "active".into(),
                created_at: now,
                updated_at: now,
            })
            .collect::<Vec<_>>();
        Ok(PaginatedCollection {
            total: items.len() as u64,
            page: 0,
            page_size: items.len() as u32,
            has_more: false,
            items,
        })
    }

    async fn list_quotas(
        &self,
        ctx: &RequestContext,
        project_id: Option<&str>,
    ) -> Result<PaginatedCollection<ProjectQuota>, ApiError> {
        let _ = self.discover_catalog(ctx).await;
        let project = match self.project(ctx, project_id) {
            Some(project) => project.to_owned(),
            None => self
                .discover_scopes(ctx)
                .await?
                .into_iter()
                .next()
                .map(|scope| scope.id)
                .ok_or(ApiError::Unauthorized)?,
        };
        let mut entries = Vec::new();
        if let Some((base, _)) = self.endpoint("compute.server") {
            let url = Url::parse(&format!(
                "{}/os-quota-sets/{project}",
                self.scoped_base(&base, Some(&project))
            ))
            .map_err(|e| config_error(e.to_string()))?;
            if let Ok((_, body)) = self.request_json(ctx, Method::GET, url, None).await {
                if let Some(quota) = body.get("quota_set") {
                    for (field, resource_type, unit) in [
                        ("instances", "compute.server", "instances"),
                        ("cores", "compute.cpu", "cores"),
                        ("ram", "compute.memory", "MiB"),
                    ] {
                        if let Some(value) = quota
                            .get(field)
                            .and_then(Value::as_i64)
                            .and_then(|v| (v >= 0).then_some(v as u64))
                        {
                            entries.push(QuotaEntry {
                                resource_type: resource_type.into(),
                                limit: Some(value),
                                used: 0,
                                unit: unit.into(),
                            });
                        }
                    }
                }
            }
        }
        if let Some((base, _)) = self.endpoint("network.network") {
            let url = Url::parse(&format!(
                "{}/quotas/{project}",
                self.scoped_base(&base, Some(&project))
            ))
            .map_err(|e| config_error(e.to_string()))?;
            if let Ok((_, body)) = self.request_json(ctx, Method::GET, url, None).await {
                if let Some(quota) = body.get("quota").or_else(|| body.get("quotas")) {
                    for (field, resource_type, unit) in [
                        ("network", "network.network", "networks"),
                        ("subnet", "network.subnet", "subnets"),
                        ("port", "network.port", "ports"),
                        (
                            "security_group",
                            "network.security-group",
                            "security-groups",
                        ),
                    ] {
                        if let Some(value) = quota
                            .get(field)
                            .and_then(Value::as_i64)
                            .and_then(|v| (v >= 0).then_some(v as u64))
                        {
                            entries.push(QuotaEntry {
                                resource_type: resource_type.into(),
                                limit: Some(value),
                                used: 0,
                                unit: unit.into(),
                            });
                        }
                    }
                }
            }
        }
        if let Some((base, _)) = self.endpoint("block.volume") {
            let url = Url::parse(&format!(
                "{}/os-quota-sets/{project}",
                self.scoped_base(&base, Some(&project))
            ))
            .map_err(|e| config_error(e.to_string()))?;
            if let Ok((_, body)) = self.request_json(ctx, Method::GET, url, None).await {
                if let Some(quota) = body.get("quota_set") {
                    if let Some(value) = quota
                        .get("volumes")
                        .and_then(Value::as_i64)
                        .and_then(|v| (v >= 0).then_some(v as u64))
                    {
                        entries.push(QuotaEntry {
                            resource_type: "block.volume".into(),
                            limit: Some(value),
                            used: 0,
                            unit: "volumes".into(),
                        });
                    }
                    if let Some(value) = quota
                        .get("gigabytes")
                        .and_then(Value::as_i64)
                        .and_then(|v| (v >= 0).then_some(v as u64))
                    {
                        entries.push(QuotaEntry {
                            resource_type: "block.volume.size".into(),
                            limit: Some(value),
                            used: 0,
                            unit: "GiB".into(),
                        });
                    }
                }
            }
        }
        Ok(PaginatedCollection {
            items: vec![ProjectQuota {
                project_id: project,
                entries,
            }],
            total: 1,
            page: 0,
            page_size: 1,
            has_more: false,
        })
    }

    async fn list_resources(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        params: ListResourcesParams,
    ) -> Result<PaginatedCollection<Resource>, ApiError> {
        let _ = self.discover_catalog(ctx).await;
        let (base, path) = self.endpoint(resource_type).ok_or_else(|| {
            ApiError::NotImplemented(format!(
                "OpenStack capability is unavailable for {resource_type}"
            ))
        })?;
        let project = self.project(ctx, params.project_id.as_deref());
        let page_size = params.page_size.clamp(1, MAX_PAGE_SIZE);
        let mut url = Url::parse(&format!("{}/{path}", self.scoped_base(&base, project)))
            .map_err(|e| config_error(e.to_string()))?;
        // All supported OpenStack collection APIs accept bounded pagination
        // parameters (some deployments ignore `offset`; the bounded limit is
        // still enforced and the deviation is surfaced by `has_more`).
        url.query_pairs_mut()
            .append_pair("limit", &page_size.to_string());
        if !resource_type.starts_with("network.") && !resource_type.starts_with("compute.") {
            // Glance/Cinder do not accept Neutron's filter-style query fields.
        } else if resource_type == "network.network"
            || resource_type == "network.subnet"
            || resource_type == "network.port"
            || resource_type == "network.security-group"
        {
            // Neutron has no offset parameter; the bounded limit still prevents
            // unbounded browser inventories.
        } else {
            url.query_pairs_mut()
                .append_pair("offset", &(params.page.min(100) * page_size).to_string());
        }
        if resource_type.starts_with("network.") || resource_type == "compute.server" {
            if let Some(project) = project {
                url.query_pairs_mut().append_pair("project_id", project);
            }
        } else if resource_type == "image.image" {
            if let Some(project) = project {
                // Glance's project-scoped filter is named `owner`.
                url.query_pairs_mut().append_pair("owner", project);
            }
        }
        for (key, value) in &params.filters {
            if key.len() < 64 && value.len() < 256 {
                url.query_pairs_mut().append_pair(key, value);
            }
        }
        let (_, body) = self.request_json(ctx, Method::GET, url, None).await?;
        let key = if resource_type == "compute.server" {
            "servers"
        } else if resource_type == "compute.flavor" {
            "flavors"
        } else if resource_type == "image.image" {
            "images"
        } else if resource_type == "block.volume" {
            "volumes"
        } else if resource_type == "object.storage.bucket" {
            "containers"
        } else if resource_type == "network.network" {
            "networks"
        } else if resource_type == "network.subnet" {
            "subnets"
        } else if resource_type == "network.port" {
            "ports"
        } else if resource_type == "network.security-group" {
            "security_groups"
        } else {
            resource_type.rsplit('.').next().unwrap_or("items")
        };
        let values = body
            .get(key)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let page_size = page_size as usize;
        let start = 0usize;
        let items = values
            .into_iter()
            .skip(start)
            .take(page_size)
            .map(|v| Self::map_resource(resource_type, &v, project))
            .collect::<Result<Vec<_>, _>>()?;
        let total = (start + items.len()) as u64;
        Ok(PaginatedCollection {
            has_more: items.len() == page_size,
            items,
            total,
            page: params.page,
            page_size: page_size as u32,
        })
    }

    async fn get_resource(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        id: &str,
    ) -> Result<Resource, ApiError> {
        if id.is_empty() || id.len() > 256 || id.contains('/') {
            return Err(ApiError::NotFound);
        }
        let _ = self.discover_catalog(ctx).await;
        let (base, path) = self.endpoint(resource_type).ok_or_else(|| {
            ApiError::NotImplemented(format!(
                "OpenStack capability is unavailable for {resource_type}"
            ))
        })?;
        let singular = if resource_type == "compute.server" {
            "servers"
        } else if resource_type == "compute.flavor" {
            "flavors"
        } else if resource_type == "image.image" {
            "images"
        } else if resource_type == "block.volume" {
            "volumes"
        } else if resource_type == "object.storage.bucket" {
            "containers"
        } else if resource_type == "network.network" {
            "networks"
        } else if resource_type == "network.subnet" {
            "subnets"
        } else if resource_type == "network.port" {
            "ports"
        } else if resource_type == "network.security-group" {
            "security-groups"
        } else {
            path.trim_end_matches("/detail")
        };
        let url = Url::parse(&format!(
            "{}/{singular}/{id}",
            self.scoped_base(&base, self.project(ctx, None))
        ))
        .map_err(|e| config_error(e.to_string()))?;
        let (_, body) = self.request_json(ctx, Method::GET, url, None).await?;
        let value = body
            .get(match resource_type {
                "network.security-group" => "security_group",
                _ => singular.trim_end_matches('s'),
            })
            .or_else(|| body.get(singular))
            .unwrap_or(&body);
        let resource = Self::map_resource(resource_type, value, self.project(ctx, None))?;
        if let Some(project) = self.project(ctx, None) {
            if resource.project_id != "unknown" && resource.project_id != project {
                // A privileged Keystone token may technically read another
                // project, but the Araf scope is authoritative for this request.
                return Err(ApiError::NotFound);
            }
        }
        Ok(resource)
    }

    async fn create_resource(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        request: CreateResourceRequest,
    ) -> Result<Operation, ApiError> {
        if !Self::supports_create(resource_type) {
            return Err(ApiError::NotImplemented(
                "OpenStack create is not supported by the configured profile".into(),
            ));
        }
        let _ = self.discover_catalog(ctx).await;
        let (base, path) = self.endpoint(resource_type).ok_or_else(|| {
            ApiError::NotImplemented(format!(
                "OpenStack capability is unavailable for {resource_type}"
            ))
        })?;
        let url = Url::parse(&format!(
            "{}/{}",
            self.scoped_base(&base, self.project(ctx, None)),
            path.trim_end_matches("/detail")
        ))
        .map_err(|e| config_error(e.to_string()))?;
        let key = if resource_type == "compute.server" {
            "server"
        } else if resource_type == "block.volume" {
            "volume"
        } else if resource_type == "object.storage.bucket" {
            "container"
        } else {
            resource_type.rsplit('.').next().unwrap_or("resource")
        };
        let (_, body) = self
            .request_json(
                ctx,
                Method::POST,
                url,
                Some(json!({ key: request.payload })),
            )
            .await?;
        let id = body
            .get(key)
            .and_then(|v| v.get("id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let operation = self.operation(
            ctx,
            "create",
            resource_type,
            id,
            OperationState::Running,
            None,
        );
        self.persist_operation(&operation).await?;
        Ok(operation)
    }

    async fn submit_action(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        id: &str,
        request: ActionRequest,
    ) -> Result<Operation, ApiError> {
        if resource_type != "compute.server"
            || !matches!(request.action_id.as_str(), "start" | "stop" | "reboot")
        {
            return Err(ApiError::NotImplemented(
                "OpenStack action is not supported by the configured profile".into(),
            ));
        }
        let _ = self.discover_catalog(ctx).await;
        let (base, _) = self.endpoint("compute.server").ok_or_else(|| {
            ApiError::NotImplemented("OpenStack compute capability is unavailable".into())
        })?;
        let action_body = match request.action_id.as_str() {
            "start" => json!({"os-start": Value::Null}),
            "stop" => json!({"os-stop": Value::Null}),
            "reboot" => {
                json!({"reboot": {"type": request.payload.as_ref().and_then(|v| v.get("type")).and_then(Value::as_str).unwrap_or("SOFT")}})
            }
            _ => unreachable!(),
        };
        let url = Url::parse(&format!(
            "{}/servers/{id}/action",
            self.scoped_base(&base, self.project(ctx, None))
        ))
        .map_err(|e| config_error(e.to_string()))?;
        let _ = self
            .request_json(ctx, Method::POST, url, Some(action_body))
            .await?;
        let operation = self.operation(
            ctx,
            &request.action_id,
            resource_type,
            Some(id.into()),
            OperationState::Running,
            None,
        );
        self.persist_operation(&operation).await?;
        Ok(operation)
    }

    async fn delete_resource(
        &self,
        ctx: &RequestContext,
        resource_type: &str,
        id: &str,
    ) -> Result<Operation, ApiError> {
        let _ = self.discover_catalog(ctx).await;
        let (base, path) = self.endpoint(resource_type).ok_or_else(|| {
            ApiError::NotImplemented(format!(
                "OpenStack capability is unavailable for {resource_type}"
            ))
        })?;
        let singular = if resource_type == "compute.server" {
            "servers"
        } else if resource_type == "block.volume" {
            "volumes"
        } else if resource_type == "object.storage.bucket" {
            "containers"
        } else if resource_type == "network.network" {
            "networks"
        } else if resource_type == "network.subnet" {
            "subnets"
        } else if resource_type == "network.port" {
            "ports"
        } else if resource_type == "network.security-group" {
            "security-groups"
        } else {
            path.trim_end_matches("/detail")
        };
        let url = Url::parse(&format!(
            "{}/{singular}/{id}",
            self.scoped_base(&base, self.project(ctx, None))
        ))
        .map_err(|e| config_error(e.to_string()))?;
        let _ = self.request_json(ctx, Method::DELETE, url, None).await?;
        let operation = self.operation(
            ctx,
            "delete",
            resource_type,
            Some(id.into()),
            OperationState::Running,
            None,
        );
        self.persist_operation(&operation).await?;
        Ok(operation)
    }

    async fn list_operations(
        &self,
        ctx: &RequestContext,
        params: ListOperationsParams,
    ) -> Result<PaginatedCollection<Operation>, ApiError> {
        let Some(journal) = &self.journal else {
            return Ok(PaginatedCollection {
                items: vec![],
                total: 0,
                page: params.page,
                page_size: params.page_size.clamp(1, MAX_PAGE_SIZE),
                has_more: false,
            });
        };
        let selected_project = self.project(ctx, params.project_id.as_deref());
        let mut operations = journal
            .read()
            .await
            .records()
            .filter(|record| {
                params.state.is_none_or(|state| state == record.state)
                    && params
                        .action
                        .as_deref()
                        .is_none_or(|action| action == record.action)
                    && params
                        .resource_type
                        .as_deref()
                        .is_none_or(|kind| kind == record.resource_type)
                    && params
                        .resource_id
                        .as_deref()
                        .is_none_or(|id| record.resource_id.as_deref() == Some(id))
                    && selected_project
                        .is_none_or(|project| record.project_id.as_deref() == Some(project))
            })
            .map(Self::operation_from_record)
            .collect::<Vec<_>>();
        operations.sort_by_key(|operation| operation.updated_at);
        let page_size = params.page_size.clamp(1, MAX_PAGE_SIZE) as usize;
        let start = params.page as usize * page_size;
        let total = operations.len() as u64;
        let items = operations
            .into_iter()
            .skip(start)
            .take(page_size)
            .collect::<Vec<_>>();
        Ok(PaginatedCollection {
            has_more: start + items.len() < total as usize,
            items,
            total,
            page: params.page,
            page_size: page_size as u32,
        })
    }

    async fn get_operation(&self, ctx: &RequestContext, id: &str) -> Result<Operation, ApiError> {
        if let Some(operation) = self.reconcile_operation(ctx, id).await? {
            return Ok(operation);
        }
        let Some(journal) = &self.journal else {
            return Err(ApiError::NotFound);
        };
        let Some(record) = journal.read().await.get(id).cloned() else {
            return Err(ApiError::NotFound);
        };
        if let Some(project) = self.project(ctx, None) {
            if record.project_id.as_deref() != Some(project) {
                return Err(ApiError::NotFound);
            }
        }
        Ok(Self::operation_from_record(&record))
    }
}

fn map_status(status: StatusCode, value: &Value) -> ApiError {
    let detail = value
        .get("detail")
        .and_then(Value::as_str)
        .or_else(|| value.get("message").and_then(Value::as_str))
        .unwrap_or("OpenStack request failed")
        .chars()
        .filter(|c| !c.is_control())
        .take(512)
        .collect::<String>();
    match status {
        StatusCode::UNAUTHORIZED => ApiError::Unauthorized,
        StatusCode::FORBIDDEN => ApiError::Forbidden,
        StatusCode::NOT_FOUND => ApiError::NotFound,
        StatusCode::CONFLICT => ApiError::Conflict(detail),
        StatusCode::PAYLOAD_TOO_LARGE => ApiError::QuotaExceeded(detail),
        _ => ApiError::Upstream(UpstreamError::Error(detail)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_mapping_preserves_provider_fields_without_making_them_identity() {
        let resource = OpenStackAdapter::map_resource(
            "compute.server",
            &json!({"id":"srv-1","name":"web","status":"ACTIVE","tenant_id":"p-1","flavor":{"id":"tiny"}}),
            None,
        ).expect("resource maps");
        assert_eq!(resource.id, "srv-1");
        assert_eq!(resource.project_id, "p-1");
        assert_eq!(resource.status, ResourceStatus::Ready);
        assert_eq!(
            resource
                .properties
                .as_ref()
                .and_then(|p| p.get("flavor"))
                .and_then(Value::as_object)
                .and_then(|p| p.get("id"))
                .and_then(Value::as_str),
            Some("tiny")
        );
    }

    #[test]
    fn object_storage_descriptor_is_explicit_and_provider_neutral() {
        let descriptor = OpenStackAdapter::descriptor("object.storage.bucket").expect("descriptor");
        assert_eq!(descriptor.name, "Object Storage Bucket");
        assert!(!descriptor.id.contains("swift"));
        assert!(descriptor.create_schema.is_none());
        assert!(descriptor
            .supported_actions
            .iter()
            .all(|action| action.id == "delete"));
    }

    #[test]
    fn credentials_in_endpoint_are_rejected() {
        std::env::set_var(
            "OPENSTACK_AUTH_URL",
            "https://user:password@example.test/v3",
        );
        let result = OpenStackClientConfig::from_env();
        std::env::remove_var("OPENSTACK_AUTH_URL");
        assert!(result.is_err());
    }

    #[test]
    fn catalog_endpoint_selection_is_region_aware_and_keystone_v3_is_not_duplicated() {
        let adapter = OpenStackAdapter::new(
            "tenant-bff",
            OpenStackClientConfig {
                auth_url: "https://keystone.example/v3".into(),
                token: Some("token".into()),
                username: None,
                password: None,
                user_domain: "Default".into(),
                project_name: None,
                project_id: None,
                region: Some("RegionOne".into()),
                compute_url: None,
                image_url: None,
                network_url: None,
                volume_url: None,
                object_storage_url: None,
            },
        )
        .expect("adapter");
        adapter.update_catalog(&json!({
            "catalog": [{"type":"compute","endpoints":[
                {"interface":"public","region":"Other","url":"https://other.example"},
                {"interface":"public","region":"RegionOne","url":"https://compute.example/v2.1"}
            ]}]
        }));
        assert_eq!(
            adapter.endpoint("compute.server").map(|(url, _)| url),
            Some("https://compute.example/v2.1".into())
        );
        assert_eq!(
            adapter.keystone_url("projects").expect("url").as_str(),
            "https://keystone.example/v3/auth/projects"
        );
    }
}
