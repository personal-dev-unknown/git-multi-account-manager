// crates/gm_ports/src/inbound/queries/get_sync_history.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A compact view of a sync session for display in operation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHistoryEntry {
    pub session_uuid:    Uuid,
    pub repository_uuid: Uuid,
    pub full_name:       String,
    pub session_type:    String,
    pub status:          String,
    pub branch:          String,
    pub commit_sha:      Option<String>,
    pub commits_pushed:  u32,
    pub commits_pulled:  u32,
    pub error_message:   Option<String>,
    pub started_at:      DateTime<Utc>,
    pub completed_at:    Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSyncHistoryQuery {
    pub repository_uuid: Option<Uuid>,
    pub account_id:      Option<Uuid>,
    pub limit:           u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSyncHistoryResult {
    pub entries: Vec<SyncHistoryEntry>,
    pub total:   usize,
}