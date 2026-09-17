//! Azure DevOps provider plugin for Git Multi-Account Manager.
//!
//! Azure DevOps uses Personal Access Tokens (PATs) for API authentication.
//! PATs are passed via HTTP Basic auth with an empty username and the PAT as
//! the password (the standard ADO authentication pattern for REST APIs).
//!
//! Repository URLs follow the format:
//!   SSH:   git@ssh.dev.azure.com:v3/{organization}/{project}/{repo}
//!   HTTPS: https://dev.azure.com/{organization}/{project}/_git/{repo}

pub mod auth;
pub mod auth_provider;
pub mod client;
pub mod plugin;
pub mod repository_provider;

pub use auth_provider::AzureDevOpsAuthProvider;
pub use plugin::AzureDevOpsPlugin;
pub use repository_provider::AzureDevOpsRepositoryProvider;

/// Creates a new, unloaded Azure DevOps plugin instance.
pub fn create_plugin() -> AzureDevOpsPlugin {
    AzureDevOpsPlugin::new()
}