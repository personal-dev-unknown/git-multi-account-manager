// crates/gm_plugin_gitlab/src/plugin.rs
//
// GitLabPlugin is the entry point for the GitLab integration. It implements the
// kernel's Plugin trait (lifecycle management) and ProviderPlugin marker trait
// (declares which platform it serves). During on_load() it creates the concrete
// RepositoryProvider and AuthProvider instances and registers them in the service
// registry so that command handlers can look them up by type.

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
    auth_provider::GitLabAuthProvider,
    client::GitLabClient,
    repository_provider::GitLabRepositoryProvider,
};

/// The GitLab platform integration plugin.
///
/// Registers `GitLabRepositoryProvider` and `GitLabAuthProvider` in the service
/// registry during `on_load()`, making them available to the command handler layer
/// by concrete-type lookup via `kernel.get::<GitLabRepositoryProvider>()`.
pub struct GitLabPlugin {
    metadata: PluginMetadata,
    client:   Arc<GitLabClient>,
}

impl GitLabPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_plugin_gitlab".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "GitLab repository and authentication provider".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      10,
            },
            client: Arc::new(GitLabClient::new()),
        }
    }
}

impl Default for GitLabPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for GitLabPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&self, registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        let creds = registry.get::<CredentialService>().ok_or_else(|| {
            PluginError::RegistrationFailed {
                name:   self.metadata.name.clone(),
                reason: "CredentialService not in registry — InfrastructurePlugin must load before provider plugins".to_string(),
            }
        })?;

        let repo_provider = Arc::new(GitLabRepositoryProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));
        let auth_provider = Arc::new(GitLabAuthProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));

        registry.register::<GitLabRepositoryProvider>(repo_provider);
        registry.register::<GitLabAuthProvider>(auth_provider);

        // tracing::info!("GitLab plugin loaded — RepositoryProvider and AuthProvider registered");
        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("GitLab plugin unloaded");
        Ok(())
    }

    fn get_event_subscriptions(&self) -> Vec<EventSubscription> {
        vec![]
    }
}

impl ProviderPlugin for GitLabPlugin {
    fn platform_type(&self) -> PlatformType {
        PlatformType::GitLab
    }
}
