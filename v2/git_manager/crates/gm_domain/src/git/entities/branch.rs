// crates/gm_domain/src/git/entities/branch.rs
//
// The Branch entity represents a git branch within a managed repository.
// Branches are ephemeral in the sense that they are re-fetched from the
// git binary on demand rather than persisted independently — the database
// tracks only the "current_branch" field on the repository record. This
// entity exists primarily as a typed carrier for branch metadata returned
// by git executor operations such as `git branch -a` and `git status`.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// The relationship between a local branch and its remote tracking branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchTrackingStatus {
    /// The local branch has no remote tracking reference.
    Untracked,
    /// The local branch is in sync with its tracking remote.
    UpToDate,
    /// The local branch has commits not yet pushed to the remote.
    Ahead { commits: u32 },
    /// The remote has commits not yet pulled into the local branch.
    Behind { commits: u32 },
    /// Both local and remote have diverged commits.
    Diverged { ahead: u32, behind: u32 },
}

/// A git branch as observed from the domain's perspective.
/// Constructed by the GitService from the output of git executor operations.
#[derive(Debug, Clone)]
pub struct Branch {
    /// Ephemeral UUID — not persisted; generated fresh each time git output is parsed.
    uuid:              Uuid,
    repository_uuid:   Uuid,
    name:              String,
    is_remote:         bool,
    is_current:        bool,
    remote_name:       Option<String>,
    tracking_status:   BranchTrackingStatus,
    last_commit_sha:   Option<String>,
    last_commit_msg:   Option<String>,
    last_commit_at:    Option<DateTime<Utc>>,
    last_commit_author: Option<String>,
}

impl Branch {
    pub fn new(
        repository_uuid: Uuid,
        name:            String,
        is_remote:       bool,
        is_current:      bool,
    ) -> Self {
        Self {
            uuid:               Uuid::new_v4(),
            repository_uuid,
            name,
            is_remote,
            is_current,
            remote_name:        None,
            tracking_status:    BranchTrackingStatus::Untracked,
            last_commit_sha:    None,
            last_commit_msg:    None,
            last_commit_at:     None,
            last_commit_author: None,
        }
    }

    pub fn with_tracking_status(mut self, status: BranchTrackingStatus) -> Self {
        self.tracking_status = status;
        self
    }

    pub fn with_last_commit(
        mut self,
        sha: String,
        msg: String,
        author: String,
        at: DateTime<Utc>,
    ) -> Self {
        self.last_commit_sha    = Some(sha);
        self.last_commit_msg    = Some(msg);
        self.last_commit_author = Some(author);
        self.last_commit_at     = Some(at);
        self
    }

    pub fn uuid(&self)              -> Uuid                         { self.uuid }
    pub fn repository_uuid(&self)   -> Uuid                         { self.repository_uuid }
    pub fn name(&self)              -> &str                         { &self.name }
    pub fn is_remote(&self)         -> bool                         { self.is_remote }
    pub fn is_current(&self)        -> bool                         { self.is_current }
    pub fn remote_name(&self)       -> Option<&str>                 { self.remote_name.as_deref() }
    pub fn tracking_status(&self)   -> &BranchTrackingStatus        { &self.tracking_status }
    pub fn last_commit_sha(&self)   -> Option<&str>                 { self.last_commit_sha.as_deref() }
    pub fn last_commit_msg(&self)   -> Option<&str>                 { self.last_commit_msg.as_deref() }
    pub fn last_commit_author(&self)-> Option<&str>                 { self.last_commit_author.as_deref() }
    pub fn last_commit_at(&self)    -> Option<DateTime<Utc>>        { self.last_commit_at }
}