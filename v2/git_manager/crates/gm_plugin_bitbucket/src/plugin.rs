// crates/gm_plugin_bitbucket/src/plugin.rs
//
// BitbucketPlugin — kernel plugin entry point for the Bitbucket integration.
// Follows the identical pattern to GitHubPlugin and GitLabPlugin.

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
    auth_provider::BitbucketAuthProvider,
    client::BitbucketClient,
    repository_provider::BitbucketRepositoryProvider,
};

/// The Bitbucket platform integration plugin.
pub struct BitbucketPlugin {
    metadata: PluginMetadata,
    client:   Arc<BitbucketClient>,
}

impl BitbucketPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_plugin_bitbucket".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "Bitbucket repository and authentication provider".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      10,
            },
            client: Arc::new(BitbucketClient::new()),
        }
    }
}

impl Default for BitbucketPlugin {
    fn default() -> Self { Self::new() }
}

impl Plugin for BitbucketPlugin {
    fn metadata(&self) -> &PluginMetadata { &self.metadata }

    fn on_load(&self, registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        let creds = registry.get::<CredentialService>().ok_or_else(|| {
            PluginError::RegistrationFailed {
                name:   self.metadata.name.clone(),
                reason: "CredentialService not in registry — InfrastructurePlugin must load first (priority 0)".to_string(),
            }
        })?;

        let repo_provider = Arc::new(BitbucketRepositoryProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));
        let auth_provider = Arc::new(BitbucketAuthProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));

        registry.register::<BitbucketRepositoryProvider>(repo_provider);
        registry.register::<BitbucketAuthProvider>(auth_provider);

        // tracing::info!("Bitbucket plugin loaded — RepositoryProvider and AuthProvider registered");
        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("Bitbucket plugin unloaded");
        Ok(())
    }

    fn get_event_subscriptions(&self) -> Vec<EventSubscription> { vec![] }
}

impl ProviderPlugin for BitbucketPlugin {
    fn platform_type(&self) -> PlatformType { PlatformType::Bitbucket }
}