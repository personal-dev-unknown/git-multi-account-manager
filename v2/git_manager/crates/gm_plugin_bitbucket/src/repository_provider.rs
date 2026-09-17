// crates/gm_plugin_bitbucket/src/repository_provider.rs
//
// Bitbucket implementation of the `RepositoryProvider` outbound port.
//
// The credential stored in the vault is "username:app_password". The client
// splits it internally before constructing Basic auth headers, so the provider
// retrieves the full credential string and passes it through.

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::repository_provider::{
    ForkRepositoryInput, PageCursor, RemoteRepositoryInfo, RepositoryProvider,
};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::BitbucketClient;

/// Bitbucket implementation of `RepositoryProvider`.
pub struct BitbucketRepositoryProvider {
    client: Arc<BitbucketClient>,
    creds:  Arc<CredentialService>,
}

impl BitbucketRepositoryProvider {
    pub fn new(client: Arc<BitbucketClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }

    async fn get_credential(&self, account_uuid: Uuid) -> Result<String, GitManagerError> {
        self.creds
            .get_token(account_uuid)
            .await?
            .ok_or_else(|| GitManagerError::Other(format!(
                "No Bitbucket credential stored for account {account_uuid}. \
                 Use 'git-zyrix account add' or 'git-zyrix account set-token' first."
            )))
    }
}

#[async_trait]
impl RepositoryProvider for BitbucketRepositoryProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::Bitbucket
    }

    async fn list_repositories(
        &self,
        account_uuid: Uuid,
        cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let credential = self.get_credential(account_uuid).await?;
        let page       = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page   = cursor.as_ref().map(|c| c.per_page).unwrap_or(50);

        tracing::debug!(account = %account_uuid, page, per_page, "listing Bitbucket repositories");

        let page_resp = self.client.list_repositories(&credential, page, per_page).await?;
        let count     = page_resp.values.len() as u32;
        let has_next  = page_resp.next.is_some() || count == per_page;

        let repos: Vec<RemoteRepositoryInfo> = page_resp.values
            .into_iter()
            .map(|r| r.to_remote_info())
            .collect();

        Ok((repos, PageCursor { page: page + 1, per_page, has_next }))
    }

    async fn get_repository(
        &self,
        account_uuid: Uuid,
        full_name:    &str,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        let credential = self.get_credential(account_uuid).await?;
        let repo       = self.client.get_repository(&credential, full_name).await?;
        Ok(repo.to_remote_info())
    }

    async fn search_repositories(
        &self,
        account_uuid: Uuid,
        query:        &str,
        limit:        u32,
    ) -> Result<Vec<RemoteRepositoryInfo>, GitManagerError> {
        let credential = self.get_credential(account_uuid).await?;
        let repos      = self.client.search_repositories(&credential, query, limit).await?;
        Ok(repos.into_iter().map(|r| r.to_remote_info()).collect())
    }

    async fn fork_repository(
        &self,
        account_uuid: Uuid,
        input:        ForkRepositoryInput,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        let credential = self.get_credential(account_uuid).await?;
        let ws         = input.target_namespace.as_deref();
        let repo       = self.client.fork_repository(&credential, &input.source_full_name, ws).await?;
        Ok(repo.to_remote_info())
    }

    async fn list_team_repositories(
        &self,
        account_uuid: Uuid,
        team:         &str,
        cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let credential = self.get_credential(account_uuid).await?;
        let page       = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page   = cursor.as_ref().map(|c| c.per_page).unwrap_or(50);

        let resp      = self.client.list_team_repositories(&credential, team, page, per_page).await?;
        let count     = resp.values.len() as u32;
        let has_next  = resp.next.is_some() || count == per_page;
        let repos: Vec<RemoteRepositoryInfo> = resp.values
            .into_iter()
            .map(|r| r.to_remote_info())
            .collect();

        Ok((repos, PageCursor { page: page + 1, per_page, has_next }))
    }

    fn ssh_verification_banner(&self) -> Option<&str> {
        Some("authenticated via ssh key")
    }
}