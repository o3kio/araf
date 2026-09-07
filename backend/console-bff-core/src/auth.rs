//! Production authentication and session lifecycle handlers.
//!
//! Implements OIDC authorization-code login through a confidential BFF client
//! (ADR 0002). The browser receives only an opaque session cookie; all OAuth/O3K
//! tokens remain server-side.
//!
//! Tenant and Operator BFFs use separate OIDC clients and cookie namespaces.
//! In fixture mode, a session is created directly without an IdP.

use axum::{
    extract::{Query, State},
    http::{
        header::{InvalidHeaderValue, SET_COOKIE},
        HeaderValue,
    },
    response::{IntoResponse, Redirect, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tracing::info;

use crate::{
    error::{ApiError, BffError},
    request::RequestContext,
};

fn config_error(message: impl Into<String>) -> ApiError {
    ApiError::Upstream(crate::error::UpstreamError::Error(message.into()))
}

pub(crate) fn session_cookie_name(surface: &str) -> String {
    match surface {
        "operator-bff" => "araf_operator_session".into(),
        _ => "araf_tenant_session".into(),
    }
}

/// OIDC client configuration read from environment.
#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: String,
    pub redirect_uri: String,
    pub o3k_url: String,
    pub fixture_mode: bool,
    pub production: bool,
    pub surface: &'static str,
}

impl OidcConfig {
    /// Read OIDC configuration from environment.
    pub fn from_env(
        surface: &'static str,
        profile: crate::RuntimeProfile,
    ) -> Result<Self, ApiError> {
        let prefix = if surface == "operator-bff" {
            "ARAF_OPERATOR_OIDC"
        } else {
            "ARAF_TENANT_OIDC"
        };
        let client_id = std::env::var(format!("{prefix}_CLIENT_ID")).map_err(|_| {
            ApiError::Upstream(crate::error::UpstreamError::Error(format!(
                "{prefix}_CLIENT_ID not set"
            )))
        })?;
        let client_secret = std::env::var(format!("{prefix}_CLIENT_SECRET")).map_err(|_| {
            ApiError::Upstream(crate::error::UpstreamError::Error(format!(
                "{prefix}_CLIENT_SECRET not set"
            )))
        })?;
        if client_id.trim().is_empty() || client_secret.trim().is_empty() {
            return Err(ApiError::Upstream(crate::error::UpstreamError::Error(
                format!("{prefix}_CLIENT_ID and {prefix}_CLIENT_SECRET must not be empty"),
            )));
        }
        let issuer_url = required_url(&format!("{prefix}_ISSUER_URL"), profile, true)?;
        validate_issuer(&issuer_url, profile)?;
        let redirect_uri = required_url(&format!("{prefix}_REDIRECT_URI"), profile, true)?;
        let o3k_url = required_url("O3K_URL", profile, false)?;
        Ok(Self {
            client_id,
            client_secret,
            issuer_url,
            redirect_uri,
            o3k_url,
            fixture_mode: false,
            production: profile == crate::RuntimeProfile::Production,
            surface,
        })
    }

    /// Build a fixture config (no real OIDC required).
    pub fn fixture(surface: &'static str) -> Self {
        Self {
            client_id: "araf-fixture".into(),
            client_secret: "unused-fixture-secret".into(),
            issuer_url: "http://localhost:8080".into(),
            redirect_uri: "http://localhost:3000/login/callback".into(),
            o3k_url: "http://127.0.0.1:8080".into(),
            fixture_mode: true,
            production: false,
            surface,
        }
    }
}

fn required_url(
    name: &str,
    profile: crate::RuntimeProfile,
    https_only: bool,
) -> Result<String, ApiError> {
    let value = std::env::var(name).map_err(|_| {
        ApiError::Upstream(crate::error::UpstreamError::Error(format!(
            "{name} is required"
        )))
    })?;
    let url = reqwest::Url::parse(value.trim()).map_err(|_| {
        ApiError::Upstream(crate::error::UpstreamError::Error(format!(
            "{name} must be a valid absolute URL"
        )))
    })?;
    if url.host_str().is_none() || url.username() != "" || url.password().is_some() {
        return Err(ApiError::Upstream(crate::error::UpstreamError::Error(
            format!("{name} must not contain credentials and must include a host"),
        )));
    }
    if https_only && profile == crate::RuntimeProfile::Production && url.scheme() != "https" {
        return Err(ApiError::Upstream(crate::error::UpstreamError::Error(
            format!("{name} must use HTTPS in production"),
        )));
    }
    Ok(value.trim().trim_end_matches('/').to_owned())
}

/// Authorization-code callback query parameters.
#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

/// Current session status exposed to the frontend.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatus {
    pub authenticated: bool,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub surface: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScopeChoice {
    pub id: String,
    pub kind: String,
    pub name: Option<String>,
    pub domain_id: Option<String>,
    pub can_request_token: bool,
}

#[derive(Debug, Deserialize)]
pub struct SelectScopeRequest {
    pub project_id: String,
}

const DISCOVERY_MAX_BYTES: usize = 64 * 1024;
const OIDC_HTTP_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Deserialize)]
struct OidcDiscovery {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: Option<String>,
    token_endpoint_auth_methods_supported: Option<Vec<String>>,
}

fn oidc_client() -> Result<reqwest::Client, ApiError> {
    reqwest::Client::builder()
        .timeout(OIDC_HTTP_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| {
            ApiError::Upstream(crate::error::UpstreamError::Error(format!(
                "OIDC client initialization failed: {error}"
            )))
        })
}

fn validate_oidc_endpoint(
    name: &str,
    value: &str,
    issuer: &reqwest::Url,
    production: bool,
) -> Result<(), ApiError> {
    let url = reqwest::Url::parse(value).map_err(|_| {
        config_error(format!(
            "OIDC discovery {name} must be a valid absolute URL"
        ))
    })?;
    if (production
        && (url.scheme() != "https"
            || url.scheme() != issuer.scheme()
            || url.host_str() != issuer.host_str()
            || url.port_or_known_default() != issuer.port_or_known_default()))
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(config_error(format!(
            "OIDC discovery {name} must be HTTPS, host-qualified, credential-free, fragment-free, and use the configured issuer origin"
        )));
    }
    Ok(())
}

fn validate_issuer(value: &str, profile: crate::RuntimeProfile) -> Result<(), ApiError> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| config_error("OIDC issuer must be a valid absolute URL"))?;
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || (profile == crate::RuntimeProfile::Production && url.scheme() != "https")
    {
        return Err(config_error(
            "OIDC issuer must be a host-qualified HTTPS URL without credentials, query, or fragment",
        ));
    }
    Ok(())
}

async fn read_bounded_body(mut response: reqwest::Response) -> Result<Vec<u8>, ApiError> {
    if response
        .content_length()
        .is_some_and(|length| length > DISCOVERY_MAX_BYTES as u64)
    {
        return Err(config_error(
            "OIDC discovery response exceeds the maximum size",
        ));
    }
    let mut body = Vec::with_capacity(
        response
            .content_length()
            .unwrap_or(DISCOVERY_MAX_BYTES as u64)
            .min(DISCOVERY_MAX_BYTES as u64) as usize,
    );
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| config_error("OIDC discovery response is unreadable"))?
    {
        if body.len() + chunk.len() > DISCOVERY_MAX_BYTES {
            return Err(config_error(
                "OIDC discovery response exceeds the maximum size",
            ));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn discover_oidc(config: &OidcConfig) -> Result<OidcDiscovery, ApiError> {
    let issuer = reqwest::Url::parse(&config.issuer_url)
        .map_err(|_| config_error("OIDC issuer is invalid"))?;
    let discovery_url = reqwest::Url::parse(&format!(
        "{}/.well-known/openid-configuration",
        config.issuer_url.trim_end_matches('/')
    ))
    .map_err(|_| config_error("OIDC discovery URL is invalid"))?;
    let response = oidc_client()?
        .get(discovery_url)
        .send()
        .await
        .map_err(|error| {
            ApiError::Upstream(crate::error::UpstreamError::Error(format!(
                "OIDC discovery unavailable: {error}"
            )))
        })?;
    if !response.status().is_success() {
        return Err(config_error(format!(
            "OIDC discovery failed: {}",
            response.status()
        )));
    }
    let bytes = read_bounded_body(response).await?;
    let metadata: OidcDiscovery = serde_json::from_slice(&bytes)
        .map_err(|_| config_error("OIDC discovery metadata is malformed"))?;
    let configured = issuer.to_string().trim_end_matches('/').to_owned();
    let discovered = reqwest::Url::parse(&metadata.issuer)
        .map_err(|_| config_error("OIDC discovery issuer is invalid"))?
        .to_string()
        .trim_end_matches('/')
        .to_owned();
    if configured != discovered {
        return Err(config_error(
            "OIDC discovery issuer does not match configured issuer",
        ));
    }
    validate_oidc_endpoint(
        "authorization_endpoint",
        &metadata.authorization_endpoint,
        &issuer,
        config.production,
    )?;
    validate_oidc_endpoint(
        "token_endpoint",
        &metadata.token_endpoint,
        &issuer,
        config.production,
    )?;
    if let Some(userinfo) = &metadata.userinfo_endpoint {
        validate_oidc_endpoint("userinfo_endpoint", userinfo, &issuer, config.production)?;
    }
    Ok(metadata)
}

fn set_session_cookie(
    response: &mut Response,
    session_token: &str,
    surface: &str,
) -> Result<(), InvalidHeaderValue> {
    let name = session_cookie_name(surface);
    let cookie =
        format!("{name}={session_token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400; Secure");
    response
        .headers_mut()
        .append(SET_COOKIE, HeaderValue::from_str(&cookie)?);
    Ok(())
}

fn clear_session_cookie(response: &mut Response, surface: &str) {
    let name = session_cookie_name(surface);
    if let Ok(value) = HeaderValue::from_str(&format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0; Secure"
    )) {
        response.headers_mut().append(SET_COOKIE, value);
    }
}

fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn authorization_url(
    endpoint: &str,
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    challenge: &str,
) -> Result<String, ApiError> {
    let mut url = reqwest::Url::parse(endpoint)
        .map_err(|_| config_error("OIDC authorization endpoint is invalid"))?;
    {
        let mut query = url.query_pairs_mut();
        query
            .append_pair("response_type", "code")
            .append_pair("client_id", client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("scope", "openid profile")
            .append_pair("state", state)
            .append_pair("code_challenge", challenge)
            .append_pair("code_challenge_method", "S256");
    }
    Ok(url.into())
}

/// Initiate OIDC login.
/// In production this redirects to the IdP authorization endpoint.
/// In fixture mode, redirects to the callback.
pub async fn login(State(state): State<crate::handlers::AppState>) -> Result<Redirect, BffError> {
    if state.oidc.fixture_mode {
        return Ok(Redirect::to(
            "/api/v1/auth/callback?code=fixture&state=mock",
        ));
    }
    let metadata = discover_oidc(&state.oidc)
        .await
        .map_err(|_| BffError::new(ApiError::Unauthorized, "auth"))?;
    let (auth_state, code_verifier) = state.sessions.issue_auth_state().await;
    let url = authorization_url(
        &metadata.authorization_endpoint,
        &state.oidc.client_id,
        &state.oidc.redirect_uri,
        &auth_state,
        &pkce_challenge(&code_verifier),
    )
    .map_err(|_| BffError::new(ApiError::Unauthorized, "auth"))?;
    Ok(Redirect::to(&url))
}

/// Handle the OIDC authorization-code callback.
///
/// In fixture mode, creates a session with synthetic data.
/// In production, exchanges the code for tokens and then creates a session.
pub async fn auth_callback(
    State(state): State<crate::handlers::AppState>,
    Query(params): Query<AuthCallbackQuery>,
) -> Result<Response, BffError> {
    if !state.oidc.fixture_mode {
        let Some(auth_state) = params.state.as_deref() else {
            return Err(BffError::new(ApiError::Unauthorized, "auth"));
        };
        let Some(code_verifier) = state.sessions.consume_auth_state(auth_state).await else {
            return Err(BffError::new(ApiError::Unauthorized, "auth"));
        };
        let metadata = discover_oidc(&state.oidc)
            .await
            .map_err(|_| BffError::new(ApiError::Unauthorized, "auth"))?;
        let token_response =
            exchange_code_for_tokens(&state.oidc, &metadata, &params.code, &code_verifier).await?;
        return create_authenticated_session(&state, &metadata, token_response).await;
    }
    let session_token = state
        .sessions
        .create(
            "fixture-user".into(),
            "Fixture User".into(),
            state.oidc.surface,
            None,
            None,
            None,
        )
        .await;

    let home = match state.oidc.surface {
        "operator-bff" => "/platform/overview",
        _ => "/",
    };

    let mut response = Redirect::to(home).into_response();
    if let Some(session) = state.sessions.get(&session_token).await {
        crate::csrf::set_csrf_cookie(&mut response, &session.csrf_token)
            .map_err(|_| BffError::new(ApiError::Internal, "auth"))?;
    }
    set_session_cookie(&mut response, &session_token, state.oidc.surface)
        .map_err(|_e| BffError::new(ApiError::Internal, "auth"))?;
    info!(surface = state.oidc.surface, "session created");
    Ok(response)
}

async fn exchange_system_token(url: &str, access_token: &str) -> Result<String, ApiError> {
    crate::o3k_client::O3kClient::new(crate::o3k_client::O3kClientConfig {
        base_url: url.to_owned(),
        token: "session-exchange".to_owned(),
    })
    .exchange_federated_system_token(access_token)
    .await
    .map(|response| response.token.id)
    .map_err(|error| ApiError::Upstream(crate::error::UpstreamError::Error(error.to_string())))
}

async fn create_authenticated_session(
    state: &crate::handlers::AppState,
    metadata: &OidcDiscovery,
    token_response: OidcTokenResponse,
) -> Result<Response, BffError> {
    let external_access_token = token_response.access_token.clone();
    let userinfo_url = metadata
        .userinfo_endpoint
        .as_deref()
        .ok_or_else(|| BffError::new(ApiError::Unauthorized, "auth"))?;
    let userinfo = fetch_userinfo(userinfo_url, &external_access_token).await?;
    let session_token = state
        .sessions
        .create_with_ttl(
            userinfo.sub,
            userinfo.name.unwrap_or_else(|| "User".into()),
            state.oidc.surface,
            Some(external_access_token.clone()),
            token_response.refresh_token,
            if state.oidc.surface == "operator-bff" {
                Some(
                    exchange_system_token(&state.oidc.o3k_url, &external_access_token)
                        .await
                        .map_err(|_| BffError::new(ApiError::Unauthorized, "auth"))?,
                )
            } else {
                None
            },
            Duration::from_secs(token_response.expires_in.min(86_400)),
        )
        .await;
    let home = if state.oidc.surface == "operator-bff" {
        "/platform/overview"
    } else {
        "/"
    };
    let mut response = Redirect::to(home).into_response();
    if let Some(session) = state.sessions.get(&session_token).await {
        crate::csrf::set_csrf_cookie(&mut response, &session.csrf_token)
            .map_err(|_| BffError::new(ApiError::Internal, "auth"))?;
    }
    set_session_cookie(&mut response, &session_token, state.oidc.surface)
        .map_err(|_| BffError::new(ApiError::Internal, "auth"))?;
    info!(surface = state.oidc.surface, "session created");
    Ok(response)
}

/// Logout: destroy the server-side session and clear the session cookie.
pub async fn logout(
    State(state): State<crate::handlers::AppState>,
    request: RequestContext,
) -> Response {
    // Extract session token from the request context's correlation id as fallback.
    // In a real implementation the middleware extracts the cookie before the handler.
    // For now we rely on the middleware layer to provide the session token.
    if let Some(session_token) = request.session.session_token.as_deref() {
        state.sessions.destroy(session_token).await;
    }
    let mut response = Response::new(axum::body::Body::empty());
    clear_session_cookie(&mut response, state.oidc.surface);
    info!("session destroyed");
    response
}

/// Return the current session status without exposing tokens.
pub async fn session_status(
    State(_state): State<crate::handlers::AppState>,
    request: RequestContext,
) -> Json<SessionStatus> {
    let session = request.session;
    if session.authenticated {
        Json(SessionStatus {
            authenticated: true,
            user_id: session.user_id.clone(),
            user_name: session.user_name.clone(),
            surface: Some(session.surface.to_string()),
        })
    } else {
        Json(SessionStatus {
            authenticated: false,
            user_id: None,
            user_name: None,
            surface: None,
        })
    }
}

pub async fn discover_scopes(
    State(state): State<crate::handlers::AppState>,
    request: RequestContext,
) -> Result<Json<Vec<ScopeChoice>>, BffError> {
    let Some(access_token) = request.session.oidc_access_token.as_deref() else {
        return Err(BffError::new(
            ApiError::Unauthorized,
            request.correlation_id(),
        ));
    };
    let response = crate::o3k_client::O3kClient::new(crate::o3k_client::O3kClientConfig {
        base_url: state.oidc.o3k_url.clone(),
        token: "session-exchange".to_owned(),
    })
    .discover_federated_scopes(access_token)
    .await
    .map_err(|_| BffError::new(ApiError::Unauthorized, request.correlation_id()))?;
    Ok(Json(
        response
            .scopes
            .into_iter()
            .map(|scope| ScopeChoice {
                id: scope.id,
                kind: scope.kind,
                name: scope.name,
                domain_id: scope.domain_id,
                can_request_token: scope.can_request_token,
            })
            .collect(),
    ))
}

pub async fn select_scope(
    State(state): State<crate::handlers::AppState>,
    request: RequestContext,
    Json(body): Json<SelectScopeRequest>,
) -> Result<Response, BffError> {
    let Some(access_token) = request.session.oidc_access_token.as_deref() else {
        return Err(BffError::new(
            ApiError::Unauthorized,
            request.correlation_id(),
        ));
    };
    let native_token = crate::o3k_client::O3kClient::new(crate::o3k_client::O3kClientConfig {
        base_url: state.oidc.o3k_url.clone(),
        token: "session-exchange".to_owned(),
    })
    .exchange_federated_token(access_token, &body.project_id)
    .await
    .map_err(|_| BffError::new(ApiError::Unauthorized, request.correlation_id()))?
    .token
    .id;
    let Some(session_token) = request.session.session_token.as_deref() else {
        return Err(BffError::new(
            ApiError::Unauthorized,
            request.correlation_id(),
        ));
    };
    if !state
        .sessions
        .set_o3k_token(session_token, native_token)
        .await
    {
        return Err(BffError::new(
            ApiError::Unauthorized,
            request.correlation_id(),
        ));
    }
    Ok(Response::new(axum::body::Body::empty()))
}

/// OIDC token response from the provider.
#[derive(Debug, Deserialize, Serialize)]
struct OidcTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

/// Userinfo from the OIDC provider.
#[derive(Debug, Deserialize, Serialize)]
struct OidcUserinfo {
    sub: String,
    name: Option<String>,
    preferred_username: Option<String>,
}

/// Exchange authorization code for tokens at the OIDC provider.
async fn exchange_code_for_tokens(
    config: &OidcConfig,
    metadata: &OidcDiscovery,
    code: &str,
    code_verifier: &str,
) -> Result<OidcTokenResponse, ApiError> {
    let client = oidc_client()?;
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &config.redirect_uri),
        ("client_id", &config.client_id),
        ("code_verifier", code_verifier),
    ];

    let default_methods = ["client_secret_basic".to_owned()];
    let methods = metadata
        .token_endpoint_auth_methods_supported
        .as_deref()
        .unwrap_or(&default_methods);
    let method = methods
        .iter()
        .find(|method| {
            method.as_str() == "client_secret_basic" || method.as_str() == "client_secret_post"
        })
        .ok_or_else(|| {
            config_error("OIDC provider advertises no supported token authentication method")
        })?;
    let request = client.post(&metadata.token_endpoint);
    let response = if method == "client_secret_basic" {
        request
            .basic_auth(&config.client_id, Some(&config.client_secret))
            .form(&params)
            .send()
            .await
    } else {
        request
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", config.redirect_uri.as_str()),
                ("client_id", config.client_id.as_str()),
                ("client_secret", config.client_secret.as_str()),
                ("code_verifier", code_verifier),
            ])
            .send()
            .await
    };
    let resp = response.map_err(|e| {
        ApiError::Upstream(crate::error::UpstreamError::Error(format!(
            "OIDC token exchange failed: {e}"
        )))
    })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let _body = resp.text().await;
        return Err(ApiError::Upstream(crate::error::UpstreamError::Error(
            format!("OIDC token exchange failed: {status}"),
        )));
    }

    resp.json::<OidcTokenResponse>().await.map_err(|e| {
        ApiError::Upstream(crate::error::UpstreamError::Error(format!(
            "OIDC token response parse failed: {e}"
        )))
    })
}

/// Fetch userinfo from the OIDC provider.
async fn fetch_userinfo(userinfo_url: &str, access_token: &str) -> Result<OidcUserinfo, ApiError> {
    let client = oidc_client()?;
    let resp = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            ApiError::Upstream(crate::error::UpstreamError::Error(format!(
                "OIDC userinfo fetch failed: {e}"
            )))
        })?;

    if !resp.status().is_success() {
        return Err(ApiError::Upstream(crate::error::UpstreamError::Error(
            format!("OIDC userinfo fetch failed: {}", resp.status()),
        )));
    }
    resp.json::<OidcUserinfo>().await.map_err(|e| {
        ApiError::Upstream(crate::error::UpstreamError::Error(format!(
            "OIDC userinfo parse failed: {e}"
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fixture::FixtureAdapter, handlers::AppState, session::SessionStore};
    use axum::extract::{Query, State};
    use std::sync::Arc;
    use wiremock::{
        matchers::{body_string_contains, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn production_login_uses_rfc7636_s256_with_valid_verifier_length() {
        let sessions = SessionStore::new();
        let (_, verifier) = sessions.issue_auth_state().await;
        assert!((43..=128).contains(&verifier.len()));

        let challenge = pkce_challenge(&verifier);
        assert_eq!(challenge.len(), 43);
        assert_ne!(challenge, verifier);
    }

    #[tokio::test]
    async fn production_callback_keeps_oidc_tokens_server_side_and_sets_opaque_cookies() {
        let idp = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issuer": idp.uri(),
                "authorization_endpoint": format!("{}/authorize", idp.uri()),
                "token_endpoint": format!("{}/protocol/openid-connect/token", idp.uri()),
                "userinfo_endpoint": format!("{}/userinfo", idp.uri()),
                "token_endpoint_auth_methods_supported": ["client_secret_basic"]
            })))
            .mount(&idp)
            .await;
        Mock::given(method("POST"))
            .and(path("/protocol/openid-connect/token"))
            .and(header(
                "authorization",
                "Basic dGVuYW50LWNsaWVudDpjbGllbnQtc2VjcmV0",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "external-secret",
                "refresh_token": "refresh-secret",
                "expires_in": 900
            })))
            .mount(&idp)
            .await;
        Mock::given(method("GET"))
            .and(path("/userinfo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "sub": "tenant-user",
                "name": "Tenant User"
            })))
            .mount(&idp)
            .await;

        let sessions = SessionStore::new();
        let (state_value, _verifier) = sessions.issue_auth_state().await;
        let state = AppState {
            upstream: Arc::new(FixtureAdapter::new("tenant-bff")),
            oidc: OidcConfig {
                client_id: "tenant-client".into(),
                client_secret: "client-secret".into(),
                issuer_url: idp.uri(),
                redirect_uri: "https://tenant.example.test/api/v1/auth/callback".into(),
                o3k_url: "http://127.0.0.1:9".into(),
                fixture_mode: false,
                production: false,
                surface: "tenant-bff",
            },
            sessions: sessions.clone(),
        };

        let response = auth_callback(
            State(state),
            Query(AuthCallbackQuery {
                code: "authorization-code".into(),
                state: Some(state_value),
            }),
        )
        .await
        .expect("callback succeeds");

        let cookies: Vec<_> = response.headers().get_all(SET_COOKIE).iter().collect();
        assert_eq!(cookies.len(), 2);
        let cookie_text = cookies
            .iter()
            .map(|value| value.to_str().expect("cookie value"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(cookie_text.contains("araf_tenant_session="));
        assert!(cookie_text.contains("araf_csrf="));
        assert!(!cookie_text.contains("external-secret"));
        assert!(!cookie_text.contains("refresh-secret"));
        assert!(cookie_text.contains("HttpOnly"));
        assert!(cookie_text.contains("Secure"));

        let session_cookie = cookies
            .iter()
            .find_map(|value| {
                value
                    .to_str()
                    .ok()
                    .and_then(|cookie| cookie.strip_prefix("araf_tenant_session="))
                    .map(|cookie| cookie.split(';').next().expect("session token"))
            })
            .expect("opaque session cookie");
        let session = sessions.get(session_cookie).await.expect("server session");
        assert_eq!(
            session.oidc_access_token.as_deref(),
            Some("external-secret")
        );
        assert_eq!(
            session.oidc_refresh_token.as_deref(),
            Some("refresh-secret")
        );
        assert_ne!(session_cookie, "external-secret");
    }

    fn discovery_config(issuer: String, production: bool) -> OidcConfig {
        OidcConfig {
            client_id: "client".into(),
            client_secret: "secret".into(),
            issuer_url: issuer,
            redirect_uri: "https://console.example/callback".into(),
            o3k_url: "https://o3k.example".into(),
            fixture_mode: false,
            production,
            surface: "tenant-bff",
        }
    }

    #[tokio::test]
    async fn discovery_rejects_wrong_issuer_and_missing_metadata() {
        let idp = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issuer": "http://other.example",
                "token_endpoint": format!("{}/token", idp.uri())
            })))
            .mount(&idp)
            .await;
        assert!(discover_oidc(&discovery_config(idp.uri(), false))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn production_discovery_rejects_http_and_credentials() {
        let idp = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issuer": idp.uri(),
                "authorization_endpoint": format!("{}/authorize", idp.uri()),
                "token_endpoint": format!("https://user:pass@example/token")
            })))
            .mount(&idp)
            .await;
        assert!(discover_oidc(&discovery_config(idp.uri(), true))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn bounded_discovery_body_accepts_limit_and_rejects_over_limit() {
        let idp = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/exact"))
            .respond_with(
                ResponseTemplate::new(200).set_body_bytes(vec![b'x'; DISCOVERY_MAX_BYTES]),
            )
            .mount(&idp)
            .await;
        Mock::given(method("GET"))
            .and(path("/over"))
            .respond_with(
                ResponseTemplate::new(200).set_body_bytes(vec![b'x'; DISCOVERY_MAX_BYTES + 1]),
            )
            .mount(&idp)
            .await;
        let client = oidc_client().expect("bounded client");
        let exact = client
            .get(format!("{}/exact", idp.uri()))
            .send()
            .await
            .expect("exact response");
        assert_eq!(
            read_bounded_body(exact).await.expect("exact body").len(),
            DISCOVERY_MAX_BYTES
        );
        let over = client
            .get(format!("{}/over", idp.uri()))
            .send()
            .await
            .expect("oversized response");
        assert!(read_bounded_body(over).await.is_err());
    }

    #[test]
    fn authorization_url_preserves_provider_query_parameters() {
        let url = authorization_url(
            "https://idp.example/authorize?tenant=foo",
            "client",
            "https://console.example/callback",
            "state",
            "challenge",
        )
        .expect("authorization URL");
        let parsed = reqwest::Url::parse(&url).expect("parsed authorization URL");
        let pairs: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();
        assert_eq!(pairs.get("tenant"), Some(&"foo".to_owned()));
        assert_eq!(pairs.get("code_challenge_method"), Some(&"S256".to_owned()));
    }

    #[test]
    fn production_issuer_rejects_query_fragment_credentials_and_http() {
        for issuer in [
            "http://idp.example/issuer",
            "https://user:pass@idp.example/issuer",
            "https://idp.example/issuer?tenant=foo",
            "https://idp.example/issuer#fragment",
        ] {
            assert!(validate_issuer(issuer, crate::RuntimeProfile::Production).is_err());
        }
        assert!(validate_issuer(
            "https://idp.example/realms/cloud",
            crate::RuntimeProfile::Production
        )
        .is_ok());
    }

    #[tokio::test]
    async fn discovery_rejects_unsupported_token_auth_methods() {
        let idp = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issuer": idp.uri(),
                "authorization_endpoint": format!("{}/authorize", idp.uri()),
                "token_endpoint": format!("{}/token", idp.uri()),
                "token_endpoint_auth_methods_supported": ["private_key_jwt"]
            })))
            .mount(&idp)
            .await;
        let config = discovery_config(idp.uri(), false);
        let metadata = discover_oidc(&config).await.expect("metadata");
        assert!(
            exchange_code_for_tokens(&config, &metadata, "code", "verifier")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn client_secret_post_is_sent_in_form_when_advertised() {
        let idp = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issuer": idp.uri(),
                "authorization_endpoint": format!("{}/authorize", idp.uri()),
                "token_endpoint": format!("{}/token", idp.uri()),
                "token_endpoint_auth_methods_supported": ["client_secret_post"]
            })))
            .mount(&idp)
            .await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(body_string_contains("client_secret=secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "token",
                "expires_in": 60
            })))
            .mount(&idp)
            .await;
        let config = discovery_config(idp.uri(), false);
        let metadata = discover_oidc(&config).await.expect("metadata");
        exchange_code_for_tokens(&config, &metadata, "code", "verifier")
            .await
            .expect("client_secret_post exchange");
    }
}
