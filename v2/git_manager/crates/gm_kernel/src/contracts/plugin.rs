// crates/gm_kernel/src/contracts/plugin.rs
//
// The Plugin trait is the fundamental contract every extension must implement.
// It declares the plugin's identity (name, version, dependencies), its lifecycle
// hooks (on_load, on_unload), and what it contributes to the running system
// (registered services, event subscriptions).
//
// ── Why a trait and not a struct ──────────────────────────────────────────────
// A Plugin is behaviour, not data. The concrete implementation varies wildly:
// a GitHub plugin makes HTTP calls, a CLI plugin renders terminal output, a
// workflow plugin registers step handlers. All of them share the same lifecycle
// contract, which is why a trait models the relationship correctly. The
// concrete types are opaque to the kernel — it only ever holds `Arc<dyn Plugin>`.
//
// ── Thread safety ─────────────────────────────────────────────────────────────
// `Send + Sync` bounds are required because:
//   1. The kernel stores plugins in a DashMap, which may be accessed from
//      multiple async tasks simultaneously.
//   2. Plugin lifecycle methods (on_load, on_unload) are called from the tokio
//      runtime's thread pool.
//   3. Services registered by plugins are also stored behind Arc, requiring the
//      underlying data to be Sync.

use std::sync::Arc;
use gm_shared::errors::PluginError;

/// Immutable metadata declared by a plugin at load time.
/// The kernel validates this metadata before calling on_load().
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Globally unique identifier: "gm_plugin_github", "gm_interface_cli", etc.
    /// Snake-case, all lowercase, no spaces.
    pub name:              String,
    /// Semantic version string: "1.0.0"
    pub version:           String,
    /// Human-readable description shown in diagnostic output.
    pub description:       String,
    /// Names of other plugins that must be loaded before this one.
    /// The kernel performs a topological sort and loads dependencies first.
    pub dependencies:      Vec<String>,
    /// The minimum kernel version this plugin requires. The kernel rejects
    /// plugins that declare a minimum version higher than the running version.
    pub min_kernel_version: String,
    /// Load priority within the same dependency tier. Higher numbers load first.
    /// Used to ensure, for example, that auth plugins load before repository
    /// plugins that depend on authentication.
    pub load_priority:     u16,
}

/// Describes one event subscription that the plugin wants to receive.
#[derive(Debug, Clone)]
pub struct EventSubscription {
    /// The event type string, e.g. "RepositoryCloned". Must match one of the
    /// constants in `gm_shared::constants::events`.
    pub event_type:    String,
    /// The name of the handler method to invoke (used for logging/diagnostics).
    pub handler_name:  String,
    /// Whether the handler should be invoked concurrently with other handlers
    /// for the same event (async) or must complete before the next handler runs.
    pub is_async:      bool,
    /// Priority — higher numbers are called first for the same event type.
    pub priority:      u16,
}

/// The core plugin contract.
///
/// Every plugin in the system — provider plugins, interface plugins, workflow
/// plugins — implements this trait. The kernel uses it as the universal handle
/// for lifecycle management, never touching the concrete types.
pub trait Plugin: Send + Sync + 'static {
    /// Returns the plugin's static metadata.
    /// Called before on_load() to validate compatibility and sort the load order.
    fn metadata(&self) -> &PluginMetadata;

    /// Called once when the plugin is loaded, in dependency order.
    /// The plugin should use this opportunity to:
    ///   - Register its services in the service registry
    ///   - Validate that its dependencies have been loaded
    ///   - Initialise any internal state
    ///   - Return Err if any required precondition is not met
    ///
    /// The kernel does NOT retry failed on_load() calls. A failure here aborts
    /// the boot sequence for the entire application.
    fn on_load(&self, registry: Arc<crate::service_registry::ServiceRegistry>) -> Result<(), PluginError>;

    /// Called in reverse dependency order when the application shuts down, or
    /// when the plugin is dynamically unloaded. The plugin should release all
    /// resources it holds (close HTTP connections, drain queues, etc.).
    fn on_unload(&self) -> Result<(), PluginError>;

    /// Returns the event subscriptions this plugin wants the event bus to wire.
    /// Called after on_load() succeeds. The kernel iterates these and registers
    /// the plugin as a handler for each listed event type.
    fn get_event_subscriptions(&self) -> Vec<EventSubscription>;
}