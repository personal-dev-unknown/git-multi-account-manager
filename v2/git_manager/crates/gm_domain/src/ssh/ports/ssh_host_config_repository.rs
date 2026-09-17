// crates/gm_domain/src/ssh/ports/ssh_host_config_repository.rs

use async_trait::async_trait;
use uuid::Uuid;

use crate::ssh::entities::SshHostConfig;
use gm_shared::errors::SshError;

#[async_trait]
pub trait SshHostConfigRepository: Send + Sync + std::fmt::Debug {
    async fn save(&self, config: &SshHostConfig) -> Result<(), SshError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SshHostConfig>, SshError>;
    async fn find_by_host_alias(&self, alias: &str) -> Result<Option<SshHostConfig>, SshError>;
    async fn find_by_account(&self, account_id: Uuid) -> Result<Option<SshHostConfig>, SshError>;
    async fn list_all_active(&self) -> Result<Vec<SshHostConfig>, SshError>;
    async fn deactivate_for_account(&self, account_id: Uuid) -> Result<(), SshError>;
}