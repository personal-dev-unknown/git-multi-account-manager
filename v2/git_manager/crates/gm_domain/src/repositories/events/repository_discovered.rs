// crates/gm_domain/src/repositories/events/repository_discovered.rs
//
// Published by RepositoryService::register_discovered() after a repository
// record is written to the database for the first time. Listeners can use this
// event to initiate background jobs — for example, pre-fetching repository
// statistics from the platform API, or updating the UI's repo count badge.

use uuid::Uuid;
use gm_shared::constants::events::REPOSITORY_DISCOVERED;

#[derive(Debug, Clone)]
pub struct RepositoryDiscovered {
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    /// "owner/repo" format — useful for display without a follow-up DB query.
    pub full_name:       String,
}

impl RepositoryDiscovered {
    pub fn event_type() -> &'static str {
        REPOSITORY_DISCOVERED
    }
}