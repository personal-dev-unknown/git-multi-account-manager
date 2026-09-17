// crates/gm_kernel/src/bootstrap.rs
//
// bootstrap() is the single function that assembles the entire runtime. It is
// called exactly once, in the binary entry point (apps/cli/main.rs, apps/web,
// etc.), and returns a ready-to-use Arc<Kernel>. After this function returns,
// the kernel is fully operational: all plugins are loaded, all event subscriptions
// are wired, and the lifecycle's startup hooks have run.
//
// ── Architectural separation of concerns ─────────────────────────────────────
// A key design decision: bootstrap() does NOT connect to the database. The
// database connection pool is created in the binary entry point using sqlx,
// which is an infrastructure concern (gm_adapters) rather than a kernel concern.
// The concrete EventStore implementation (SqlxEventStore from gm_adapters) is
// created by the binary and passed in as `Arc<dyn EventStore>`. This keeps sqlx
// out of gm_kernel entirely and respects the dependency direction rule:
//
//   Adapters → Kernel (not the reverse)
//
// The same principle applies to the concrete plugin implementations: they come
// from the binary as `Vec<Box<dyn Plugin>>`. The kernel knows about the Plugin
// trait; it never knows about GitHubPlugin or any concrete type.
//
// ── Boot sequence ─────────────────────────────────────────────────────────────
//   1. Initialise structured tracing (MUST be first — all subsequent steps emit logs)
//   2. Create ServiceRegistry — the foundation all other subsystems depend on
//   3. Create EventBus — wires the audit handler before any plugin loads, so
//      no events escape the audit trail even during the boot sequence itself
//   4. Create CommandBus — middleware chain: logging → validation → auth → handler
//   5. Create WorkflowRegistry + WorkflowRunner
//   6. Create LifecycleManager
//   7. Create CredentialVault — derives the machine key (fails fast if derivation fails)
//   8. Load static plugins in topological dependency order
//   9. Scan plugin_dir for dynamic .so plugins (if configured)
//  10. Wire plugin event subscriptions to the event bus
//  11. Run lifecycle startup hooks in ascending priority order
//  12. Return Arc<Kernel>
//
// ── Failure semantics ─────────────────────────────────────────────────────────
// Any failure in steps 1–11 returns Err and the process should exit. There is
// no partial-boot recovery: if a plugin fails to load, the entire boot fails
// rather than continuing with a degraded set of plugins. This is intentional —
// a partially-loaded system is harder to reason about than a clean failure.
//
// ── Tracing initialisation idempotency ───────────────────────────────────────
// tracing_subscriber::try_init() is used instead of init() so that tests that
// call bootstrap() more than once in the same process (e.g. integration tests)
// do not panic on the second call. The returned Err from try_init() is silently
// ignored — the only failure case is "already initialised", which is harmless.

use std::collections::HashMap;
use std::sync::Arc;

use gm_shared::errors::GitManagerError;

use crate::{
    command_bus::{
        CommandBus,
        middleware::{AuthMiddleware, LoggingMiddleware, ValidationMiddleware},
    },
    contracts::plugin::Plugin,
    event_bus::{AuditEventHandler, EventBus, EventStore},
    kernel::Kernel,
    lifecycle::LifecycleManager,
    plugin_system::{PluginLifecycle, PluginLoader, PluginRegistry, PluginValidator},
    security::{CredentialVault, PermissionChecker},
    service_registry::ServiceRegistry,
    workflow_engine::{WorkflowRegistry, WorkflowRunner},
};

// ─────────────────────────────────────────────────────────────────────────────
// AppConfig
// ─────────────────────────────────────────────────────────────────────────────

/// All configuration the application needs at startup.
///
/// The `database_url` and `sqlite_path` fields are consumed by the binary entry
/// point to create the sqlx pool before calling bootstrap(). They are included
/// here so the binary has a single config struct to manage. bootstrap() itself
/// only reads `log_level`, `web_addr`, and `plugin_dir`.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// MySQL connection URL.
    /// Format: "mysql://user:password@host/database"
    /// Used by the binary entry point to create the sqlx pool and run migrations.
    pub database_url: String,

    /// If set, uses SQLite at this path instead of MySQL.
    /// ":memory:" uses an in-memory database — fast for integration tests.
    /// Used by the binary entry point (not by bootstrap directly).
    pub sqlite_path: Option<String>,

    /// RUST_LOG compatible filter string. Controls which log levels and crates
    /// emit log output. Examples: "info", "gm_kernel=debug,info", "warn".
    pub log_level: String,

    /// The address and port the web interface plugin should bind to.
    /// Example: "127.0.0.1:5000"
    pub web_addr: String,

    /// If set, the kernel scans this directory for dynamic .so/.dylib plugin files.
    /// Each file must export the `_gm_plugin_create` C ABI function.
    /// Failures for individual plugins are logged and skipped; they do not abort boot.
    pub plugin_dir: Option<String>,
}

impl AppConfig {
    /// Reads configuration from environment variables with production-safe defaults.
    ///
    /// Environment variables checked (new `GIT_MANAGER_*` naming preferred,
    /// legacy `GIT_ZYRIX_*` fallback for backward compatibility):
    ///   GIT_MANAGER_DB_URL  or  GIT_ZYRIX_DB_URL       — MySQL connection URL
    ///   GIT_MANAGER_SQLITE_PATH or GIT_ZYRIX_SQLITE_PATH — SQLite path
    ///   RUST_LOG               — log filter string
    ///   GIT_MANAGER_WEB_ADDR or GIT_ZYRIX_WEB_ADDR      — bind address
    ///   GIT_MANAGER_PLUGIN_DIR or GIT_ZYRIX_PLUGIN_DIR   — plugin directory
    pub fn from_env() -> Self {
        // Load .env by walking up the directory tree from CWD.
        let _ = crate::env_loader::try_load_dotenv();

        fn env_or(name: &str, legacy: &str) -> Result<String, std::env::VarError> {
            std::env::var(name).or_else(|_| std::env::var(legacy))
        }

        let database_url = env_or("GIT_MANAGER_DB_URL", "GIT_ZYRIX_DB_URL").unwrap_or_default();

        let sqlite_path = env_or("GIT_MANAGER_SQLITE_PATH", "GIT_ZYRIX_SQLITE_PATH").ok()
            .or_else(|| {
                // If neither MySQL URL nor SQLite path is set, default to
                // ~/.local/share/git-manager/git_manager.db so the app works
                // without any environment configuration for local development.
                if !database_url.is_empty() { return None; }
                let home = std::env::var("HOME").ok()
                    .or_else(|| dirs_next::home_dir().map(|p| p.to_string_lossy().to_string()))
                    .unwrap_or_default();
                if home.is_empty() { return None; }
                let path = std::path::Path::new(&home).join(".local/share/git-manager/git_manager.db");
                tracing::info!(
                    "GIT_MANAGER_DB_URL not set — using SQLite at {}",
                    path.display(),
                );
                let _ = std::fs::create_dir_all(path.parent().unwrap());
                Some(path.to_string_lossy().to_string())
            });

        Self {
            database_url,
            sqlite_path,
            log_level:    std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            web_addr:     env_or("GIT_MANAGER_WEB_ADDR", "GIT_ZYRIX_WEB_ADDR")
                              .unwrap_or_else(|_| "127.0.0.1:5008".to_string()),
            plugin_dir:   env_or("GIT_MANAGER_PLUGIN_DIR", "GIT_ZYRIX_PLUGIN_DIR").ok(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// bootstrap()
// ─────────────────────────────────────────────────────────────────────────────

/// Assembles the kernel and returns a ready-to-use `Arc<Kernel>`.
///
/// # Parameters
///
/// - `config`         — Application configuration (log level, web address, plugin dir).
/// - `static_plugins` — Plugins compiled into the binary. The kernel does not
///                      import any concrete plugin crate; the binary supplies them.
/// - `event_store`    — The persistence backend for the audit event log. Typically
///                      a `SqlxEventStore` from `gm_adapters`, created by the binary
///                      after opening the database pool.
///
/// # Errors
///
/// Returns `Err` if any of the following fail:
/// - The credential vault cannot derive its machine key
/// - A plugin's declared dependency is not in `static_plugins`
/// - A plugin's `min_kernel_version` is newer than the running kernel version
/// - The plugin dependency graph contains a cycle
/// - A plugin's `on_load()` returns `Err`
/// - Any lifecycle startup hook returns `Err`
pub async fn bootstrap(
    config:         AppConfig,
    static_plugins: Vec<Box<dyn Plugin>>,
    event_store:    Arc<dyn EventStore>,
) -> Result<Arc<Kernel>, GitManagerError> {

    // ── Step 1: Tracing ───────────────────────────────────────────────────────
    // Must happen before any tracing::info!/warn!/error! calls.
    // try_init() is idempotent so integration tests can call bootstrap() multiple
    // times in the same process without panicking.
    init_tracing(&config.log_level);

    // tracing::info!(
    //     log_level  = %config.log_level,
    //     web_addr   = %config.web_addr,
    //     plugin_dir = ?config.plugin_dir,
    //     "Git Manager booting — kernel v{}",
    //     crate::plugin_system::KERNEL_VERSION
    // );

    // ── Step 2: Service registry ──────────────────────────────────────────────
    // Created first because the event bus, command bus, and plugins all need it.
    // It starts empty; adapters and plugins populate it during their load phase.
    let service_registry = Arc::new(ServiceRegistry::new());

    // ── Step 3: Event bus + audit handler ─────────────────────────────────────
    // The audit handler is registered BEFORE any plugin loads so that no events
    // escape the audit trail, including events emitted by on_load() hooks.
    let event_bus = Arc::new(EventBus::new());
    let audit_handler = Arc::new(AuditEventHandler::new(Arc::clone(&event_store)));
    event_bus.subscribe_all(audit_handler);

    tracing::debug!("event bus initialised with audit handler");

    // ── Step 4: Command bus ───────────────────────────────────────────────────
    // Middlewares run in order: logging wraps everything; validation checks
    // structural correctness; auth checks permissions. All are constructed here
    // so the command bus is fully operational before plugins register handlers.
    let permission_checker = Arc::new(PermissionChecker::new());
    let middlewares: Vec<Arc<dyn crate::command_bus::middleware::Middleware>> = vec![
        Arc::new(LoggingMiddleware::default()),
        Arc::new(ValidationMiddleware::default()),
        Arc::new(AuthMiddleware::new(Arc::clone(&permission_checker))),
    ];
    let command_bus = Arc::new(CommandBus::new(middlewares));

    tracing::debug!("command bus initialised with 3 middlewares");

    // ── Step 5: Workflow engine ───────────────────────────────────────────────
    let workflow_registry = Arc::new(WorkflowRegistry::new());
    let workflow_runner   = Arc::new(WorkflowRunner::new(
        Arc::clone(&workflow_registry),
        Arc::clone(&service_registry),
        Arc::clone(&event_bus),
    ));

    // ── Step 6: Lifecycle manager ─────────────────────────────────────────────
    let lifecycle = Arc::new(LifecycleManager::new());

    // ── Step 7: Credential vault ──────────────────────────────────────────────
    // Fails fast here if the machine-key derivation fails (e.g. /etc/machine-id
    // is unreadable). Better to abort cleanly at boot than to silently store
    // credentials under a fallback key that differs between reboots.
    let credential_vault = Arc::new(
        CredentialVault::new()
            .map_err(|e| {
                tracing::error!(error = %e, "credential vault initialisation failed");
                e
            })?
    );

    tracing::debug!("credential vault initialised");

    // ── Step 8: Load static plugins in topological order ─────────────────────
    let plugin_registry = Arc::new(PluginRegistry::new());
    let plugin_loader   = PluginLoader::new(
        Arc::clone(&plugin_registry),
        Arc::clone(&service_registry),
    );

    // Collect metadata for the topological sort. This validates version
    // compatibility, checks for dependency cycles, and resolves a stable load
    // order BEFORE calling on_load() on any plugin.
    let metadata_list: Vec<_> = static_plugins.iter()
        .map(|p| p.metadata().clone())
        .collect();

    let sorted_metadata = PluginValidator::topological_sort(&metadata_list)
        .map_err(|e| {
            tracing::error!(error = %e, "plugin dependency resolution failed");
            GitManagerError::Plugin(e)
        })?;

    // Build a name → plugin map so we can consume them in topological order.
    // HashMap::remove() gives us ownership of each plugin exactly once.
    let mut plugin_map: HashMap<String, Box<dyn Plugin>> = static_plugins
        .into_iter()
        .map(|p| (p.metadata().name.clone(), p))
        .collect();

    for meta in &sorted_metadata {
        let plugin = plugin_map
            .remove(&meta.name)
            .expect("topological sort returned a name not present in plugin_map — this is a bug");

        // tracing::info!(
        //     plugin   = %meta.name,
        //     version  = %meta.version,
        //     priority = meta.load_priority,
        //     "loading static plugin"
        // );

        plugin_loader.load_static(plugin)
            .map_err(|e| {
                tracing::error!(plugin = %meta.name, error = %e, "static plugin load failed");
                GitManagerError::Plugin(e)
            })?;
    }

    // tracing::info!("{} static plugins loaded", plugin_registry.count());

    // ── Step 9: Dynamic plugin directory ─────────────────────────────────────
    // Dynamic plugins are optional. Load failures for individual .so files are
    // logged and skipped (load_directory does not abort on individual failures).
    if let Some(dir) = &config.plugin_dir {
        // tracing::info!(dir = %dir, "scanning for dynamic plugins");
        plugin_loader.load_directory(dir)
            .map_err(|e| {
                tracing::error!(dir = %dir, error = %e, "plugin directory scan failed");
                GitManagerError::Plugin(e)
            })?;
        // tracing::info!("total plugins after dynamic load: {}", plugin_registry.count());
    }

    // ── Step 10: Wire event subscriptions ────────────────────────────────────
    // Now that all plugins are loaded and their services are in the registry,
    // wire each plugin's declared event subscriptions to the event bus.
    // This is done in a separate pass so all services are available before
    // any subscription handler could be triggered.
    let plugin_lifecycle = PluginLifecycle::new(
        Arc::clone(&plugin_registry),
        Arc::clone(&event_bus),
    );
    plugin_lifecycle.wire_subscriptions();

    tracing::debug!("plugin event subscriptions wired");

    // ── Step 11: Lifecycle startup hooks ─────────────────────────────────────
    // Adapters and plugins register startup hooks in their on_load() calls
    // (e.g. "warm the connection pool", "verify the SSH agent is running").
    // These run now, after all plugins are loaded. A startup failure aborts boot.
    lifecycle.startup().await
        .map_err(|e| {
            tracing::error!(error = %e, "lifecycle startup hook failed");
            e
        })?;

    // tracing::info!(
    //     plugins   = plugin_registry.count(),
    //     services  = service_registry.len(),
    //     "Git Manager boot complete"
    // );

    // ── Step 12: Assemble and return the Kernel ───────────────────────────────
    Ok(Arc::new(Kernel {
        plugin_registry,
        command_bus,
        event_bus,
        workflow_runner,
        service_registry,
        lifecycle,
        credential_vault,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tracing initialisation
// ─────────────────────────────────────────────────────────────────────────────

/// Initialises the global tracing subscriber.
///
/// Uses try_init() rather than init() so that this function is safe to call
/// multiple times in the same process — which happens when integration tests
/// each set up their own kernel instance. The `let _ =` suppresses the harmless
/// "already initialised" error on subsequent calls.
///
/// The env_filter parses the level string using RUST_LOG semantics, falling back
/// to a plain "info" level if the string cannot be parsed. This prevents a
/// misconfigured RUST_LOG value from crashing the process at startup.
fn init_tracing(level: &str) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_new(level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).with_thread_ids(false))
        .try_init();
}