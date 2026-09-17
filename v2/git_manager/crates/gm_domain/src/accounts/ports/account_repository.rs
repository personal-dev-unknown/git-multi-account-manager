//! The AccountRepository port — the domain's contract for account persistence.
//!
//! This trait declares what the domain needs from storage without specifying
//! how storage works. The domain's AccountService holds an `Arc<dyn AccountRepository>`
//! and calls these methods. At runtime the kernel wires in either
//! `MySqlAccountRepository` or `SqliteAccountRepository` from gm_adapters — the
//! domain never knows or cares which.
//!
//! The `Send + Sync` bounds are mandatory because the kernel stores the
//! trait object in an `Arc` and dispatches commands to domain handlers from
//! multiple async tasks simultaneously.

use async_trait::async_trait;
use uuid::Uuid;

use crate::accounts::entities::Account;
use gm_shared::errors::AccountError;

/// Storage contract for account records.
#[async_trait]
pub trait AccountRepository: Send + Sync + std::fmt::Debug {
    /// Persists a new account or updates an existing one.
    /// Uses an upsert based on the account's UUID — the repository never
    /// silently replaces a different account with the same alias.
    async fn save(&self, account: &Account) -> Result<(), AccountError>;

    /// Retrieves an account by its UUID. Returns `None` if no account with
    /// that UUID exists; returns `Err` only on storage-level failures.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, AccountError>;

    /// Finds an account by its (alias, platform_id) composite key.
    /// Aliases are unique per platform — the same alias on two different
    /// platforms refers to two different accounts.
    async fn find_by_alias(
        &self,
        alias:       &str,
        platform_id: Uuid,
    ) -> Result<Option<Account>, AccountError>;

    /// Returns all accounts ordered by alias ascending.
    async fn list_all(&self) -> Result<Vec<Account>, AccountError>;

    /// Returns all accounts for a specific platform, ordered by alias.
    async fn list_by_platform(
        &self,
        platform_id: Uuid,
    ) -> Result<Vec<Account>, AccountError>;

    /// Permanently removes an account from storage.
    /// The caller (AccountService) must verify no repositories or SSH keys
    /// are still linked before calling this — the storage layer may enforce
    /// referential integrity via foreign keys and return an error if links exist.
    async fn delete(&self, id: Uuid) -> Result<(), AccountError>;

    /// Atomically sets one account as the platform default and clears the
    /// `is_default` flag on all other accounts in the same platform.
    ///
    /// This operation MUST be atomic at the database level — a non-atomic
    /// implementation would leave a window where either no account or two
    /// accounts appear as default, which breaks the CLI's account auto-selection.
    /// Implementations must use a database transaction.
    async fn set_default(
        &self,
        account_id:  Uuid,
        platform_id: Uuid,
    ) -> Result<(), AccountError>;

    /// Returns the number of accounts registered with a specific platform.
    /// Used by AccountService to decide whether deleting an account requires
    /// re-assigning the default.
    async fn count_by_platform(&self, platform_id: Uuid) -> Result<u64, AccountError>;
}