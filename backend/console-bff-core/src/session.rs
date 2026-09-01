//! Server-side session management with opaque cookies.
//!
//! The browser receives an opaque session token as an HttpOnly, Secure, SameSite
//! cookie. All OAuth/O3K tokens are stored server-side in memory (production
//! deployments should use a shared session store such as Redis).
//!
//! Session expiry, rotation on privilege escalation, and explicit logout are
//! enforced here.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Default session lifetime (24 hours).
pub const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(86400);
const AUTH_STATE_TTL: Duration = Duration::from_secs(600);

/// Server-side session data.
///
/// The BFF holds the OIDC/O3K tokens and any other confidential state here.
/// Only the opaque `session_token` is sent to the browser.
#[derive(Clone, Debug)]
pub struct SessionData {
    pub user_id: String,
    pub user_name: String,
    pub surface: &'static str,
    pub oidc_access_token: Option<String>,
    pub oidc_refresh_token: Option<String>,
    pub o3k_token: Option<String>,
    pub created_at: Instant,
    pub expires_at: Instant,
    pub csrf_token: String,
}

impl SessionData {
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// In-memory session store.
///
/// For MVP this is a shared HashMap behind a RwLock. Production deployments
/// should replace this with a Redis-backed store for HA and session persistence.
#[derive(Debug, Default)]
pub struct SessionStore {
    sessions: RwLock<HashMap<String, SessionData>>,
    auth_states: RwLock<HashMap<String, Instant>>,
}

impl SessionStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Create a new session and return the opaque session token.
    pub async fn create(
        &self,
        user_id: String,
        user_name: String,
        surface: &'static str,
        oidc_access_token: Option<String>,
        oidc_refresh_token: Option<String>,
        o3k_token: Option<String>,
    ) -> String {
        let session_token = Uuid::new_v4().to_string();
        let now = Instant::now();
        let csrf_token = Uuid::new_v4().to_string();

        let session = SessionData {
            user_id,
            user_name,
            surface,
            oidc_access_token,
            oidc_refresh_token,
            o3k_token,
            created_at: now,
            expires_at: now + DEFAULT_SESSION_TTL,
            csrf_token,
        };

        self.sessions
            .write()
            .await
            .insert(session_token.clone(), session);
        session_token
    }

    /// Issue a short-lived, single-use OIDC callback state value.
    pub async fn issue_auth_state(&self) -> String {
        let state = Uuid::new_v4().to_string();
        self.auth_states
            .write()
            .await
            .insert(state.clone(), Instant::now() + AUTH_STATE_TTL);
        state
    }

    /// Consume an OIDC callback state value, rejecting expiry and replay.
    pub async fn consume_auth_state(&self, state: &str) -> bool {
        let mut states = self.auth_states.write().await;
        matches!(states.remove(state), Some(expires_at) if Instant::now() < expires_at)
    }

    /// Look up a session by its opaque token.
    pub async fn get(&self, session_token: &str) -> Option<SessionData> {
        let sessions = self.sessions.read().await;
        sessions.get(session_token).cloned()
    }

    /// Validate and return session data, returning None if expired or missing.
    pub async fn validate(&self, session_token: &str) -> Option<SessionData> {
        let session = self.get(session_token).await?;
        if session.is_expired() {
            self.destroy(session_token).await;
            return None;
        }
        Some(session)
    }

    /// Destroy a session (logout).
    pub async fn destroy(&self, session_token: &str) {
        self.sessions.write().await.remove(session_token);
    }

    /// Rotate the session token (call after privilege change).
    /// Returns a new token; the old token is invalidated.
    pub async fn rotate(&self, old_token: &str) -> Option<String> {
        let session = self.get(old_token).await?;
        let new_token = Uuid::new_v4().to_string();

        let mut sessions = self.sessions.write().await;
        sessions.remove(old_token);
        sessions.insert(new_token.clone(), session);

        Some(new_token)
    }

    /// Number of active sessions (for health monitoring).
    pub async fn active_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Clean up expired sessions.
    pub async fn reap_expired(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, s| !s.is_expired());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_and_validate_session() {
        let store = SessionStore::new();
        let token = store
            .create(
                "user-1".into(),
                "Test User".into(),
                "tenant-bff",
                None,
                None,
                None,
            )
            .await;
        let session = store
            .validate(&token)
            .await
            .expect("session should be valid");
        assert_eq!(session.user_id, "user-1");
        assert_eq!(session.surface, "tenant-bff");
    }

    #[tokio::test]
    async fn destroy_session() {
        let store = SessionStore::new();
        let token = store
            .create(
                "user-1".into(),
                "Test User".into(),
                "tenant-bff",
                None,
                None,
                None,
            )
            .await;
        store.destroy(&token).await;
        assert!(store.validate(&token).await.is_none());
    }

    #[tokio::test]
    async fn expired_session_is_invalid() {
        let store = Arc::new(SessionStore::default());
        let token = store
            .create(
                "user-1".into(),
                "Test User".into(),
                "tenant-bff",
                None,
                None,
                None,
            )
            .await;

        // Manually expire the session
        {
            let mut sessions = store.sessions.write().await;
            if let Some(s) = sessions.get_mut(&token) {
                s.expires_at = Instant::now() - Duration::from_secs(1);
            }
        }

        assert!(store.validate(&token).await.is_none());
    }

    #[tokio::test]
    async fn rotate_session() {
        let store = SessionStore::new();
        let old = store
            .create(
                "user-1".into(),
                "Test User".into(),
                "tenant-bff",
                None,
                None,
                None,
            )
            .await;
        let new = store.rotate(&old).await.expect("rotate should succeed");
        assert_ne!(old, new);

        // Old token is invalidated
        assert!(store.validate(&old).await.is_none());
        // New token is valid
        assert!(store.validate(&new).await.is_some());
    }

    #[tokio::test]
    async fn auth_state_is_single_use() {
        let store = SessionStore::new();
        let state = store.issue_auth_state().await;
        assert!(store.consume_auth_state(&state).await);
        assert!(!store.consume_auth_state(&state).await);
    }
}
