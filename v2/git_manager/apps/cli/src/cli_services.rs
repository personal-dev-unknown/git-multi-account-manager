use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_ports::outbound::git_op_repository::GitOpRepository;
use gm_ports::outbound::repository_provider::{PageCursor, RepositoryProvider};

use gm_adapters::git::zig_git_executor::ZigGitExecutor;
use gm_domain::accounts::services::account_service::AccountService;
use gm_domain::accounts::value_objects::AuthMethod as DomainAuthMethod;
use gm_domain::configuration::services::config_service::ConfigService;
use gm_domain::repositories::entities::Repository;
use gm_domain::repositories::ports::RepositoryRepository;
use gm_domain::repositories::value_objects::RepositoryUrl;
use gm_domain::repositories::entities::repository::Visibility;
use gm_domain::ssh::ports::SshKeyRepository;
use gm_domain::ssh::services::ssh_service::SshService;
use gm_domain::ssh::value_objects::KeyType;
use gm_domain::git::ports::git_executor::{
    CloneOptions, CommitOptions, GitExecutor as _, PullOptions, PushOptions,
};
use gm_kernel::security::{AuthStrategy, CloneFallbackEngine, CloneUrlResolver, CredentialService, ResolvedStrategy};
use gm_ports::inbound::commands::{
    AddAccountCommand, CloneRepositoryCommand, GenerateSshKeyCommand,
    PullRepositoryCommand, PushRepositoryCommand, TestSshConnectionCommand,
};
use gm_interface_cli::services::{
    CliServices, DryRunPreview, GitOpResult, GitStatusEntry,
    OperationLogEntry, RepoDetectionResult, SshTestResult, SshValidationItemDto,
    SshValidationResultDto,
};
use gm_shared::{
    errors::GitManagerError,
    models::{account::AccountDto, repository::RepositoryDto, ssh_key::SshKeyDto},
};

use crate::platform_helpers::{platform_info, platform_display_name};
use crate::git_helpers::run_git;

#[allow(dead_code)]
pub(crate) struct ConcreteCliServices {
    pub account_svc:      Arc<AccountService>,
    pub ssh_service:      Arc<SshService>,
    pub config_svc:       Arc<ConfigService>,
    pub repo_repo:        Arc<dyn RepositoryRepository>,
    pub git_executor:     Arc<ZigGitExecutor>,
    pub git_op_repo:      Arc<dyn GitOpRepository>,
    pub cred_service:     Arc<CredentialService>,
    pub url_resolver:     Arc<CloneUrlResolver>,
    pub ssh_key_repo:     Arc<dyn SshKeyRepository>,
    pub fallback_engine:  CloneFallbackEngine,
    pub providers:        HashMap<Uuid, Arc<dyn RepositoryProvider>>,
}

impl ConcreteCliServices {
    fn enrich_platform_names(mut accounts: Vec<AccountDto>) -> Vec<AccountDto> {
        for a in &mut accounts {
            if a.platform_name.is_none() {
                a.platform_name = platform_display_name(a.platform_id);
            }
        }
        accounts
    }
}

#[async_trait]
impl CliServices for ConcreteCliServices {

    async fn list_accounts(&self, platform_id: Option<Uuid>) -> Result<Vec<AccountDto>, GitManagerError> {
        let accounts = if let Some(pid) = platform_id {
            self.account_svc.list_accounts_by_platform(pid).await.map_err(GitManagerError::Account)?
        } else {
            self.account_svc.list_accounts().await.map_err(GitManagerError::Account)?
        };
        Ok(Self::enrich_platform_names(accounts))
    }

    async fn get_account(&self, uuid: Uuid) -> Result<Option<AccountDto>, GitManagerError> {
        match self.account_svc.get_account(uuid).await {
            Ok(account) => {
                let mut dto = account.to_dto();
                dto.platform_name = platform_display_name(dto.platform_id);
                Ok(Some(dto))
            }
            Err(gm_shared::errors::AccountError::NotFound { .. }) => Ok(None),
            Err(e) => Err(GitManagerError::Account(e)),
        }
    }

    async fn get_account_by_alias(
        &self,
        alias:       &str,
        platform_id: Option<Uuid>,
    ) -> Result<Option<AccountDto>, GitManagerError> {
        let accounts = self.list_accounts(platform_id).await?;
        Ok(accounts.into_iter().find(|a| a.alias == alias))
    }

    async fn add_account(&self, cmd: AddAccountCommand) -> Result<AccountDto, GitManagerError> {
        let auth = DomainAuthMethod::from_str(&cmd.auth_method).ok_or_else(|| {
            GitManagerError::Other(format!("unknown auth_method '{}'", cmd.auth_method))
        })?;
        let result = self.account_svc
            .add_account(cmd.alias, cmd.platform_id, cmd.username, cmd.email, auth)
            .await
            .map_err(GitManagerError::Account)?;
        let mut dto = result.account.to_dto();
        dto.platform_name = platform_display_name(dto.platform_id);
        Ok(dto)
    }

    async fn remove_account(&self, uuid: Uuid) -> Result<(), GitManagerError> {
        self.account_svc.remove_account(uuid).await
            .map(|_| ())
            .map_err(GitManagerError::Account)
    }

    async fn set_default_account(&self, account_uuid: Uuid, platform_id: Uuid) -> Result<(), GitManagerError> {
        self.account_svc.set_default_account(account_uuid, platform_id).await
            .map(|_| ())
            .map_err(GitManagerError::Account)
    }

    async fn store_account_token(&self, account_uuid: Uuid, token: &str) -> Result<(), GitManagerError> {
        self.cred_service.store_token(account_uuid, token).await
    }

    async fn list_ssh_keys(&self, account_uuid: Uuid) -> Result<Vec<SshKeyDto>, GitManagerError> {
        self.ssh_service.list_keys_for_account(account_uuid).await
            .map(|keys| keys.into_iter().map(|k| k.to_dto()).collect())
            .map_err(GitManagerError::Ssh)
    }

    async fn generate_ssh_key(&self, cmd: GenerateSshKeyCommand) -> Result<SshKeyDto, GitManagerError> {
        let account   = self.account_svc.get_account(cmd.account_uuid).await
            .map_err(GitManagerError::Account)?;
        let (slug, hostname, port) = platform_info(account.platform_id());
        let key_type  = KeyType::from_str(&cmd.key_type)
            .unwrap_or(KeyType::Ed25519);
        let ssh_dir   = dirs_next::home_dir()
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
            account.uuid(),
            account.alias(),
            slug,
            account.email(),
            hostname,
            key_type,
            ssh_dir,
            port,
            cmd.add_to_agent,
            cmd.passphrase.as_deref(),
            10_000,
        ).await.map_err(GitManagerError::Ssh)?;

        Ok(result.ssh_key.to_dto())
    }

    async fn test_ssh_connection(&self, cmd: TestSshConnectionCommand) -> Result<SshTestResult, GitManagerError> {
        let timeout = cmd.timeout_ms.unwrap_or(10_000);
        let result  = self.ssh_service.test_connection(cmd.account_uuid, timeout).await
            .map_err(GitManagerError::Ssh)?;
        if result.event.success {
            let _ = self.account_svc.activate_account(cmd.account_uuid).await;
        }
        Ok(SshTestResult {
            success:  result.event.success,
            username: result.event.username,
            error:    result.event.error,
        })
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

    async fn add_key_to_agent(&self, key_uuid: Uuid, passphrase: Option<&str>) -> Result<(), GitManagerError> {
        let ssh_key = self.ssh_key_repo.find_by_id(key_uuid).await
            .map_err(GitManagerError::Ssh)?
            .ok_or_else(|| GitManagerError::Other(
                format!("SSH key with UUID {key_uuid} not found")
            ))?;
        self.ssh_service.add_to_agent(ssh_key.account_id(), passphrase).await
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

    async fn clone_repository(&self, cmd: CloneRepositoryCommand) -> Result<RepositoryDto, GitManagerError> {
        if cmd.account_uuid.is_none() {
            let dest_path = match &cmd.destination {
                Some(d) => std::path::PathBuf::from(d),
                None => {
                    let repo_name = cmd.url.trim_end_matches(".git")
                        .split(['/', ':'])
                        .next_back()
                        .unwrap_or("repo");
                    std::env::current_dir()
                        .unwrap_or_else(|_| std::path::PathBuf::from("."))
                        .join(repo_name)
                }
            };

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

            let full_name = cmd.url.trim_end_matches(".git")
                .split([':','/'])
                .rev()
                .take(2)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("/");
            let name = full_name.split('/').next_back().unwrap_or(&full_name).to_string();

            let url_vo = RepositoryUrl::new(cmd.url.clone())
                .map_err(|e| GitManagerError::Other(e.to_string()))?;

            let mut repo = Repository::new_discovered(
                uuid::Uuid::nil(),                      // anonymous account
                platform_id_from_url(&cmd.url),          // detect from URL
                name,
                full_name,
                url_vo,
                clone_result.branch.clone(),
                Visibility::Public,
            );
            repo.mark_cloned(
                clone_result.local_path.to_string_lossy().to_string(),
                clone_result.commit_sha.clone(),
            );
            self.repo_repo.save(&repo).await?;

            return Ok(repo.to_dto());
        }

        let account_uuid = cmd.account_uuid
            .ok_or_else(|| GitManagerError::Other(
                "account_uuid is required for authenticated clone".to_string()
            ))?;
        let account = self.account_svc.get_account(account_uuid).await
            .map_err(GitManagerError::Account)?;

        let active_key = self.ssh_service.get_active_key(account_uuid).await
            .map_err(GitManagerError::Ssh)?
            .ok_or_else(|| GitManagerError::Other(
                "No active SSH key for this account. Run 'git-zyrix ssh generate --account <alias>' first.".to_string()
            ))?;

        let (_slug, hostname, _port) = platform_info(account.platform_id());
        let strategy = match account.auth_method() {
            DomainAuthMethod::Ssh           => AuthStrategy::Ssh,
            DomainAuthMethod::HttpsPat      => AuthStrategy::HttpsPat,
            DomainAuthMethod::HttpsPassword => AuthStrategy::HttpsPassword,
            DomainAuthMethod::OAuth         => AuthStrategy::OAuth,
            DomainAuthMethod::Anonymous     => AuthStrategy::Anonymous,
        };
        let resolved_url = CloneUrlResolver::build_url(
            &cmd.url,
            &strategy,
            &hostname,
            account.ssh_host_alias().unwrap_or(account.alias()),
        );

        let dest_path = match &cmd.destination {
            Some(d) => std::path::PathBuf::from(d),
            None => {
                let repo_name = resolved_url.trim_end_matches(".git")
                    .split(['/', ':'])
                    .next_back()
                    .unwrap_or("repo");
                std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join(repo_name)
            }
        };

        let clone_opts = CloneOptions {
            url:             resolved_url.clone(),
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
            primary: strategy.clone(),
            fallbacks: vec![AuthStrategy::Anonymous],
        };
        let attempt = self.fallback_engine.clone_with_fallback(&clone_opts, &resolved, Some(account_uuid)).await?;
        let clone_result = attempt.result.ok_or_else(|| GitManagerError::Other("clone returned no result".to_string()))?;

        let full_name = resolved_url
            .trim_end_matches(".git")
            .split([':','/'])
            .rev()
            .take(2)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("/");
        let name = full_name.split('/').next_back().unwrap_or(&full_name).to_string();

        let url_vo = RepositoryUrl::new(resolved_url.clone())
            .map_err(|e| GitManagerError::Other(e.to_string()))?;

        let mut repo = Repository::new_discovered(
            account_uuid,
            account.platform_id(),
            name,
            full_name,
            url_vo,
            clone_result.branch.clone(),
            Visibility::Private,
        );
        repo.mark_cloned(
            clone_result.local_path.to_string_lossy().to_string(),
            clone_result.commit_sha.clone(),
        );
        self.repo_repo.save(&repo).await?;

        Ok(repo.to_dto())
    }

    async fn git_pull(&self, cmd: PullRepositoryCommand) -> Result<GitOpResult, GitManagerError> {
        let repo = self.repo_repo.find_by_id(cmd.repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(
                format!("Repository {} not found — clone it first", cmd.repository_uuid)
            ))?;
        let local_path = repo.local_path()
            .ok_or_else(|| GitManagerError::Other("Repository not cloned locally yet".to_string()))?;

        let active_key = self.ssh_service.get_active_key(cmd.account_uuid).await
            .map_err(GitManagerError::Ssh)?
            .ok_or_else(|| GitManagerError::Other("No active SSH key for this account".to_string()))?;

        let pull_opts = PullOptions {
            repo_path:    std::path::PathBuf::from(local_path),
            ssh_key_path: std::path::PathBuf::from(active_key.private_key_path()),
            rebase:       cmd.rebase,
            branch:       cmd.branch.clone(),
        };
        let result = self.git_executor.pull(pull_opts).await
            .map_err(GitManagerError::Git)?;

        let _ = self.git_op_repo.record(
            cmd.repository_uuid, cmd.account_uuid, "pull", "success",
            Some(&result.commit_sha), None, None,
        ).await;

        Ok(GitOpResult {
            commits_transferred: result.commits_pulled,
            current_sha:         Some(result.commit_sha),
            had_conflicts:       false,
        })
    }

    async fn git_push(&self, cmd: PushRepositoryCommand) -> Result<GitOpResult, GitManagerError> {
        let repo = self.repo_repo.find_by_id(cmd.repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(
                format!("Repository {} not found", cmd.repository_uuid)
            ))?;
        let local_path = repo.local_path()
            .ok_or_else(|| GitManagerError::Other("Repository not cloned locally yet".to_string()))?;

        let active_key = self.ssh_service.get_active_key(cmd.account_uuid).await
            .map_err(GitManagerError::Ssh)?
            .ok_or_else(|| GitManagerError::Other("No active SSH key for this account".to_string()))?;

        self.git_executor.stage_all(std::path::Path::new(local_path)).await
            .map_err(GitManagerError::Git)?;

        let account = self.account_svc.get_account(cmd.account_uuid).await
            .map_err(GitManagerError::Account)?;
        let commit_opts = CommitOptions {
            repo_path:    std::path::PathBuf::from(local_path),
            message:      cmd.commit_message.clone(),
            author_name:  account.display_name().unwrap_or(account.username()).to_string(),
            author_email: account.email().to_string(),
            amend:        false,
        };
        let commit_result = self.git_executor.commit(commit_opts).await
            .map_err(GitManagerError::Git)?;

        let push_opts = PushOptions {
            repo_path:    std::path::PathBuf::from(local_path),
            ssh_key_path: std::path::PathBuf::from(active_key.private_key_path()),
            remote:       "origin".to_string(),
            branch:       cmd.branch.clone(),
            force:        cmd.force,
        };
        let push_result = self.git_executor.push(push_opts).await
            .map_err(GitManagerError::Git)?;

        let _ = self.git_op_repo.record(
            cmd.repository_uuid, cmd.account_uuid, "push", "success",
            Some(&commit_result.sha), None, None,
        ).await;

        Ok(GitOpResult {
            commits_transferred: push_result.commits_pushed,
            current_sha:         Some(commit_result.sha),
            had_conflicts:       false,
        })
    }

    async fn git_status(&self, repo_path: &std::path::Path) -> Result<Vec<GitStatusEntry>, GitManagerError> {
        let status = self.git_executor.status(repo_path).await
            .map_err(GitManagerError::Git)?;
        Ok(status.entries.into_iter().map(|e| GitStatusEntry {
            path:   e.path,
            status: format!("{}{}", e.index_status, e.working_status),
        }).collect())
    }

    async fn dry_run_pull(&self, repository_uuid: Uuid, account_uuid: Uuid) -> Result<DryRunPreview, GitManagerError> {
        let repo = self.repo_repo.find_by_id(repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("Repository {repository_uuid} not found")))?;
        let local_path = repo.local_path()
            .ok_or_else(|| GitManagerError::Other("Repository not cloned locally".to_string()))?;

        let pb_path = std::path::PathBuf::from(local_path);
        let active_key = self.ssh_service.get_active_key(account_uuid).await
            .map_err(GitManagerError::Ssh)?
            .ok_or_else(|| GitManagerError::Other("No active SSH key for this account".to_string()))?;

        self.git_executor.fetch(&pb_path, std::path::Path::new(active_key.private_key_path())).await
            .map_err(GitManagerError::Git)?;

        let status = self.git_executor.status(&pb_path).await
            .map_err(GitManagerError::Git)?;

        let branch = run_git(&pb_path, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
        let remote_url = run_git(&pb_path, &["remote", "get-url", "origin"]).unwrap_or_default();

        Ok(DryRunPreview {
            summary: format!(
                "Would pull from '{}' into '{}' (current branch: {branch})",
                remote_url, repo.name(),
            ),
            commits: status.behind,
            conflicts_predicted: status.has_conflicts,
            details: vec![
                format!("Local branch:  {branch}"),
                format!("Remote ahead:  {} commit(s)", status.behind),
                format!("Local ahead:   {} commit(s)", status.ahead),
                if status.behind > 0 { format!("Would fetch {} new commit(s)", status.behind) } else { "Already up to date".to_string() },
            ],
        })
    }

    async fn dry_run_push(&self, repository_uuid: Uuid, _account_uuid: Uuid) -> Result<DryRunPreview, GitManagerError> {
        let repo = self.repo_repo.find_by_id(repository_uuid).await?
            .ok_or_else(|| GitManagerError::Other(format!("Repository {repository_uuid} not found")))?;
        let local_path = repo.local_path()
            .ok_or_else(|| GitManagerError::Other("Repository not cloned locally".to_string()))?;

        let pb_path = std::path::PathBuf::from(local_path);
        let status = self.git_executor.status(&pb_path).await
            .map_err(GitManagerError::Git)?;

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
            summary: format!(
                "Would push {} file(s) to '{}' on branch '{branch}'",
                status.entries.len(),
                repo.name(),
            ),
            commits: status.ahead.max(1),
            conflicts_predicted: false,
            details,
        })
    }

    async fn detect_repository(&self, path: &std::path::Path) -> Result<RepoDetectionResult, GitManagerError> {
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Ok(RepoDetectionResult {
                tracked: false,
                account_alias: None,
                platform_slug: None,
                platform_name: None,
                remote_url: None,
                local_path: None,
                current_branch: None,
                repository_name: None,
            });
        }

        let remote_url = run_git(path, &["remote", "get-url", "origin"]);
        let current_branch = run_git(path, &["rev-parse", "--abbrev-ref", "HEAD"]);
        let canonical = path.canonicalize().ok();

        let tracked = if let Ok(repos) = self.repo_repo.list_cloned().await {
            repos.into_iter().find(|r| {
                canonical.is_some() && r.local_path().map_or(false, |lp| {
                    std::path::Path::new(lp).canonicalize().ok() == canonical
                })
            })
        } else {
            None
        };

        if let Some(repo) = tracked {
            let account = self.account_svc.get_account(repo.account_id()).await.ok();
            let (slug, _hostname, _port) = account.as_ref()
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
                tracked: false,
                account_alias: None,
                platform_slug: None,
                platform_name: None,
                remote_url,
                local_path: path.to_str().map(|s| s.to_string()),
                current_branch,
                repository_name: None,
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

    async fn list_recent_operations(
        &self,
        account_uuid: Option<Uuid>,
        limit:        u32,
    ) -> Result<Vec<OperationLogEntry>, GitManagerError> {
        let ops = self.git_op_repo.list_recent(account_uuid, limit).await?;
        Ok(ops.into_iter().map(|o| OperationLogEntry {
            uuid:        o.uuid,
            op_type:     o.op_type,
            status:      o.status,
            account:     o.account_alias,
            repository:  o.repository_name,
            started_at:  o.started_at,
            duration_ms: o.duration_ms,
            error:       o.error_message,
        }).collect())
    }
}

/// Map a remote URL to a known platform UUID so anonymous clones get a
/// valid platform_id FK in the repositories table.
fn platform_id_from_url(url: &str) -> Uuid {
    if url.contains("github.com") {
        Uuid::parse_str("00000000-0001-0000-0000-000000000001").unwrap()
    } else if url.contains("gitlab.com") {
        Uuid::parse_str("00000000-0002-0000-0000-000000000001").unwrap()
    } else if url.contains("bitbucket.org") {
        Uuid::parse_str("00000000-0003-0000-0000-000000000001").unwrap()
    } else if url.contains("dev.azure.com") {
        Uuid::parse_str("00000000-0004-0000-0000-000000000001").unwrap()
    } else if url.contains("sourceforge.net") || url.contains("code.sf.net") {
        Uuid::parse_str("00000000-0005-0000-0000-000000000001").unwrap()
    } else {
        // Unknown/self-hosted as fallback
        Uuid::parse_str("00000000-0006-0000-0000-000000000001").unwrap()
    }
}
