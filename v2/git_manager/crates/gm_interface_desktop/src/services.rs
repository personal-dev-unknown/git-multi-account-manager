// crates/gm_interface_desktop/src/services.rs
//
// `DesktopServices` is the application-level service facade for all Tauri
// command handlers. It mirrors the pattern established by `CliServices` and
// `WebServices` — a single trait object that isolates Tauri command handlers
// from every infrastructure layer.

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use gm_shared::{
    errors::GitManagerError,
    models::{account::AccountDto, repository::RepositoryDto, ssh_key::SshKeyDto},
};
use gm_ports::inbound::commands::{
    AddAccountCommand, CloneRepositoryCommand, GenerateSshKeyCommand,
    PullRepositoryCommand, PushRepositoryCommand, TestSshConnectionCommand,
};

// ── Return types specific to desktop commands ─────────────────────────────────

/// Result of a git pull or push operation, serializable for the Svelte frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopGitOpResult {
    pub commits_transferred: u32,
    pub current_sha:         Option<String>,
    pub had_conflicts:       bool,
}

/// Result of an SSH connection test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopSshTestResult {
    pub success:  bool,
    pub username: Option<String>,
    pub error:    Option<String>,
}

/// Single status entry from `git status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopGitStatusEntry {
    pub path:   String,
    /// Two-character porcelain code: "M ", " M", "A ", "D ", "??"
    pub status: String,
}

// ── The trait ─────────────────────────────────────────────────────────────────

/// The application-level service facade for all Tauri command handlers.
///
/// `Send + Sync + 'static` are required because Tauri stores this in managed
/// state and calls it from multiple async tasks concurrently.
#[async_trait]
pub trait DesktopServices: Send + Sync + 'static {
    // ── Account operations ─────────────────────────────────────────────────
    async fn list_accounts(&self, platform_id: Option<Uuid>)  -> Result<Vec<AccountDto>, GitManagerError>;
    async fn get_account(&self, uuid: Uuid)                    -> Result<Option<AccountDto>, GitManagerError>;
    async fn add_account(&self, cmd: AddAccountCommand)        -> Result<AccountDto, GitManagerError>;
    async fn remove_account(&self, uuid: Uuid)                 -> Result<(), GitManagerError>;
    async fn set_default_account(&self, account_uuid: Uuid, platform_id: Uuid) -> Result<(), GitManagerError>;
    async fn store_account_token(&self, account_uuid: Uuid, token: String) -> Result<(), GitManagerError>;

    // ── SSH key operations ─────────────────────────────────────────────────
    async fn list_ssh_keys(&self, account_uuid: Uuid)          -> Result<Vec<SshKeyDto>, GitManagerError>;
    async fn generate_ssh_key(&self, cmd: GenerateSshKeyCommand) -> Result<SshKeyDto, GitManagerError>;
    async fn test_ssh_connection(&self, cmd: TestSshConnectionCommand) -> Result<DesktopSshTestResult, GitManagerError>;
    async fn add_key_to_agent(&self, account_uuid: Uuid)       -> Result<(), GitManagerError>;

    // ── Repository operations ──────────────────────────────────────────────
    async fn list_repositories(&self, account_uuid: Option<Uuid>) -> Result<Vec<RepositoryDto>, GitManagerError>;
    async fn clone_repository(&self, cmd: CloneRepositoryCommand)  -> Result<RepositoryDto, GitManagerError>;
    async fn list_remote_repositories(&self, account_uuid: Uuid, page: u32, per_page: u32) -> Result<Vec<String>, GitManagerError>;

    // ── Git operations ─────────────────────────────────────────────────────
    async fn git_pull(&self, cmd: PullRepositoryCommand)  -> Result<DesktopGitOpResult, GitManagerError>;
    async fn git_push(&self, cmd: PushRepositoryCommand)  -> Result<DesktopGitOpResult, GitManagerError>;
    async fn git_status(&self, repository_uuid: Uuid)     -> Result<Vec<DesktopGitStatusEntry>, GitManagerError>;
}

// ── AppState newtype ──────────────────────────────────────────────────────────

/// Newtype wrapping `Arc<dyn DesktopServices>` for Tauri managed state.
pub struct AppState(pub Arc<dyn DesktopServices>);

impl AppState {
    pub fn new(services: impl DesktopServices) -> Self {
        Self(Arc::new(services))
    }

    pub fn services(&self) -> &dyn DesktopServices {
        self.0.as_ref()
    }
}
