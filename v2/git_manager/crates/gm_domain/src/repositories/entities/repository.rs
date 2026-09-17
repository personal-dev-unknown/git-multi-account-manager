// crates/gm_domain/src/repositories/entities/repository.rs
//
// The Repository entity. A repository can be in two states:
// "discovered" (we know it exists on the platform) and "cloned" (it has a
// local copy on disk). Both states are represented by the same struct; the
// local_path and is_cloned fields differentiate them.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::repositories::value_objects::{CloneStatus, RepositoryUrl};
use gm_shared::models::repository::{RepositoryDto, RepositoryVisibility};

/// Visibility as known by the domain — mirrors the shared DTO enum to avoid
/// the domain depending on the shared type's serialization derives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

impl Visibility {
    pub fn to_shared(&self) -> RepositoryVisibility {
        match self {
            Visibility::Public   => RepositoryVisibility::Public,
            Visibility::Private  => RepositoryVisibility::Private,
            Visibility::Internal => RepositoryVisibility::Internal,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "public"   => Visibility::Public,
            "internal" => Visibility::Internal,
            _          => Visibility::Private,
        }
    }
}

/// A Git repository managed by Git Manager.
#[derive(Debug, Clone)]
pub struct Repository {
    uuid:            Uuid,
    account_id:      Uuid,
    platform_id:     Uuid,
    name:            String,
    full_name:       String,
    description:     Option<String>,
    local_path:      Option<String>,
    remote_url:      RepositoryUrl,
    clone_url_ssh:   Option<String>,
    clone_url_https: Option<String>,
    default_branch:  String,
    current_branch:  Option<String>,
    visibility:      Visibility,
    is_cloned:       bool,
    is_archived:     bool,
    is_forked:       bool,
    last_commit_sha: Option<String>,
    last_synced_at:  Option<DateTime<Utc>>,
    created_at:      DateTime<Utc>,
}

impl Repository {
    /// Constructs a newly discovered repository (not yet cloned).
    pub fn new_discovered(
        account_id:   Uuid,
        platform_id:  Uuid,
        name:         String,
        full_name:    String,
        remote_url:   RepositoryUrl,
        default_branch: String,
        visibility:   Visibility,
    ) -> Self {
        Self {
            uuid:            Uuid::new_v4(),
            account_id,
            platform_id,
            name,
            full_name,
            description:     None,
            local_path:      None,
            remote_url,
            clone_url_ssh:   None,
            clone_url_https: None,
            default_branch,
            current_branch:  None,
            visibility,
            is_cloned:       false,
            is_archived:     false,
            is_forked:       false,
            last_commit_sha: None,
            last_synced_at:  None,
            created_at:      Utc::now(),
        }
    }

    /// Rehydrates from the database without re-running construction logic.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        uuid:            Uuid,
        account_id:      Uuid,
        platform_id:     Uuid,
        name:            String,
        full_name:       String,
        description:     Option<String>,
        local_path:      Option<String>,
        remote_url:      RepositoryUrl,
        clone_url_ssh:   Option<String>,
        clone_url_https: Option<String>,
        default_branch:  String,
        current_branch:  Option<String>,
        visibility:      Visibility,
        is_cloned:       bool,
        is_archived:     bool,
        is_forked:       bool,
        last_commit_sha: Option<String>,
        last_synced_at:  Option<DateTime<Utc>>,
        created_at:      DateTime<Utc>,
    ) -> Self {
        Self {
            uuid, account_id, platform_id, name, full_name, description,
            local_path, remote_url, clone_url_ssh, clone_url_https,
            default_branch, current_branch, visibility, is_cloned, is_archived,
            is_forked, last_commit_sha, last_synced_at, created_at,
        }
    }

    // ── Domain operations ─────────────────────────────────────────────────────

    /// Records that a successful clone has completed, setting the local path
    /// and the HEAD commit SHA from the clone output.
    pub fn mark_cloned(&mut self, local_path: String, commit_sha: String) {
        self.local_path      = Some(local_path);
        self.is_cloned       = true;
        self.last_commit_sha = Some(commit_sha);
        self.last_synced_at  = Some(Utc::now());
    }

    /// Updates the post-sync state after a push or pull completes.
    pub fn record_sync(&mut self, commit_sha: String) {
        self.last_commit_sha = Some(commit_sha);
        self.last_synced_at  = Some(Utc::now());
    }

    /// Returns the clone status as a computed value from the entity's state.
    pub fn clone_status(&self) -> CloneStatus {
        if self.is_cloned { CloneStatus::Cloned }
        else              { CloneStatus::NotCloned }
    }

    // ── Getters ───────────────────────────────────────────────────────────────
    pub fn uuid(&self)            -> Uuid                    { self.uuid }
    pub fn account_id(&self)      -> Uuid                    { self.account_id }
    pub fn platform_id(&self)     -> Uuid                    { self.platform_id }
    pub fn name(&self)            -> &str                    { &self.name }
    pub fn full_name(&self)       -> &str                    { &self.full_name }
    pub fn description(&self)     -> Option<&str>            { self.description.as_deref() }
    pub fn local_path(&self)      -> Option<&str>            { self.local_path.as_deref() }
    pub fn remote_url(&self)      -> &RepositoryUrl          { &self.remote_url }
    pub fn default_branch(&self)  -> &str                    { &self.default_branch }
    pub fn current_branch(&self)  -> Option<&str>            { self.current_branch.as_deref() }
    pub fn visibility(&self)      -> &Visibility             { &self.visibility }
    pub fn is_cloned(&self)       -> bool                    { self.is_cloned }
    pub fn is_archived(&self)     -> bool                    { self.is_archived }
    pub fn is_forked(&self)       -> bool                    { self.is_forked }
    pub fn last_commit_sha(&self) -> Option<&str>            { self.last_commit_sha.as_deref() }
    pub fn last_synced_at(&self)  -> Option<DateTime<Utc>>   { self.last_synced_at }
    pub fn created_at(&self)      -> DateTime<Utc>           { self.created_at }

    pub fn to_dto(&self) -> RepositoryDto {
        RepositoryDto {
            uuid:                self.uuid,
            account_id:          self.account_id,
            platform_id:         self.platform_id,
            name:                self.name.clone(),
            full_name:           self.full_name.clone(),
            description:         self.description.clone(),
            local_path:          self.local_path.clone(),
            remote_url:          self.remote_url.as_str().to_string(),
            clone_url_ssh:       self.clone_url_ssh.clone(),
            clone_url_https:     self.clone_url_https.clone(),
            default_branch:      self.default_branch.clone(),
            current_branch:      self.current_branch.clone(),
            visibility:          self.visibility.to_shared(),
            is_cloned:           self.is_cloned,
            is_archived:         self.is_archived,
            is_forked:           self.is_forked,
            last_commit_sha:     self.last_commit_sha.clone(),
            last_commit_message: None,
            last_commit_author:  None,
            last_commit_at:      None,
            last_synced_at:      self.last_synced_at,
            primary_language:    None,
            stargazers_count:    0,
            forks_count:         0,
            open_issues_count:   0,
            created_at:          self.created_at,
        }
    }
}