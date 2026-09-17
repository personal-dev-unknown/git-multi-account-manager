// crates/gm_domain/src/ssh/ports/ssh_key_repository.rs
//
// Persistence port for SshKey entities. The SshService holds an
// Arc<dyn SshKeyRepository> and calls these methods without knowing whether
// the backing store is MySQL, SQLite, or an in-memory mock for tests.

use async_trait::async_trait;
use uuid::Uuid;

use crate::ssh::entities::SshKey;
use gm_shared::errors::SshError;

#[async_trait]
pub trait SshKeyRepository: Send + Sync + std::fmt::Debug {
    /// Persists a new key or updates an existing one by UUID.
    async fn save(&self, key: &SshKey) -> Result<(), SshError>;

    /// Retrieves a key by its UUID. Returns None if not found.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SshKey>, SshError>;

    /// Returns the active SSH key for an account, if one exists.
    /// "Active" means is_active = true. There should be at most one per account.
    async fn find_active_for_account(&self, account_id: Uuid) -> Result<Option<SshKey>, SshError>;

    /// Returns all SSH keys for an account (active and inactive) ordered by created_at desc.
    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<SshKey>, SshError>;

    /// Looks up a key by its SHA-256 fingerprint. Used to detect duplicate keys
    /// when the user tries to add a key they already have in the system.
    async fn find_by_fingerprint(&self, fingerprint: &str) -> Result<Option<SshKey>, SshError>;

    /// Deactivates all SSH keys for an account. Called before creating a new key
    /// to ensure only one active key exists per account at any time.
    async fn deactivate_all_for_account(&self, account_id: Uuid) -> Result<(), SshError>;
}