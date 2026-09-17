// crates/gm_ports/src/inbound/commands/test_ssh_connection.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::errors::GitManagerError;

/// Command: test whether an account's SSH key authenticates with its platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSshConnectionCommand {
    pub account_uuid: Uuid,
    /// Timeout in milliseconds. If None, the application default is used.
    pub timeout_ms:   Option<u32>,
}

impl TestSshConnectionCommand {
    pub fn validate(&self) -> Result<(), GitManagerError> { Ok(()) }
}