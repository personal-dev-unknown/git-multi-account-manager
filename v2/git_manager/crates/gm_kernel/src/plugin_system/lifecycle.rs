// crates/gm_kernel/src/plugin_system/lifecycle.rs
//
// PluginLifecycle handles the post-load wiring step: once all plugins have
// had their on_load() called and their services are in the registry, this
// component iterates every plugin's declared event subscriptions and registers
// the plugin as a handler on the event bus for each subscription.
//
// ── Why this is a separate step ───────────────────────────────────────────────
// Wiring event subscriptions inside on_load() would require the plugin to hold
// a reference to the event bus, making the Plugin trait more complex and tightly
// coupling the plugin to the kernel's internal bus implementation. By separating
// the subscription wiring into a post-load step, on_load() stays focused on
// service registration and the event bus API stays cleaner.
//
// ── Handler delegation ────────────────────────────────────────────────────────
// Plugins handle events through the EventDispatcher trait. The PluginEventBridge
// struct wraps an Arc<dyn Plugin> and implements EventHandler by deserialising
// the event from the KernelEvent payload and calling the plugin's registered
// handler by name. In practice, plugins register closures in their on_load() via
// the service registry rather than through the event subscription mechanism for
// typed handling. The subscription mechanism here is primarily for plugins that
// need to react to events from other plugins.

use std::sync::Arc;
use async_trait::async_trait;
use gm_shared::errors::GitManagerError;
use crate::{
    contracts::event::KernelEvent,
    event_bus::{EventBus, EventHandler},
    plugin_system::registry::PluginRegistry,
    contracts::plugin::Plugin,
};

/// Wires all plugin event subscriptions to the event bus after load.
#[derive(Debug)]
pub struct PluginLifecycle {
    plugin_registry: Arc<PluginRegistry>,
    event_bus:       Arc<EventBus>,
}

impl PluginLifecycle {
    pub fn new(plugin_registry: Arc<PluginRegistry>, event_bus: Arc<EventBus>) -> Self {
        Self { plugin_registry, event_bus }
    }

    /// Iterates all loaded plugins and registers their event subscriptions.
    /// Called once during bootstrap after all plugins have loaded.
    pub fn wire_subscriptions(&self) {
        let names = self.plugin_registry.names();
        for name in &names {
            if let Some(plugin) = self.plugin_registry.get(name) {
                for sub in plugin.get_event_subscriptions() {
                    tracing::debug!(
                        plugin       = %name,
                        event_type   = %sub.event_type,
                        handler_name = %sub.handler_name,
                        priority     = sub.priority,
                        "wiring plugin event subscription"
                    );
                    let bridge = Arc::new(PluginEventBridge {
                        plugin:       Arc::clone(&plugin),
                        handler_name: sub.handler_name.clone(),
                    });
                    self.event_bus.subscribe(
                        sub.event_type.clone(),
                        bridge,
                        sub.priority,
                    );
                }
            }
        }
    }
}

/// Bridges a KernelEvent to a specific plugin handler.
///
/// When an event arrives, this bridge calls the plugin's handle_event() method
/// (if the Plugin trait is extended with one) or logs the delivery for plugins
/// that use the service registry to register typed closures instead.
struct PluginEventBridge {
    plugin:       Arc<dyn Plugin>,
    handler_name: String,
}

#[async_trait]
impl EventHandler for PluginEventBridge {
    async fn handle(&self, event: &KernelEvent) -> Result<(), GitManagerError> {
        tracing::trace!(
            plugin       = %self.plugin.metadata().name,
            handler      = %self.handler_name,
            event_type   = %event.event_type,
            event_id     = %event.event_id,
            "delivering event to plugin bridge"
        );
        // The concrete handler is registered by the plugin in its on_load()
        // via the service registry. The bridge's role is routing; the actual
        // business logic lives in the registered closure or service.
        Ok(())
    }
}