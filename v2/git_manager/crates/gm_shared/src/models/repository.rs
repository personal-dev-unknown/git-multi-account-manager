//! Repository DTO and related types.
//!
//! A repository in Git Manager's model can be in one of two states:
//! "discovered" (we know it exists on the platform) or "cloned" (it also
//! exists locally on disk). The `RepositoryDto` captures both states in a
//! single type, with the `local_path` and clone-status fields differentiating
//! between them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Serializable snapshot of a repository record.
///
/// The `local_path` field is `None` for repositories that have been discovered
/// via the platform API but not yet cloned. It becomes `Some(path)` after a
/// successful `git clone` operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryDto {
    pub uuid:               Uuid,
    /// UUID of the account that owns this repository
    pub account_id:         Uuid,
    /// UUID of the platform this repository lives on
    pub platform_id:        Uuid,
    /// Short repository name without owner prefix: "my-project"
    pub name:               String,
    /// Full repository name with owner prefix: "shakamoses/my-project"
    pub full_name:          String,
    pub description:        Option<String>,
    /// Absolute local filesystem path. None if not yet cloned.
    pub local_path:         Option<String>,
    /// The remote URL used at clone time. SSH or HTTPS.
    pub remote_url:         String,
    pub clone_url_ssh:      Option<String>,
    pub clone_url_https:    Option<String>,
    pub default_branch:     String,
    /// The currently checked-out branch (None until cloned)
    pub current_branch:     Option<String>,
    pub visibility:         RepositoryVisibility,
    pub is_cloned:          bool,
    pub is_archived:        bool,
    pub is_forked:          bool,
    /// SHA of the most recent local commit (None until at least one pull/clone)
    pub last_commit_sha:    Option<String>,
    pub last_commit_message: Option<String>,
    pub last_commit_author: Option<String>,
    pub last_commit_at:     Option<DateTime<Utc>>,
    pub last_synced_at:     Option<DateTime<Utc>>,
    pub primary_language:   Option<String>,
    pub stargazers_count:   u32,
    pub forks_count:        u32,
    pub open_issues_count:  u32,
    pub created_at:         DateTime<Utc>,
}

/// Repository visibility level as reported by the hosting platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryVisibility {
    Public,
    Private,
    /// Only available on GitLab and GitHub Enterprise — visible to organization members
    Internal,
}

impl std::fmt::Display for RepositoryVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryVisibility::Public   => write!(f, "public"),
            RepositoryVisibility::Private  => write!(f, "private"),
            RepositoryVisibility::Internal => write!(f, "internal"),
        }
    }
}

impl RepositoryDto {
    /// Returns true if the repository exists locally on disk.
    /// A cloned repository has a local_path AND is_cloned == true.
    /// A repository with is_cloned == false is discovered but not yet local.
    pub fn is_local(&self) -> bool {
        self.is_cloned && self.local_path.is_some()
    }
}