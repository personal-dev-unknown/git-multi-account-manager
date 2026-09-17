// crates/gm_ports/src/inbound/commands/generate_ssh_key.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::errors::GitManagerError;

/// Command: generate a new SSH key pair for an account and set it up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateSshKeyCommand {
    pub account_uuid:    Uuid,
    /// "ed25519" (recommended) or "rsa"
    pub key_type:        String,
    /// If None, the account's email is used as the key comment.
    pub comment:         Option<String>,
    /// If Some, the generated private key is encrypted with this passphrase.
    pub passphrase:      Option<String>,
    /// Whether to automatically add the key to the SSH agent after generation.
    pub add_to_agent:    bool,
}

impl GenerateSshKeyCommand {
    pub fn validate(&self) -> Result<(), GitManagerError> {
        let valid_types = ["ed25519", "rsa", "ecdsa"];
        if !valid_types.contains(&self.key_type.as_str()) {
            return Err(GitManagerError::Other(format!(
                "key_type must be one of: {}", valid_types.join(", ")
            )));
        }
        Ok(())
    }
}