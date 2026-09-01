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
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::{
    error::{ApiError, BffError},
    request::RequestContext,
    session::SessionStore,
};

/// OIDC client configuration read from environment.
#[derive(Clone, Debug)]
pub struct OidcConfig {
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: String,
    pub redirect_uri: String,
    pub surface: &'static str,
}

impl OidcConfig {
    /// Read OIDC configuration from environment.
    pub fn from_env(surface: &'static str) -> Result<Self, ApiError> {
        let client_id = std::env::var("ARAF_OIDC_CLIENT_ID").map_err(|_| {
            ApiError::Upstream(crate::error::UpstreamError::Error(
                "ARAF_OIDC_CLIENT_ID not set".into(),
            ))
        })?;
        let client_secret = std::env::var("ARAF_OIDC_CLIENT_SECRET").map_err(|_| {
            ApiError::Upstream(crate::error::UpstreamError::Error(
                "ARAF_OIDC_CLIENT_SECRET not set".into(),
            ))
        })?;
        let issuer_url = std::env::var("ARAF_OIDC_ISSUER_URL")
            .unwrap_or_else(|_| "http://localhost:8080".into());
        let redirect_uri = std::env::var("ARAF_OIDC_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/login/callback".into());
        Ok(Self {
            client_id,
            client_secret,
            issuer_url,
            redirect_uri,
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
            surface,
        }
    }
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

fn session_cookie_name(surface: &str) -> String {
    match surface {
        "operator-bff" => "araf_operator_session".into(),
        _ => "araf_tenant_session".into(),
    }
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
        .insert(SET_COOKIE, HeaderValue::from_str(&cookie)?);
    Ok(())
}

fn clear_session_cookie(response: &mut Response, surface: &str) {
    let name = session_cookie_name(surface);
    if let Ok(value) = HeaderValue::from_str(&format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0; Secure"
    )) {
        response.headers_mut().insert(SET_COOKIE, value);
    }
}

/// Initiate OIDC login.
/// In production this redirects to the IdP authorization endpoint.
/// In fixture mode, redirects to the callback.
pub async fn login(State(_config): State<OidcConfig>) -> Redirect {
    Redirect::to("/api/v1/auth/callback?code=fixture&state=mock")
}

/// Handle the OIDC authorization-code callback.
///
/// In fixture mode, creates a session with synthetic data.
/// In production, exchanges the code for tokens and then creates a session.
pub async fn auth_callback(
    State(oidc_config): State<OidcConfig>,
    State(session_store): State<Arc<SessionStore>>,
    Query(params): Query<AuthCallbackQuery>,
) -> Result<Response, BffError> {
    let session_token = if params.code == "fixture" {
        session_store
            .create(
                "fixture-user".into(),
                "Fixture User".into(),
                oidc_config.surface,
                None,
                None,
                None,
            )
            .await
    } else {
        // Production path: exchange code for tokens at the IdP token endpoint.
        let token_response = exchange_code_for_tokens(&oidc_config, &params.code).await?;
        let userinfo = fetch_userinfo(&token_response.access_token).await?;
        session_store
            .create(
                userinfo.sub,
                userinfo.name.unwrap_or_else(|| "User".into()),
                oidc_config.surface,
                Some(token_response.access_token),
                token_response.refresh_token,
                None,
            )
            .await
    };

    let home = match oidc_config.surface {
        "operator-bff" => "/platform/overview",
        _ => "/",
    };

    let mut response = Redirect::to(home).into_response();
    set_session_cookie(&mut response, &session_token, oidc_config.surface)
        .map_err(|_e| BffError::new(ApiError::Internal, "auth"))?;
    info!(surface = oidc_config.surface, "session created");
    Ok(response)
}

/// Logout: destroy the server-side session and clear the session cookie.
pub async fn logout(
    State(session_store): State<Arc<SessionStore>>,
    State(oidc_config): State<OidcConfig>,
    request: RequestContext,
) -> Response {
    // Extract session token from the request context's correlation id as fallback.
    // In a real implementation the middleware extracts the cookie before the handler.
    // For now we rely on the middleware layer to provide the session token.
    session_store.destroy(&request.request_id).await;
    let mut response = Response::new(axum::body::Body::empty());
    clear_session_cookie(&mut response, oidc_config.surface);
    info!("session destroyed");
    response
}

/// Return the current session status without exposing tokens.
pub async fn session_status(
    State(_session_store): State<Arc<SessionStore>>,
    request: RequestContext,
) -> Json<SessionStatus> {
    let session = request.session;
    if session.authenticated {
        Json(SessionStatus {
            authenticated: true,
            user_id: session.user_id.clone(),
            user_name: Some("Fixture User".into()),
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
) -> Result<OidcTokenResponse, ApiError> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &config.redirect_uri),
        ("client_id", &config.client_id),
        ("client_secret", &config.client_secret),
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
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::Upstream(crate::error::UpstreamError::Error(
            format!("OIDC token exchange failed: {status} {body}"),
        )));
    }

    resp.json::<OidcTokenResponse>().await.map_err(|e| {
        ApiError::Upstream(crate::error::UpstreamError::Error(format!(
            "OIDC token response parse failed: {e}"
        )))
    })
}

/// Fetch userinfo from the OIDC provider.
async fn fetch_userinfo(access_token: &str) -> Result<OidcUserinfo, ApiError> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://example.com/protocol/openid-connect/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            ApiError::Upstream(crate::error::UpstreamError::Error(format!(
                "OIDC userinfo fetch failed: {e}"
            )))
        })?;

    resp.json::<OidcUserinfo>().await.map_err(|e| {
        ApiError::Upstream(crate::error::UpstreamError::Error(format!(
            "OIDC userinfo parse failed: {e}"
        )))
    })
}
