// Web interface plugin implementation.
use std::sync::Arc;
use async_trait::async_trait;
use gm_kernel::{
    contracts::plugin::{EventSubscription, Plugin, PluginMetadata},
    contracts::interface::InterfacePlugin,
    kernel::Kernel,
    service_registry::ServiceRegistry,
};
use gm_shared::errors::{GitManagerError, PluginError};

pub struct WebPlugin { metadata: PluginMetadata }

impl WebPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "gm_interface_web".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: "HTTP web interface with SSE for real-time updates".to_string(),
                dependencies: vec![],
                min_kernel_version: "1.0.0".to_string(),
                load_priority: 50,
            },
        }
    }
}

impl Default for WebPlugin { fn default() -> Self { Self::new() } }

impl Plugin for WebPlugin {
    fn metadata(&self) -> &PluginMetadata { &self.metadata }
    fn on_load(&self, _registry: Arc<ServiceRegistry>) -> Result<(), PluginError> {
        tracing::info!("Web interface plugin loaded");
        Ok(())
    }
    fn on_unload(&self) -> Result<(), PluginError> { Ok(()) }
    fn get_event_subscriptions(&self) -> Vec<EventSubscription> { vec![] }
}

#[async_trait]
impl InterfacePlugin for WebPlugin {
    async fn run(&self, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
        let addr = std::env::var("GIT_MANAGER_WEB_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:5008".to_string());
        crate::app::run_server(kernel, &addr).await
    }
}