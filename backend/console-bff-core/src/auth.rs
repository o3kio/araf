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
    pub authorization_url: String,
    pub userinfo_url: String,
    pub o3k_url: String,
    pub fixture_mode: bool,
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
        let redirect_uri = required_url(&format!("{prefix}_REDIRECT_URI"), profile, true)?;
        let authorization_url = std::env::var(format!("{prefix}_AUTHORIZATION_URL"))
            .unwrap_or_else(|_| {
                format!(
                    "{}/protocol/openid-connect/auth",
                    issuer_url.trim_end_matches('/')
                )
            });
        let userinfo_url = std::env::var(format!("{prefix}_USERINFO_URL")).unwrap_or_else(|_| {
            format!(
                "{}/protocol/openid-connect/userinfo",
                issuer_url.trim_end_matches('/')
            )
        });
        let o3k_url = required_url("O3K_URL", profile, false)?;
        Ok(Self {
            client_id,
            client_secret,
            issuer_url,
            redirect_uri,
            authorization_url,
            userinfo_url,
            o3k_url,
            fixture_mode: false,
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
            authorization_url: "/api/v1/auth/callback?code=fixture&state=mock".into(),
            userinfo_url: "http://localhost:8080/userinfo".into(),
            o3k_url: "http://127.0.0.1:8080".into(),
            fixture_mode: true,
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

fn encode_query_component(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Initiate OIDC login.
/// In production this redirects to the IdP authorization endpoint.
/// In fixture mode, redirects to the callback.
pub async fn login(State(state): State<crate::handlers::AppState>) -> Redirect {
    if state.oidc.fixture_mode {
        return Redirect::to("/api/v1/auth/callback?code=fixture&state=mock");
    }
    let (auth_state, code_verifier) = state.sessions.issue_auth_state().await;
    Redirect::to(&format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope=openid%20profile&state={}&code_challenge={}&code_challenge_method=S256",
        state.oidc.authorization_url,
        encode_query_component(&state.oidc.client_id),
        encode_query_component(&state.oidc.redirect_uri),
        encode_query_component(&auth_state),
        encode_query_component(&pkce_challenge(&code_verifier)),
    ))
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
        let token_response =
            exchange_code_for_tokens(&state.oidc, &params.code, &code_verifier).await?;
        return create_authenticated_session(&state, token_response).await;
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
    token_response: OidcTokenResponse,
) -> Result<Response, BffError> {
    let external_access_token = token_response.access_token.clone();
    let userinfo = fetch_userinfo(&state.oidc.userinfo_url, &external_access_token).await?;
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
    code: &str,
    code_verifier: &str,
) -> Result<OidcTokenResponse, ApiError> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &config.redirect_uri),
        ("client_id", &config.client_id),
        ("client_secret", &config.client_secret),
        ("code_verifier", code_verifier),
    ];

    let token_url = format!(
        "{}/protocol/openid-connect/token",
        config.issuer_url.trim_end_matches('/')
    );
    let resp = client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
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
    let client = reqwest::Client::new();
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
        matchers::{method, path},
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
        Mock::given(method("POST"))
            .and(path("/protocol/openid-connect/token"))
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
                authorization_url: format!("{}/authorize", idp.uri()),
                userinfo_url: format!("{}/userinfo", idp.uri()),
                o3k_url: "http://127.0.0.1:9".into(),
                fixture_mode: false,
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
}
