// crates/gm_domain/src/repositories/ports/repository_repository.rs
//
// The persistence port for Repository entities. The domain service calls
// these methods; the MySQL and SQLite adapters implement them.

use async_trait::async_trait;
use uuid::Uuid;

use crate::repositories::entities::Repository;
use gm_shared::errors::GitManagerError;

#[async_trait]
pub trait RepositoryRepository: Send + Sync {
    async fn save(&self, repo: &Repository) -> Result<(), GitManagerError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Repository>, GitManagerError>;
    async fn find_by_local_path(&self, path: &str) -> Result<Option<Repository>, GitManagerError>;
    async fn find_by_full_name(&self, full_name: &str, account_id: Uuid) -> Result<Option<Repository>, GitManagerError>;
    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Repository>, GitManagerError>;
    async fn list_cloned(&self) -> Result<Vec<Repository>, GitManagerError>;
    async fn delete(&self, id: Uuid) -> Result<(), GitManagerError>;
}