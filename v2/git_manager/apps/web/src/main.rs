// apps/web/src/main.rs
//
// Git Manager Web binary — Phase 10.
//
// Identical wiring strategy to apps/cli/src/main.rs. The only differences:
//   - Registers WebServicesHandle (not CliServicesHandle) after bootstrap
//   - Runs WebPlugin::run(kernel) which starts the Axum HTTP server
//
// Everything else — adapters, domain services, InfrastructurePlugin,
// ConcreteWebServices — follows the same pattern.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_domain::accounts::ports::AccountRepository;
use gm_domain::ssh::ports::{SshKeyRepository, SshHostConfigRepository};
use gm_domain::ssh::services::ssh_service::SshOperations;
use gm_domain::configuration::ConfigRepository;
use gm_domain::repositories::ports::RepositoryRepository;
use gm_ports::outbound::git_op_repository::GitOpRepository;
use gm_kernel::event_bus::EventStore;

use gm_ports::outbound::repository_provider::{PageCursor, RepositoryProvider};

// ── Adapters ───────────────────────────────────────────────────────────────────
use gm_adapters::persistence::mysql::{
    MySqlAccountRepository, MySqlConfigRepository, MySqlGitOperationRepository,
    MySqlPlatformRepository, MySqlRepositoryRepository, MySqlSshHostConfigRepository,
    MySqlSshKeyRepository, create_mysql_pool, SqlxEventStore,
};
use gm_adapters::persistence::sqlite::{
    SqliteAccountRepository, SqliteConfigRepository, SqliteGitOperationRepository,
    SqliteRepositoryRepository, SqliteSshHostConfigRepository, SqliteSshKeyRepository,
    SqliteEventStore, SqlitePlatformRepository, create_sqlite_pool, setup_sqlite_schema,
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
use gm_domain::configuration::services::config_service::ConfigService;
use gm_domain::repositories::entities::Repository;
// RepositoryRepository imported above
use gm_domain::repositories::value_objects::RepositoryUrl;
use gm_domain::repositories::entities::repository::Visibility;
use gm_domain::ssh::services::ssh_service::SshService;
use gm_domain::ssh::value_objects::KeyType;

// ── Kernel ─────────────────────────────────────────────────────────────────────
use gm_kernel::{
    bootstrap::{AppConfig, bootstrap},
    contracts::plugin::{EventSubscription, Plugin, PluginMetadata},
    security::{CloneFallbackEngine, CloneUrlResolver, CredentialService, CredentialVault, ResolvedStrategy, AuthStrategy, register_child_killer},
    service_registry::ServiceRegistry,
};
use gm_ports::outbound::CredentialStore;
use gm_ports::inbound::commands::{
    AddAccountCommand, CloneRepositoryCommand, GenerateSshKeyCommand,
    PullRepositoryCommand, PushRepositoryCommand, TestSshConnectionCommand,
};
use gm_domain::git::ports::git_executor::{
    CloneOptions, CommitOptions, PullOptions, PushOptions,
};

// ── Web interface ──────────────────────────────────────────────────────────────
use gm_interface_web::services::{
    DryRunPreview, GitOpResult, GitStatusEntry, OperationLogEntry, RepoDetectionResult,
    SshTestResult, SshValidationItemDto, SshValidationResultDto,
    WebServices, WebServicesHandle,
};
use gm_shared::{
    errors::{GitManagerError, PluginError},
    models::{account::AccountDto, platform::PlatformDto, repository::RepositoryDto, ssh_key::SshKeyDto},
};
use gm_ports::outbound::PlatformRepository;

// ═══════════════════════════════════════════════════════════════════════════════
// InfrastructurePlugin (identical to CLI — must be present in every binary)
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
// Platform info helpers (shared by both binaries)
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
        _                                       => ("custom",       "localhost",         22),
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
// ConcreteWebServices
// ═══════════════════════════════════════════════════════════════════════════════

#[allow(dead_code)]
struct ConcreteWebServices {
    account_svc:     Arc<AccountService>,
    ssh_service:     Arc<SshService>,
    config_svc:      Arc<ConfigService>,
    repo_repo:       Arc<dyn RepositoryRepository>,
    platform_repo:   Arc<dyn PlatformRepository>,
    git_executor:    Arc<ZigGitExecutor>,
    git_op_repo:     Arc<dyn GitOpRepository>,
    cred_service:    Arc<CredentialService>,
    url_resolver:    Arc<CloneUrlResolver>,
    fallback_engine: CloneFallbackEngine,
    providers:       HashMap<Uuid, Arc<dyn RepositoryProvider>>,
}

impl ConcreteWebServices {
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
impl WebServices for ConcreteWebServices {

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

    async fn get_account_by_alias(&self, alias: &str, platform_id: Option<Uuid>) -> Result<Option<AccountDto>, GitManagerError> {
        let accounts = self.list_accounts(platform_id).await?;
        Ok(accounts.into_iter().find(|a| a.alias == alias))
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
        std::fs::create_dir_all(&ssh_dir)
            .map_err(|e| GitManagerError::Other(format!("cannot create ~/.ssh/gitzyrix: {e}")))?;
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

    async fn test_ssh_connection(&self, cmd: TestSshConnectionCommand) -> Result<SshTestResult, GitManagerError> {
        let timeout = cmd.timeout_ms.unwrap_or(10_000);
        let result  = self.ssh_service.test_connection(cmd.account_uuid, timeout).await.map_err(GitManagerError::Ssh)?;
        let success = result.event.success;
        if success {
            let _ = self.account_svc.activate_account(cmd.account_uuid).await;
        }
        Ok(SshTestResult { success, username: result.event.username, error: result.event.error })
    }

    async fn list_repositories(&self, account_uuid: Option<Uuid>) -> Result<Vec<RepositoryDto>, GitManagerError> {
        let repos = if let Some(uuid) = account_uuid {
            self.repo_repo.list_by_account(uuid).await?
        } else {
            self.repo_repo.list_cloned().await?
        };
        Ok(repos.into_iter().map(|r| r.to_dto()).collect())
    }

    async fn clone_repository(&self, cmd: CloneRepositoryCommand) -> Result<RepositoryDto, GitManagerError> {
        let dest_path = match &cmd.destination {
            Some(d) => std::path::PathBuf::from(d),
            None => {
                let name = cmd.url.trim_end_matches(".git").split(['/', ':']).next_back().unwrap_or("repo");
                std::env::current_dir().unwrap_or_default().join(name)
            }
        };

        if let Some(account_uuid) = cmd.account_uuid {
            let account    = self.account_svc.get_account(account_uuid).await.map_err(GitManagerError::Account)?;
            let active_key = self.ssh_service.get_active_key(account_uuid).await.map_err(GitManagerError::Ssh)?
                .ok_or_else(|| GitManagerError::Other("No active SSH key for this account".to_string()))?;
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
            let full_name    = cmd.url.trim_end_matches(".git").split([':','/'])
                .rev().take(2).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("/");
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
            let full_name    = cmd.url.trim_end_matches(".git").split([':','/'])
                .rev().take(2).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("/");
            let name         = full_name.split('/').next_back().unwrap_or(&full_name).to_string();
            let url_vo       = RepositoryUrl::new(cmd.url.clone()).map_err(|e| GitManagerError::Other(e.to_string()))?;
            let mut repo     = Repository::new_discovered(uuid::Uuid::nil(), uuid::Uuid::nil(), name, full_name, url_vo, clone_result.branch.clone(), Visibility::Public);
            repo.mark_cloned(clone_result.local_path.to_string_lossy().to_string(), clone_result.commit_sha.clone());
            self.repo_repo.save(&repo).await?;
            Ok(repo.to_dto())
        }
    }

    async fn git_pull(&self, cmd: PullRepositoryCommand) -> Result<GitOpResult, GitManagerError> {
        let repo       = self.repo_repo.find_by_id(cmd.repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("Repository {} not found", cmd.repository_uuid)))?;
        let local_path = repo.local_path().ok_or_else(|| GitManagerError::Other("Repository not cloned".to_string()))?;
        let active_key = self.ssh_service.get_active_key(cmd.account_uuid).await.map_err(GitManagerError::Ssh)?
            .ok_or_else(|| GitManagerError::Other("No active SSH key".to_string()))?;
        let result     = self.git_executor.pull(PullOptions {
            repo_path:    std::path::PathBuf::from(local_path),
            ssh_key_path: std::path::PathBuf::from(active_key.private_key_path()),
            rebase:       cmd.rebase,
            branch:       cmd.branch.clone(),
        }).await.map_err(GitManagerError::Git)?;
        Ok(GitOpResult { commits_transferred: result.commits_pulled, current_sha: Some(result.commit_sha), had_conflicts: false })
    }

    async fn git_push(&self, cmd: PushRepositoryCommand) -> Result<GitOpResult, GitManagerError> {
        let repo       = self.repo_repo.find_by_id(cmd.repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("Repository {} not found", cmd.repository_uuid)))?;
        let local_path = repo.local_path().ok_or_else(|| GitManagerError::Other("Repository not cloned".to_string()))?;
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
        Ok(GitOpResult { commits_transferred: push.commits_pushed, current_sha: Some(commit.sha), had_conflicts: false })
    }

    async fn git_status(&self, repository_uuid: Uuid) -> Result<Vec<GitStatusEntry>, GitManagerError> {
        let repo = self.repo_repo.find_by_id(repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("Repository {} not found", repository_uuid)))?;
        let local = repo.local_path().ok_or_else(|| GitManagerError::Other("Not cloned locally".to_string()))?;
        let status = self.git_executor.status(std::path::Path::new(local)).await.map_err(GitManagerError::Git)?;
        Ok(status.entries.into_iter().map(|e| GitStatusEntry {
            path:   e.path,
            status: format!("{}{}", e.index_status, e.working_status),
        }).collect())
    }

    async fn list_platforms(&self) -> Result<Vec<PlatformDto>, GitManagerError> {
        self.platform_repo.list_active().await
    }

    async fn store_account_token(&self, account_uuid: Uuid, token: &str) -> Result<(), GitManagerError> {
        self.cred_service.store_token(account_uuid, token).await
    }

    async fn validate_ssh_setup(&self, account_uuid: Uuid, timeout_ms: u32) -> Result<SshValidationResultDto, GitManagerError> {
        let result = self.ssh_service.validate_setup(account_uuid, timeout_ms).await;
        Ok(SshValidationResultDto {
            all_passed: result.all_passed,
            checks:     result.checks.into_iter().map(|c| SshValidationItemDto {
                name:   c.name.to_string(),
                passed: c.passed,
                detail: c.detail,
            }).collect(),
        })
    }

    async fn add_key_to_agent(&self, _key_uuid: Uuid, _passphrase: Option<&str>) -> Result<(), GitManagerError> {
        Err(GitManagerError::Other(
            "Use add-key-to-agent with --account instead of --key-uuid".to_string()
        ))
    }

    async fn detect_repository(&self, path: &str) -> Result<RepoDetectionResult, GitManagerError> {
        let pb_path = std::path::Path::new(path);
        if !pb_path.join(".git").exists() {
            return Ok(RepoDetectionResult {
                tracked: false, account_alias: None, platform_slug: None,
                platform_name: None, remote_url: None, local_path: None,
                current_branch: None, repository_name: None,
            });
        }
        let remote_url = run_git(pb_path, &["remote", "get-url", "origin"]);
        let current_branch = run_git(pb_path, &["rev-parse", "--abbrev-ref", "HEAD"]);
        let canonical = pb_path.canonicalize().ok();
        let tracked = self.repo_repo.list_cloned().await.ok().and_then(|repos| {
            repos.into_iter().find(|r| {
                canonical.is_some() && r.local_path().map_or(false, |lp| {
                    std::path::Path::new(lp).canonicalize().ok() == canonical
                })
            })
        });
        if let Some(repo) = tracked {
            let account = self.account_svc.get_account(repo.account_id()).await.ok();
            let (slug, _, _) = account.as_ref()
                .map(|a| platform_info(a.platform_id()))
                .unwrap_or(("", "", 22));
            Ok(RepoDetectionResult {
                tracked: true,
                account_alias: account.as_ref().map(|a| a.alias().to_string()),
                platform_slug: Some(slug.to_string()),
                platform_name: account.and_then(|a| platform_display_name(a.platform_id())),
                remote_url,
                local_path: repo.local_path().map(|s| s.to_string()),
                current_branch,
                repository_name: Some(repo.name().to_string()),
            })
        } else {
            Ok(RepoDetectionResult {
                tracked: false, account_alias: None, platform_slug: None,
                platform_name: None, remote_url,
                local_path: Some(path.to_string()), current_branch, repository_name: None,
            })
        }
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

    async fn dry_run_pull(&self, repository_uuid: Uuid, account_uuid: Uuid) -> Result<DryRunPreview, GitManagerError> {
        let repo = self.repo_repo.find_by_id(repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("Repository {repository_uuid} not found")))?;
        let local_path = repo.local_path()
            .ok_or_else(|| GitManagerError::Other("Repository not cloned locally".to_string()))?;
        let pb_path = std::path::PathBuf::from(local_path);

        let active_key = self.ssh_service.get_active_key(account_uuid).await
            .map_err(GitManagerError::Ssh)?
            .ok_or_else(|| GitManagerError::Other("No active SSH key".to_string()))?;
        let _ = self.git_executor.fetch(&pb_path, std::path::Path::new(active_key.private_key_path())).await;
        let status = self.git_executor.status(&pb_path).await.map_err(GitManagerError::Git)?;

        let branch = run_git(&pb_path, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
        let remote_url = run_git(&pb_path, &["remote", "get-url", "origin"]).unwrap_or_default();

        Ok(DryRunPreview {
            summary: format!("Would pull from '{remote_url}' into '{}' (current branch: {branch})", repo.name()),
            commits: status.behind,
            conflicts_predicted: status.has_conflicts,
            details: vec![
                format!("Local branch:  {branch}"),
                format!("Remote ahead:  {} commit(s)", status.behind),
                format!("Local ahead:   {} commit(s)", status.ahead),
            ],
        })
    }

    async fn dry_run_push(&self, repository_uuid: Uuid, _account_uuid: Uuid) -> Result<DryRunPreview, GitManagerError> {
        let repo = self.repo_repo.find_by_id(repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("Repository {repository_uuid} not found")))?;
        let local_path = repo.local_path()
            .ok_or_else(|| GitManagerError::Other("Repository not cloned locally".to_string()))?;
        let pb_path = std::path::PathBuf::from(local_path);
        let status = self.git_executor.status(&pb_path).await.map_err(GitManagerError::Git)?;
        let branch = run_git(&pb_path, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
        let mut details = vec![
            format!("Branch:        {branch}"),
            format!("Files changed: {}", status.entries.len()),
        ];
        for e in &status.entries {
            details.push(format!("  {}{}  {}", e.index_status, e.working_status, e.path));
        }
        if status.ahead > 0 {
            details.push(format!("Local ahead:   {} commit(s) not yet pushed", status.ahead));
        }
        Ok(DryRunPreview {
            summary: format!("Would push {} file(s) to '{}' on branch '{branch}'", status.entries.len(), repo.name()),
            commits: status.ahead.max(1),
            conflicts_predicted: false,
            details,
        })
    }

    async fn config_list(&self) -> Result<Vec<(String, String)>, GitManagerError> {
        let cfg = self.config_svc.load().await?;
        let mut pairs = Vec::new();
        for (k, raw) in cfg.all() {
            let decoded: String = serde_json::from_str(raw).unwrap_or_else(|_| raw.clone());
            pairs.push((k.clone(), decoded));
        }
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(pairs)
    }

    async fn config_get(&self, key: &str) -> Result<Option<String>, GitManagerError> {
        let cfg = self.config_svc.load().await?;
        Ok(cfg.get_raw(key).and_then(|raw| serde_json::from_str(raw).ok()))
    }

    async fn config_set(&self, key: &str, value: &str) -> Result<(), GitManagerError> {
        self.config_svc.set(key, &value).await?;
        Ok(())
    }

    async fn list_recent_operations(&self, account_uuid: Option<Uuid>, limit: u32) -> Result<Vec<OperationLogEntry>, GitManagerError> {
        let ops = self.git_op_repo.list_recent(account_uuid, limit).await?;
        Ok(ops.into_iter().map(|o| OperationLogEntry {
            uuid: o.uuid, op_type: o.op_type, status: o.status,
            account: o.account_alias, repository: o.repository_name,
            started_at: o.started_at, duration_ms: o.duration_ms, error: o.error_message,
        }).collect())
    }
}

/// Run a git command in a repo path and return trimmed stdout, or None on error.
fn run_git(repo_path: &std::path::Path, args: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else { None }
        })
}


// ═══════════════════════════════════════════════════════════════════════════════
// main()
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env();
    gm_interface_web::banner::print_banner(env!("CARGO_PKG_VERSION"), &config.web_addr);

    let use_sqlite = config.database_url.is_empty();

    let (account_repo, ssh_key_repo, ssh_host_cfg_repo, repo_repo,
         git_op_repo, config_repo, event_store, platform_repo): (
        Arc<dyn AccountRepository>,
        Arc<dyn SshKeyRepository>,
        Arc<dyn SshHostConfigRepository>,
        Arc<dyn RepositoryRepository>,
        Arc<dyn GitOpRepository>,
        Arc<dyn ConfigRepository>,
        Arc<dyn EventStore>,
        Arc<dyn PlatformRepository>,
    ) = if use_sqlite {
        let sqlite_path = config.sqlite_path.as_deref().unwrap_or(":memory:");
        let pool = create_sqlite_pool(sqlite_path).await?;
        setup_sqlite_schema(&pool).await?;
        bootstrap_anonymous_account_sqlite(&pool).await?;

        let account_repo      = Arc::new(SqliteAccountRepository::new(pool.clone())) as Arc<dyn AccountRepository>;
        let ssh_key_repo      = Arc::new(SqliteSshKeyRepository::new(pool.clone())) as Arc<dyn SshKeyRepository>;
        let ssh_host_cfg_repo = Arc::new(SqliteSshHostConfigRepository::new(pool.clone())) as Arc<dyn SshHostConfigRepository>;
        let repo_repo         = Arc::new(SqliteRepositoryRepository::new(pool.clone())) as Arc<dyn RepositoryRepository>;
        let git_op_repo       = Arc::new(SqliteGitOperationRepository::new(pool.clone())) as Arc<dyn GitOpRepository>;
        let config_repo       = Arc::new(SqliteConfigRepository::new(pool.clone())) as Arc<dyn ConfigRepository>;
        let event_store       = Arc::new(SqliteEventStore::new(pool.clone())) as Arc<dyn EventStore>;
        let platform_repo     = Arc::new(SqlitePlatformRepository::new(pool)) as Arc<dyn PlatformRepository>;

        (account_repo, ssh_key_repo, ssh_host_cfg_repo, repo_repo,
         git_op_repo, config_repo, event_store, platform_repo)
    } else {
        let pool = create_mysql_pool(&config.database_url).await?;
        sqlx::migrate!("../../crates/gm_adapters/src/persistence/migrations").run(&pool).await?;
        bootstrap_anonymous_account(&pool).await?;

        let account_repo      = Arc::new(MySqlAccountRepository::new(pool.clone())) as Arc<dyn AccountRepository>;
        let ssh_key_repo      = Arc::new(MySqlSshKeyRepository::new(pool.clone())) as Arc<dyn SshKeyRepository>;
        let ssh_host_cfg_repo = Arc::new(MySqlSshHostConfigRepository::new(pool.clone())) as Arc<dyn SshHostConfigRepository>;
        let repo_repo         = Arc::new(MySqlRepositoryRepository::new(pool.clone())) as Arc<dyn RepositoryRepository>;
        let git_op_repo       = Arc::new(MySqlGitOperationRepository::new(pool.clone())) as Arc<dyn GitOpRepository>;
        let config_repo       = Arc::new(MySqlConfigRepository::new(pool.clone())) as Arc<dyn ConfigRepository>;
        let event_store       = Arc::new(SqlxEventStore::new(pool.clone())) as Arc<dyn EventStore>;
        let platform_repo     = Arc::new(MySqlPlatformRepository::new(pool)) as Arc<dyn PlatformRepository>;

        (account_repo, ssh_key_repo, ssh_host_cfg_repo, repo_repo,
         git_op_repo, config_repo, event_store, platform_repo)
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
    let config_svc = Arc::new(ConfigService::new(Arc::clone(&config_repo)));

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
    ];

    let kernel = bootstrap(config, plugins, event_store).await?;

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

    let web_services = ConcreteWebServices {
        account_svc,
        ssh_service,
        config_svc,
        repo_repo,
        platform_repo,
        git_executor,
        git_op_repo,
        cred_service,
        url_resolver,
        fallback_engine,
        providers,
    };

    kernel.register::<WebServicesHandle>(Arc::new(WebServicesHandle::new(web_services)));
    tracing::info!("WebServicesHandle registered in kernel service registry");

    use gm_kernel::contracts::interface::InterfacePlugin as _;
    gm_interface_web::WebPlugin::new().run(kernel).await?;

    Ok(())
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