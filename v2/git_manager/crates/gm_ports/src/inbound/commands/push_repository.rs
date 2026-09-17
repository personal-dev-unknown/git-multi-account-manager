// crates/gm_ports/src/inbound/commands/push_repository.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::errors::GitManagerError;

/// Command: stage all changes, commit, and push to the remote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRepositoryCommand {
    pub repository_uuid: Uuid,
    pub account_uuid:    Uuid,
    pub commit_message:  String,
    pub branch:          String,
    pub force:           bool,
    pub dry_run:         bool,
}

impl PushRepositoryCommand {
    pub fn validate(&self) -> Result<(), GitManagerError> {
        if self.commit_message.trim().is_empty() {
            return Err(GitManagerError::Other("commit message cannot be empty".to_string()));
        }
        if self.branch.trim().is_empty() {
            return Err(GitManagerError::Other("branch name cannot be empty".to_string()));
        }
        Ok(())
    }
}