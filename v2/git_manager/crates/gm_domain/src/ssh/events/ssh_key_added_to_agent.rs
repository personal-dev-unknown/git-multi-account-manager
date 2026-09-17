// crates/gm_domain/src/ssh/events/ssh_key_added_to_agent.rs
use uuid::Uuid;
use gm_shared::constants::events::SSH_KEY_ADDED_TO_AGENT;

#[derive(Debug, Clone)]
pub struct SshKeyAddedToAgent {
    pub ssh_key_uuid: Uuid,
    pub account_id:   Uuid,
}

impl SshKeyAddedToAgent {
    pub fn event_type() -> &'static str { SSH_KEY_ADDED_TO_AGENT }
}