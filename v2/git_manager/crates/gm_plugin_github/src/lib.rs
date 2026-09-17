//! GitHub provider plugin for Git Multi-Account Manager.
//!
//! Exports the plugin factory function `create_plugin()` that the binary
//! entry point calls when building the static plugin list for bootstrap.

pub mod auth;
pub mod auth_provider;
pub mod client;
pub mod plugin;
pub mod repository_provider;

pub use auth_provider::GitHubAuthProvider;
pub use plugin::GitHubPlugin;
pub use repository_provider::GitHubRepositoryProvider;

/// Creates a new, unloaded GitHub plugin instance.
/// The binary entry point boxes this and passes it to `bootstrap()`.
pub fn create_plugin() -> GitHubPlugin {
    GitHubPlugin::new()
}