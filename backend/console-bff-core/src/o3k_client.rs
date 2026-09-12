//! Narrow, hand-written O3K native HTTP client.
//!
//! Targets the `/o3k/v1` surface defined in `crates/o3k-api` and
//! `crates/o3k-native-api` of the authoritative O3K repository. All types
//! here are manually derived from inspection of those sources and the
//! `contracts/native-resource-envelope-v1.schema.json` contract.
//!
//! This client is intentionally minimal: it covers only the routes the Araf
//! MVP needs for M7. It does not attempt to be a generic O3K SDK.

use std::collections::HashMap;

use futures_util::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

const DISCOVERY_RESPONSE_MAX_BYTES: usize = 64 * 1024;
const JSON_RESPONSE_MAX_BYTES: usize = 4 * 1024 * 1024;

/// Configuration needed to talk to an O3K native API.
#[derive(Clone, Debug)]
pub struct O3kClientConfig {
    /// Base URL of the O3K HTTP gateway, e.g. `http://127.0.0.1:8080`.
    pub base_url: String,
    /// Bearer token used for `Authorization: Bearer <token>`.
    pub token: String,
}

impl O3kClientConfig {
    /// Build configuration from environment variables.
    ///
    /// - `O3K_URL` sets the gateway base URL and is always required.
    /// - `O3K_TOKEN` is an optional development fallback. Production calls use
    ///   the native token held by the current server-side BFF session.
    ///
    /// Per M3-O3K-002 there is no production OIDC exchange in the BFF yet,
    /// so the token is supplied directly. Do not put end-user tokens in
    /// browser storage.
    pub fn from_env() -> Result<Self, O3kClientError> {
        let base_url = std::env::var("O3K_URL")
            .map_err(|_| O3kClientError::Configuration("O3K_URL is required".into()))?;
        let url = reqwest::Url::parse(base_url.trim()).map_err(|_| {
            O3kClientError::Configuration("O3K_URL must be a valid absolute URL".into())
        })?;
        if url.host_str().is_none() || url.username() != "" || url.password().is_some() {
            return Err(O3kClientError::Configuration(
                "O3K_URL must include a host and no credentials".into(),
            ));
        }
        let token = std::env::var("O3K_TOKEN").unwrap_or_default();
        Ok(Self {
            base_url: base_url.trim().trim_end_matches('/').into(),
            token,
        })
    }
}

/// Errors that can occur when calling the O3K native API.
#[derive(Debug, thiserror::Error, Clone)]
pub enum O3kClientError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("upstream error ({status}): {title} - {detail}")]
    Upstream {
        status: u16,
        title: String,
        detail: String,
    },
    #[error("not implemented by O3K native API in M7: {0}")]
    NotImplemented(String),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

impl From<reqwest::Error> for O3kClientError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
    }
}

/// Native resource envelope as defined by
/// `contracts/native-resource-envelope-v1.schema.json`.
#[derive(Clone, Debug, Deserialize)]
pub struct NativeResourceEnvelope {
    #[serde(rename = "api_version")]
    pub api_version: String,
    pub kind: String,
    pub metadata: NativeMetadata,
    pub spec: serde_json::Value,
    pub status: serde_json::Value,
}

/// Metadata block of a native resource envelope.
#[derive(Clone, Debug, Deserialize)]
pub struct NativeMetadata {
    pub id: String,
    pub owner_scope: Option<String>,
    pub generation: i64,
    pub region: Option<String>,
    #[serde(rename = "availability_domain")]
    pub availability_domain: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    pub annotations: Option<HashMap<String, String>>,
}

/// Canonical O3K Operation response (`crates/o3k-kernel/src/operation.rs`).
#[derive(Clone, Debug, Deserialize)]
pub struct NativeOperation {
    pub id: String,
    pub service: String,
    pub action: String,
    pub actor: String,
    #[serde(rename = "owner_scope")]
    pub owner_scope: String,
    #[serde(rename = "resource_type")]
    pub resource_type: String,
    #[serde(rename = "resource_id")]
    pub resource_id: Option<String>,
    pub state: String,
    pub attempt: u32,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error: Option<String>,
    #[serde(rename = "request_id")]
    pub request_id: Option<String>,
}

/// Result returned by a native resource mutation.
///
/// Concrete create/delete routes return this shape with HTTP 202 (or 201/204
/// when the mutation is synchronous). The `operation_id` is the canonical
/// identity used to poll for completion.
#[derive(Clone, Debug, Deserialize)]
pub struct MutationResult {
    #[serde(rename = "operation_id")]
    pub operation_id: String,
    #[serde(rename = "resource_id")]
    pub resource_id: Option<String>,
    pub complete: bool,
    pub resource: Option<serde_json::Value>,
}

/// Response from `GET /o3k/v1/compute/servers`.
#[derive(Clone, Debug, Deserialize)]
pub struct ServerListResponse {
    pub items: Vec<serde_json::Value>,
    pub next_cursor: Option<String>,
}

/// Discovered service from `GET /o3k/v1/services`.
#[derive(Clone, Debug, Deserialize)]
pub struct DiscoveredService {
    pub id: String,
    pub namespace: String,
    #[serde(rename = "service_version")]
    pub service_version: String,
    pub ownership: Option<String>,
    #[serde(rename = "lifecycle_state")]
    pub lifecycle_state: Option<String>,
}

/// Response from `GET /o3k/v1/services`.
#[derive(Clone, Debug, Deserialize)]
pub struct ServicesResponse {
    pub services: Vec<DiscoveredService>,
    pub count: usize,
}

/// Discovered resource type from `GET /o3k/v1/resource-types`.
#[derive(Clone, Debug, Deserialize)]
pub struct DiscoveredResourceType {
    pub namespace: String,
    pub name: String,
    pub service: String,
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub collection: String,
    pub scope: String,
    pub ready: bool,
    #[serde(rename = "lifecycle_actions")]
    pub lifecycle_actions: HashMap<String, String>,
    #[serde(default)]
    pub placement: Option<String>,
    #[serde(default)]
    pub regions: Vec<String>,
    #[serde(default, rename = "availability_domain_selection")]
    pub availability_domain_selection: Option<String>,
    #[serde(default)]
    pub schema: Option<DiscoveredSchemaReference>,
    #[serde(default)]
    pub actions: Vec<DiscoveredActionMetadata>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DiscoveredSchemaReference {
    pub id: String,
    pub version: String,
    pub representation: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DiscoveredActionMetadata {
    pub name: String,
    pub action_id: String,
    pub target: String,
    #[serde(default)]
    pub input: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub asynchronous: bool,
}

/// Response from `GET /o3k/v1/resource-types`.
#[derive(Clone, Debug, Deserialize)]
pub struct ResourceTypesResponse {
    #[serde(rename = "resource_types")]
    pub resource_types: Vec<DiscoveredResourceType>,
    pub count: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DiscoveredAvailabilityDomain {
    pub id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DiscoveredRegion {
    pub id: String,
    #[serde(default, rename = "availability_domains")]
    pub availability_domains: Vec<DiscoveredAvailabilityDomain>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RegionsResponse {
    pub regions: Vec<DiscoveredRegion>,
    pub count: usize,
}

/// Response from `GET /o3k/v1/identity/me`.
#[derive(Clone, Debug, Deserialize)]
pub struct CurrentContext {
    pub authenticated: bool,
    pub principal_id: String,
    pub principal_kind: String,
    pub principal_name: String,
    pub effective_scope_id: String,
    pub effective_scope_kind: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OperatorProfile {
    pub profile: String,
    pub scope: String,
    pub principal_id: String,
    pub audit_id: String,
}

/// Async HTTP client for the O3K native API.
#[derive(Clone, Debug)]
pub struct O3kClient {
    http: reqwest::Client,
    base_url: String,
    token: String,
}

impl O3kClient {
    /// Create a new client from the given configuration.
    pub fn new(config: O3kClientConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: config.base_url.trim_end_matches('/').to_owned(),
            token: config.token,
        }
    }

    /// Clone this client with the native token held by one server-side BFF
    /// session. Tokens never come from browser input or process-global state.
    pub fn with_token(&self, token: impl Into<String>) -> Self {
        Self {
            http: self.http.clone(),
            base_url: self.base_url.clone(),
            token: token.into(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn path_segment(value: &str) -> String {
        value
            .bytes()
            .flat_map(|byte| {
                if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                    vec![byte as char]
                } else {
                    format!("%{byte:02X}").chars().collect()
                }
            })
            .collect()
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, O3kClientError> {
        let response = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        Self::handle_response(response, JSON_RESPONSE_MAX_BYTES).await
    }

    async fn get_discovery_json<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
    ) -> Result<T, O3kClientError> {
        let response = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        Self::handle_response(response, DISCOVERY_RESPONSE_MAX_BYTES).await
    }

    async fn post_json<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> Result<T, O3kClientError> {
        let response = self
            .http
            .post(url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;
        Self::handle_response(response, JSON_RESPONSE_MAX_BYTES).await
    }

    async fn post_mutation_json<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> Result<T, O3kClientError> {
        let response = self
            .http
            .post(url)
            .header("Authorization", self.auth_header())
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&body)
            .send()
            .await?;
        Self::handle_response(response, JSON_RESPONSE_MAX_BYTES).await
    }

    async fn delete_json<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
    ) -> Result<T, O3kClientError> {
        let response = self
            .http
            .delete(url)
            .header("Authorization", self.auth_header())
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .send()
            .await?;
        Self::handle_response(response, JSON_RESPONSE_MAX_BYTES).await
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(
        response: reqwest::Response,
        max_bytes: usize,
    ) -> Result<T, O3kClientError> {
        let status = response.status().as_u16();
        let declared_length = response.content_length();
        if declared_length.is_some_and(|length| length > max_bytes as u64) {
            return Err(O3kClientError::InvalidResponse(format!(
                "response exceeds {max_bytes} byte limit"
            )));
        }
        if response.status().is_success() {
            let mut body = Vec::with_capacity(
                declared_length
                    .map(|length| length as usize)
                    .unwrap_or(0)
                    .min(max_bytes),
            );
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(O3kClientError::from)?;
                if body.len().saturating_add(chunk.len()) > max_bytes {
                    return Err(O3kClientError::InvalidResponse(format!(
                        "success response exceeds {max_bytes} byte limit"
                    )));
                }
                body.extend_from_slice(&chunk);
            }
            serde_json::from_slice(&body).map_err(|e| {
                O3kClientError::InvalidResponse(format!("failed to parse success body: {e}"))
            })
        } else {
            let mut body = Vec::with_capacity(
                declared_length
                    .map(|length| length as usize)
                    .unwrap_or(0)
                    .min(max_bytes),
            );
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(O3kClientError::from)?;
                if body.len().saturating_add(chunk.len()) > max_bytes {
                    return Err(O3kClientError::InvalidResponse(format!(
                        "error response exceeds {max_bytes} byte limit"
                    )));
                }
                body.extend_from_slice(&chunk);
            }
            let body_text = String::from_utf8_lossy(&body).into_owned();
            // Try to extract O3K Problem Details fields.
            let (title, detail) =
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&body_text) {
                    (
                        value
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Upstream error")
                            .to_owned(),
                        value
                            .get("detail")
                            .or_else(|| value.get("message"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(&body_text)
                            .to_owned(),
                    )
                } else {
                    ("Upstream error".to_owned(), body_text)
                };
            Err(O3kClientError::Upstream {
                status,
                title,
                detail,
            })
        }
    }

    /// GET /o3k/v1/services
    pub async fn list_services(&self) -> Result<Vec<DiscoveredService>, O3kClientError> {
        let response: ServicesResponse = self
            .get_discovery_json(&self.url("/o3k/v1/services"))
            .await?;
        Ok(response.services)
    }

    /// GET /o3k/v1/resource-types
    pub async fn list_resource_types(&self) -> Result<Vec<DiscoveredResourceType>, O3kClientError> {
        let response: ResourceTypesResponse = self
            .get_discovery_json(&self.url("/o3k/v1/resource-types"))
            .await?;
        Ok(response.resource_types)
    }

    /// GET /o3k/v1/resource-schemas/{namespace}/{collection}/{version}
    pub async fn get_resource_schema(
        &self,
        namespace: &str,
        collection: &str,
        version: &str,
    ) -> Result<serde_json::Value, O3kClientError> {
        self.get_discovery_json(&self.url(&format!(
            "/o3k/v1/resource-schemas/{}/{}/{}",
            Self::path_segment(namespace),
            Self::path_segment(collection),
            Self::path_segment(version)
        )))
        .await
    }

    /// GET /o3k/v1/regions
    pub async fn list_regions(&self) -> Result<Vec<DiscoveredRegion>, O3kClientError> {
        let response: RegionsResponse = self
            .get_discovery_json(&self.url("/o3k/v1/regions"))
            .await?;
        Ok(response.regions)
    }

    /// GET /o3k/v1/compute/servers
    pub async fn list_compute_servers(
        &self,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<ServerListResponse, O3kClientError> {
        let mut url = self.url("/o3k/v1/compute/servers");
        let mut params = Vec::new();
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(cursor) = cursor {
            params.push(("cursor", cursor.to_owned()));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(
                &params
                    .iter()
                    .map(|(k, v)| format!("{k}={}", Self::path_segment(v)))
                    .collect::<Vec<_>>()
                    .join("&"),
            );
        }
        self.get_json(&url).await
    }

    /// GET /o3k/v1/compute/servers/{id}
    pub async fn get_compute_server(
        &self,
        id: &str,
    ) -> Result<NativeResourceEnvelope, O3kClientError> {
        self.get_json(&self.url(&format!(
            "/o3k/v1/compute/servers/{}",
            Self::path_segment(id)
        )))
        .await
    }

    /// POST /o3k/v1/compute/servers
    pub async fn create_compute_server(
        &self,
        payload: serde_json::Value,
    ) -> Result<MutationResult, O3kClientError> {
        let body = serde_json::json!({
            "api_version": "o3k.io/v1",
            "kind": "compute:server",
            "spec": payload,
        });
        self.post_json(&self.url("/o3k/v1/compute/servers"), body)
            .await
    }

    /// Start a compute server.
    ///
    /// **Upstream gap:** the O3K native API exposes concrete create/delete
    /// routes for `compute:server` but no native start/stop route. The
    /// actions are defined in the manifest (`compute:StartServer`) but are
    /// not bound to HTTP endpoints in M7. This method therefore returns
    /// `O3kClientError::NotImplemented` rather than inventing a route.
    pub async fn start_compute_server(&self, _id: &str) -> Result<MutationResult, O3kClientError> {
        Err(O3kClientError::NotImplemented(
            "native start route for compute:server is not available".to_owned(),
        ))
    }

    /// Stop a compute server.
    ///
    /// **Upstream gap:** same as `start_compute_server`; no native stop route
    /// exists in M7.
    pub async fn stop_compute_server(&self, _id: &str) -> Result<MutationResult, O3kClientError> {
        Err(O3kClientError::NotImplemented(
            "native stop route for compute:server is not available".to_owned(),
        ))
    }

    /// DELETE /o3k/v1/compute/servers/{id}
    pub async fn delete_compute_server(&self, id: &str) -> Result<MutationResult, O3kClientError> {
        self.delete_json(&self.url(&format!(
            "/o3k/v1/compute/servers/{}",
            Self::path_segment(id)
        )))
        .await
    }

    /// GET /o3k/v1/operations/{id}
    pub async fn get_operation(&self, id: &str) -> Result<NativeOperation, O3kClientError> {
        self.get_json(&self.url(&format!("/o3k/v1/operations/{}", Self::path_segment(id))))
            .await
    }

    /// GET /o3k/v1/identity/me
    pub async fn get_identity_me(&self) -> Result<CurrentContext, O3kClientError> {
        self.get_json(&self.url("/o3k/v1/identity/me")).await
    }

    /// GET /o3k/v1/operator/profile. O3K performs the system/operator check.
    pub async fn get_operator_profile(&self) -> Result<OperatorProfile, O3kClientError> {
        self.get_json(&self.url("/o3k/v1/operator/profile")).await
    }

    /// POST /o3k/v1/identity/scopes
    pub async fn discover_federated_scopes(
        &self,
        external_access_token: &str,
    ) -> Result<FederatedScopesResponse, O3kClientError> {
        self.post_json(
            &self.url("/o3k/v1/identity/scopes"),
            serde_json::json!({"federated": {"access_token": external_access_token}}),
        )
        .await
    }

    /// POST /o3k/v1/identity/tokens for one selected project.
    pub async fn exchange_federated_token(
        &self,
        external_access_token: &str,
        project_id: &str,
    ) -> Result<IssuedNativeTokenResponse, O3kClientError> {
        self.post_json(
            &self.url("/o3k/v1/identity/tokens"),
            serde_json::json!({
                "auth": {
                    "method": "federated",
                    "project_id": project_id,
                    "federated": {"access_token": external_access_token}
                }
            }),
        )
        .await
    }

    /// POST /o3k/v1/identity/tokens for an explicitly authorized operator.
    pub async fn exchange_federated_system_token(
        &self,
        external_access_token: &str,
    ) -> Result<IssuedNativeTokenResponse, O3kClientError> {
        self.post_json(
            &self.url("/o3k/v1/identity/tokens"),
            serde_json::json!({
                "auth": {
                    "method": "federated",
                    "federated": {
                        "access_token": external_access_token,
                        "scope": {"kind": "system"}
                    }
                }
            }),
        )
        .await
    }

    /// GET /o3k/v1/{namespace}/{collection}
    pub async fn list_generic_resources(
        &self,
        namespace: &str,
        collection: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<ServerListResponse, O3kClientError> {
        let mut url = self.url(&format!(
            "/o3k/v1/{}/{}",
            Self::path_segment(namespace),
            Self::path_segment(collection)
        ));
        let mut params = Vec::new();
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(cursor) = cursor {
            params.push(("cursor", cursor.to_owned()));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(
                &params
                    .iter()
                    .map(|(k, v)| format!("{k}={}", Self::path_segment(v)))
                    .collect::<Vec<_>>()
                    .join("&"),
            );
        }
        self.get_json(&url).await
    }

    /// GET /o3k/v1/{namespace}/{collection}/{id}
    pub async fn get_generic_resource(
        &self,
        namespace: &str,
        collection: &str,
        id: &str,
    ) -> Result<NativeResourceEnvelope, O3kClientError> {
        self.get_json(&self.url(&format!(
            "/o3k/v1/{}/{}/{}",
            Self::path_segment(namespace),
            Self::path_segment(collection),
            Self::path_segment(id)
        )))
        .await
    }

    pub async fn create_generic_resource(
        &self,
        namespace: &str,
        collection: &str,
        kind: &str,
        payload: serde_json::Value,
    ) -> Result<MutationResult, O3kClientError> {
        self.post_mutation_json(
            &self.url(&format!(
                "/o3k/v1/{}/{}",
                Self::path_segment(namespace),
                Self::path_segment(collection)
            )),
            serde_json::json!({"api_version":"o3k.io/v1", "kind": kind, "spec": payload}),
        )
        .await
    }

    pub async fn update_generic_resource(
        &self,
        namespace: &str,
        collection: &str,
        id: &str,
        kind: &str,
        payload: serde_json::Value,
    ) -> Result<MutationResult, O3kClientError> {
        let response = self
            .http
            .put(self.url(&format!(
                "/o3k/v1/{}/{}/{}",
                Self::path_segment(namespace),
                Self::path_segment(collection),
                Self::path_segment(id)
            )))
            .header("Authorization", self.auth_header())
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&serde_json::json!({"api_version":"o3k.io/v1", "kind": kind, "spec": payload}))
            .send()
            .await?;
        Self::handle_response(response, JSON_RESPONSE_MAX_BYTES).await
    }

    pub async fn delete_generic_resource(
        &self,
        namespace: &str,
        collection: &str,
        id: &str,
    ) -> Result<MutationResult, O3kClientError> {
        self.delete_json(&self.url(&format!(
            "/o3k/v1/{}/{}/{}",
            Self::path_segment(namespace),
            Self::path_segment(collection),
            Self::path_segment(id)
        )))
        .await
    }

    pub async fn invoke_generic_action(
        &self,
        namespace: &str,
        collection: &str,
        id: &str,
        action: &str,
        input: serde_json::Value,
    ) -> Result<MutationResult, O3kClientError> {
        self.post_mutation_json(
            &self.url(&format!(
                "/o3k/v1/{}/{}/{}/actions/{}",
                Self::path_segment(namespace),
                Self::path_segment(collection),
                Self::path_segment(id),
                Self::path_segment(action)
            )),
            serde_json::json!({"input": input}),
        )
        .await
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FederatedScopeDescriptor {
    pub id: String,
    pub kind: String,
    pub name: Option<String>,
    pub domain_id: Option<String>,
    pub can_request_token: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FederatedScopesResponse {
    pub scopes: Vec<FederatedScopeDescriptor>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IssuedNativeTokenResponse {
    pub token: IssuedNativeToken,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IssuedNativeToken {
    pub id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn config_from_env_uses_defaults() {
        // We cannot safely mutate the process environment here because tests
        // run concurrently. This test only verifies the default URL.
        let cfg = O3kClientConfig {
            base_url: "http://127.0.0.1:8080".to_owned(),
            token: "test-token".to_owned(),
        };
        assert_eq!(cfg.base_url, "http://127.0.0.1:8080");
    }

    #[tokio::test]
    async fn federated_exchange_and_scope_discovery_use_native_contract() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/o3k/v1/identity/scopes"))
            .and(body_json(serde_json::json!({
                "federated": {"access_token": "external-secret"}
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "scopes": [{"id":"project-a","kind":"project","name":"Project A","domain_id":"default","can_request_token":true}]
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/o3k/v1/identity/tokens"))
            .and(body_json(serde_json::json!({
                "auth": {"method":"federated","project_id":"project-a","federated":{"access_token":"external-secret"}}
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "token": {"id":"native-project-a"}
            })))
            .mount(&server)
            .await;

        let client = O3kClient::new(O3kClientConfig {
            base_url: server.uri(),
            token: "unused".into(),
        });
        let scopes = client
            .discover_federated_scopes("external-secret")
            .await
            .expect("scopes");
        assert_eq!(scopes.scopes[0].id, "project-a");
        let token = client
            .exchange_federated_token("external-secret", "project-a")
            .await
            .expect("exchange");
        assert_eq!(token.token.id, "native-project-a");
    }

    #[tokio::test]
    async fn discovery_rejects_body_larger_than_bounded_limit() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/o3k/v1/services"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![
                b' ';
                DISCOVERY_RESPONSE_MAX_BYTES
                    + 1
            ]))
            .mount(&server)
            .await;

        let client = O3kClient::new(O3kClientConfig {
            base_url: server.uri(),
            token: "unused".into(),
        });
        let error = client
            .list_services()
            .await
            .expect_err("oversized discovery");
        assert!(error.to_string().contains("byte limit"));
    }

    #[tokio::test]
    async fn discovery_retains_exact_limit_before_parsing() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/o3k/v1/services"))
            .respond_with(
                ResponseTemplate::new(200).set_body_bytes(vec![b' '; DISCOVERY_RESPONSE_MAX_BYTES]),
            )
            .mount(&server)
            .await;

        let client = O3kClient::new(O3kClientConfig {
            base_url: server.uri(),
            token: "unused".into(),
        });
        let error = client.list_services().await.expect_err("invalid JSON");
        assert!(!error.to_string().contains("byte limit"));
    }

    #[tokio::test]
    async fn discovery_reads_converged_resource_schema_and_locations() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/o3k/v1/resource-types"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "resource_types": [{
                    "namespace": "compute", "name": "server", "service": "compute",
                    "schema_version": "v1", "collection": "servers", "scope": "regional",
                    "ready": true, "lifecycle_actions": {"list":"compute:ListServers", "create":"compute:CreateServer"},
                    "placement": "regional", "regions": ["eu-test"],
                    "availability_domain_selection": "optional",
                    "schema": {"id":"https://o3k.io/schemas/compute/servers/v1", "version":"v1", "representation":"native-resource-envelope"},
                    "actions": [{"name":"CreateServer", "action_id":"compute:CreateServer", "target":"collection", "output":"https://o3k.io/contracts/native-mutation-result-v1.schema.json", "asynchronous":true}]
                }], "count": 1
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/o3k/v1/resource-schemas/compute/servers/v1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "allOf": [{"$ref":"envelope"}, {"type":"object", "properties":{"spec":{"type":"object", "required":["name"]}}}]
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/o3k/v1/regions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "regions": [{"id":"eu-test", "availability_domains":[{"id":"eu-test-a"}]}], "count": 1
            })))
            .mount(&server)
            .await;

        let client = O3kClient::new(O3kClientConfig {
            base_url: server.uri(),
            token: "unused".into(),
        });
        let types = client.list_resource_types().await.expect("resource types");
        assert_eq!(types[0].regions, ["eu-test"]);
        assert_eq!(types[0].actions[0].action_id, "compute:CreateServer");
        let schema = client
            .get_resource_schema("compute", "servers", "v1")
            .await
            .expect("schema");
        assert_eq!(schema["allOf"][1]["properties"]["spec"]["type"], "object");
        let regions = client.list_regions().await.expect("regions");
        assert_eq!(regions[0].availability_domains[0].id, "eu-test-a");
    }
}
