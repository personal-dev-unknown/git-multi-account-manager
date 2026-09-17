use uuid::Uuid;
use gm_shared::constants::events::CLONE_PROGRESS_UPDATED;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CloneProgressUpdated {
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    pub url:             String,
    /// Percentage as f64 with 2 decimal places (0.00 — 100.00).
    pub percentage:      f64,
    pub bytes_transferred: u64,
    pub total_bytes:     u64,
    pub speed_bps:       u64,
    pub stage:           String,
}

impl CloneProgressUpdated {
    pub fn event_type() -> &'static str { CLONE_PROGRESS_UPDATED }
}
