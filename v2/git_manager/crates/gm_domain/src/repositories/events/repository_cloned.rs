// crates/gm_domain/src/repositories/events/repository_cloned.rs
//
// Published after git clone completes successfully and the repository record
// is updated with is_cloned=true and the local_path. The workflow engine
// subscribes to this event to trigger the "configure_local_identity" workflow
// step that writes the per-repository git config (user.email, user.name).
// Interface plugins subscribe to update clone progress displays.

use uuid::Uuid;
use gm_shared::constants::events::REPOSITORY_CLONED;

#[derive(Debug, Clone)]
pub struct RepositoryCloned {
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    /// "owner/repo" — for display without additional DB lookup.
    pub full_name:       String,
    /// Absolute path on the local filesystem where the repository was cloned.
    pub local_path:      String,
    /// The HEAD commit SHA after the successful clone.
    pub commit_sha:      String,
}

impl RepositoryCloned {
    pub fn event_type() -> &'static str {
        REPOSITORY_CLONED
    }
}