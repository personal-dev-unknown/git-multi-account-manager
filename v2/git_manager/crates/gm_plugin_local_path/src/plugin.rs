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
    auth_provider::LocalPathAuthProvider,
    repository_provider::LocalPathRepositoryProvider,
};

pub struct LocalPathPlugin {
    metadata: PluginMetadata,
}

impl LocalPathPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_plugin_local_path".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "Local Path platform provider for filesystem repositories".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      10,
            },
        }
    }
}

impl Default for LocalPathPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for LocalPathPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&self, registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        let creds = registry.get::<CredentialService>().ok_or_else(|| {
            PluginError::RegistrationFailed {
                name:   self.metadata.name.clone(),
                reason: "CredentialService not in registry".to_string(),
            }
        })?;

        let repo_provider = Arc::new(LocalPathRepositoryProvider::new(Arc::clone(&creds)));
        let auth_provider = Arc::new(LocalPathAuthProvider::new(Arc::clone(&creds)));

        registry.register::<LocalPathRepositoryProvider>(repo_provider);
        registry.register::<LocalPathAuthProvider>(auth_provider);

        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("Local Path plugin unloaded");
        Ok(())
    }

    fn get_event_subscriptions(&self) -> Vec<EventSubscription> {
        vec![]
    }
}

impl ProviderPlugin for LocalPathPlugin {
    fn platform_type(&self) -> PlatformType {
        PlatformType::LocalPath
    }
}
