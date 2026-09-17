// crates/gm_domain/src/repositories/services/repository_service.rs

use std::sync::Arc;
use uuid::Uuid;

use crate::repositories::{
    entities::Repository,
    events::{RepositoryCloned, RepositoryDiscovered, RepositorySynced},
    ports::RepositoryRepository,
    value_objects::RepositoryUrl,
};
use crate::repositories::entities::repository::Visibility;
use gm_shared::errors::GitManagerError;
use gm_shared::models::repository::RepositoryDto;

#[derive(Debug)]
pub struct DiscoverResult {
    pub repository: Repository,
    pub event:      RepositoryDiscovered,
}

#[derive(Debug)]
pub struct CloneResult {
    pub repository: Repository,
    pub event:      RepositoryCloned,
}

/// Domain service orchestrating repository discovery, clone tracking, and sync state.
#[derive(Debug)]
pub struct RepositoryService<R: RepositoryRepository> {
    repository: Arc<R>,
}

impl<R: RepositoryRepository> RepositoryService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Registers a repository discovered via the platform API.
    /// Called before cloning — creates the record in NotCloned state.
    pub async fn register_discovered(
        &self,
        account_id:     Uuid,
        platform_id:    Uuid,
        name:           String,
        full_name:      String,
        remote_url:     String,
        default_branch: String,
        visibility:     Visibility,
    ) -> Result<DiscoverResult, GitManagerError> {
        // Idempotent: if already registered, return the existing record
        if let Some(existing) = self.repository
            .find_by_full_name(&full_name, account_id)
            .await?
        {
            let event = RepositoryDiscovered {
                repository_uuid: existing.uuid(),
                account_id,
                full_name:       existing.full_name().to_string(),
            };
            return Ok(DiscoverResult { repository: existing, event });
        }

        let url = RepositoryUrl::new(remote_url).map_err(|e| {
            GitManagerError::Other(format!("invalid remote URL: {e}"))
        })?;

        let repo = Repository::new_discovered(
            account_id, platform_id, name, full_name.clone(), url, default_branch, visibility,
        );
        self.repository.save(&repo).await?;

        let event = RepositoryDiscovered {
            repository_uuid: repo.uuid(),
            account_id,
            full_name,
        };
        Ok(DiscoverResult { repository: repo, event })
    }

    /// Marks a repository as successfully cloned, storing the local path and commit SHA.
    pub async fn mark_cloned(
        &self,
        repo_uuid:   Uuid,
        local_path:  String,
        commit_sha:  String,
    ) -> Result<CloneResult, GitManagerError> {
        let mut repo = self.get_repository(repo_uuid).await?;
        repo.mark_cloned(local_path, commit_sha.clone());
        self.repository.save(&repo).await?;

        let event = RepositoryCloned {
            repository_uuid: repo_uuid,
            account_id:      repo.account_id(),
            full_name:       repo.full_name().to_string(),
            local_path:      repo.local_path().unwrap_or("").to_string(),
            commit_sha,
        };
        Ok(CloneResult { repository: repo, event })
    }

    /// Updates sync state after a push or pull completes.
    pub async fn record_sync(
        &self,
        repo_uuid:  Uuid,
        commit_sha: String,
    ) -> Result<RepositorySynced, GitManagerError> {
        let mut repo = self.get_repository(repo_uuid).await?;
        repo.record_sync(commit_sha.clone());
        self.repository.save(&repo).await?;
        Ok(RepositorySynced {
            repository_uuid: repo_uuid,
            account_id:      repo.account_id(),
            commit_sha,
        })
    }

    pub async fn get_repository(&self, uuid: Uuid) -> Result<Repository, GitManagerError> {
        self.repository
            .find_by_id(uuid)
            .await?
            .ok_or_else(|| GitManagerError::Other(format!("repository {uuid} not found")))
    }

    pub async fn find_by_local_path(&self, path: &str) -> Result<Option<Repository>, GitManagerError> {
        self.repository.find_by_local_path(path).await
    }

    pub async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<RepositoryDto>, GitManagerError> {
        let repos = self.repository.list_by_account(account_id).await?;
        Ok(repos.iter().map(Repository::to_dto).collect())
    }

    pub async fn list_cloned(&self) -> Result<Vec<RepositoryDto>, GitManagerError> {
        let repos = self.repository.list_cloned().await?;
        Ok(repos.iter().map(Repository::to_dto).collect())
    }
}