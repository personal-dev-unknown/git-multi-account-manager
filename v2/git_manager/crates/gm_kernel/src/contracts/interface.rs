// crates/gm_kernel/src/contracts/interface.rs
//
// The InterfacePlugin trait marks a plugin that presents the application to
// a user. Interface plugins own the application's I/O surface: the CLI renders
// output to the terminal, the web plugin runs an HTTP server, the desktop
// plugin starts a Tauri window. The kernel boots and then calls run() on all
// loaded interface plugins, which enter their own event loops.
//
// An interface plugin is always the last plugin type to start (after all
// provider plugins have loaded) because it depends on the service registry
// being fully populated before it can dispatch user commands.

use std::sync::Arc;
use async_trait::async_trait;
use gm_shared::errors::GitManagerError;

/// Marker trait for plugins that provide a user interface.
/// In addition to the Plugin lifecycle, interface plugins expose a run() method
/// that takes control of the process until the user exits.
#[async_trait]
pub trait InterfacePlugin: crate::contracts::plugin::Plugin {
    /// Enters the interface's main loop. This method should not return until
    /// the user requests shutdown (Ctrl+C in the CLI, browser window close, etc.).
    ///
    /// The kernel passes itself as an Arc so the interface can dispatch commands
    /// and query services throughout its lifetime.
    async fn run(&self, kernel: Arc<crate::kernel::Kernel>) -> Result<(), GitManagerError>;
}