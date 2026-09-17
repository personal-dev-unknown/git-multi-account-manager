// crates/gm_domain/src/sync/events/sync_started.rs
use uuid::Uuid;
use gm_shared::constants::events::SYNC_STARTED;

#[derive(Debug, Clone)]
pub struct SyncStarted {
    pub session_uuid:    Uuid,
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    pub session_type:    String,
}

impl SyncStarted {
    pub fn event_type() -> &'static str { SYNC_STARTED }
}