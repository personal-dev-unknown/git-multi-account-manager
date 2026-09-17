// crates/gm_interface_web/src/services.rs
// The type-erased application services facade for web handlers.
// Same pattern as gm_interface_cli/src/services.rs.

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use gm_shared::{errors::GitManagerError, models::{account::AccountDto, platform::PlatformDto, repository::RepositoryDto, ssh_key::SshKeyDto}};
use gm_ports::inbound::commands::{
    AddAccountCommand, CloneRepositoryCommand, GenerateSshKeyCommand,
    PullRepositoryCommand, PushRepositoryCommand, TestSshConnectionCommand,
};

pub struct SshTestResult          { pub success: bool, pub username: Option<String>, pub error: Option<String> }
pub struct SshValidationItemDto   { pub name: String, pub passed: bool, pub detail: String }
pub struct SshValidationResultDto { pub all_passed: bool, pub checks: Vec<SshValidationItemDto> }
pub struct GitOpResult            { pub commits_transferred: u32, pub current_sha: Option<String>, pub had_conflicts: bool }
pub struct GitStatusEntry         { pub path: String, pub status: String }
pub struct DryRunPreview          { pub summary: String, pub commits: u32, pub conflicts_predicted: bool, pub details: Vec<String> }
pub struct RepoDetectionResult    { pub tracked: bool, pub account_alias: Option<String>, pub platform_slug: Option<String>, pub platform_name: Option<String>, pub remote_url: Option<String>, pub local_path: Option<String>, pub current_branch: Option<String>, pub repository_name: Option<String> }
pub struct OperationLogEntry      { pub uuid: Uuid, pub op_type: String, pub status: String, pub account: Option<String>, pub repository: Option<String>, pub started_at: chrono::DateTime<chrono::Utc>, pub duration_ms: Option<u32>, pub error: Option<String> }

#[async_trait]
pub trait WebServices: Send + Sync {
    async fn list_accounts(&self, platform_id: Option<Uuid>) -> Result<Vec<AccountDto>, GitManagerError>;
    async fn get_account(&self, uuid: Uuid) -> Result<Option<AccountDto>, GitManagerError>;
    async fn get_account_by_alias(&self, alias: &str, platform_id: Option<Uuid>) -> Result<Option<AccountDto>, GitManagerError>;
    async fn add_account(&self, cmd: AddAccountCommand) -> Result<AccountDto, GitManagerError>;
    async fn remove_account(&self, uuid: Uuid) -> Result<(), GitManagerError>;
    async fn set_default_account(&self, account_uuid: Uuid, platform_id: Uuid) -> Result<(), GitManagerError>;
    async fn store_account_token(&self, account_uuid: Uuid, token: &str) -> Result<(), GitManagerError>;

    async fn list_ssh_keys(&self, account_uuid: Uuid) -> Result<Vec<SshKeyDto>, GitManagerError>;
    async fn generate_ssh_key(&self, cmd: GenerateSshKeyCommand) -> Result<SshKeyDto, GitManagerError>;
    async fn test_ssh_connection(&self, cmd: TestSshConnectionCommand) -> Result<SshTestResult, GitManagerError>;
    async fn validate_ssh_setup(&self, account_uuid: Uuid, timeout_ms: u32) -> Result<SshValidationResultDto, GitManagerError>;
    async fn add_key_to_agent(&self, key_uuid: Uuid, passphrase: Option<&str>) -> Result<(), GitManagerError>;

    async fn list_repositories(&self, account_uuid: Option<Uuid>) -> Result<Vec<RepositoryDto>, GitManagerError>;
    async fn clone_repository(&self, cmd: CloneRepositoryCommand) -> Result<RepositoryDto, GitManagerError>;
    async fn detect_repository(&self, path: &str) -> Result<RepoDetectionResult, GitManagerError>;
    async fn list_remote_repositories(&self, account_uuid: Uuid, page: u32, per_page: u32) -> Result<Vec<String>, GitManagerError>;

    async fn git_pull(&self, cmd: PullRepositoryCommand) -> Result<GitOpResult, GitManagerError>;
    async fn git_push(&self, cmd: PushRepositoryCommand) -> Result<GitOpResult, GitManagerError>;
    async fn git_status(&self, repository_uuid: Uuid) -> Result<Vec<GitStatusEntry>, GitManagerError>;
    async fn dry_run_pull(&self, repository_uuid: Uuid, account_uuid: Uuid) -> Result<DryRunPreview, GitManagerError>;
    async fn dry_run_push(&self, repository_uuid: Uuid, account_uuid: Uuid) -> Result<DryRunPreview, GitManagerError>;

    async fn config_list(&self) -> Result<Vec<(String, String)>, GitManagerError>;
    async fn config_get(&self, key: &str) -> Result<Option<String>, GitManagerError>;
    async fn config_set(&self, key: &str, value: &str) -> Result<(), GitManagerError>;

    async fn list_recent_operations(&self, account_uuid: Option<Uuid>, limit: u32) -> Result<Vec<OperationLogEntry>, GitManagerError>;

    async fn list_platforms(&self) -> Result<Vec<PlatformDto>, GitManagerError>;
}

/// Newtype so Arc<dyn WebServices> can be stored in the TypeId service registry.
pub struct WebServicesHandle(pub Arc<dyn WebServices>);

impl WebServicesHandle {
    pub fn new(inner: impl WebServices + 'static) -> Self { Self(Arc::new(inner)) }
    pub fn services(&self) -> &dyn WebServices { self.0.as_ref() }
}
