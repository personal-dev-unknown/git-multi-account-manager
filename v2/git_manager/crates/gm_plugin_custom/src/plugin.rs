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
    auth_provider::CustomAuthProvider,
    client::CustomClient,
    repository_provider::CustomRepositoryProvider,
};

pub struct CustomPlugin {
    metadata: PluginMetadata,
    client:   Arc<CustomClient>,
}

impl CustomPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_plugin_custom".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "Custom/Self-Hosted Git platform provider".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      10,
            },
            client: Arc::new(CustomClient::new()),
        }
    }
}

impl Default for CustomPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CustomPlugin {
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

        let repo_provider = Arc::new(CustomRepositoryProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));
        let auth_provider = Arc::new(CustomAuthProvider::new(
            Arc::clone(&self.client),
            Arc::clone(&creds),
        ));

        registry.register::<CustomRepositoryProvider>(repo_provider);
        registry.register::<CustomAuthProvider>(auth_provider);

        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("Custom plugin unloaded");
        Ok(())
    }

    fn get_event_subscriptions(&self) -> Vec<EventSubscription> {
        vec![]
    }
}

impl ProviderPlugin for CustomPlugin {
    fn platform_type(&self) -> PlatformType {
        // Return SelfHosted variant with empty default that gets
        // replaced by the specific host during account configuration.
        PlatformType::SelfHosted("custom".to_string())
    }
}
