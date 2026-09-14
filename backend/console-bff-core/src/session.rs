//! Server-side session management with opaque cookies.
//!
//! The browser receives an opaque session token as an HttpOnly, Secure, SameSite
//! cookie. All OAuth/O3K tokens are stored server-side. Production deployments
//! use the encrypted durable implementation so replicas can share sessions
//! safely.
//!
//! Session expiry, rotation on privilege escalation, and explicit logout are
//! enforced here. Deployments spanning hosts must provide the same encryption
//! key through a secret and a shared filesystem with atomic rename semantics
//! (or replace this implementation with an equivalent transactional store).

use aes_gcm::{
    aead::{consts::U12, Aead, KeyInit},
    Aes256Gcm, Nonce,
};
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
const SESSION_FILE_MAGIC: &[u8] = b"ARAFSESS1";

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

/// Session store. Development/test may use an in-memory HashMap; production
/// startup requires `ARAF_SESSION_STORE_PATH` and an AES-256 key in
/// `ARAF_SESSION_STORE_KEY`.
#[derive(Debug)]
pub struct SessionStore {
    sessions: RwLock<HashMap<String, SessionData>>,
    auth_states: RwLock<HashMap<String, (Instant, String)>>,
    durable_path: Option<Arc<PathBuf>>,
    durable_key: Option<[u8; 32]>,
    file_lock: Arc<tokio::sync::Mutex<()>>,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            auth_states: RwLock::new(HashMap::new()),
            durable_path: None,
            durable_key: None,
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

    /// Open an encrypted durable store for multi-replica production use.
    /// The key must be a 32-byte AES-256 key supplied by the deployment secret.
    pub fn new_durable_encrypted(
        path: impl Into<PathBuf>,
        key: [u8; 32],
    ) -> Result<Arc<Self>, std::io::Error> {
        let path = path.into();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        Ok(Arc::new(Self {
            durable_path: Some(Arc::new(path)),
            durable_key: Some(key),
            ..Self::default()
        }))
    }

    fn decode_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let Some(key) = self.durable_key else {
            return Ok(bytes.to_vec());
        };
        if bytes.len() < SESSION_FILE_MAGIC.len() + 12
            || &bytes[..SESSION_FILE_MAGIC.len()] != SESSION_FILE_MAGIC
        {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "encrypted session store header is invalid",
            ));
        }
        let nonce_start = SESSION_FILE_MAGIC.len();
        let nonce_end = nonce_start + 12;
        Aes256Gcm::new_from_slice(&key)
            .map_err(|error| std::io::Error::other(error.to_string()))?
            .decrypt(
                Nonce::<U12>::from_slice(&bytes[nonce_start..nonce_end]),
                &bytes[nonce_end..],
            )
            .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error.to_string()))
    }

    fn encode_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let Some(key) = self.durable_key else {
            return Ok(bytes.to_vec());
        };
        let nonce_uuid = Uuid::new_v4();
        let nonce = &nonce_uuid.as_bytes()[..12];
        let ciphertext = Aes256Gcm::new_from_slice(&key)
            .map_err(|error| std::io::Error::other(error.to_string()))?
            .encrypt(Nonce::<U12>::from_slice(nonce), bytes)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let mut encoded = Vec::with_capacity(SESSION_FILE_MAGIC.len() + 12 + ciphertext.len());
        encoded.extend_from_slice(SESSION_FILE_MAGIC);
        encoded.extend_from_slice(nonce);
        encoded.extend_from_slice(&ciphertext);
        Ok(encoded)
    }

    fn deserialize_sessions(
        &self,
        bytes: &[u8],
    ) -> Result<HashMap<String, SessionData>, std::io::Error> {
        let persisted: HashMap<String, PersistedSession> =
            serde_json::from_slice(&self.decode_bytes(bytes)?)
                .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error))?;
        let now = Instant::now();
        Ok(persisted
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
            .collect())
    }

    fn persisted_bytes(
        &self,
        sessions: &HashMap<String, SessionData>,
    ) -> Result<Vec<u8>, std::io::Error> {
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
        self.encode_bytes(
            &serde_json::to_vec(&persisted)
                .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error))?,
        )
    }

    async fn reload_from_disk(&self) -> Result<(), std::io::Error> {
        let Some(path) = self.durable_path.as_deref() else {
            return Ok(());
        };
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                // A missing authority must not leave this replica serving its
                // last in-memory snapshot. Fail closed until the authority is
                // restored and a subsequent request reloads it.
                self.sessions.write().await.clear();
                return Ok(());
            }
            Err(error) => {
                self.sessions.write().await.clear();
                return Err(error);
            }
        };
        let sessions = match self.deserialize_sessions(&bytes) {
            Ok(sessions) => sessions,
            Err(error) => {
                self.sessions.write().await.clear();
                return Err(error);
            }
        };
        *self.sessions.write().await = sessions;
        Ok(())
    }

    async fn mutate<F, R>(&self, mutator: F) -> Result<R, std::io::Error>
    where
        F: FnOnce(&mut HashMap<String, SessionData>) -> R,
    {
        let Some(path) = self.durable_path.as_deref() else {
            return Ok(mutator(&mut *self.sessions.write().await));
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
        let mut sessions = match fs::read(path) {
            Ok(bytes) => self.deserialize_sessions(&bytes)?,
            Err(error) if error.kind() == ErrorKind::NotFound => HashMap::new(),
            Err(error) => return Err(error),
        };
        let result = mutator(&mut sessions);
        let encoded = self.persisted_bytes(&sessions)?;
        // A process id is not unique across containers/hosts sharing a volume;
        // include a UUID so concurrent replicas cannot target the same temp
        // file before the atomic rename.
        let temporary = PathBuf::from(format!(
            "{}.tmp-{}-{}",
            path.display(),
            std::process::id(),
            Uuid::new_v4()
        ));
        if let Err(error) = fs::write(&temporary, encoded) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(error) = fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600)) {
                let _ = fs::remove_file(&temporary);
                return Err(error);
            }
        }
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        *self.sessions.write().await = sessions;
        Ok(result)
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

        let token_for_store = session_token.clone();
        let _ = self
            .mutate(move |sessions| {
                sessions.insert(token_for_store, session);
            })
            .await;
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
        if self.reload_from_disk().await.is_err() {
            return None;
        }
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
        let token = session_token.to_owned();
        let _ = self.mutate(move |sessions| sessions.remove(&token)).await;
    }

    pub async fn set_o3k_token(&self, session_token: &str, token: String) -> bool {
        let session_token = session_token.to_owned();
        self.mutate(move |sessions| {
            let Some(session) = sessions.get_mut(&session_token) else {
                return false;
            };
            session.o3k_token = Some(token);
            true
        })
        .await
        .unwrap_or(false)
    }

    pub async fn set_openstack_project(&self, session_token: &str, project_id: String) -> bool {
        let session_token = session_token.to_owned();
        self.mutate(move |sessions| {
            let Some(session) = sessions.get_mut(&session_token) else {
                return false;
            };
            session.openstack_project_id = Some(project_id);
            true
        })
        .await
        .unwrap_or(false)
    }

    pub async fn set_openstack_token(&self, session_token: &str, token: String) -> bool {
        let session_token = session_token.to_owned();
        self.mutate(move |sessions| {
            let Some(session) = sessions.get_mut(&session_token) else {
                return false;
            };
            session.openstack_token = Some(token);
            true
        })
        .await
        .unwrap_or(false)
    }

    /// Rotate the session token (call after privilege change).
    /// Returns a new token; the old token is invalidated.
    pub async fn rotate(&self, old_token: &str) -> Option<String> {
        let new_token = Uuid::new_v4().to_string();
        let old_token = old_token.to_owned();
        let new_token_for_store = new_token.clone();
        self.mutate(move |sessions| {
            let session = sessions.remove(&old_token)?;
            sessions.insert(new_token_for_store, session);
            Some(new_token)
        })
        .await
        .ok()
        .flatten()
    }

    /// Number of active sessions (for health monitoring).
    pub async fn active_count(&self) -> usize {
        if self.reload_from_disk().await.is_err() {
            return 0;
        }
        self.sessions.read().await.len()
    }

    /// Clean up expired sessions.
    pub async fn reap_expired(&self) {
        let _ = self
            .mutate(|sessions| sessions.retain(|_, s| !s.is_expired()))
            .await;
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

    #[tokio::test]
    async fn encrypted_store_hides_tokens_and_merges_replica_writes() {
        let path =
            std::env::temp_dir().join(format!("araf-session-encrypted-{}.bin", Uuid::new_v4()));
        let key = [7u8; 32];
        let first = SessionStore::new_durable_encrypted(&path, key).expect("encrypted store");
        let second = SessionStore::new_durable_encrypted(&path, key).expect("second replica");
        let (first_token, second_token) = tokio::join!(
            first.create(
                "user-a".into(),
                "A".into(),
                "tenant-bff",
                Some("access-a".into()),
                None,
                None
            ),
            second.create(
                "user-b".into(),
                "B".into(),
                "operator-bff",
                Some("access-b".into()),
                None,
                None
            ),
        );
        let reopened = SessionStore::new_durable_encrypted(&path, key).expect("reopen");
        assert_eq!(
            reopened.validate(&first_token).await.unwrap().user_id,
            "user-a"
        );
        assert_eq!(
            reopened.validate(&second_token).await.unwrap().user_id,
            "user-b"
        );
        let bytes = fs::read(&path).expect("encrypted bytes");
        assert!(!bytes
            .windows(b"access-a".len())
            .any(|window| window == b"access-a"));
        assert!(!bytes
            .windows(b"access-b".len())
            .any(|window| window == b"access-b"));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("lock"));
    }

    #[tokio::test]
    async fn durable_replicas_preserve_concurrent_session_creates() {
        let path =
            std::env::temp_dir().join(format!("araf-session-concurrent-{}.bin", Uuid::new_v4()));
        let key = [9u8; 32];
        let first = SessionStore::new_durable_encrypted(&path, key).expect("first replica");
        let second = SessionStore::new_durable_encrypted(&path, key).expect("second replica");
        let first_task = {
            let store = first.clone();
            tokio::spawn(async move {
                for index in 0..8 {
                    store
                        .create(
                            format!("tenant-{index}"),
                            "Tenant".into(),
                            "tenant-bff",
                            Some(format!("tenant-token-{index}")),
                            None,
                            None,
                        )
                        .await;
                }
            })
        };
        let second_task = {
            let store = second.clone();
            tokio::spawn(async move {
                for index in 0..8 {
                    store
                        .create(
                            format!("operator-{index}"),
                            "Operator".into(),
                            "operator-bff",
                            Some(format!("operator-token-{index}")),
                            None,
                            None,
                        )
                        .await;
                }
            })
        };
        first_task.await.expect("first writer");
        second_task.await.expect("second writer");

        let reopened = SessionStore::new_durable_encrypted(&path, key).expect("reopen");
        assert_eq!(reopened.active_count().await, 16);
        let bytes = fs::read(&path).expect("session file");
        assert!(!bytes
            .windows(b"tenant-token-0".len())
            .any(|window| window == b"tenant-token-0"));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("lock"));
    }

    #[tokio::test]
    async fn deleted_authority_invalidates_existing_replica() {
        let path =
            std::env::temp_dir().join(format!("araf-session-deleted-{}.json", Uuid::new_v4()));
        let store = SessionStore::new_durable(&path).expect("store");
        let token = store
            .create("user".into(), "User".into(), "tenant-bff", None, None, None)
            .await;
        assert!(store.validate(&token).await.is_some());
        fs::remove_file(&path).expect("remove authority");
        assert!(store.validate(&token).await.is_none());
        assert_eq!(store.active_count().await, 0);
        let _ = fs::remove_file(path.with_extension("lock"));
    }

    #[tokio::test]
    async fn corrupt_authority_fails_closed() {
        let path =
            std::env::temp_dir().join(format!("araf-session-corrupt-{}.json", Uuid::new_v4()));
        let store = SessionStore::new_durable(&path).expect("store");
        let token = store
            .create("user".into(), "User".into(), "tenant-bff", None, None, None)
            .await;
        fs::write(&path, b"not-json").expect("corrupt authority");
        assert!(store.validate(&token).await.is_none());
        assert_eq!(store.active_count().await, 0);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("lock"));
    }
}
