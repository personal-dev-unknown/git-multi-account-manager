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
    auth_provider::CloudStorageAuthProvider,
    repository_provider::CloudStorageRepositoryProvider,
};

pub struct CloudStoragePlugin {
    metadata: PluginMetadata,
}

impl CloudStoragePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_plugin_cloud_storage".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "Cloud Storage platform provider (S3, GCS, Azure Blob)".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      10,
            },
        }
    }
}

impl Default for CloudStoragePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CloudStoragePlugin {
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

        let repo_provider = Arc::new(CloudStorageRepositoryProvider::new(Arc::clone(&creds)));
        let auth_provider = Arc::new(CloudStorageAuthProvider::new(Arc::clone(&creds)));

        registry.register::<CloudStorageRepositoryProvider>(repo_provider);
        registry.register::<CloudStorageAuthProvider>(auth_provider);

        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("Cloud Storage plugin unloaded");
        Ok(())
    }

    fn get_event_subscriptions(&self) -> Vec<EventSubscription> {
        vec![]
    }
}

impl ProviderPlugin for CloudStoragePlugin {
    fn platform_type(&self) -> PlatformType {
        PlatformType::CloudStorage("generic".to_string())
    }
}
