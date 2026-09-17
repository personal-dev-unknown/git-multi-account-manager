// crates/gm_domain/src/git/events/commit_created.rs
use uuid::Uuid;
use gm_shared::constants::events::COMMIT_CREATED;

#[derive(Debug, Clone)]
pub struct CommitCreated {
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    pub commit_sha:      String,
    pub commit_message:  String,
}

impl CommitCreated {
    pub fn event_type() -> &'static str { COMMIT_CREATED }
}