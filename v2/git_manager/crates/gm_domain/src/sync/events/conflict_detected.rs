// crates/gm_domain/src/sync/events/conflict_detected.rs
use uuid::Uuid;
use gm_shared::constants::events::CONFLICT_DETECTED;

#[derive(Debug, Clone)]
pub struct ConflictDetected {
    pub session_uuid:     Uuid,
    pub repository_uuid:  Uuid,
    pub account_id:       Uuid,
    pub conflicted_files: Vec<String>,
}

impl ConflictDetected {
    pub fn event_type() -> &'static str { CONFLICT_DETECTED }
}