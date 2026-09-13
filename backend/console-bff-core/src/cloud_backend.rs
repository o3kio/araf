//! Server-side backend contract metadata.
//!
//! `Upstream` is the request contract consumed by handlers.  This module keeps
//! provenance and capability semantics explicit so adding a provider cannot
//! leak provider branches into React or turn compatibility state into cloud
//! authority.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    #[default]
    O3k,
    OpenStack,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationProvenance {
    O3k,
    OpenStack,
}

/// A capability is an observed, deployable feature rather than a role guess.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendCapability {
    pub id: String,
    pub enabled: bool,
    pub reason: Option<String>,
}

/// Implemented by every production backend in addition to `Upstream`.
pub trait CloudBackend: crate::upstream::Upstream {
    fn provenance(&self) -> OperationProvenance {
        match self.backend_kind() {
            BackendKind::O3k => OperationProvenance::O3k,
            BackendKind::OpenStack => OperationProvenance::OpenStack,
        }
    }
}
