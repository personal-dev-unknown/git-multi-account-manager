// crates/gm_ports/src/outbound/storage_provider.rs
//
// The StorageProvider outbound port — generic key-value storage for plugin
// configuration and state. Plugins use this to persist their own settings
// (e.g., a GitHub plugin storing the OAuth callback port it last used)
// without needing their own database tables.
//
// The v1 implementation in gm_adapters uses the filesystem (atomic writes
// via the Zig layer) to store JSON files under ~/.git-zyrix/plugins/{name}/.
// Future adapters could delegate to Redis, S3, or the main MySQL database.

use async_trait::async_trait;
use gm_shared::errors::GitManagerError;

#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Stores a JSON-serialisable value under a namespaced key.
    /// The namespace is typically the plugin name.
    async fn set<V: serde::Serialize + Send + Sync>(
        &self,
        namespace: &str,
        key:       &str,
        value:     &V,
    ) -> Result<(), GitManagerError>;

    /// Retrieves and deserialises a value by namespace and key.
    /// Returns None if the key has not been set.
    async fn get<V: serde::de::DeserializeOwned>(
        &self,
        namespace: &str,
        key:       &str,
    ) -> Result<Option<V>, GitManagerError>;

    /// Removes a key from storage. Returns Ok(()) even if it did not exist.
    async fn delete(&self, namespace: &str, key: &str) -> Result<(), GitManagerError>;
}