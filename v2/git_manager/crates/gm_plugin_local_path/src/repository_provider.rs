use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::repository_provider::{
    PageCursor, RemoteRepositoryInfo, RepositoryProvider,
};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::url_utils::{is_git_repository, parse_local_path};

#[derive(Debug)]
pub struct LocalPathRepositoryProvider {
    creds: Arc<CredentialService>,
}

impl LocalPathRepositoryProvider {
    pub fn new(creds: Arc<CredentialService>) -> Self {
        Self { creds }
    }
}

#[async_trait]
impl RepositoryProvider for LocalPathRepositoryProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::LocalPath
    }

    async fn list_repositories(
        &self,
        account_uuid: Uuid,
        cursor: Option<PageCursor>,
    ) -> Result<(Vec<RemoteRepositoryInfo>, PageCursor), GitManagerError> {
        let base_path = self.creds
            .get_token(account_uuid)
            .await?
            .unwrap_or_else(|| dirs_next::home_dir()
                .map(|d| d.to_string_lossy().to_string())
                .unwrap_or_default());

        let dir = std::path::Path::new(&base_path);
        let mut infos = Vec::new();

        if dir.exists() {
            let mut entries: Vec<_> = match dir.read_dir() {
                Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
                Err(_) => vec![],
            };
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let path = entry.path();
                if path.is_dir() && is_git_repository(&path) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    infos.push(RemoteRepositoryInfo {
                        full_name:        name.clone(),
                        name,
                        description:      Some(format!("Local repository at {}", path.display())),
                        clone_url_ssh:    String::new(),
                        clone_url_https:  path.to_string_lossy().to_string(),
                        default_branch:   "main".to_string(),
                        is_private:       true,
                        is_forked:        false,
                        is_archived:      false,
                        stargazers:       0,
                        forks:            0,
                        open_issues:      0,
                        primary_language: None,
                    });
                }
            }
        }

        let page = cursor.as_ref().map(|c| c.page).unwrap_or(1);
        let per_page = cursor.as_ref().map(|c| c.per_page).unwrap_or(100);
        let start = ((page - 1) * per_page) as usize;
        let page_items: Vec<RemoteRepositoryInfo> = infos.into_iter().skip(start).take(per_page as usize).collect();
        let has_next = start + (per_page as usize) < page_items.len();

        Ok((page_items, PageCursor {
            page: page + 1,
            per_page,
            has_next,
        }))
    }

    async fn get_repository(
        &self,
        _account_uuid: Uuid,
        full_name: &str,
    ) -> Result<RemoteRepositoryInfo, GitManagerError> {
        let info = parse_local_path(full_name)
            .ok_or_else(|| GitManagerError::Other(format!("Invalid local path: {full_name}")))?;

        if !is_git_repository(&info.absolute) {
            return Err(GitManagerError::Other(format!(
                "Not a git repository: {}", info.absolute.display()
            )));
        }

        Ok(RemoteRepositoryInfo {
            full_name:        full_name.to_string(),
            name:             info.repo,
            description:      Some(format!("Local repository at {}", info.absolute.display())),
            clone_url_ssh:    String::new(),
            clone_url_https:  info.absolute.to_string_lossy().to_string(),
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
