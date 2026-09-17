// crates/gm_domain/src/git/entities/commit.rs
//
// The Commit entity represents a git commit created or observed during a
// managed operation. Like Branch, commits are ephemeral domain objects — they
// are constructed from git executor output rather than stored independently.
// The persistent record of "what is the last commit on this repo" lives in
// Repository::last_commit_sha and SyncSession::commit_sha. This entity
// provides the richer per-commit context needed during an active operation.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A git commit as parsed from git log or git commit output.
#[derive(Debug, Clone)]
pub struct Commit {
    uuid:            Uuid,
    repository_uuid: Uuid,
    sha:             String,
    short_sha:       String,
    message:         String,
    author_name:     String,
    author_email:    String,
    committed_at:    DateTime<Utc>,
    parent_shas:     Vec<String>,
    is_merge:        bool,
}

impl Commit {
    pub fn new(
        repository_uuid: Uuid,
        sha:             String,
        message:         String,
        author_name:     String,
        author_email:    String,
        committed_at:    DateTime<Utc>,
        parent_shas:     Vec<String>,
    ) -> Self {
        // The conventional short SHA is the first 7 characters.
        let short_sha = sha.chars().take(7).collect();
        let is_merge  = parent_shas.len() > 1;
        Self {
            uuid: Uuid::new_v4(),
            repository_uuid,
            sha,
            short_sha,
            message,
            author_name,
            author_email,
            committed_at,
            is_merge,
            parent_shas,
        }
    }

    /// Returns the first line of the commit message — the "subject" line
    /// as defined by git convention. Used in compact log display.
    pub fn subject(&self) -> &str {
        self.message.lines().next().unwrap_or("")
    }

    pub fn uuid(&self)            -> Uuid             { self.uuid }
    pub fn repository_uuid(&self) -> Uuid             { self.repository_uuid }
    pub fn sha(&self)             -> &str             { &self.sha }
    pub fn short_sha(&self)       -> &str             { &self.short_sha }
    pub fn message(&self)         -> &str             { &self.message }
    pub fn author_name(&self)     -> &str             { &self.author_name }
    pub fn author_email(&self)    -> &str             { &self.author_email }
    pub fn committed_at(&self)    -> DateTime<Utc>    { self.committed_at }
    pub fn parent_shas(&self)     -> &[String]        { &self.parent_shas }
    pub fn is_merge(&self)        -> bool             { self.is_merge }
}