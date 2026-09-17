// crates/gm_ports/src/outbound/repository_provider.rs
//
// The RepositoryProvider outbound port — what a platform plugin must implement
// to make repositories browsable and clonable via the platform's API.
// GitHub, GitLab, Bitbucket, and AzureDevOps plugins each implement this trait.
// The domain never calls this directly; the kernel's workflow engine uses it to
// list and discover repositories before handing them to the domain service.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

/// Metadata about a remote repository returned by the platform API.
/// This is the provider's view — richer than the domain's RepositoryUrl
/// because it includes platform-specific metadata like star counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRepositoryInfo {
    pub full_name:       String,
    pub name:            String,
    pub description:     Option<String>,
    pub clone_url_ssh:   String,
    pub clone_url_https: String,
    pub default_branch:  String,
    pub is_private:      bool,
    pub is_forked:       bool,
    pub is_archived:     bool,
    pub stargazers:      u32,
    pub forks:           u32,
    pub open_issues:     u32,
    pub primary_language: Option<String>,
}

/// Pagination cursor for multi-page repository lists.
#[derive(Debug, Clone)]
pub struct PageCursor {
    pub page:     u32,
    pub per_page: u32,
    pub has_next: bool,
}

/// Input parameters for forking a repository.
#[derive(Debug, Clone)]
pub struct ForkRepositoryInput {
    /// The full name of the source repository (e.g. "owner/repo").
    pub source_full_name: String,
    /// Optional target namespace (org or user) to fork into.
    /// If None, the platform's default (usually the authenticated user) is used.
    pub target_namespace: Option<String>,
}

/// The platform-specific repository access trait.
/// Provider plugins implement this; the kernel calls it during the clone wizard.
///
/// All methods added after the original four have default stub implementations
/// that return an error so plugins retain compile-time backward compatibility.
#[async_trait]
pub trait RepositoryProvider: Send + Sync {
    /// Returns the platform type this provider handles.
    fn platform_type(&self) -> PlatformType;

    /// Lists repositories accessible to the authenticated account.
    async fn list_repositories(
        &self,
        account_uuid: Uuid,
        cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError>;

    /// Fetches metadata for a single repository by its full name (owner/repo).
    async fn get_repository(
        &self,
        account_uuid: Uuid,
        full_name:    &str,
    ) -> Result<RemoteRepositoryInfo, GitManagerError>;

    /// Searches repositories matching the given query string.
    async fn search_repositories(
        &self,
        account_uuid: Uuid,
        query:        &str,
        limit:        u32,
    ) -> Result<Vec<RemoteRepositoryInfo>, GitManagerError>;

    // ── Phase 2 extensions with default stubs ───────────────────────────────

    /// Forks a repository to the authenticated user's account (or a target namespace).
    async fn fork_repository(
        &self,
        _account_uuid: Uuid,
        _input:        ForkRepositoryInput,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        Err(GitManagerError::Other(format!(
            "{} does not support forking",
            self.platform_type(),
        )))
    }

    /// Lists repositories belonging to a specific organisation (GitHub, Azure DevOps).
    async fn list_organization_repositories(
        &self,
        _account_uuid:  Uuid,
        _organization:  &str,
        _cursor:        Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        Err(GitManagerError::Other(format!(
            "{} does not support organisation repository listing",
            self.platform_type(),
        )))
    }

    /// Lists repositories belonging to a specific group (GitLab).
    async fn list_group_repositories(
        &self,
        _account_uuid: Uuid,
        _group:        &str,
        _cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        Err(GitManagerError::Other(format!(
            "{} does not support group repository listing",
            self.platform_type(),
        )))
    }

    /// Lists repositories belonging to a team (Bitbucket).
    async fn list_team_repositories(
        &self,
        _account_uuid: Uuid,
        _team:         &str,
        _cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        Err(GitManagerError::Other(format!(
            "{} does not support team repository listing",
            self.platform_type(),
        )))
    }

    /// Lists repositories the authenticated user has starred (GitHub).
    async fn list_starred_repositories(
        &self,
        _account_uuid: Uuid,
        _cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        Err(GitManagerError::Other(format!(
            "{} does not support listing starred repositories",
            self.platform_type(),
        )))
    }

    /// Returns the platform-specific SSH banner string that indicates a successful
    /// authentication when present in the SSH connection response.
    /// Used by SshService to verify the SSH key was accepted by the platform.
    fn ssh_verification_banner(&self) -> Option<&str> {
        None
    }
}