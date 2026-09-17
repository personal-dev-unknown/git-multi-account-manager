use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::repository_provider::{
    PageCursor, RemoteRepositoryInfo, RepositoryProvider,
};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::CustomClient;

#[allow(dead_code)]
#[derive(Debug)]
pub struct CustomRepositoryProvider {
    client: Arc<CustomClient>,
    creds:  Arc<CredentialService>,
}

impl CustomRepositoryProvider {
    pub fn new(client: Arc<CustomClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }
}

#[async_trait]
impl RepositoryProvider for CustomRepositoryProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::SelfHosted("custom".to_string())
    }

    async fn list_repositories(
        &self,
        _account_uuid: Uuid,
        cursor: Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        // Self-hosted servers require explicit URL entry — no standard API.
        let next_cursor = PageCursor {
            page:     cursor.as_ref().map(|c| c.page).unwrap_or(1) + 1,
            per_page: cursor.as_ref().map(|c| c.per_page).unwrap_or(100),
            has_next: false,
        };
        Ok((vec![], next_cursor))
    }

    async fn get_repository(
        &self,
        _account_uuid: Uuid,
        full_name: &str,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        Err(GitManagerError::Other(format!(
            "Cannot look up '{}' on a self-hosted server. \
             Use manual URL entry with 'git-zyrix clone'.", full_name
        )))
    }

    async fn search_repositories(
        &self,
        _account_uuid: Uuid,
        _query: &str,
        _limit: u32,
    ) -> Result<Vec<RemoteRepositoryInfo>, GitManagerError> {
        Ok(vec![])
    }
}
