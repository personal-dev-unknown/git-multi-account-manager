use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::repository_provider::{
    PageCursor, RemoteRepositoryInfo, RepositoryProvider,
};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::url_utils::generate_url;

#[allow(dead_code)]
#[derive(Debug)]
pub struct CloudStorageRepositoryProvider {
    creds: Arc<CredentialService>,
}

impl CloudStorageRepositoryProvider {
    pub fn new(creds: Arc<CredentialService>) -> Self {
        Self { creds }
    }
}

#[async_trait]
impl RepositoryProvider for CloudStorageRepositoryProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::CloudStorage("generic".to_string())
    }

    async fn list_repositories(
        &self,
        _account_uuid: Uuid,
        cursor: Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        // Cloud storage doesn't provide API-level repository listing.
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
        // Full name is "bucket/repo" for cloud storage
        let parts: Vec<&str> = full_name.split('/').collect();
        if parts.len() < 2 {
            return Err(GitManagerError::Other(format!(
                "Invalid cloud storage reference: '{full_name}'. Format: bucket/repo"
            )));
        }
        let bucket = parts[0];
        let repo = parts[1];

        Ok(RemoteRepositoryInfo {
            full_name:        full_name.to_string(),
            name:             repo.to_string(),
            description:      Some(format!("Cloud storage repository at {bucket}")),
            clone_url_ssh:    String::new(),
            clone_url_https:  generate_url("generic", bucket, repo),
            default_branch:   "main".to_string(),
            is_private:       true,
            is_forked:        false,
            is_archived:      false,
            stargazers:       0,
            forks:            0,
            open_issues:      0,
            primary_language: None,
        })
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
