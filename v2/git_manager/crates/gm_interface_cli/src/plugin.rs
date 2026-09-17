// crates/gm_interface_cli/src/plugin.rs
//
// CliPlugin is the kernel entry point for the terminal interface. The binary
// entry point (apps/cli/main.rs) instantiates it with `CliPlugin::new()`,
// boxes it, and passes it to `bootstrap()`. The kernel then calls:
//
//   1. `metadata()` — to log the CLI plugin loaded
//   2. `on_load(registry)` — to let the CLI register anything it needs
//   3. `run(kernel)` — after all provider plugins are loaded; this is where
//      the Clap parser takes over and the terminal session begins
//
// The CLI plugin's on_load() is minimal because it does not provide services
// to other plugins — it consumes services from the registry. The only thing
// it does in on_load() is log that it is present so that diagnostics show the
// CLI interface in the loaded-plugins list.
//
// ── Blocking vs async ─────────────────────────────────────────────────────────
// Clap's argument parsing is synchronous, but command handlers are async because
// they call database repositories and platform APIs. The run() method is declared
// `async` (required by InterfacePlugin), and each handler is awaited normally
// inside the tokio runtime the binary entry point started.
//
// Interactive TUI sessions (ratatui-based views) use `spawn_blocking` so that
// the blocking crossterm event loop does not starve the tokio scheduler.

use std::sync::Arc;
use async_trait::async_trait;

use gm_kernel::{
    contracts::plugin::{EventSubscription, Plugin, PluginMetadata},
    contracts::interface::InterfacePlugin,
    kernel::Kernel,
    service_registry::ServiceRegistry,
};
use gm_shared::{
    constants::events::*,
    errors::{GitManagerError, PluginError},
};

use crate::app::Cli;

pub struct CliPlugin {
    metadata: PluginMetadata,
}

impl CliPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_interface_cli".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "Terminal interface: full-featured CLI with TUI elements".to_string(),
                // The CLI depends on all provider plugins being loaded first
                // so it can list available platforms when adding an account.
                // Provider plugins have load_priority = 10; CLI has 50.
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      50,
            },
        }
    }
}

impl Default for CliPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CliPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&self, _registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        // The CLI plugin does not register any services for other plugins to use.
        // Its only contribution is the user-facing interface provided by run().
        tracing::info!("CLI interface plugin loaded");
        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("CLI interface plugin unloaded");
        Ok(())
    }

    fn get_event_subscriptions(&self) -> Vec<EventSubscription> {
        // Every domain event the CLI wants terminal notifications for.
        // The kernel's PluginLifecycle uses this list for diagnostics/logging;
        // the actual handler is registered via subscribe_all() in run().
        let sub = |event_type: &'static str, handler_name: &'static str| EventSubscription {
            event_type:   event_type.to_string(),
            handler_name: handler_name.to_string(),
            is_async:     true,
            priority:     10,
        };
        vec![
            // Account
            sub(ACCOUNT_ADDED,           "on_account_added"),
            sub(ACCOUNT_REMOVED,         "on_account_removed"),
            sub(ACCOUNT_STATUS_CHANGED,  "on_account_status_changed"),
            // SSH keys
            sub(SSH_KEY_GENERATED,       "on_ssh_key_generated"),
            sub(SSH_KEY_TESTED,          "on_ssh_key_tested"),
            sub(SSH_KEY_ADDED_TO_AGENT,  "on_ssh_key_added_to_agent"),
            // Repositories
            sub(REPOSITORY_DISCOVERED,  "on_repository_discovered"),
            sub(REPOSITORY_CLONED,      "on_repository_cloned"),
            sub(REPOSITORY_SYNCED,      "on_repository_synced"),
            // Git operations
            sub(COMMIT_CREATED,            "on_commit_created"),
            sub(BRANCH_CREATED,            "on_branch_created"),
            sub(MERGE_CONFLICT_DETECTED,   "on_merge_conflict_detected"),
            // Sync sessions
            sub(SYNC_STARTED,    "on_sync_started"),
            sub(SYNC_COMPLETED,  "on_sync_completed"),
            sub(CONFLICT_DETECTED, "on_conflict_detected"),
            // Workflows
            sub(WORKFLOW_STARTED,   "on_workflow_started"),
            sub(WORKFLOW_COMPLETED, "on_workflow_completed"),
            sub(WORKFLOW_FAILED,    "on_workflow_failed"),
        ]
    }
}

#[async_trait]
impl InterfacePlugin for CliPlugin {
    /// Parse command-line arguments and dispatch to the appropriate handler.
    ///
    /// This is called by the kernel after every provider plugin has been loaded
    /// and the service registry is fully populated. From here the CLI takes
    /// exclusive ownership of execution until the command completes (or the
    /// user requests an interactive session exit).
    async fn run(&self, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
        use clap::Parser;
        use crate::ui::theme_engine::ThemeRegistry;

        // ── Initialise the theme engine ──────────────────────────────────────
        let theme_registry = Arc::new(std::sync::Mutex::new(ThemeRegistry::new()));
        {
            let mut lock = theme_registry.lock()
                .expect("cli plugin: theme_registry Mutex poisoned");
            lock.load_state();
        }
        kernel.register::<std::sync::Mutex<ThemeRegistry>>(Arc::clone(&theme_registry));
        crate::ui::colors::init_theme(Arc::clone(&theme_registry));

        // Register the terminal notification handler for all domain events.
        // Must happen before any command is dispatched so that events fired
        // during the first command are not missed.
        kernel.event_bus.subscribe_all(
            crate::ui::event_handler::CliEventHandler::new()
        );

        // Detect whether a real subcommand was provided.
        // Flags like --verbose and --no-banner do not count as subcommands.
        let raw: Vec<String>  = std::env::args().collect();
        let has_subcommand    = raw.iter().skip(1).any(|a| !a.starts_with('-'));

        // When run on a real terminal with no subcommand, ask which mode to use.
        // Interactive (index 0) is the default — pressing Enter immediately
        // launches the menu without the user needing to type anything.
        if !has_subcommand && atty::is(atty::Stream::Stdin) {
            let mode = dialoguer::Select::new()
                .with_prompt("Select mode  (↑↓ arrows, Enter to confirm)")
                .items(&[
                    "Interactive  — menu-driven (default, recommended)",
                    "Subcommand   — git-zyrix account add --alias work ...",
                ])
                .default(0)
                .interact()
                .unwrap_or(0); // default to interactive if the prompt fails

            if mode == 0 {
                let handle = kernel
                    .get::<crate::services::CliServicesHandle>()
                    .ok_or_else(|| GitManagerError::Other(
                        "CliServicesHandle not registered — check apps/cli/main.rs bootstrap"
                            .to_string(),
                    ))?;
                return crate::ui::interactive::run_menu(handle).await;
            } else {
                // mode == 1: persistent subcommand REPL shell.
                return crate::ui::shell::run_shell(kernel).await;
            }
        }

        let cli = Cli::parse();
        crate::app::dispatch(cli, kernel).await
    }
}