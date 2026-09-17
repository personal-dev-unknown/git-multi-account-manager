use std::sync::Arc;
use gm_kernel::{
    contracts::plugin::{EventSubscription, Plugin, PluginMetadata},
    security::CredentialService,
    service_registry::ServiceRegistry,
};
use gm_shared::errors::PluginError;

pub struct InfrastructurePlugin {
    metadata:     PluginMetadata,
    cred_service: Arc<CredentialService>,
}

impl InfrastructurePlugin {
    pub fn new(cred_service: Arc<CredentialService>) -> Self {
        Self {
            metadata: PluginMetadata {
                name:               "gm_infrastructure".to_string(),
                version:            env!("CARGO_PKG_VERSION").to_string(),
                description:        "Registers kernel infrastructure services before plugins load".to_string(),
                dependencies:       vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority:      0,
            },
            cred_service,
        }
    }
}

impl Plugin for InfrastructurePlugin {
    fn metadata(&self) -> &PluginMetadata { &self.metadata }

    fn on_load(&self, registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        registry.register::<CredentialService>(Arc::clone(&self.cred_service));
        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> { Ok(()) }
    fn get_event_subscriptions(&self) -> Vec<EventSubscription> { vec![] }
}
