//! Server-side session management with opaque cookies.
//!
//! The browser receives an opaque session token as an HttpOnly, Secure, SameSite
//! cookie. All OAuth/O3K tokens are stored server-side in memory (production
//! deployments should use a shared session store such as Redis).
//!
//! Session expiry, rotation on privilege escalation, and explicit logout are
//! enforced here. Production may use the durable file-backed implementation;
//! deployments spanning hosts should provide an encrypted shared store.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::ErrorKind,
    path::PathBuf,
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
    pub openstack_token: Option<String>,
    pub openstack_project_id: Option<String>,
    pub created_at: Instant,
    pub expires_at: Instant,
    pub csrf_token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PersistedSession {
    user_id: String,
    user_name: String,
    surface: String,
    oidc_access_token: Option<String>,
    oidc_refresh_token: Option<String>,
    o3k_token: Option<String>,
    openstack_token: Option<String>,
    openstack_project_id: Option<String>,
    ttl_seconds: u64,
    csrf_token: String,
}

impl SessionData {
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// Session store. Development/test use an in-memory HashMap; production
/// startup requires the durable path configured by `ARAF_SESSION_STORE_PATH`.
#[derive(Debug)]
pub struct SessionStore {
    sessions: RwLock<HashMap<String, SessionData>>,
    auth_states: RwLock<HashMap<String, (Instant, String)>>,
    durable_path: Option<Arc<PathBuf>>,
    file_lock: Arc<tokio::sync::Mutex<()>>,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            auth_states: RwLock::new(HashMap::new()),
            durable_path: None,
            file_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }
}

impl SessionStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn new_durable(path: impl Into<PathBuf>) -> Result<Arc<Self>, std::io::Error> {
        let path = path.into();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        let store = Arc::new(Self {
            durable_path: Some(Arc::new(path)),
            ..Self::default()
        });
        Ok(store)
    }

    async fn reload_from_disk(&self) -> Result<(), std::io::Error> {
        let Some(path) = self.durable_path.as_deref() else {
            return Ok(());
        };
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        let persisted: HashMap<String, PersistedSession> = serde_json::from_slice(&bytes)
            .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error))?;
        let now = Instant::now();
        let sessions = persisted
            .into_iter()
            .filter_map(|(token, session)| {
                let ttl = Duration::from_secs(session.ttl_seconds);
                (session.ttl_seconds > 0).then_some((
                    token,
                    SessionData {
                        user_id: session.user_id,
                        user_name: session.user_name,
                        surface: if session.surface == "operator-bff" {
                            "operator-bff"
                        } else {
                            "tenant-bff"
                        },
                        oidc_access_token: session.oidc_access_token,
                        oidc_refresh_token: session.oidc_refresh_token,
                        o3k_token: session.o3k_token,
                        openstack_token: session.openstack_token,
                        openstack_project_id: session.openstack_project_id,
                        created_at: now,
                        expires_at: now + ttl,
                        csrf_token: session.csrf_token,
                    },
                ))
            })
            .collect();
        *self.sessions.write().await = sessions;
        Ok(())
    }

    async fn persist_to_disk(&self) -> Result<(), std::io::Error> {
        let Some(path) = self.durable_path.as_deref() else {
            return Ok(());
        };
        let _guard = self.file_lock.lock().await;
        let lock_path = path.with_extension("lock");
        let lock_file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;
        fs2::FileExt::lock_exclusive(&lock_file)?;
        let sessions = self.sessions.read().await;
        let persisted: HashMap<_, _> = sessions
            .iter()
            .filter(|(_, session)| !session.is_expired())
            .map(|(token, session)| {
                (
                    token.clone(),
                    PersistedSession {
                        user_id: session.user_id.clone(),
                        user_name: session.user_name.clone(),
                        surface: session.surface.to_owned(),
                        oidc_access_token: session.oidc_access_token.clone(),
                        oidc_refresh_token: session.oidc_refresh_token.clone(),
                        o3k_token: session.o3k_token.clone(),
                        openstack_token: session.openstack_token.clone(),
                        openstack_project_id: session.openstack_project_id.clone(),
                        ttl_seconds: session
                            .expires_at
                            .saturating_duration_since(Instant::now())
                            .as_secs(),
                        csrf_token: session.csrf_token.clone(),
                    },
                )
            })
            .collect();
        drop(sessions);
        let temporary = PathBuf::from(format!("{}.tmp-{}", path.display(), std::process::id()));
        fs::write(
            &temporary,
            serde_json::to_vec(&persisted)
                .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error))?,
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))?;
        }
        fs::rename(&temporary, path)?;
        Ok(())
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
        self.create_with_ttl(
            user_id,
            user_name,
            surface,
            oidc_access_token,
            oidc_refresh_token,
            o3k_token,
            DEFAULT_SESSION_TTL,
        )
        .await
    }

    // The explicit fields keep confidential session material visibly separated
    // at call sites; the argument count is intentional for this security boundary.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_ttl(
        &self,
        user_id: String,
        user_name: String,
        surface: &'static str,
        oidc_access_token: Option<String>,
        oidc_refresh_token: Option<String>,
        o3k_token: Option<String>,
        ttl: Duration,
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
            openstack_token: None,
            openstack_project_id: None,
            created_at: now,
            expires_at: now + ttl,
            csrf_token,
        };

        self.sessions
            .write()
            .await
            .insert(session_token.clone(), session);
        let _ = self.persist_to_disk().await;
        session_token
    }

    pub async fn issue_auth_state(&self) -> (String, String) {
        let state = Uuid::new_v4().to_string();
        let code_verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        self.auth_states.write().await.insert(
            state.clone(),
            (Instant::now() + AUTH_STATE_TTL, code_verifier.clone()),
        );
        (state, code_verifier)
    }

    pub async fn consume_auth_state(&self, state: &str) -> Option<String> {
        let mut states = self.auth_states.write().await;
        match states.remove(state) {
            Some((expires_at, verifier)) if Instant::now() < expires_at => Some(verifier),
            _ => None,
        }
    }

    /// Look up a session by its opaque token.
    pub async fn get(&self, session_token: &str) -> Option<SessionData> {
        let _ = self.reload_from_disk().await;
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
        let _ = self.reload_from_disk().await;
        self.sessions.write().await.remove(session_token);
        let _ = self.persist_to_disk().await;
    }

    pub async fn set_o3k_token(&self, session_token: &str, token: String) -> bool {
        let _ = self.reload_from_disk().await;
        let mut sessions = self.sessions.write().await;
        let Some(session) = sessions.get_mut(session_token) else {
            return false;
        };
        session.o3k_token = Some(token);
        drop(sessions);
        let _ = self.persist_to_disk().await;
        true
    }

    pub async fn set_openstack_project(&self, session_token: &str, project_id: String) -> bool {
        let _ = self.reload_from_disk().await;
        let mut sessions = self.sessions.write().await;
        let Some(session) = sessions.get_mut(session_token) else {
            return false;
        };
        session.openstack_project_id = Some(project_id);
        drop(sessions);
        let _ = self.persist_to_disk().await;
        true
    }

    pub async fn set_openstack_token(&self, session_token: &str, token: String) -> bool {
        let _ = self.reload_from_disk().await;
        let mut sessions = self.sessions.write().await;
        let Some(session) = sessions.get_mut(session_token) else {
            return false;
        };
        session.openstack_token = Some(token);
        drop(sessions);
        let _ = self.persist_to_disk().await;
        true
    }

    /// Rotate the session token (call after privilege change).
    /// Returns a new token; the old token is invalidated.
    pub async fn rotate(&self, old_token: &str) -> Option<String> {
        let session = self.get(old_token).await?;
        let new_token = Uuid::new_v4().to_string();

        let mut sessions = self.sessions.write().await;
        sessions.remove(old_token);
        sessions.insert(new_token.clone(), session);
        drop(sessions);
        let _ = self.persist_to_disk().await;

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
        drop(sessions);
        let _ = self.persist_to_disk().await;
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
    async fn durable_store_survives_reopen_and_revocation() {
        let path = std::env::temp_dir().join(format!("araf-session-{}.json", Uuid::new_v4()));
        let first = SessionStore::new_durable(&path).expect("durable store");
        let token = first
            .create(
                "user-1".into(),
                "User".into(),
                "tenant-bff",
                Some("secret".into()),
                None,
                None,
            )
            .await;
        let second = SessionStore::new_durable(&path).expect("second replica");
        assert_eq!(
            second
                .validate(&token)
                .await
                .expect("session replicated")
                .user_id,
            "user-1"
        );
        second.destroy(&token).await;
        assert!(
            first.validate(&token).await.is_none(),
            "revocation must cross replicas"
        );
        let _ = std::fs::remove_file(path);
    }
}
