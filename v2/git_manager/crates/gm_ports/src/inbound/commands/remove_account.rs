// crates/gm_ports/src/inbound/commands/remove_account.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::errors::GitManagerError;

/// Command: permanently remove a Git hosting account and all associated data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAccountCommand {
    pub account_uuid: Uuid,
    /// If true, also delete the SSH key files from the filesystem and
    /// remove the Host block from ~/.ssh/config.
    pub cleanup_ssh:  bool,
}

impl RemoveAccountCommand {
    pub fn validate(&self) -> Result<(), GitManagerError> { Ok(()) }
}