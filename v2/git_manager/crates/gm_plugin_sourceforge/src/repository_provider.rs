use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::repository_provider::{
    PageCursor, RemoteRepositoryInfo, RepositoryProvider,
};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::SourceForgeClient;

#[derive(Debug)]
pub struct SourceForgeRepositoryProvider {
    client: Arc<SourceForgeClient>,
    creds:  Arc<CredentialService>,
}

impl SourceForgeRepositoryProvider {
    pub fn new(client: Arc<SourceForgeClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }

    async fn username_for(&self, account_uuid: Uuid) -> Result<String, GitManagerError> {
        self.creds
            .get_token(account_uuid)
            .await?
            .ok_or_else(|| GitManagerError::Other(
                format!("no SourceForge credential found for account {account_uuid}")
            ))
    }
}

#[async_trait]
impl RepositoryProvider for SourceForgeRepositoryProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::SourceForge
    }

    async fn list_repositories(
        &self,
        account_uuid: Uuid,
        cursor:       Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let username = self.username_for(account_uuid).await?;
        let page     = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page = cursor.as_ref().map(|c| c.per_page).unwrap_or(100);

        let projects = self.client.list_projects(&username).await?;

        // SourceForge API doesn't support server-side pagination for user projects.
        // We fake pagination client-side.
        let start = ((page - 1) * per_page) as usize;
        let infos: Vec<RemoteRepositoryInfo> = projects
            .iter()
            .skip(start)
            .take(per_page as usize)
            .map(|p| p.to_remote_info(&username))
            .collect();

        let has_next = start + (per_page as usize) < projects.len();
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
        let username = self.username_for(account_uuid).await?;
        // Extract project name from full_name (username/project)
        let project_name = full_name.split('/').next_back().unwrap_or(full_name);

        let project = self.client.get_project(project_name).await?;
        Ok(project.to_remote_info(&username))
    }

    async fn search_repositories(
        &self,
        account_uuid: Uuid,
        query:        &str,
        limit:        u32,
    ) -> Result<Vec<RemoteRepositoryInfo>, GitManagerError> {
        let username = self.username_for(account_uuid).await?;
        let projects = self.client.list_projects(&username).await?;

        let q = query.to_lowercase();
        let results: Vec<RemoteRepositoryInfo> = projects
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&q)
                || p.summary.as_deref().unwrap_or("").to_lowercase().contains(&q))
            .take(limit as usize)
            .map(|p| p.to_remote_info(&username))
            .collect();
        Ok(results)
    }

    fn ssh_verification_banner(&self) -> Option<&str> {
        Some("SourceForge")
    }
}
