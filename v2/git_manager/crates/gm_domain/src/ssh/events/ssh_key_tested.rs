// crates/gm_domain/src/ssh/events/ssh_key_tested.rs
use uuid::Uuid;
use gm_shared::constants::events::SSH_KEY_TESTED;

#[derive(Debug, Clone)]
pub struct SshKeyTested {
    pub ssh_key_uuid: Uuid,
    pub account_id:   Uuid,
    pub success:      bool,
    /// The platform-confirmed username if the test succeeded.
    pub username:     Option<String>,
    /// The failure reason if success is false.
    pub error:        Option<String>,
}

impl SshKeyTested {
    pub fn event_type() -> &'static str { SSH_KEY_TESTED }
}