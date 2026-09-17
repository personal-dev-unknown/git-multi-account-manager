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
    auth_provider::SourceForgeAuthProvider,
    client::SourceForgeClient,
    repository_provider::SourceForgeRepositoryProvider,
};

pub struct SourceForgePlugin {
    metadata: PluginMetadata,
    client:   Arc<SourceForgeClient>,
}

impl SourceForgePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_plugin_sourceforge".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "SourceForge repository and authentication provider".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      10,
            },
            client: Arc::new(SourceForgeClient::new()),
        }
    }
}

impl Default for SourceForgePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for SourceForgePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&self, registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        let creds = registry.get::<CredentialService>().ok_or_else(|| {
            PluginError::RegistrationFailed {
                name:   self.metadata.name.clone(),
                reason: "CredentialService not in registry — register it in bootstrap before loading plugins".to_string(),
            }
        })?;

        let repo_provider = Arc::new(SourceForgeRepositoryProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));
        let auth_provider = Arc::new(SourceForgeAuthProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));

        registry.register::<SourceForgeRepositoryProvider>(repo_provider);
        registry.register::<SourceForgeAuthProvider>(auth_provider);

        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("SourceForge plugin unloaded");
        Ok(())
    }

    fn get_event_subscriptions(&self) -> Vec<EventSubscription> {
        vec![]
    }
}

impl ProviderPlugin for SourceForgePlugin {
    fn platform_type(&self) -> PlatformType {
        PlatformType::SourceForge
    }
}
