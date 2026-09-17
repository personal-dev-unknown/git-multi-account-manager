// crates/gm_ports/src/inbound/commands/sync_repository.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::errors::GitManagerError;

/// Command: bidirectional sync — pull then push.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRepositoryCommand {
    pub repository_uuid: Uuid,
    pub account_uuid:    Uuid,
    pub branch:          String,
    pub commit_message:  String,
}

impl SyncRepositoryCommand {
    pub fn validate(&self) -> Result<(), GitManagerError> {
        if self.commit_message.trim().is_empty() {
            return Err(GitManagerError::Other("commit message cannot be empty".to_string()));
        }
        Ok(())
    }
}