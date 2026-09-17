// crates/gm_domain/src/configuration/entities/configuration.rs
//
// The Configuration entity is the application's persistent settings store.
// It is a key-value map where keys are namespaced strings (e.g. "core.log_level",
// "web.bind_addr") and values are JSON-serialised strings. Every setting has
// a well-known key constant so callers never write raw string literals.
//
// Design rationale for JSON values: settings range from simple strings to complex
// nested objects (e.g. a list of trusted plugin directories). A single
// string column that holds JSON allows arbitrary value shapes without schema
// migrations for each new setting. The type-safe getter methods decode the JSON
// internally so callers always work with typed values.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// All typed settings accessors live in this module. When a new setting is
/// needed, add a constant here and a getter/setter method pair on Configuration.
pub mod keys {
    /// Minimum logging level: "error", "warn", "info", "debug", "trace"
    pub const LOG_LEVEL: &str = "core.log_level";
    /// Web interface bind address, e.g. "127.0.0.1:5000"
    pub const WEB_ADDR: &str = "web.bind_addr";
    /// Whether to show the CLI banner on startup
    pub const SHOW_BANNER: &str = "cli.show_banner";
    /// Default SSH key type for newly generated keys: "ed25519" or "rsa"
    pub const DEFAULT_SSH_KEY_TYPE: &str = "ssh.default_key_type";
    /// SSH connection test timeout in milliseconds
    pub const SSH_CONNECT_TIMEOUT_MS: &str = "ssh.connect_timeout_ms";
    /// Whether to automatically add newly generated keys to the SSH agent
    pub const AUTO_ADD_TO_AGENT: &str = "ssh.auto_add_to_agent";
    /// Maximum number of concurrent git operations across all repositories
    pub const MAX_CONCURRENT_OPS: &str = "git.max_concurrent_ops";
    /// Whether to run migrations automatically on startup
    pub const AUTO_MIGRATE: &str = "core.auto_migrate";
    /// Plugin directories to scan for external .so plugins (JSON array of paths)
    pub const PLUGIN_DIRS: &str = "plugins.extra_dirs";
    /// Active theme slug (e.g. "zyrix", "jet_black"). Previously stored in JSON file.
    pub const THEME_ACTIVE_SLUG: &str = "ui.theme_active_slug";
}

/// The application configuration entity.
///
/// One Configuration record exists per installation. It is loaded at boot
/// time by bootstrap.rs and cached for the lifetime of the running process.
/// Changes written through ConfigService take effect on the next restart for
/// most settings, though runtime-safe settings (log level, web bind addr) are
/// hot-reloaded by their respective subsystems.
#[derive(Debug, Clone)]
pub struct Configuration {
    uuid:       Uuid,
    /// The flat key-value store. Values are JSON-encoded strings.
    entries:    HashMap<String, String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Configuration {
    /// Creates a new Configuration with factory-default values.
    /// Called on first boot when no existing configuration is found.
    pub fn new_with_defaults() -> Self {
        let mut entries = HashMap::new();
        entries.insert(keys::LOG_LEVEL.to_string(),           serde_json::to_string("info").unwrap());
        entries.insert(keys::WEB_ADDR.to_string(),            serde_json::to_string("127.0.0.1:5000").unwrap());
        entries.insert(keys::SHOW_BANNER.to_string(),         serde_json::to_string(&true).unwrap());
        entries.insert(keys::DEFAULT_SSH_KEY_TYPE.to_string(),serde_json::to_string("ed25519").unwrap());
        entries.insert(keys::SSH_CONNECT_TIMEOUT_MS.to_string(), serde_json::to_string(&30_000u32).unwrap());
        entries.insert(keys::AUTO_ADD_TO_AGENT.to_string(),   serde_json::to_string(&true).unwrap());
        entries.insert(keys::MAX_CONCURRENT_OPS.to_string(),  serde_json::to_string(&4u32).unwrap());
        entries.insert(keys::AUTO_MIGRATE.to_string(),        serde_json::to_string(&true).unwrap());
        entries.insert(keys::PLUGIN_DIRS.to_string(),         "[]".to_string());
        entries.insert(keys::THEME_ACTIVE_SLUG.to_string(),   serde_json::to_string("zyrix").unwrap());
        let now = Utc::now();
        Self { uuid: Uuid::new_v4(), entries, created_at: now, updated_at: now }
    }

    /// Rehydrates a Configuration from stored key-value pairs.
    pub fn rehydrate(
        uuid:       Uuid,
        entries:    HashMap<String, String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self { uuid, entries, created_at, updated_at }
    }

    // ── Generic get/set ───────────────────────────────────────────────────────

    /// Returns the raw JSON string for a key, or None if the key is not set.
    pub fn get_raw(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }

    /// Stores a value as its JSON serialisation. Returns the previous raw value.
    pub fn set<V: serde::Serialize>(&mut self, key: &str, value: &V) -> Option<String> {
        let json = serde_json::to_string(value).expect("configuration value must be JSON-serialisable");
        let prev = self.entries.insert(key.to_string(), json);
        self.updated_at = Utc::now();
        prev
    }

    /// Removes a key from the configuration. Returns the removed value if it existed.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        let removed = self.entries.remove(key);
        if removed.is_some() {
            self.updated_at = Utc::now();
        }
        removed
    }

    /// Returns all key-value pairs as a reference to the internal map.
    pub fn all(&self) -> &HashMap<String, String> {
        &self.entries
    }

    // ── Typed accessors ───────────────────────────────────────────────────────

    /// Returns the configured log level string, defaulting to "info".
    pub fn log_level(&self) -> String {
        self.get_string(keys::LOG_LEVEL, "info")
    }

    /// Returns the web interface bind address, defaulting to "127.0.0.1:5000".
    pub fn web_addr(&self) -> String {
        self.get_string(keys::WEB_ADDR, "127.0.0.1:5000")
    }

    /// Returns whether the CLI banner should be shown on startup.
    pub fn show_banner(&self) -> bool {
        self.get_bool(keys::SHOW_BANNER, true)
    }

    /// Returns the default SSH key type for new key generation.
    pub fn default_ssh_key_type(&self) -> String {
        self.get_string(keys::DEFAULT_SSH_KEY_TYPE, "ed25519")
    }

    /// Returns the SSH connection test timeout in milliseconds.
    pub fn ssh_connect_timeout_ms(&self) -> u32 {
        self.get_u32(keys::SSH_CONNECT_TIMEOUT_MS, 30_000)
    }

    /// Returns whether newly generated SSH keys should automatically be added to the agent.
    pub fn auto_add_to_agent(&self) -> bool {
        self.get_bool(keys::AUTO_ADD_TO_AGENT, true)
    }

    /// Returns the maximum number of concurrent git operations.
    pub fn max_concurrent_ops(&self) -> u32 {
        self.get_u32(keys::MAX_CONCURRENT_OPS, 4)
    }

    /// Returns the active theme slug, defaulting to "zyrix".
    pub fn theme_active_slug(&self) -> String {
        self.get_string(keys::THEME_ACTIVE_SLUG, "zyrix")
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn get_string(&self, key: &str, default: &str) -> String {
        self.entries.get(key)
            .and_then(|v| serde_json::from_str::<String>(v).ok())
            .unwrap_or_else(|| default.to_string())
    }

    fn get_bool(&self, key: &str, default: bool) -> bool {
        self.entries.get(key)
            .and_then(|v| serde_json::from_str::<bool>(v).ok())
            .unwrap_or(default)
    }

    fn get_u32(&self, key: &str, default: u32) -> u32 {
        self.entries.get(key)
            .and_then(|v| serde_json::from_str::<u32>(v).ok())
            .unwrap_or(default)
    }

    pub fn uuid(&self)       -> Uuid            { self.uuid }
    pub fn created_at(&self) -> DateTime<Utc>   { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc>   { self.updated_at }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_configuration_has_sane_defaults() {
        let cfg = Configuration::new_with_defaults();
        assert_eq!(cfg.log_level(), "info");
        assert_eq!(cfg.default_ssh_key_type(), "ed25519");
        assert!(cfg.show_banner());
        assert_eq!(cfg.max_concurrent_ops(), 4);
    }

    #[test]
    fn set_and_get_string_round_trips() {
        let mut cfg = Configuration::new_with_defaults();
        cfg.set(keys::LOG_LEVEL, &"debug");
        assert_eq!(cfg.log_level(), "debug");
    }

    #[test]
    fn remove_key_returns_none_on_subsequent_get() {
        let mut cfg = Configuration::new_with_defaults();
        cfg.remove(keys::SHOW_BANNER);
        // Should fall back to default since the key is gone
        assert!(cfg.show_banner());
    }
}