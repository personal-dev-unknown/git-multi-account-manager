//! Plugin lifecycle error variants.

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum PluginError {
    /// The plugin's .so file could not be loaded by libloading, or the
    /// create_plugin() C ABI symbol was not found in the loaded library.
    #[error("Failed to load plugin '{name}': {reason}")]
    LoadFailed { name: String, reason: String },

    /// The loaded plugin does not implement a required method from the Plugin
    /// contract. This typically means the plugin was compiled against an older
    /// kernel API version and is missing a method added in a later version.
    #[error("Plugin '{name}' does not implement required method: '{missing_method}'")]
    ContractViolation { name: String, missing_method: String },

    /// The plugin declared a dependency on another plugin that has not been
    /// loaded. Dependencies are resolved using a topological sort — if a cycle
    /// or missing dependency is found, boot fails with this error.
    #[error("Plugin '{plugin}' requires '{required}' which is not loaded")]
    DependencyMissing { plugin: String, required: String },

    /// The plugin declared that it requires a minimum kernel version that is
    /// higher than the running kernel version. The user needs to upgrade the
    /// application before the plugin can be used.
    #[error("Plugin '{plugin}' requires kernel ≥{required}, running {actual}")]
    VersionIncompatible { plugin: String, required: String, actual: String },

    /// The plugin's on_load() hook returned an error when trying to register
    /// its services with the service registry.
    #[error("Failed to register services for plugin '{name}': {reason}")]
    RegistrationFailed { name: String, reason: String },

    /// Two plugins with the same name were loaded. Plugin names must be
    /// globally unique within a running instance.
    #[error("Plugin '{name}' is already registered — plugin names must be unique")]
    Duplicate { name: String },

    /// A circular dependency was detected in the plugin dependency graph.
    /// The kernel refuses to boot rather than enter an undefined loading order.
    /// The `cycle` field contains the plugin names forming the cycle.
    #[error("Circular plugin dependency: {}", cycle.join(" → "))]
    CircularDependency { cycle: Vec<String> },
}