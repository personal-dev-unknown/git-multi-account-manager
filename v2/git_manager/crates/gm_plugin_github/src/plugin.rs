// crates/gm_plugin_github/src/plugin.rs
//
// GitHubPlugin is the entry point for the GitHub integration. It implements
// the kernel's Plugin trait (lifecycle management) and ProviderPlugin marker
// trait (declares which platform it serves). During on_load() it creates the
// concrete RepositoryProvider and AuthProvider instances and registers them
// in the service registry so that command handlers can look them up by type.

use std::sync::Arc;

use gm_kernel::{
    contracts::plugin::{EventSubscription, Plugin, PluginMetadata},
    contracts::provider::ProviderPlugin,
    security::CredentialService,
    service_registry::ServiceRegistry,
};
use gm_shared::errors::PluginError;
use gm_shared::models::platform::PlatformType;

use crate::{
    auth_provider::GitHubAuthProvider,
    client::GitHubClient,
    repository_provider::GitHubRepositoryProvider,
};

/// The GitHub platform integration plugin.
/// Registers GitHubRepositoryProvider and GitHubAuthProvider in the
/// service registry during on_load(), making them available to the
/// command handler layer by concrete type lookup.
pub struct GitHubPlugin {
    metadata: PluginMetadata,
    client:   Arc<GitHubClient>,
}

impl GitHubPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_plugin_github".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "GitHub repository and authentication provider".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      10,
            },
            client: Arc::new(GitHubClient::new()),
        }
    }
}

impl Default for GitHubPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for GitHubPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&self, registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        // Fetch the CredentialService that was registered by the binary entry
        // point during bootstrap. This gives the provider a way to look up
        // plaintext PATs for any account UUID at call time.
        let creds = registry.get::<CredentialService>().ok_or_else(|| {
            PluginError::RegistrationFailed {
                name:   self.metadata.name.clone(),
                reason: "CredentialService not in registry — register it in bootstrap before loading plugins".to_string(),
            }
        })?;

        let repo_provider = Arc::new(GitHubRepositoryProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));
        let auth_provider = Arc::new(GitHubAuthProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));

        // Register under concrete types so command handlers can look them up.
        registry.register::<GitHubRepositoryProvider>(repo_provider);
        registry.register::<GitHubAuthProvider>(auth_provider);

        // tracing::info!("GitHub plugin loaded — RepositoryProvider and AuthProvider registered");
        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("GitHub plugin unloaded");
        Ok(())
    }

    fn get_event_subscriptions(&self) -> Vec<EventSubscription> {
        // GitHub plugin has no event subscriptions in v1.
        // A future version could subscribe to AccountAdded events to
        // auto-detect GitHub accounts and validate their credentials.
        vec![]
    }
}

impl ProviderPlugin for GitHubPlugin {
    fn platform_type(&self) -> PlatformType {
        PlatformType::GitHub
    }
}