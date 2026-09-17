mod infrastructure_plugin;
mod platform_helpers;
mod git_helpers;
mod cli_services;
mod first_run;

use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use gm_ports::outbound::git_op_repository::GitOpRepository;
use gm_ports::outbound::repository_provider::RepositoryProvider;

use gm_domain::accounts::ports::AccountRepository;
use gm_domain::ssh::ports::{SshKeyRepository, SshHostConfigRepository};
use gm_domain::ssh::services::ssh_service::SshOperations;
use gm_domain::configuration::ConfigRepository;
use gm_domain::repositories::ports::RepositoryRepository;

use gm_adapters::persistence::mysql::{
    MySqlAccountRepository, MySqlConfigRepository, MySqlGitOperationRepository,
    MySqlRepositoryRepository, MySqlSshHostConfigRepository, MySqlSshKeyRepository,
    create_mysql_pool, SqlxEventStore,
};
use gm_adapters::persistence::sqlite::{
    SqliteAccountRepository, SqliteConfigRepository, SqliteGitOperationRepository,
    SqliteRepositoryRepository, SqliteSshHostConfigRepository, SqliteSshKeyRepository,
    SqliteEventStore, create_sqlite_pool, setup_sqlite_schema,
};
use gm_adapters::platform::zig_credential_store::ZigCredentialStore;
use gm_adapters::ssh::zig_ssh_provider::ZigSshProvider;
use gm_adapters::git::zig_git_executor::ZigGitExecutor;
use gm_adapters::git::ZigDiskSpaceChecker;
use gm_adapters::git::ffi;
use gm_domain::git::ports::git_executor::GitExecutor;

use gm_domain::accounts::services::account_service::AccountService;
use gm_domain::configuration::services::config_service::ConfigService;
use gm_domain::ssh::services::ssh_service::SshService;

use gm_kernel::{
    bootstrap::{AppConfig, bootstrap},
    event_bus::EventStore,
    security::{Cancelled, CloneFallbackEngine, CloneUrlResolver, CredentialService, CredentialVault, install_signal_handler, register_child_killer},
};
use gm_ports::outbound::CredentialStore;

use gm_interface_cli::services::CliServicesHandle;

use infrastructure_plugin::InfrastructurePlugin;
use cli_services::ConcreteCliServices;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if first_run::ensure_database_configured() {
        std::process::exit(0);
    }

    let config = AppConfig::from_env();

    let database_url = std::env::var("GIT_MANAGER_DB_URL")
        .or_else(|_| std::env::var("GIT_ZYRIX_DB_URL"))
        .unwrap_or_default();
    let sqlite_path = std::env::var("GIT_MANAGER_SQLITE_PATH")
        .or_else(|_| std::env::var("GIT_ZYRIX_SQLITE_PATH"))
        .ok();
    let use_sqlite = database_url.is_empty();

    // ── Database pool, schema, repos ──────────────────────────────────────
    let (account_repo, ssh_key_repo, ssh_host_cfg_repo, repo_repo,
         git_op_repo, config_repo, event_store): (
        Arc<dyn AccountRepository>,
        Arc<dyn SshKeyRepository>,
        Arc<dyn SshHostConfigRepository>,
        Arc<dyn RepositoryRepository>,
        Arc<dyn GitOpRepository>,
        Arc<dyn ConfigRepository>,
        Arc<dyn EventStore>,
    ) = if use_sqlite {
        let sqlite_path = sqlite_path.as_deref().unwrap_or(":memory:");
        let pool = create_sqlite_pool(sqlite_path).await?;
        setup_sqlite_schema(&pool).await?;
        bootstrap_anonymous_account_sqlite(&pool).await?;

        let account_repo      = Arc::new(SqliteAccountRepository::new(pool.clone())) as Arc<dyn AccountRepository>;
        let ssh_key_repo      = Arc::new(SqliteSshKeyRepository::new(pool.clone())) as Arc<dyn SshKeyRepository>;
        let ssh_host_cfg_repo = Arc::new(SqliteSshHostConfigRepository::new(pool.clone())) as Arc<dyn SshHostConfigRepository>;
        let repo_repo         = Arc::new(SqliteRepositoryRepository::new(pool.clone())) as Arc<dyn RepositoryRepository>;
        let git_op_repo       = Arc::new(SqliteGitOperationRepository::new(pool.clone())) as Arc<dyn GitOpRepository>;
        let config_repo       = Arc::new(SqliteConfigRepository::new(pool.clone())) as Arc<dyn ConfigRepository>;
        let event_store       = Arc::new(SqliteEventStore::new(pool)) as Arc<dyn EventStore>;

        (account_repo, ssh_key_repo, ssh_host_cfg_repo, repo_repo,
         git_op_repo, config_repo, event_store)
    } else {
        let pool = create_mysql_pool(&database_url).await?;

        sqlx::migrate!("../../crates/gm_adapters/src/persistence/migrations")
            .run(&pool)
            .await?;
        bootstrap_anonymous_account(&pool).await?;

        let account_repo      = Arc::new(MySqlAccountRepository::new(pool.clone())) as Arc<dyn AccountRepository>;
        let ssh_key_repo      = Arc::new(MySqlSshKeyRepository::new(pool.clone())) as Arc<dyn SshKeyRepository>;
        let ssh_host_cfg_repo = Arc::new(MySqlSshHostConfigRepository::new(pool.clone())) as Arc<dyn SshHostConfigRepository>;
        let repo_repo         = Arc::new(MySqlRepositoryRepository::new(pool.clone())) as Arc<dyn RepositoryRepository>;
        let git_op_repo       = Arc::new(MySqlGitOperationRepository::new(pool.clone())) as Arc<dyn GitOpRepository>;
        let config_repo       = Arc::new(MySqlConfigRepository::new(pool.clone())) as Arc<dyn ConfigRepository>;
        let event_store       = Arc::new(SqlxEventStore::new(pool)) as Arc<dyn EventStore>;

        (account_repo, ssh_key_repo, ssh_host_cfg_repo, repo_repo,
         git_op_repo, config_repo, event_store)
    };

    // ── Shared infrastructure ─────────────────────────────────────────────
    let cred_store   = Arc::new(ZigCredentialStore::new());
    let ssh_provider = Arc::new(ZigSshProvider::new()) as Arc<dyn SshOperations>;
    let git_executor = Arc::new(ZigGitExecutor::new());

    let vault        = Arc::new(CredentialVault::new()?);
    let cred_service = Arc::new(CredentialService::new(
        Arc::clone(&cred_store) as Arc<dyn CredentialStore>,
        Arc::clone(&vault),
    ));

    // ── Domain services ───────────────────────────────────────────────────
    let account_svc = Arc::new(AccountService::new(Arc::clone(&account_repo)));
    let ssh_service = Arc::new(SshService::new(
        Arc::clone(&ssh_key_repo),
        Arc::clone(&ssh_host_cfg_repo),
        Arc::clone(&ssh_provider),
    ));
    let config_svc = Arc::new(ConfigService::new(Arc::clone(&config_repo)));

    let url_resolver = Arc::new(CloneUrlResolver::new(Arc::clone(&cred_service)));
    use gm_interface_cli::ui::cli_progress::CliProgressReporter;
    let progress_reporter = CliProgressReporter::new("Cloning repository");
    let disk_checker = Arc::new(ZigDiskSpaceChecker);

    // ── Ctrl+C signal handler ──
    let cancelled = Cancelled::new();
    install_signal_handler(cancelled.clone());
    register_child_killer(|| unsafe { ffi::gm_git_kill_process(0) });

    let fallback_engine = CloneFallbackEngine::new(
        Arc::clone(&git_executor) as Arc<dyn GitExecutor>,
        Arc::clone(&url_resolver),
    )
    .with_progress(progress_reporter.callback())
    .with_disk_checker(disk_checker)
    .with_cancellation(cancelled);

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

    // Build provider map from registered plugin providers
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

    let cli_services = ConcreteCliServices {
        account_svc,
        ssh_service,
        config_svc,
        repo_repo,
        git_executor,
        git_op_repo,
        cred_service,
        url_resolver,
        ssh_key_repo,
        fallback_engine,
        providers,
    };
    kernel.register::<CliServicesHandle>(Arc::new(CliServicesHandle::new(cli_services)));

    use gm_kernel::contracts::interface::InterfacePlugin as _;
    gm_interface_cli::CliPlugin::new()
        .run(kernel)
        .await?;

    Ok(())
}

/// Bootstrap the anonymous system account for MySQL.
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

/// Bootstrap the anonymous system account for SQLite.
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
