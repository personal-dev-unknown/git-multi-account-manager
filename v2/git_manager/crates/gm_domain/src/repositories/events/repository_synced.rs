// crates/gm_domain/src/repositories/events/repository_synced.rs
//
// Published after any successful push or pull operation updates the repository's
// last_commit_sha and last_synced_at fields. Lightweight — carries only the
// UUIDs and the new commit SHA. Listeners that need more context re-fetch
// from the repository store.

use uuid::Uuid;
use gm_shared::constants::events::REPOSITORY_SYNCED;

#[derive(Debug, Clone)]
pub struct RepositorySynced {
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    /// The HEAD commit SHA after the sync completed.
    pub commit_sha:      String,
}

impl RepositorySynced {
    pub fn event_type() -> &'static str {
        REPOSITORY_SYNCED
    }
}