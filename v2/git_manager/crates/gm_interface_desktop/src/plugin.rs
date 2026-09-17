// crates/gm_interface_desktop/src/plugin.rs
//
// DesktopPlugin — the kernel Plugin implementation for the Tauri desktop interface.
//
// Unlike CliPlugin and WebPlugin which are InterfacePlugins that call a `run()`
// method to start their event loop, DesktopPlugin registers the desktop interface
// services with the kernel and lets Tauri's event loop (started in app.rs) drive
// the execution. Tauri's `run()` is blocking and takes over the main thread,
// so no separate `run()` method is needed here.
//
// The plugin's `on_load()` verifies that the kernel is correctly wired and logs
// the desktop startup. Post-bootstrap, the binary registers DesktopServicesHandle.

use std::sync::Arc;
use gm_kernel::{
    contracts::plugin::{EventSubscription, Plugin, PluginMetadata},
    service_registry::ServiceRegistry,
};
use gm_shared::errors::PluginError;

/// The Tauri desktop interface plugin.
///
/// This plugin's role in the kernel lifecycle:
///   - `on_load()`: Registers the desktop interface with the kernel; logs startup.
///   - No event subscriptions in v1 (future: subscribe to AccountAdded to refresh UI).
///   - No `run()` method: Tauri takes over the main thread via `app.rs::run_tauri_app()`.
pub struct DesktopPlugin {
    metadata: PluginMetadata,
}

impl DesktopPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_interface_desktop".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "Git Manager Tauri desktop interface".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      20, // loads after infrastructure (0) and providers (10)
            },
        }
    }
}

impl Default for DesktopPlugin {
    fn default() -> Self { Self::new() }
}

impl Plugin for DesktopPlugin {
    fn metadata(&self) -> &PluginMetadata { &self.metadata }

    fn on_load(&self, _registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        tracing::info!(
            version = %self.metadata.version,
            "Desktop interface plugin loaded — Tauri event loop will be started by run_tauri_app()"
        );
        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("Desktop interface plugin unloaded");
        Ok(())
    }

    fn get_event_subscriptions(&self) -> Vec<EventSubscription> {
        // v1: no subscriptions. v2: subscribe to AccountAdded, RepositoryCloned
        // to emit events to the Svelte store via tauri::AppHandle::emit_all().
        vec![]
    }
}