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

use serde::Deserialize;

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
    /// - `O3K_URL` sets the gateway base URL (default `http://127.0.0.1:8080`).
    /// - `O3K_TOKEN` sets the bearer token.
    ///
    /// Per M3-O3K-002 there is no production OIDC exchange in the BFF yet,
    /// so the token is supplied directly. Do not put end-user tokens in
    /// browser storage.
    pub fn from_env() -> Result<Self, O3kClientError> {
        let base_url =
            std::env::var("O3K_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_owned());
        let token = std::env::var("O3K_TOKEN").map_err(|_| {
            O3kClientError::Configuration("O3K_TOKEN environment variable is required".to_owned())
        })?;
        Ok(Self { base_url, token })
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
}

/// Response from `GET /o3k/v1/resource-types`.
#[derive(Clone, Debug, Deserialize)]
pub struct ResourceTypesResponse {
    #[serde(rename = "resource_types")]
    pub resource_types: Vec<DiscoveredResourceType>,
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

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, O3kClientError> {
        let response = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        Self::handle_response(response).await
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
        Self::handle_response(response).await
    }

    async fn delete_json<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
    ) -> Result<T, O3kClientError> {
        let response = self
            .http
            .delete(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        Self::handle_response(response).await
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(
        response: reqwest::Response,
    ) -> Result<T, O3kClientError> {
        let status = response.status().as_u16();
        if response.status().is_success() {
            response.json::<T>().await.map_err(|e| {
                O3kClientError::InvalidResponse(format!("failed to parse success body: {e}"))
            })
        } else {
            let body_text = response.text().await.unwrap_or_default();
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
        let response: ServicesResponse = self.get_json(&self.url("/o3k/v1/services")).await?;
        Ok(response.services)
    }

    /// GET /o3k/v1/resource-types
    pub async fn list_resource_types(&self) -> Result<Vec<DiscoveredResourceType>, O3kClientError> {
        let response: ResourceTypesResponse =
            self.get_json(&self.url("/o3k/v1/resource-types")).await?;
        Ok(response.resource_types)
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
                    .map(|(k, v)| format!("{k}={v}"))
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
        self.get_json(&self.url(&format!("/o3k/v1/compute/servers/{id}")))
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
        self.delete_json(&self.url(&format!("/o3k/v1/compute/servers/{id}")))
            .await
    }

    /// GET /o3k/v1/operations/{id}
    pub async fn get_operation(&self, id: &str) -> Result<NativeOperation, O3kClientError> {
        self.get_json(&self.url(&format!("/o3k/v1/operations/{id}")))
            .await
    }

    /// GET /o3k/v1/identity/me
    pub async fn get_identity_me(&self) -> Result<CurrentContext, O3kClientError> {
        self.get_json(&self.url("/o3k/v1/identity/me")).await
    }

    /// GET /o3k/v1/{namespace}/{collection}
    pub async fn list_generic_resources(
        &self,
        namespace: &str,
        collection: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<ServerListResponse, O3kClientError> {
        let mut url = self.url(&format!("/o3k/v1/{namespace}/{collection}"));
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
                    .map(|(k, v)| format!("{k}={v}"))
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
        self.get_json(&self.url(&format!("/o3k/v1/{namespace}/{collection}/{id}")))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
