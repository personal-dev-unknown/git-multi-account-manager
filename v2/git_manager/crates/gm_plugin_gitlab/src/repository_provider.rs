// crates/gm_plugin_gitlab/src/repository_provider.rs
//
// GitLab implementation of the `RepositoryProvider` outbound port.
//
// The provider translates GitLab REST API v4 responses into the platform-agnostic
// `RemoteRepositoryInfo` type defined in gm_ports. This allows the clone wizard
// and repository discovery commands to work identically regardless of which
// provider is active for a given account.
//
// ── Security ─────────────────────────────────────────────────────────────────
// The PAT is retrieved from the CredentialService at call time rather than stored
// in the provider struct. This means a rotated token is automatically used on the
// next API call without restarting the application.

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::repository_provider::{
    ForkRepositoryInput, PageCursor, RemoteRepositoryInfo, RepositoryProvider,
};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::GitLabClient;

/// GitLab implementation of `RepositoryProvider`.
pub struct GitLabRepositoryProvider {
    client: Arc<GitLabClient>,
    creds:  Arc<CredentialService>,
}

impl GitLabRepositoryProvider {
    pub fn new(client: Arc<GitLabClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }

    async fn get_token(&self, account_uuid: Uuid) -> Result<String, GitManagerError> {
        self.creds
            .get_token(account_uuid)
            .await?
            .ok_or_else(|| GitManagerError::Other(format!(
                "No GitLab credential stored for account {account_uuid}. \
                 Use 'git-zyrix account add' or 'git-zyrix account set-token' first."
            )))
    }
}

#[async_trait]
impl RepositoryProvider for GitLabRepositoryProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::GitLab
    }

    async fn list_repositories(
        &self,
        account_uuid: Uuid,
        cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let token    = self.get_token(account_uuid).await?;
        let page     = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page = cursor.as_ref().map(|c| c.per_page).unwrap_or(50);

        tracing::debug!(account = %account_uuid, page, per_page, "listing GitLab projects");

        let projects = self.client.list_repositories(&token, page, per_page).await?;
        let count    = projects.len() as u32;
        let has_next = count == per_page;

        let repos: Vec<RemoteRepositoryInfo> = projects
            .into_iter()
            .map(|p| p.to_remote_info())
            .collect();

        Ok((repos, PageCursor { page: page + 1, per_page, has_next }))
    }

    async fn get_repository(
        &self,
        account_uuid: Uuid,
        full_name:    &str,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        let token   = self.get_token(account_uuid).await?;
        let project = self.client.get_repository(&token, full_name).await?;
        Ok(project.to_remote_info())
    }

    async fn search_repositories(
        &self,
        account_uuid: Uuid,
        query:        &str,
        limit:        u32,
    ) -> Result<Vec<RemoteRepositoryInfo>, GitManagerError> {
        let token    = self.get_token(account_uuid).await?;
        let projects = self.client.search_repositories(&token, query, limit).await?;
        Ok(projects.into_iter().map(|p| p.to_remote_info()).collect())
    }

    async fn fork_repository(
        &self,
        account_uuid: Uuid,
        input:        ForkRepositoryInput,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        let token      = self.get_token(account_uuid).await?;
        let project    = self.client.get_repository(&token, &input.source_full_name).await?;
        let namespace  = input.target_namespace.as_deref();
        let forked     = self.client.fork_project(&token, project.id, namespace).await?;
        Ok(forked.to_remote_info())
    }

    async fn list_group_repositories(
        &self,
        account_uuid: Uuid,
        group:        &str,
        cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let token    = self.get_token(account_uuid).await?;
        let page     = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page = cursor.as_ref().map(|c| c.per_page).unwrap_or(50);

        let projects  = self.client.list_group_projects(&token, group, page, per_page).await?;
        let count     = projects.len() as u32;
        let has_next  = count == per_page;
        let repos: Vec<RemoteRepositoryInfo> = projects.into_iter().map(|p| p.to_remote_info()).collect();

        Ok((repos, PageCursor { page: page + 1, per_page, has_next }))
    }

    fn ssh_verification_banner(&self) -> Option<&str> {
        Some("Welcome to GitLab")
    }
}
