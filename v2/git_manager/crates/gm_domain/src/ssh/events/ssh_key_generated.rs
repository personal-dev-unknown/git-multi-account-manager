// crates/gm_domain/src/ssh/events/ssh_key_generated.rs
use uuid::Uuid;
use gm_shared::constants::events::SSH_KEY_GENERATED;

#[derive(Debug, Clone)]
pub struct SshKeyGenerated {
    pub ssh_key_uuid: Uuid,
    pub account_id:   Uuid,
    pub fingerprint:  String,
    pub public_key:   String,
    pub key_type_str: String,
    pub host_alias:   String,
}

impl SshKeyGenerated {
    pub fn event_type() -> &'static str { SSH_KEY_GENERATED }
}