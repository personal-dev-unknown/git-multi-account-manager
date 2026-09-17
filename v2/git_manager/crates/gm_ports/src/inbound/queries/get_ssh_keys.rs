// crates/gm_ports/src/inbound/queries/get_ssh_keys.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::models::ssh_key::SshKeyDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSshKeysQuery {
    pub account_id:  Option<Uuid>,
    pub active_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSshKeysResult {
    pub keys:  Vec<SshKeyDto>,
    pub total: usize,
}