// crates/gm_domain/src/git/events/merge_conflict_detected.rs
use uuid::Uuid;
use gm_shared::constants::events::MERGE_CONFLICT_DETECTED;

#[derive(Debug, Clone)]
pub struct MergeConflictDetected {
    pub repository_uuid:   Uuid,
    pub account_id:        Uuid,
    /// Paths of all conflicted files at the time of detection.
    pub conflicted_files:  Vec<String>,
    /// The branch being merged when the conflict was detected.
    pub source_branch:     String,
    /// The branch being merged into.
    pub target_branch:     String,
}

impl MergeConflictDetected {
    pub fn event_type() -> &'static str { MERGE_CONFLICT_DETECTED }
}