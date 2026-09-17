// apps/desktop/src/main.rs
//
// Git Manager Desktop binary (Tauri).
//
// ── Wiring strategy ───────────────────────────────────────────────────────────
// The binary's job is identical to apps/cli and apps/web: resolve ALL generic
// type parameters to concrete types, create every adapter, create domain
// services, and boot the kernel. The only difference:
//   - After bootstrap, registers `AppState` (not CliServicesHandle or
//     WebServicesHandle) in Tauri managed state.
//   - Calls `gm_interface_desktop::run_tauri_app(app_state)` instead of
//     running a CLI read-eval-print loop or Axum server.
//
// ── Main thread and Tokio ─────────────────────────────────────────────────────
// Tauri requires the main thread. We cannot annotate main with #[tokio::main]
// because Tauri's `.run()` also needs the main thread. Instead:
//   1. Create a multi-thread Tokio runtime manually.
//   2. Use `runtime.block_on(init_services())` to perform the async initialization.
//   3. Drop the runtime — the initialized state (AppState) is Move-captured.
//   4. Call `run_tauri_app(app_state)` which blocks on the main thread under Tauri.
//
// Tauri creates its own internal Tokio runtime for async command handlers, so
// async commands keep working after step 3.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_ports::outbound::repository_provider::{PageCursor, RepositoryProvider};

// ── Adapters ───────────────────────────────────────────────────────────────────
use gm_adapters::persistence::mysql::{
    MySqlAccountRepository, MySqlRepositoryRepository,
    MySqlSshHostConfigRepository, MySqlSshKeyRepository, create_mysql_pool, SqlxEventStore,
};
use gm_adapters::platform::zig_credential_store::ZigCredentialStore;
use gm_adapters::ssh::zig_ssh_provider::ZigSshProvider;
use gm_adapters::git::zig_git_executor::ZigGitExecutor;
use gm_adapters::git::ZigDiskSpaceChecker;
use gm_adapters::git::ffi;

// ── Domain ─────────────────────────────────────────────────────────────────────
use gm_domain::git::ports::git_executor::GitExecutor;
use gm_domain::accounts::services::account_service::AccountService;
use gm_domain::accounts::value_objects::AuthMethod as DomainAuthMethod;
use gm_domain::repositories::entities::Repository;
use gm_domain::repositories::ports::RepositoryRepository;
use gm_domain::repositories::value_objects::RepositoryUrl;
use gm_domain::repositories::entities::repository::Visibility;
use gm_domain::ssh::services::ssh_service::SshService;
use gm_domain::ssh::value_objects::KeyType;

// ── Kernel ─────────────────────────────────────────────────────────────────────
use gm_kernel::{
    bootstrap::{AppConfig, bootstrap},
    contracts::plugin::{EventSubscription, Plugin, PluginMetadata},
    security::{AuthStrategy, CloneFallbackEngine, CloneUrlResolver, CredentialService, CredentialVault, ResolvedStrategy, register_child_killer},
    service_registry::ServiceRegistry,
};
use gm_ports::outbound::CredentialStore;
use gm_ports::inbound::commands::{
    AddAccountCommand, CloneRepositoryCommand, GenerateSshKeyCommand,
    PullRepositoryCommand, PushRepositoryCommand, TestSshConnectionCommand,
};
use gm_domain::accounts::ports::AccountRepository;
use gm_domain::ssh::ports::{SshKeyRepository, SshHostConfigRepository};
use gm_domain::ssh::services::ssh_service::SshOperations;
use gm_kernel::event_bus::EventStore;

use gm_adapters::persistence::sqlite::{
    SqliteAccountRepository, SqliteSshKeyRepository, SqliteSshHostConfigRepository,
    SqliteRepositoryRepository, SqliteEventStore, create_sqlite_pool, setup_sqlite_schema,
};

use gm_domain::git::ports::git_executor::{
    CloneOptions, CommitOptions, PullOptions, PushOptions,
};

// ── Desktop interface ──────────────────────────────────────────────────────────
use gm_interface_desktop::{
    AppState, DesktopServices, DesktopGitOpResult, DesktopSshTestResult, DesktopGitStatusEntry,
    run_tauri_app, DesktopPlugin,
};
use gm_shared::{
    errors::{GitManagerError, PluginError},
    models::{account::AccountDto, repository::RepositoryDto, ssh_key::SshKeyDto},
};

// ═══════════════════════════════════════════════════════════════════════════════
// InfrastructurePlugin (identical across all three binaries)
// ═══════════════════════════════════════════════════════════════════════════════

struct InfrastructurePlugin {
    metadata:     PluginMetadata,
    cred_service: Arc<CredentialService>,
}

impl InfrastructurePlugin {
    fn new(cred_service: Arc<CredentialService>) -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_infrastructure".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "Registers kernel infrastructure services before plugins load".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      0,
            },
            cred_service,
        }
    }
}

impl Plugin for InfrastructurePlugin {
    fn metadata(&self) -> &PluginMetadata { &self.metadata }

    fn on_load(&self, registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        registry.register::<CredentialService>(Arc::clone(&self.cred_service));
        tracing::info!("InfrastructurePlugin: CredentialService registered");
        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> { Ok(()) }
    fn get_event_subscriptions(&self) -> Vec<EventSubscription> { vec![] }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Platform info helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn platform_info(platform_id: Uuid) -> (&'static str, &'static str, u16) {
    match platform_id.to_string().as_str() {
        "00000000-0001-0000-0000-000000000001" => ("github",       "github.com",        22),
        "00000000-0002-0000-0000-000000000001" => ("gitlab",       "gitlab.com",        22),
        "00000000-0003-0000-0000-000000000001" => ("bitbucket",    "bitbucket.org",     22),
        "00000000-0004-0000-0000-000000000001" => ("azure_devops", "ssh.dev.azure.com", 22),
        "00000000-0005-0000-0000-000000000001" => ("sourceforge",  "git.code.sf.net",   22),
        "00000000-0006-0000-0000-000000000001" => ("self_hosted",  "localhost",         22),
        "00000000-0007-0000-0000-000000000001" => ("cloud_storage","",                  22),
        "00000000-0008-0000-0000-000000000001" => ("local_path",   "",                  22),
        _                                       => ("custom",       "github.com",        22),
    }
}

fn platform_display_name(platform_id: Uuid) -> Option<String> {
    match platform_id.to_string().as_str() {
        "00000000-0001-0000-0000-000000000001" => Some("GitHub".to_string()),
        "00000000-0002-0000-0000-000000000001" => Some("GitLab".to_string()),
        "00000000-0003-0000-0000-000000000001" => Some("Bitbucket".to_string()),
        "00000000-0004-0000-0000-000000000001" => Some("Azure DevOps".to_string()),
        "00000000-0005-0000-0000-000000000001" => Some("SourceForge".to_string()),
        "00000000-0006-0000-0000-000000000001" => Some("Self-Hosted".to_string()),
        "00000000-0007-0000-0000-000000000001" => Some("Cloud Storage".to_string()),
        "00000000-0008-0000-0000-000000000001" => Some("Local Path".to_string()),
        _                                       => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConcreteDesktopServices — full production implementation of DesktopServices
// ═══════════════════════════════════════════════════════════════════════════════

#[allow(dead_code)]
struct ConcreteDesktopServices {
    account_svc:     Arc<AccountService>,
    ssh_service:     Arc<SshService>,
    repo_repo:       Arc<dyn RepositoryRepository>,
    git_executor:    Arc<ZigGitExecutor>,
    cred_service:    Arc<CredentialService>,
    url_resolver:    Arc<CloneUrlResolver>,
    fallback_engine: CloneFallbackEngine,
    providers:       HashMap<Uuid, Arc<dyn RepositoryProvider>>,
}

impl ConcreteDesktopServices {
    fn enrich(mut accounts: Vec<AccountDto>) -> Vec<AccountDto> {
        for a in &mut accounts {
            if a.platform_name.is_none() {
                a.platform_name = platform_display_name(a.platform_id);
            }
        }
        accounts
    }
}

#[async_trait]
impl DesktopServices for ConcreteDesktopServices {
    async fn list_accounts(&self, platform_id: Option<Uuid>) -> Result<Vec<AccountDto>, GitManagerError> {
        let accounts = if let Some(pid) = platform_id {
            self.account_svc.list_accounts_by_platform(pid).await.map_err(GitManagerError::Account)?
        } else {
            self.account_svc.list_accounts().await.map_err(GitManagerError::Account)?
        };
        Ok(Self::enrich(accounts))
    }

    async fn get_account(&self, uuid: Uuid) -> Result<Option<AccountDto>, GitManagerError> {
        match self.account_svc.get_account(uuid).await {
            Ok(a) => {
                let mut dto = a.to_dto();
                dto.platform_name = platform_display_name(dto.platform_id);
                Ok(Some(dto))
            }
            Err(gm_shared::errors::AccountError::NotFound { .. }) => Ok(None),
            Err(e) => Err(GitManagerError::Account(e)),
        }
    }

    async fn add_account(&self, cmd: AddAccountCommand) -> Result<AccountDto, GitManagerError> {
        let auth = DomainAuthMethod::from_str(&cmd.auth_method)
            .ok_or_else(|| GitManagerError::Other(format!("unknown auth_method '{}'", cmd.auth_method)))?;
        let result = self.account_svc.add_account(cmd.alias, cmd.platform_id, cmd.username, cmd.email, auth)
            .await.map_err(GitManagerError::Account)?;
        let mut dto = result.account.to_dto();
        dto.platform_name = platform_display_name(dto.platform_id);
        Ok(dto)
    }

    async fn remove_account(&self, uuid: Uuid) -> Result<(), GitManagerError> {
        self.account_svc.remove_account(uuid).await.map(|_| ()).map_err(GitManagerError::Account)
    }

    async fn set_default_account(&self, account_uuid: Uuid, platform_id: Uuid) -> Result<(), GitManagerError> {
        self.account_svc.set_default_account(account_uuid, platform_id).await.map(|_| ()).map_err(GitManagerError::Account)
    }

    async fn store_account_token(&self, account_uuid: Uuid, token: String) -> Result<(), GitManagerError> {
        self.cred_service.store_token(account_uuid, &token).await
    }

    async fn list_ssh_keys(&self, account_uuid: Uuid) -> Result<Vec<SshKeyDto>, GitManagerError> {
        self.ssh_service.list_keys_for_account(account_uuid).await
            .map(|keys| keys.into_iter().map(|k| k.to_dto()).collect())
            .map_err(GitManagerError::Ssh)
    }

    async fn generate_ssh_key(&self, cmd: GenerateSshKeyCommand) -> Result<SshKeyDto, GitManagerError> {
        let account              = self.account_svc.get_account(cmd.account_uuid).await.map_err(GitManagerError::Account)?;
        let (slug, hostname, port) = platform_info(account.platform_id());
        let key_type             = KeyType::from_str(&cmd.key_type).unwrap_or(KeyType::Ed25519);
        let ssh_dir              = dirs_next::home_dir()
            .ok_or_else(|| GitManagerError::Other("cannot determine home directory".to_string()))?
            .join(".ssh").join("gitzyrix");
        std::fs::create_dir_all(&ssh_dir).map_err(|e| GitManagerError::Other(format!("cannot create ~/.ssh/gitzyrix: {e}")))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&ssh_dir, std::fs::Permissions::from_mode(0o700));
        }
        let result = self.ssh_service.generate_and_setup_key(
            account.uuid(), account.alias(), slug, account.email(), hostname,
            key_type, ssh_dir, port, cmd.add_to_agent, cmd.passphrase.as_deref(), 10_000,
        ).await.map_err(GitManagerError::Ssh)?;
        Ok(result.ssh_key.to_dto())
    }

    async fn test_ssh_connection(&self, cmd: TestSshConnectionCommand) -> Result<DesktopSshTestResult, GitManagerError> {
        let timeout = cmd.timeout_ms.unwrap_or(10_000);
        let result  = self.ssh_service.test_connection(cmd.account_uuid, timeout).await.map_err(GitManagerError::Ssh)?;
        let success = result.event.success;
        if success {
            let _ = self.account_svc.activate_account(cmd.account_uuid).await;
        }
        Ok(DesktopSshTestResult { success, username: result.event.username, error: result.event.error })
    }

    async fn add_key_to_agent(&self, account_uuid: Uuid) -> Result<(), GitManagerError> {
        self.ssh_service.add_to_agent(account_uuid, None).await
            .map(|_| ())
            .map_err(GitManagerError::Ssh)
    }

    async fn list_repositories(&self, account_uuid: Option<Uuid>) -> Result<Vec<RepositoryDto>, GitManagerError> {
        let repos = if let Some(uuid) = account_uuid {
            self.repo_repo.list_by_account(uuid).await?
        } else {
            self.repo_repo.list_cloned().await?
        };
        Ok(repos.into_iter().map(|r| r.to_dto()).collect())
    }

    async fn list_remote_repositories(&self, account_uuid: Uuid, page: u32, per_page: u32) -> Result<Vec<String>, GitManagerError> {
        let account = self.account_svc.get_account(account_uuid).await
            .map_err(GitManagerError::Account)?;
        let provider = self.providers.get(&account.platform_id())
            .ok_or_else(|| GitManagerError::Other(
                "No repository provider registered for this platform".to_string()
            ))?;
        let cursor = Some(PageCursor { page, per_page, has_next: false });
        let (repos, _) = provider.list_repositories(account_uuid, cursor).await?;
        Ok(repos.into_iter().map(|r| r.full_name).collect())
    }

    async fn clone_repository(&self, cmd: CloneRepositoryCommand) -> Result<RepositoryDto, GitManagerError> {
        let dest_path = match &cmd.destination {
            Some(d) => std::path::PathBuf::from(d),
            None => {
                let name = cmd.url.trim_end_matches(".git").split(['/', ':']).next_back().unwrap_or("repo");
                dirs_next::home_dir().unwrap_or_default().join("git-repos").join(name)
            }
        };

        if let Some(account_uuid) = cmd.account_uuid {
            let account    = self.account_svc.get_account(account_uuid).await.map_err(GitManagerError::Account)?;
            let active_key = self.ssh_service.get_active_key(account_uuid).await.map_err(GitManagerError::Ssh)?
                .ok_or_else(|| GitManagerError::Other("No active SSH key — generate one first".to_string()))?;
            let clone_opts = CloneOptions {
                url:             cmd.url.clone(),
                destination:     dest_path.clone(),
                ssh_key_path:    Some(std::path::PathBuf::from(active_key.private_key_path())),
                ssh_host_alias:  account.ssh_host_alias().map(|s| s.to_string()),
                branch:          cmd.branch.clone(),
                depth:           Some(cmd.depth).filter(|d| *d > 0),
                filter:          cmd.filter.clone(),
                sparse_checkout: cmd.sparse_checkout.as_ref().map(|s| s.split(',').map(|p| p.trim().to_string()).collect()),
                single_branch:   cmd.single_branch,
                no_checkout:     cmd.no_checkout,
                recurse_submodules: cmd.recurse_submodules,
                tags_mode:       cmd.to_tags_mode(),
                upload_pack:     cmd.upload_pack.clone(),
                ..Default::default()
            };
            let resolved = ResolvedStrategy {
                primary: AuthStrategy::Ssh,
                fallbacks: vec![AuthStrategy::Anonymous],
            };
            let attempt = self.fallback_engine.clone_with_fallback(&clone_opts, &resolved, Some(account_uuid)).await?;
            let clone_result = attempt.result.ok_or_else(|| GitManagerError::Other("clone returned no result".to_string()))?;
            let full_name    = cmd.url.trim_end_matches(".git").split([':','/']).rev().take(2)
                .collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("/");
            let name         = full_name.split('/').next_back().unwrap_or(&full_name).to_string();
            let url_vo       = RepositoryUrl::new(cmd.url.clone()).map_err(|e| GitManagerError::Other(e.to_string()))?;
            let mut repo     = Repository::new_discovered(account_uuid, account.platform_id(), name, full_name, url_vo, clone_result.branch.clone(), Visibility::Private);
            repo.mark_cloned(clone_result.local_path.to_string_lossy().to_string(), clone_result.commit_sha.clone());
            self.repo_repo.save(&repo).await?;
            Ok(repo.to_dto())
        } else {
            let clone_opts = CloneOptions {
                url:             cmd.url.clone(),
                destination:     dest_path.clone(),
                ssh_key_path:    None,
                ssh_host_alias:  None,
                branch:          cmd.branch.clone(),
                depth:           Some(cmd.depth).filter(|d| *d > 0),
                filter:          cmd.filter.clone(),
                sparse_checkout: cmd.sparse_checkout.as_ref().map(|s| s.split(',').map(|p| p.trim().to_string()).collect()),
                single_branch:   cmd.single_branch,
                no_checkout:     cmd.no_checkout,
                recurse_submodules: cmd.recurse_submodules,
                tags_mode:       cmd.to_tags_mode(),
                upload_pack:     cmd.upload_pack.clone(),
                ..Default::default()
            };
            let resolved = ResolvedStrategy {
                primary: AuthStrategy::Anonymous,
                fallbacks: vec![],
            };
            let attempt = self.fallback_engine.clone_with_fallback(&clone_opts, &resolved, None).await?;
            let clone_result = attempt.result.ok_or_else(|| GitManagerError::Other("clone returned no result".to_string()))?;
            let full_name    = cmd.url.trim_end_matches(".git").split([':','/']).rev().take(2)
                .collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("/");
            let name         = full_name.split('/').next_back().unwrap_or(&full_name).to_string();
            let url_vo       = RepositoryUrl::new(cmd.url.clone()).map_err(|e| GitManagerError::Other(e.to_string()))?;
            let mut repo     = Repository::new_discovered(uuid::Uuid::nil(), uuid::Uuid::nil(), name, full_name, url_vo, clone_result.branch.clone(), Visibility::Public);
            repo.mark_cloned(clone_result.local_path.to_string_lossy().to_string(), clone_result.commit_sha.clone());
            self.repo_repo.save(&repo).await?;
            Ok(repo.to_dto())
        }
    }

    async fn git_pull(&self, cmd: PullRepositoryCommand) -> Result<DesktopGitOpResult, GitManagerError> {
        let repo       = self.repo_repo.find_by_id(cmd.repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("Repository {} not found", cmd.repository_uuid)))?;
        let local_path = repo.local_path().ok_or_else(|| GitManagerError::Other("Not cloned locally".to_string()))?;
        let active_key = self.ssh_service.get_active_key(cmd.account_uuid).await.map_err(GitManagerError::Ssh)?
            .ok_or_else(|| GitManagerError::Other("No active SSH key".to_string()))?;
        let result     = self.git_executor.pull(PullOptions {
            repo_path:    std::path::PathBuf::from(local_path),
            ssh_key_path: std::path::PathBuf::from(active_key.private_key_path()),
            rebase:       cmd.rebase,
            branch:       cmd.branch.clone(),
        }).await.map_err(GitManagerError::Git)?;
        Ok(DesktopGitOpResult { commits_transferred: result.commits_pulled, current_sha: Some(result.commit_sha), had_conflicts: false })
    }

    async fn git_push(&self, cmd: PushRepositoryCommand) -> Result<DesktopGitOpResult, GitManagerError> {
        let repo       = self.repo_repo.find_by_id(cmd.repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("Repository {} not found", cmd.repository_uuid)))?;
        let local_path = repo.local_path().ok_or_else(|| GitManagerError::Other("Not cloned locally".to_string()))?;
        let account    = self.account_svc.get_account(cmd.account_uuid).await.map_err(GitManagerError::Account)?;
        let active_key = self.ssh_service.get_active_key(cmd.account_uuid).await.map_err(GitManagerError::Ssh)?
            .ok_or_else(|| GitManagerError::Other("No active SSH key".to_string()))?;
        self.git_executor.stage_all(std::path::Path::new(local_path)).await.map_err(GitManagerError::Git)?;
        let commit = self.git_executor.commit(CommitOptions {
            repo_path:    std::path::PathBuf::from(local_path),
            message:      cmd.commit_message.clone(),
            author_name:  account.display_name().unwrap_or(account.username()).to_string(),
            author_email: account.email().to_string(),
            amend:        false,
        }).await.map_err(GitManagerError::Git)?;
        let push = self.git_executor.push(PushOptions {
            repo_path:    std::path::PathBuf::from(local_path),
            ssh_key_path: std::path::PathBuf::from(active_key.private_key_path()),
            remote:       "origin".to_string(),
            branch:       cmd.branch.clone(),
            force:        cmd.force,
        }).await.map_err(GitManagerError::Git)?;
        Ok(DesktopGitOpResult { commits_transferred: push.commits_pushed, current_sha: Some(commit.sha), had_conflicts: false })
    }

    async fn git_status(&self, repository_uuid: Uuid) -> Result<Vec<DesktopGitStatusEntry>, GitManagerError> {
        let repo  = self.repo_repo.find_by_id(repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("Repository {} not found", repository_uuid)))?;
        let local = repo.local_path().ok_or_else(|| GitManagerError::Other("Not cloned locally".to_string()))?;
        let status = self.git_executor.status(std::path::Path::new(local)).await.map_err(GitManagerError::Git)?;
        Ok(status.entries.into_iter().map(|e| DesktopGitStatusEntry {
            path:   e.path,
            status: format!("{}{}", e.index_status, e.working_status),
        }).collect())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Async service initialization
// ═══════════════════════════════════════════════════════════════════════════════

async fn init_services() -> Result<AppState, Box<dyn std::error::Error>> {
    let config = AppConfig::from_env();

    let use_sqlite = config.database_url.is_empty();

    let (account_repo, ssh_key_repo, ssh_host_cfg_repo, repo_repo, event_store): (
        Arc<dyn AccountRepository>,
        Arc<dyn SshKeyRepository>,
        Arc<dyn SshHostConfigRepository>,
        Arc<dyn RepositoryRepository>,
        Arc<dyn EventStore>,
    ) = if use_sqlite {
        let sqlite_path = config.sqlite_path.as_deref().unwrap_or(":memory:");
        let pool = create_sqlite_pool(sqlite_path).await?;
        setup_sqlite_schema(&pool).await?;
        bootstrap_anonymous_account_sqlite(&pool).await?;

        let account_repo      = Arc::new(SqliteAccountRepository::new(pool.clone())) as Arc<dyn AccountRepository>;
        let ssh_key_repo      = Arc::new(SqliteSshKeyRepository::new(pool.clone())) as Arc<dyn SshKeyRepository>;
        let ssh_host_cfg_repo = Arc::new(SqliteSshHostConfigRepository::new(pool.clone())) as Arc<dyn SshHostConfigRepository>;
        let repo_repo         = Arc::new(SqliteRepositoryRepository::new(pool.clone())) as Arc<dyn RepositoryRepository>;
        let event_store       = Arc::new(SqliteEventStore::new(pool)) as Arc<dyn EventStore>;

        (account_repo, ssh_key_repo, ssh_host_cfg_repo, repo_repo, event_store)
    } else {
        let pool = create_mysql_pool(&config.database_url).await?;
        sqlx::migrate!("../../crates/gm_adapters/src/persistence/migrations").run(&pool).await?;
        bootstrap_anonymous_account(&pool).await?;

        let account_repo      = Arc::new(MySqlAccountRepository::new(pool.clone())) as Arc<dyn AccountRepository>;
        let ssh_key_repo      = Arc::new(MySqlSshKeyRepository::new(pool.clone())) as Arc<dyn SshKeyRepository>;
        let ssh_host_cfg_repo = Arc::new(MySqlSshHostConfigRepository::new(pool.clone())) as Arc<dyn SshHostConfigRepository>;
        let repo_repo         = Arc::new(MySqlRepositoryRepository::new(pool.clone())) as Arc<dyn RepositoryRepository>;
        let event_store       = Arc::new(SqlxEventStore::new(pool)) as Arc<dyn EventStore>;

        (account_repo, ssh_key_repo, ssh_host_cfg_repo, repo_repo, event_store)
    };

    let cred_store    = Arc::new(ZigCredentialStore::new());
    let ssh_provider  = Arc::new(ZigSshProvider::new()) as Arc<dyn SshOperations>;
    let git_executor  = Arc::new(ZigGitExecutor::new());

    let vault        = Arc::new(CredentialVault::new()?);
    let cred_service = Arc::new(CredentialService::new(
        Arc::clone(&cred_store) as Arc<dyn CredentialStore>,
        Arc::clone(&vault),
    ));

    let account_svc = Arc::new(AccountService::new(Arc::clone(&account_repo)));
    let ssh_service = Arc::new(SshService::new(
        Arc::clone(&ssh_key_repo),
        Arc::clone(&ssh_host_cfg_repo),
        Arc::clone(&ssh_provider),
    ));

    let url_resolver = Arc::new(CloneUrlResolver::new(Arc::clone(&cred_service)));
    register_child_killer(|| unsafe { ffi::gm_git_kill_process(0) });
    use gm_adapters::git::progress_reporter::tracing_progress;
    let disk_checker = Arc::new(ZigDiskSpaceChecker);
    let fallback_engine = CloneFallbackEngine::new(
        Arc::clone(&git_executor) as Arc<dyn GitExecutor>,
        Arc::clone(&url_resolver),
    )
    .with_progress(Arc::new(tracing_progress()))
    .with_disk_checker(disk_checker);

    let plugins: Vec<Box<dyn gm_kernel::contracts::plugin::Plugin>> = vec![
        Box::new(InfrastructurePlugin::new(Arc::clone(&cred_service))),
        Box::new(gm_plugin_github::create_plugin()),
        Box::new(gm_plugin_gitlab::create_plugin()),
        Box::new(gm_plugin_bitbucket::create_plugin()),
        Box::new(gm_plugin_azure_devops::create_plugin()),
        Box::new(gm_plugin_sourceforge::create_plugin()),
        Box::new(gm_plugin_custom::create_plugin()),
        Box::new(gm_plugin_cloud_storage::create_plugin()),
        Box::new(gm_plugin_local_path::create_plugin()),
        Box::new(DesktopPlugin::new()),
    ];
    let kernel = bootstrap(config, plugins, event_store).await?;
    tracing::info!("Kernel bootstrapped — starting Tauri desktop interface");

    let mut providers: HashMap<Uuid, Arc<dyn RepositoryProvider>> = HashMap::new();
    macro_rules! insert_provider {
        ($uuid:literal, $ty:ty) => {
            if let Some(p) = kernel.get::<$ty>() {
                providers.insert(Uuid::parse_str($uuid).unwrap(), p as Arc<dyn RepositoryProvider>);
            }
        };
    }
    insert_provider!("00000000-0001-0000-0000-000000000001", gm_plugin_github::GitHubRepositoryProvider);
    insert_provider!("00000000-0002-0000-0000-000000000001", gm_plugin_gitlab::GitLabRepositoryProvider);
    insert_provider!("00000000-0003-0000-0000-000000000001", gm_plugin_bitbucket::BitbucketRepositoryProvider);
    insert_provider!("00000000-0004-0000-0000-000000000001", gm_plugin_azure_devops::AzureDevOpsRepositoryProvider);
    insert_provider!("00000000-0005-0000-0000-000000000001", gm_plugin_sourceforge::SourceForgeRepositoryProvider);

    let desktop_services = ConcreteDesktopServices {
        account_svc,
        ssh_service,
        repo_repo,
        git_executor,
        cred_service,
        url_resolver,
        fallback_engine,
        providers,
    };

    Ok(AppState::new(desktop_services))
}

async fn bootstrap_anonymous_account(pool: &sqlx::MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    let anon_uuid = uuid::Uuid::nil();
    let exists: bool = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM accounts WHERE uuid = ?"
    )
    .bind(anon_uuid.to_string())
    .fetch_one(pool)
    .await
    .map(|c| c > 0)
    .unwrap_or(false);

    if !exists {
        sqlx::query(r#"
            INSERT INTO accounts
                (uuid,                            platform_id,                   alias,       username,   email,
                 display_name,    auth_method,   ssh_host_alias,   status,   is_default,   last_used_at)
            VALUES
                (?,
                 (SELECT id FROM platforms WHERE uuid = '00000000-0008-0000-0000-000000000001'),
                 'anonymous',     'anonymous',   'anonymous@local',
                 'Anonymous User','anonymous',   NULL,              'active', 0,             NULL)
        "#)
        .bind(anon_uuid.to_string())
        .execute(pool)
        .await?;
        tracing::info!("bootstrapped anonymous system account");
    }
    Ok(())
}

async fn bootstrap_anonymous_account_sqlite(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let anon_uuid = uuid::Uuid::nil();
    let exists: bool = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM accounts WHERE uuid = ?"
    )
    .bind(anon_uuid.to_string())
    .fetch_one(pool)
    .await
    .map(|c| c > 0)
    .unwrap_or(false);

    if !exists {
        sqlx::query(r#"
            INSERT INTO accounts
                (uuid,                            platform_id,                   alias,       username,   email,
                 display_name,    auth_method,   ssh_host_alias,   status,   is_default,   last_used_at)
            VALUES
                (?,
                 (SELECT id FROM platforms WHERE uuid = '00000000-0008-0000-0000-000000000001'),
                 'anonymous',     'anonymous',   'anonymous@local',
                 'Anonymous User','anonymous',   NULL,              'active', 0,             NULL)
        "#)
        .bind(anon_uuid.to_string())
        .execute(pool)
        .await?;
        tracing::info!("bootstrapped anonymous system account (SQLite)");
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// main() — standard (NOT tokio::main) to preserve the main thread for Tauri
// ═══════════════════════════════════════════════════════════════════════════════

fn main() {
    // Initialize tracing before anything else
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_env("RUST_LOG")
                .add_directive("git_manager_desktop=info".parse().unwrap())
        )
        .init();

    // Create a Tokio runtime for async initialization.
    // This runtime is used ONLY for init_services() — Tauri creates its own
    // runtime for async command handlers.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("gm-init")
        .build()
        .expect("failed to create Tokio runtime for desktop initialization");

    let app_state = runtime
        .block_on(init_services())
        .expect("failed to initialize Git Manager services — check database connection and environment variables");

    tracing::info!("Services initialized successfully");
    //drop the runtime to prevent pool timeout errors where Tauri creates its own runtime
    // and tries to use the pool
    // drop(runtime);
    // NOTE: runtime is NOT dropped here intentionally.
    // SQLx pool tasks are spawned on this runtime; dropping it kills them and
    // causes "pool timed out" errors on every Tauri command. The runtime drops
    // naturally at end of main() when run_tauri_app() returns (window closed).
    // Tauri creates its own internal runtime for async command dispatch.

    // Start the Tauri event loop — this blocks the main thread until the window closes.
    run_tauri_app(app_state);

    // runtime is dropped here — pool cleanup happens on process exit.
}