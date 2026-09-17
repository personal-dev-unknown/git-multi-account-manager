// crates/gm_kernel/src/plugin_system/registry.rs
//
// The PluginRegistry is the authoritative record of what plugins are currently
// loaded. It is populated during bootstrap and read by the kernel throughout
// the application lifetime. After bootstrap completes, writes happen only when
// a plugin is dynamically unloaded (a future feature).
//
// The registry stores plugins by name (their unique string identifier) rather
// than by TypeId because plugin names are the stable API — the concrete Rust
// types are opaque to the kernel after load time.

use dashmap::DashMap;
use std::sync::Arc;
use gm_shared::errors::PluginError;
use crate::contracts::plugin::Plugin;

#[derive(Default)]
pub struct PluginRegistry {
    plugins: DashMap<String, Arc<dyn Plugin>>,
}

impl std::fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field("plugins", &self.names())
            .finish()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a plugin into the registry.
    /// Returns Err if a plugin with the same name is already registered.
    pub fn insert(&self, plugin: Arc<dyn Plugin>) -> Result<(), PluginError> {
        let name = plugin.metadata().name.clone();
        if self.plugins.contains_key(&name) {
            return Err(PluginError::Duplicate { name });
        }
        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Returns the plugin with the given name, or None if not loaded.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(name).map(|e| Arc::clone(e.value()))
    }

    /// Returns true if a plugin with the given name is loaded.
    pub fn contains(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Returns the number of loaded plugins.
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Returns the names of all loaded plugins, sorted alphabetically.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.plugins.iter()
            .map(|e| e.key().clone())
            .collect();
        names.sort();
        names
    }
}