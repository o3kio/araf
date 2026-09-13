//! Durable correlation journal for OpenStack CompatibilityOperations.
//!
//! The journal stores intent and last-observed authority only. It cannot mark
//! a resource successful without a fresh authoritative adapter observation.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::model::OperationState;

const MAX_RECORDS: usize = 1_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityRecord {
    pub operation_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub project_id: Option<String>,
    pub correlation_id: String,
    pub state: OperationState,
    pub observed_status: Option<String>,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct CompatibilityJournal {
    path: PathBuf,
    records: BTreeMap<String, CompatibilityRecord>,
}

impl CompatibilityJournal {
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_owned();
        let records = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(error) => return Err(error),
        };
        Ok(Self { path, records })
    }

    pub fn insert(&mut self, record: CompatibilityRecord) -> std::io::Result<()> {
        self.records.insert(record.operation_id.clone(), record);
        while self.records.len() > MAX_RECORDS {
            let oldest = self
                .records
                .iter()
                .min_by_key(|(_, record)| record.updated_at)
                .map(|(id, _)| id.clone());
            if let Some(id) = oldest {
                self.records.remove(&id);
            } else {
                break;
            }
        }
        self.flush()
    }

    pub fn get(&self, operation_id: &str) -> Option<&CompatibilityRecord> {
        self.records.get(operation_id)
    }

    pub fn update_authoritative_state(
        &mut self,
        operation_id: &str,
        status: &str,
        state: OperationState,
    ) -> std::io::Result<bool> {
        let Some(record) = self.records.get_mut(operation_id) else {
            return Ok(false);
        };
        record.observed_status = Some(status.to_owned());
        record.state = state;
        record.updated_at = OffsetDateTime::now_utc();
        self.flush()?;
        Ok(true)
    }

    pub fn records(&self) -> impl Iterator<Item = &CompatibilityRecord> {
        self.records.values()
    }

    fn flush(&self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = self.path.with_extension("tmp");
        fs::write(
            &temporary,
            serde_json::to_vec(&self.records).map_err(std::io::Error::other)?,
        )?;
        fs::rename(temporary, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_survives_reopen_and_only_authority_updates_state() {
        let path = std::env::temp_dir().join(format!("araf-compat-{}.json", uuid::Uuid::new_v4()));
        let mut journal = CompatibilityJournal::open(&path).expect("open");
        journal
            .insert(CompatibilityRecord {
                operation_id: "openstack-compat-1".into(),
                action: "delete".into(),
                resource_type: "compute.server".into(),
                resource_id: Some("srv".into()),
                project_id: Some("project".into()),
                correlation_id: "corr".into(),
                state: OperationState::Running,
                observed_status: None,
                updated_at: OffsetDateTime::now_utc(),
            })
            .expect("insert");
        drop(journal);
        let mut reopened = CompatibilityJournal::open(&path).expect("reopen");
        assert_eq!(
            reopened.get("openstack-compat-1").map(|r| r.state),
            Some(OperationState::Running)
        );
        reopened
            .update_authoritative_state("openstack-compat-1", "DELETED", OperationState::Succeeded)
            .expect("update");
        assert_eq!(
            CompatibilityJournal::open(&path)
                .expect("reopen")
                .get("openstack-compat-1")
                .map(|r| r.state),
            Some(OperationState::Succeeded)
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn journal_retention_is_bounded() {
        let path = std::env::temp_dir().join(format!(
            "araf-compat-retention-{}.json",
            uuid::Uuid::new_v4()
        ));
        let mut journal = CompatibilityJournal::open(&path).expect("open");
        for index in 0..=MAX_RECORDS {
            journal
                .insert(CompatibilityRecord {
                    operation_id: format!("operation-{index}"),
                    action: "create".into(),
                    resource_type: "compute.server".into(),
                    resource_id: None,
                    project_id: None,
                    correlation_id: index.to_string(),
                    state: OperationState::Running,
                    observed_status: None,
                    updated_at: OffsetDateTime::now_utc(),
                })
                .expect("insert");
        }
        assert_eq!(journal.records().count(), MAX_RECORDS);
        let _ = fs::remove_file(path);
    }
}
