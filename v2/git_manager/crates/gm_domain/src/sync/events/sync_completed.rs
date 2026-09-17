// crates/gm_domain/src/sync/events/sync_completed.rs
use uuid::Uuid;
use gm_shared::constants::events::SYNC_COMPLETED;

#[derive(Debug, Clone)]
pub struct SyncCompleted {
    pub session_uuid:    Uuid,
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    pub commit_sha:      String,
    pub commits_pushed:  u32,
    pub commits_pulled:  u32,
    pub success:         bool,
}

impl SyncCompleted {
    pub fn event_type() -> &'static str { SYNC_COMPLETED }
}