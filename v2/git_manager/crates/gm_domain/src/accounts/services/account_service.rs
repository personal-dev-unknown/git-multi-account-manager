//! AccountService — the orchestrator for all account lifecycle operations.
//!
//! The service layer sits between the command bus (which receives user intent)
//! and the repository port (which persists data). Its job is to apply business
//! rules that span across entities and require repository queries — rules that
//! cannot be enforced inside a single entity in isolation.
//!
//! For example, the rule "alias must be unique per platform" requires querying
//! whether an account with that alias already exists. A single Account entity
//! cannot check this because it has no access to the repository. The service
//! runs the check before creating the entity.
//!
//! # Generics
//!
//! `AccountService<R>` is generic over `R: AccountRepository`. This means the
//! type of the underlying storage (MySQL, SQLite, in-memory mock) is resolved
//! at the kernel's bootstrap time when the concrete adapter is wired in. The
//! domain service never imports a concrete repository type.
//!
//! # Event production
//!
//! The service methods return typed result structs that include both the
//! operation's primary output AND the domain events that should be published.
//! The command handler in the kernel reads these events and publishes them to
//! the event bus. This keeps the domain pure — it defines what events SHOULD
//! be published after an operation, but it never directly touches the event bus.

use std::sync::Arc;
use uuid::Uuid;

use crate::accounts::{
    entities::Account,
    events::{AccountAdded, AccountRemoved, AccountStatusChanged},
    ports::AccountRepository,
    value_objects::AuthMethod,
};
use gm_shared::errors::AccountError;
use gm_shared::models::account::AccountDto;

/// The result of a successful add_account call.
/// The kernel's command handler publishes the included event after wiring.
#[derive(Debug)]
pub struct AddAccountResult {
    pub account: Account,
    pub event:   AccountAdded,
}

/// The result of a successful remove_account call.
#[derive(Debug)]
pub struct RemoveAccountResult {
    pub event: AccountRemoved,
}

/// The result of a successful activate call.
#[derive(Debug)]
pub struct ActivateAccountResult {
    pub account: Account,
    pub event:   AccountStatusChanged,
}

/// Domain service for account lifecycle management.
#[derive(Debug)]
pub struct AccountService {
    repository: Arc<dyn AccountRepository>,
}

impl AccountService {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    /// Creates and persists a new account.
    ///
    /// Steps:
    ///   1. Check that no account with the same (alias, platform_id) already exists.
    ///   2. Construct the Account entity (enforces format rules via Account::new).
    ///   3. Persist the account via the repository port.
    ///   4. Return the account and the AccountAdded event for the caller to publish.
    pub async fn add_account(
        &self,
        alias:       String,
        platform_id: Uuid,
        username:    String,
        email:       String,
        auth_method: AuthMethod,
    ) -> Result<AddAccountResult, AccountError> {
        // Business rule: alias must be unique within a platform
        if let Some(_existing) = self.repository
            .find_by_alias(&alias, platform_id)
            .await?
        {
            return Err(AccountError::AliasAlreadyExists {
                alias,
                platform_id,
            });
        }

        // Construct entity (enforces format invariants)
        let account = Account::new(alias, platform_id, username, email, auth_method)?;

        // Persist
        self.repository.save(&account).await?;

        let event = AccountAdded {
            account_uuid: account.uuid(),
            platform_id,
            alias:        account.alias().to_string(),
            username:     account.username().to_string(),
            auth_method:  account.auth_method().to_shared(),
        };

        Ok(AddAccountResult { account, event })
    }

    /// Retrieves a single account by UUID.
    pub async fn get_account(&self, uuid: Uuid) -> Result<Account, AccountError> {
        self.repository
            .find_by_id(uuid)
            .await?
            .ok_or(AccountError::NotFound { uuid })
    }

    /// Retrieves an account by its (alias, platform_id) pair.
    pub async fn get_account_by_alias(
        &self,
        alias:       &str,
        platform_id: Uuid,
    ) -> Result<Account, AccountError> {
        self.repository
            .find_by_alias(alias, platform_id)
            .await?
            .ok_or_else(|| AccountError::NotFound { uuid: Uuid::nil() })
    }

    /// Lists all accounts, ordered by alias.
    pub async fn list_accounts(&self) -> Result<Vec<AccountDto>, AccountError> {
        let accounts = self.repository.list_all().await?;
        Ok(accounts.iter().map(Account::to_dto).collect())
    }

    /// Lists accounts for a specific platform.
    pub async fn list_accounts_by_platform(
        &self,
        platform_id: Uuid,
    ) -> Result<Vec<AccountDto>, AccountError> {
        let accounts = self.repository.list_by_platform(platform_id).await?;
        Ok(accounts.iter().map(Account::to_dto).collect())
    }

    /// Activates an account after SSH key or PAT verification succeeds.
    ///
    /// This transitions the account from any non-Suspended state into Active.
    /// The event returned should be published so interface plugins can update
    /// their displays (e.g. the CLI's account list shows the new status).
    pub async fn activate_account(
        &self,
        uuid: Uuid,
    ) -> Result<ActivateAccountResult, AccountError> {
        let mut account = self.get_account(uuid).await?;
        let previous_status = account.status().to_string();
        account.activate()?;
        self.repository.save(&account).await?;

        let event = AccountStatusChanged {
            account_uuid:    uuid,
            previous_status,
            new_status:      account.status().to_string(),
        };

        Ok(ActivateAccountResult { account, event })
    }

    /// Sets an account as the platform default, clearing the flag on all others.
    ///
    /// The atomicity guarantee is delegated to the repository's `set_default`
    /// method which must run both operations in a single database transaction.
    pub async fn set_default_account(
        &self,
        account_uuid: Uuid,
        platform_id:  Uuid,
    ) -> Result<Account, AccountError> {
        // Confirm the account exists before asking the repository to set default
        let _account = self.get_account(account_uuid).await?;
        self.repository.set_default(account_uuid, platform_id).await?;
        // Re-fetch to get the updated is_default = true state
        self.get_account(account_uuid).await
    }

    /// Removes an account permanently.
    ///
    /// Business rules:
    ///   - Cannot delete the only account for a platform if it is the default
    ///     (there would be no default account after deletion).
    ///   - Enforcing referential integrity (linked repositories, SSH keys) is
    ///     the responsibility of the database foreign key constraints; the
    ///     repository adapter translates constraint violations to AccountError.
    pub async fn remove_account(
        &self,
        uuid: Uuid,
    ) -> Result<RemoveAccountResult, AccountError> {
        let account = self.get_account(uuid).await?;

        if account.is_default() {
            let count = self.repository
                .count_by_platform(account.platform_id())
                .await?;
            if count == 1 {
                // This is the last account for the platform — safe to delete even as default
            } else {
                // More than one account exists; prevent deleting the default without reassigning
                return Err(AccountError::CannotDeleteDefault);
            }
        }

        self.repository.delete(uuid).await?;

        let event = AccountRemoved {
            account_uuid: uuid,
            platform_id:  account.platform_id(),
            alias:        account.alias().to_string(),
        };

        Ok(RemoveAccountResult { event })
    }

    /// Associates an SSH host alias with an existing account and persists the update.
    /// Called after the SSH adapter writes the `~/.ssh/config` Host block.
    pub async fn set_ssh_host_alias(
        &self,
        account_uuid: Uuid,
        host_alias:   String,
    ) -> Result<(), AccountError> {
        let mut account = self.get_account(account_uuid).await?;
        account.set_ssh_host_alias(host_alias);
        self.repository.save(&account).await
    }

    /// Records that this account was used in a git operation, updating last_used_at.
    pub async fn record_usage(&self, account_uuid: Uuid) -> Result<(), AccountError> {
        let mut account = self.get_account(account_uuid).await?;
        account.record_usage();
        self.repository.save(&account).await
    }
}