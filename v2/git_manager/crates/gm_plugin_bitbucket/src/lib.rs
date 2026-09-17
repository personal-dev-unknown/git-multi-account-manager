//! Bitbucket provider plugin for Git Multi-Account Manager.
//!
//! Bitbucket uses app passwords (not OAuth PATs) for API authentication when
//! using HTTP Basic auth. The credentials are stored as "username:app_password"
//! and Base64-encoded for the Authorization header.

pub mod auth;
pub mod auth_provider;
pub mod client;
pub mod plugin;
pub mod repository_provider;

pub use auth_provider::BitbucketAuthProvider;
pub use plugin::BitbucketPlugin;
pub use repository_provider::BitbucketRepositoryProvider;

/// Creates a new, unloaded Bitbucket plugin instance.
pub fn create_plugin() -> BitbucketPlugin {
    BitbucketPlugin::new()
}