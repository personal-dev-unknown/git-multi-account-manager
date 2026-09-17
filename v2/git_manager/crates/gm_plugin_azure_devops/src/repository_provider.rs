// crates/gm_plugin_azure_devops/src/repository_provider.rs
//
// Azure DevOps implementation of the `RepositoryProvider` outbound port.
//
// Azure DevOps structures resources as organization → project → repository.
// The credential format is "organization:pat". The client splits it internally.
// Repository listing fetches all projects in the org first, then all repos per project.
//
// ── Security ─────────────────────────────────────────────────────────────────
// The full credential string is retrieved from CredentialService at call time.
// A rotated PAT is automatically used on the next API call without restart.

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::repository_provider::{
    ForkRepositoryInput, PageCursor, RemoteRepositoryInfo, RepositoryProvider,
};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::AzureDevOpsClient;

/// Azure DevOps implementation of `RepositoryProvider`.
pub struct AzureDevOpsRepositoryProvider {
    client: Arc<AzureDevOpsClient>,
    creds:  Arc<CredentialService>,
}

impl AzureDevOpsRepositoryProvider {
    pub fn new(client: Arc<AzureDevOpsClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }

    async fn get_credential(&self, account_uuid: Uuid) -> Result<String, GitManagerError> {
        self.creds
            .get_token(account_uuid)
            .await?
            .ok_or_else(|| GitManagerError::Other(format!(
                "No Azure DevOps credential stored for account {account_uuid}. \
                 Use 'git-zyrix account add' or 'git-zyrix account set-token' first."
            )))
    }
}

#[async_trait]
impl RepositoryProvider for AzureDevOpsRepositoryProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::AzureDevOps
    }

    /// Lists all repositories across all projects in the organization.
    ///
    /// Azure DevOps does not have a flat repo-list endpoint; this fetches all
    /// projects first, then all repos per project. `cursor` is used only for
    /// page accounting — the client loads all repos in one pass and we slice
    /// the result by `per_page`.
    async fn list_repositories(
        &self,
        account_uuid: Uuid,
        cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let credential = self.get_credential(account_uuid).await?;
        let page       = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page   = cursor.as_ref().map(|c| c.per_page).unwrap_or(50) as usize;

        tracing::debug!(account = %account_uuid, page, "listing Azure DevOps repositories");

        let all = self.client.list_repositories(&credential).await?;

        let start    = ((page - 1) as usize) * per_page;
        let slice    = all.into_iter().skip(start).take(per_page).collect::<Vec<_>>();
        let has_next = slice.len() == per_page;

        let repos: Vec<RemoteRepositoryInfo> = slice
            .into_iter()
            .map(|(repo, org)| repo.to_remote_info(&org))
            .collect();

        Ok((repos, PageCursor { page: page + 1, per_page: per_page as u32, has_next }))
    }

    /// Fetches a single repository by its "organization/project/repo" full name.
    async fn get_repository(
        &self,
        account_uuid: Uuid,
        full_name:    &str,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        let credential = self.get_credential(account_uuid).await?;
        let (repo, org) = self.client.get_repository(&credential, full_name).await?;
        Ok(repo.to_remote_info(&org))
    }

    /// Searches repositories by name (client-side filtering across all projects).
    async fn search_repositories(
        &self,
        account_uuid: Uuid,
        query:        &str,
        limit:        u32,
    ) -> Result<Vec<RemoteRepositoryInfo>, GitManagerError> {
        let credential = self.get_credential(account_uuid).await?;
        let results    = self.client.search_repositories(&credential, query, limit).await?;
        Ok(results.into_iter().map(|(repo, org)| repo.to_remote_info(&org)).collect())
    }

    async fn fork_repository(
        &self,
        account_uuid: Uuid,
        input:        ForkRepositoryInput,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        let credential        = self.get_credential(account_uuid).await?;
        let target_name       = input.target_namespace.as_deref();
        let (repo, org)       = self.client.fork_repository(&credential, &input.source_full_name, target_name).await?;
        Ok(repo.to_remote_info(&org))
    }
}