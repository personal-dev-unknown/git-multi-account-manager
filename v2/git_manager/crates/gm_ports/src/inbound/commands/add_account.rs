// crates/gm_ports/src/inbound/commands/add_account.rs
//
// The command that arrives from any interface (CLI, Web, Desktop) when a user
// wants to register a new Git hosting account. The command carries only the raw
// user-supplied data — no domain objects yet. The kernel's command bus passes
// this through the validation middleware (which calls validate()) and then
// to the account service handler, which constructs the domain Account entity.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::errors::GitManagerError;

/// Command: register a new Git hosting account.
///
/// The `platform_id` is the UUID of the platform record in the database
/// (pre-seeded on first boot). The interface layers look up platform UUIDs
/// when the user selects a platform from the dropdown or types "github".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAccountCommand {
    /// Human-readable label chosen by the user: "work", "personal", "client-acme"
    pub alias:       String,
    pub platform_id: Uuid,
    pub username:    String,
    pub email:       String,
    /// "ssh", "https_pat", "https_password", "oauth"
    pub auth_method: String,
}

impl AddAccountCommand {
    /// Validates the raw user input before the domain sees it.
    /// Returns Err only for structural impossibilities (empty required fields).
    /// Format rules (alias charset, email format) are enforced by the domain entity.
    pub fn validate(&self) -> Result<(), GitManagerError> {
        if self.alias.trim().is_empty() {
            return Err(GitManagerError::Other("alias cannot be empty".to_string()));
        }
        if self.username.trim().is_empty() {
            return Err(GitManagerError::Other("username cannot be empty".to_string()));
        }
        if self.email.trim().is_empty() {
            return Err(GitManagerError::Other("email cannot be empty".to_string()));
        }
        Ok(())
    }
}