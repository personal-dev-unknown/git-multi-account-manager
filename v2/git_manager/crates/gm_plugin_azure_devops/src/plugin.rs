// crates/gm_plugin_azure_devops/src/plugin.rs
//
// AzureDevOpsPlugin — kernel plugin entry point for the Azure DevOps integration.
// Follows the identical pattern to GitHubPlugin, GitLabPlugin, and BitbucketPlugin.

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
    auth_provider::AzureDevOpsAuthProvider,
    client::AzureDevOpsClient,
    repository_provider::AzureDevOpsRepositoryProvider,
};

/// The Azure DevOps platform integration plugin.
///
/// Registers `AzureDevOpsRepositoryProvider` and `AzureDevOpsAuthProvider` in the
/// service registry during `on_load()`.
pub struct AzureDevOpsPlugin {
    metadata: PluginMetadata,
    client:   Arc<AzureDevOpsClient>,
}

impl AzureDevOpsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_plugin_azure_devops".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "Azure DevOps repository and authentication provider".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      10,
            },
            client: Arc::new(AzureDevOpsClient::new()),
        }
    }
}

impl Default for AzureDevOpsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AzureDevOpsPlugin {
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

        let repo_provider = Arc::new(AzureDevOpsRepositoryProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));
        let auth_provider = Arc::new(AzureDevOpsAuthProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));

        registry.register::<AzureDevOpsRepositoryProvider>(repo_provider);
        registry.register::<AzureDevOpsAuthProvider>(auth_provider);

        // tracing::info!("Azure DevOps plugin loaded — RepositoryProvider and AuthProvider registered");
        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("Azure DevOps plugin unloaded");
        Ok(())
    }

    fn get_event_subscriptions(&self) -> Vec<EventSubscription> {
        vec![]
    }
}

impl ProviderPlugin for AzureDevOpsPlugin {
    fn platform_type(&self) -> PlatformType {
        PlatformType::AzureDevOps
    }
}