// crates/gm_interface_cli/src/services.rs
//
// The application-level port that isolates every CLI command handler from the
// infrastructure layers beneath it. Command handlers call methods on this
// trait object without knowing whether they talk to MySQL, SQLite, the Zig SSH
// layer, or an in-memory mock for tests.
//
// ── Field alignment ───────────────────────────────────────────────────────────
// Every method that accepts a gm_ports command type uses the EXACT field names
// from those structs (verified against the actual files):
//
//   AddAccountCommand        → alias, platform_id, username, email, auth_method
//   CloneRepositoryCommand   → url, account_uuid, destination, branch, shallow
//   GenerateSshKeyCommand    → account_uuid, key_type, comment, passphrase, add_to_agent
//   PullRepositoryCommand    → repository_uuid, account_uuid, branch, rebase
//   PushRepositoryCommand    → repository_uuid, account_uuid, commit_message, branch, force
//   TestSshConnectionCommand → account_uuid, timeout_ms: Option<u32>

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use gm_shared::{
    errors::GitManagerError,
    models::{account::AccountDto, repository::RepositoryDto, ssh_key::SshKeyDto},
};

use gm_ports::inbound::commands::{
    AddAccountCommand, CloneRepositoryCommand, GenerateSshKeyCommand,
    PullRepositoryCommand, PushRepositoryCommand, TestSshConnectionCommand,
};

// ─────────────────────────────────────────────────────────────────────────────
// Return types for multi-field operation outcomes
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct GitOpResult {
    pub commits_transferred: u32,
    pub current_sha:         Option<String>,
    pub had_conflicts:       bool,
}

#[derive(Debug)]
pub struct SshTestResult {
    pub success:  bool,
    pub username: Option<String>,
    pub error:    Option<String>,
}

#[derive(Debug, Clone)]
pub struct SshValidationItemDto {
    pub name:   String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct SshValidationResultDto {
    pub all_passed: bool,
    pub checks:     Vec<SshValidationItemDto>,
}

#[derive(Debug)]
pub struct GitStatusEntry {
    pub path:   String,
    /// Two-character porcelain code: "M ", " M", "A ", "D ", "??"
    pub status: String,
}

#[derive(Debug)]
pub struct OperationLogEntry {
    pub uuid:        Uuid,
    pub op_type:     String,
    pub status:      String,
    pub account:     Option<String>,
    pub repository:  Option<String>,
    pub started_at:  chrono::DateTime<chrono::Utc>,
    pub duration_ms: Option<u32>,
    pub error:       Option<String>,
}

#[derive(Debug, Clone)]
pub struct DryRunPreview {
    /// Summary description of what would happen.
    pub summary:          String,
    /// Number of commits that would be transferred.
    pub commits:          u32,
    /// Whether conflicts are predicted (dry-run analysis).
    pub conflicts_predicted: bool,
    /// Detailed predictions.
    pub details:          Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RepoDetectionResult {
    pub tracked:          bool,
    pub account_alias:    Option<String>,
    pub platform_slug:    Option<String>,
    pub platform_name:    Option<String>,
    pub remote_url:       Option<String>,
    pub local_path:       Option<String>,
    pub current_branch:   Option<String>,
    pub repository_name:  Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// The facade trait
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait CliServices: Send + Sync {
    // Account operations
    async fn list_accounts(&self, platform_id: Option<Uuid>) -> Result<Vec<AccountDto>, GitManagerError>;
    async fn get_account(&self, uuid: Uuid) -> Result<Option<AccountDto>, GitManagerError>;
    async fn get_account_by_alias(&self, alias: &str, platform_id: Option<Uuid>) -> Result<Option<AccountDto>, GitManagerError>;
    async fn add_account(&self, cmd: AddAccountCommand) -> Result<AccountDto, GitManagerError>;
    async fn remove_account(&self, uuid: Uuid) -> Result<(), GitManagerError>;
    async fn set_default_account(&self, account_uuid: Uuid, platform_id: Uuid) -> Result<(), GitManagerError>;
    async fn store_account_token(&self, account_uuid: Uuid, token: &str) -> Result<(), GitManagerError>;

    // SSH key operations
    async fn list_ssh_keys(&self, account_uuid: Uuid) -> Result<Vec<SshKeyDto>, GitManagerError>;
    async fn generate_ssh_key(&self, cmd: GenerateSshKeyCommand) -> Result<SshKeyDto, GitManagerError>;
    /// timeout_ms field in TestSshConnectionCommand is Option<u32>.
    async fn test_ssh_connection(&self, cmd: TestSshConnectionCommand) -> Result<SshTestResult, GitManagerError>;
    async fn validate_ssh_setup(&self, account_uuid: Uuid, timeout_ms: u32) -> Result<SshValidationResultDto, GitManagerError>;
    async fn add_key_to_agent(&self, key_uuid: Uuid, passphrase: Option<&str>) -> Result<(), GitManagerError>;

    // Repository operations
    async fn list_repositories(&self, account_uuid: Option<Uuid>) -> Result<Vec<RepositoryDto>, GitManagerError>;
    /// CloneRepositoryCommand fields: url, account_uuid, destination, branch, depth,
    /// filter, bare, mirror, sparse_checkout, single_branch, no_checkout,
    /// recurse_submodules, tags_mode, upload_pack
    /// account_uuid = None → anonymous HTTPS clone (public repos only)
    async fn clone_repository(&self, cmd: CloneRepositoryCommand) -> Result<RepositoryDto, GitManagerError>;

    // Git operations
    /// PullRepositoryCommand fields: repository_uuid, account_uuid, branch (Option<String>), rebase (bool), dry_run (bool)
    async fn git_pull(&self, cmd: PullRepositoryCommand) -> Result<GitOpResult, GitManagerError>;
    /// PushRepositoryCommand fields: repository_uuid, account_uuid, commit_message, branch, force (bool), dry_run (bool)
    async fn git_push(&self, cmd: PushRepositoryCommand) -> Result<GitOpResult, GitManagerError>;
    async fn git_status(&self, repo_path: &std::path::Path) -> Result<Vec<GitStatusEntry>, GitManagerError>;
    /// Dry-run preview: what would happen on a pull
    async fn dry_run_pull(&self, repository_uuid: Uuid, account_uuid: Uuid) -> Result<DryRunPreview, GitManagerError>;
    /// Dry-run preview: what would happen on a push
    async fn dry_run_push(&self, repository_uuid: Uuid, account_uuid: Uuid) -> Result<DryRunPreview, GitManagerError>;
    /// Detect which account/repo is associated with the given directory
    async fn detect_repository(&self, path: &std::path::Path) -> Result<RepoDetectionResult, GitManagerError>;
    /// List repositories from the platform API (paginated)
    async fn list_remote_repositories(&self, account_uuid: Uuid, page: u32, per_page: u32) -> Result<Vec<String>, GitManagerError>;

    // Configuration
    async fn config_list(&self) -> Result<Vec<(String, String)>, GitManagerError>;
    async fn config_get(&self, key: &str) -> Result<Option<String>, GitManagerError>;
    async fn config_set(&self, key: &str, value: &str) -> Result<(), GitManagerError>;

    // Operation logs
    async fn list_recent_operations(&self, account_uuid: Option<Uuid>, limit: u32) -> Result<Vec<OperationLogEntry>, GitManagerError>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Registration newtype
// ─────────────────────────────────────────────────────────────────────────────

/// Newtype so Arc<dyn CliServices> can be stored in the TypeId service registry.
/// CLI command handlers retrieve this by type: kernel.get::<CliServicesHandle>()
pub struct CliServicesHandle(pub Arc<dyn CliServices>);

impl CliServicesHandle {
    pub fn new(inner: impl CliServices + 'static) -> Self { Self(Arc::new(inner)) }
    pub fn services(&self) -> &dyn CliServices { self.0.as_ref() }
}