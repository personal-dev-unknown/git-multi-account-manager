// crates/gm_interface_web/src/session.rs
// Simple session cookie handling for the web interface.
// In v1 the session only stores the "active account" selection
// so the dashboard can remember which account the user is managing.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Session data stored in the signed cookie.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebSession {
    /// The UUID of the account currently selected in the dashboard.
    pub active_account_id: Option<Uuid>,
}

impl WebSession {
    pub fn set_active_account(&mut self, uuid: Uuid) { self.active_account_id = Some(uuid); }
    pub fn clear_active_account(&mut self) { self.active_account_id = None; }
}