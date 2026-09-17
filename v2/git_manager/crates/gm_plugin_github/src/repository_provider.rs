// crates/gm_plugin_github/src/repository_provider.rs
//
// Implements gm_ports::outbound::RepositoryProvider for GitHub.
// When a method is called with an account_uuid, this provider looks up the
// plaintext PAT for that account via the CredentialService, then makes the
// corresponding GitHub REST API call.

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::repository_provider::{
    ForkRepositoryInput, PageCursor, RemoteRepositoryInfo, RepositoryProvider,
};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::GitHubClient;

#[derive(Debug)]
pub struct GitHubRepositoryProvider {
    client: Arc<GitHubClient>,
    creds:  Arc<CredentialService>,
}

impl GitHubRepositoryProvider {
    pub fn new(client: Arc<GitHubClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }

    async fn token_for(&self, account_uuid: Uuid) -> Result<String, GitManagerError> {
        self.creds
            .get_token(account_uuid)
            .await?
            .ok_or_else(|| GitManagerError::Other(
                format!("no GitHub credential found for account {account_uuid}")
            ))
    }
}

#[async_trait]
impl RepositoryProvider for GitHubRepositoryProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::GitHub
    }

    async fn list_repositories(
        &self,
        account_uuid: Uuid,
        cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let token    = self.token_for(account_uuid).await?;
        let page     = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page = cursor.as_ref().map(|c| c.per_page).unwrap_or(100);

        let repos = self.client.list_repositories(&token, page, per_page).await?;
        let has_next = repos.len() as u32 >= per_page;
        let infos: Vec<RemoteRepositoryInfo> = repos.iter().map(|r| r.to_remote_info()).collect();

        let next_cursor = PageCursor {
            page:     page + 1,
            per_page,
            has_next,
        };
        Ok((infos, next_cursor))
    }

    async fn get_repository(
        &self,
        account_uuid: Uuid,
        full_name:    &str,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        let token = self.token_for(account_uuid).await?;
        let repo  = self.client.get_repository(&token, full_name).await?;
        Ok(repo.to_remote_info())
    }

    async fn search_repositories(
        &self,
        account_uuid: Uuid,
        query:        &str,
        limit:        u32,
    ) -> Result<Vec<RemoteRepositoryInfo>, GitManagerError> {
        let token = self.token_for(account_uuid).await?;
        let repos = self.client.search_repositories(&token, query, limit).await?;
        Ok(repos.iter().map(|r| r.to_remote_info()).collect())
    }

    async fn fork_repository(
        &self,
        account_uuid: Uuid,
        input:        ForkRepositoryInput,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        let token = self.token_for(account_uuid).await?;
        let org   = input.target_namespace.as_deref();
        let repo  = self.client.fork_repository(&token, &input.source_full_name, org).await?;
        Ok(repo.to_remote_info())
    }

    async fn list_organization_repositories(
        &self,
        account_uuid: Uuid,
        organization: &str,
        cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let token    = self.token_for(account_uuid).await?;
        let page     = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page = cursor.as_ref().map(|c| c.per_page).unwrap_or(100);

        let repos    = self.client.list_organization_repositories(&token, organization, page, per_page).await?;
        let has_next = repos.len() as u32 >= per_page;
        let infos: Vec<RemoteRepositoryInfo> = repos.iter().map(|r| r.to_remote_info()).collect();

        Ok((infos, PageCursor { page: page + 1, per_page, has_next }))
    }

    async fn list_starred_repositories(
        &self,
        account_uuid: Uuid,
        cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let token    = self.token_for(account_uuid).await?;
        let page     = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page = cursor.as_ref().map(|c| c.per_page).unwrap_or(100);

        let repos    = self.client.list_starred_repositories(&token, page, per_page).await?;
        let has_next = repos.len() as u32 >= per_page;
        let infos: Vec<RemoteRepositoryInfo> = repos.iter().map(|r| r.to_remote_info()).collect();

        Ok((infos, PageCursor { page: page + 1, per_page, has_next }))
    }

    fn ssh_verification_banner(&self) -> Option<&str> {
        Some("Hi")
    }
}