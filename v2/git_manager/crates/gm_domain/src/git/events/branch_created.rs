// crates/gm_domain/src/git/events/branch_created.rs
use uuid::Uuid;
use gm_shared::constants::events::BRANCH_CREATED;

#[derive(Debug, Clone)]
pub struct BranchCreated {
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    pub branch_name:     String,
    /// The commit SHA at which the branch was created.
    pub base_commit_sha: String,
}

impl BranchCreated {
    pub fn event_type() -> &'static str { BRANCH_CREATED }
}