// crates/gm_domain/src/configuration/services/config_service.rs
//
// The ConfigService wraps the ConfigRepository port and provides typed
// read/write access to application settings. It is generic over R: ConfigRepository
// so the kernel can wire in MySQL, SQLite, or an in-memory test implementation
// without the service knowing which.
//
// The service intentionally does not cache the Configuration in memory — callers
// load it when needed. The kernel's bootstrap does cache it via the service
// registry after the first load, so repeated hot-path reads do not pay the
// DB round-trip cost.

use std::fmt;
use std::sync::Arc;
use crate::configuration::{
    entities::Configuration,
    ConfigRepository,
};
use gm_shared::errors::GitManagerError;

/// Domain service for reading and modifying application configuration.
pub struct ConfigService {
    repository: Arc<dyn ConfigRepository>,
}

impl fmt::Debug for ConfigService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigService").finish_non_exhaustive()
    }
}

impl ConfigService {
    pub fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }

    /// Loads the current configuration, creating a default-valued record
    /// on first boot if none exists.
    pub async fn load(&self) -> Result<Configuration, GitManagerError> {
        self.repository.load().await
    }

    /// Sets a single configuration key to the given value and persists immediately.
    /// Returns the previous raw JSON value for that key, if one existed.
    pub async fn set<V: serde::Serialize>(
        &self,
        key:   &str,
        value: &V,
    ) -> Result<Option<String>, GitManagerError> {
        let mut config = self.repository.load().await?;
        let prev = config.set(key, value);
        self.repository.save(&config).await?;
        Ok(prev)
    }

    /// Resets a key to its factory default by removing the stored value.
    /// The next read of that key will fall back to the in-code default.
    pub async fn reset(&self, key: &str) -> Result<(), GitManagerError> {
        let mut config = self.repository.load().await?;
        config.remove(key);
        self.repository.save(&config).await
    }

    /// Resets all configuration to factory defaults by replacing the record
    /// with a freshly constructed default Configuration.
    pub async fn reset_all(&self) -> Result<(), GitManagerError> {
        let defaults = Configuration::new_with_defaults();
        self.repository.save(&defaults).await
    }

    // ── Typed convenience methods ─────────────────────────────────────────────
    // These exist so callers write `config_service.log_level().await?`
    // rather than needing to load the whole Configuration struct themselves.

    pub async fn log_level(&self) -> Result<String, GitManagerError> {
        Ok(self.repository.load().await?.log_level())
    }

    pub async fn web_addr(&self) -> Result<String, GitManagerError> {
        Ok(self.repository.load().await?.web_addr())
    }

    pub async fn show_banner(&self) -> Result<bool, GitManagerError> {
        Ok(self.repository.load().await?.show_banner())
    }

    pub async fn default_ssh_key_type(&self) -> Result<String, GitManagerError> {
        Ok(self.repository.load().await?.default_ssh_key_type())
    }

    pub async fn ssh_connect_timeout_ms(&self) -> Result<u32, GitManagerError> {
        Ok(self.repository.load().await?.ssh_connect_timeout_ms())
    }

    pub async fn auto_add_to_agent(&self) -> Result<bool, GitManagerError> {
        Ok(self.repository.load().await?.auto_add_to_agent())
    }

    pub async fn max_concurrent_ops(&self) -> Result<u32, GitManagerError> {
        Ok(self.repository.load().await?.max_concurrent_ops())
    }
}