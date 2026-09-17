//! GitLab provider plugin for Git Multi-Account Manager.
//!
//! Exports the plugin factory function `create_plugin()` that the binary
//! entry point calls when building the static plugin list for bootstrap.
//! The binary passes this into `bootstrap(static_plugins, ...)` so the kernel
//! can call `on_load()` in topological dependency order.
//!
//! # Architecture
//! This plugin follows the exact pattern established by `gm_plugin_github`:
//! - `client.rs`              → GitLab REST API v4 HTTP client (stateless, token-per-call)
//! - `auth.rs`                → GitLab-specific authentication helpers
//! - `auth_provider.rs`       → `AuthProvider` trait implementation
//! - `repository_provider.rs` → `RepositoryProvider` trait implementation
//! - `plugin.rs`              → `Plugin` + `ProviderPlugin` trait implementations

pub mod auth;
pub mod auth_provider;
pub mod client;
pub mod plugin;
pub mod repository_provider;

pub use auth_provider::GitLabAuthProvider;
pub use plugin::GitLabPlugin;
pub use repository_provider::GitLabRepositoryProvider;

/// Creates a new, unloaded GitLab plugin instance.
/// The binary entry point boxes this and passes it to `bootstrap()`.
pub fn create_plugin() -> GitLabPlugin {
    GitLabPlugin::new()
}