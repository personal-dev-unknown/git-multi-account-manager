// crates/gm_ports/src/inbound/commands/pull_repository.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::errors::GitManagerError;

/// Command: pull changes from the remote into a local repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRepositoryCommand {
    pub repository_uuid: Uuid,
    pub account_uuid:    Uuid,
    pub branch:          Option<String>,
    pub rebase:          bool,
    pub dry_run:         bool,
}

impl PullRepositoryCommand {
    pub fn validate(&self) -> Result<(), GitManagerError> { Ok(()) }
}