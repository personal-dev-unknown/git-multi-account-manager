//! The Account entity — root aggregate of the accounts bounded context.
//!
//! An Account is a Git hosting account managed by Git Manager. It is a root
//! aggregate: it owns its SSH keys and SSH host configurations, and it is the
//! anchor for every repository and sync session in the system. Nothing in the
//! domain can create an SSH key, register a repository, or start a sync without
//! first resolving the owning Account.
//!
//! # Construction-time validation
//!
//! Fields are private. The ONLY way to create a valid Account is through
//! `Account::new()`, which enforces all business invariants before the struct
//! exists in memory. Entities rehydrated from the database go through
//! `Account::rehydrate()` which trusts that the database already validated the
//! data when it was first persisted. Skipping validation on rehydration avoids
//! the scenario where a schema migration changes a constraint and every read
//! fails until the data is migrated.
//!
//! # Identity
//!
//! The UUID is generated at construction time and never changes. The alias can
//! be thought of as the user-visible "handle" for the account, while the UUID
//! is the stable identity that all foreign keys reference.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::accounts::value_objects::{AccountStatus, AuthMethod};
use gm_shared::errors::AccountError;
use gm_shared::models::account::AccountDto;
use gm_shared::validation::validators::{validate_alias, validate_email};

/// A Git hosting account managed by Git Manager.
#[derive(Debug, Clone)]
pub struct Account {
    uuid:           Uuid,
    platform_id:    Uuid,
    alias:          String,
    username:       String,
    email:          String,
    display_name:   Option<String>,
    auth_method:    AuthMethod,
    ssh_host_alias: Option<String>,
    status:         AccountStatus,
    is_default:     bool,
    last_used_at:   Option<DateTime<Utc>>,
    created_at:     DateTime<Utc>,
}

impl Account {
    /// Creates a new Account, enforcing every business invariant.
    ///
    /// Business rules enforced:
    ///   1. Alias must be 2–50 lowercase letters, digits, or hyphens only.
    ///   2. Email must have a valid structure.
    ///   3. Username must not be empty or whitespace-only.
    ///
    /// On success the account starts in `AccountStatus::Unverified` because
    /// the user's SSH key or PAT has not yet been tested against the platform.
    /// The account transitions to `Active` after `activate()` is called following
    /// a successful connection test.
    pub fn new(
        alias:       String,
        platform_id: Uuid,
        username:    String,
        email:       String,
        auth_method: AuthMethod,
    ) -> Result<Self, AccountError> {
        validate_alias(&alias).map_err(|e| AccountError::InvalidAlias {
            alias:  alias.clone(),
            reason: e.to_string(),
        })?;

        validate_email(&email).map_err(|e| AccountError::InvalidEmail {
            email: {
                let _ = e; // consume the error to satisfy the borrow checker
                email.clone()
            },
        })?;

        let username = username.trim().to_string();
        if username.is_empty() {
            return Err(AccountError::EmptyUsername);
        }

        Ok(Self {
            uuid:           Uuid::new_v4(),
            platform_id,
            alias,
            username,
            email:          email.trim().to_lowercase(),
            display_name:   None,
            auth_method,
            ssh_host_alias: None,
            status:         AccountStatus::Unverified,
            is_default:     false,
            last_used_at:   None,
            created_at:     Utc::now(),
        })
    }

    /// Rehydrates an Account from a database row without re-running construction
    /// validators. Called exclusively by the MySQL and SQLite repository adapters.
    ///
    /// This method is intentionally not `pub` within the domain — it is
    /// `pub(crate)` to allow the adapters crate access while preventing
    /// arbitrary code from bypassing validation. Adapters are in `gm_adapters`,
    /// not in `gm_domain`, so they access this through a public `from_row`
    /// method on the repository implementations.
    ///
    /// The `pub` visibility here is required because gm_adapters is a separate
    /// crate; the architecture enforces the constraint via code review discipline
    /// (adapters call rehydrate, domain logic never does).
    pub fn rehydrate(
        uuid:           Uuid,
        platform_id:    Uuid,
        alias:          String,
        username:       String,
        email:          String,
        display_name:   Option<String>,
        auth_method:    AuthMethod,
        ssh_host_alias: Option<String>,
        status:         AccountStatus,
        is_default:     bool,
        last_used_at:   Option<DateTime<Utc>>,
        created_at:     DateTime<Utc>,
    ) -> Self {
        Self {
            uuid,
            platform_id,
            alias,
            username,
            email,
            display_name,
            auth_method,
            ssh_host_alias,
            status,
            is_default,
            last_used_at,
            created_at,
        }
    }

    // ── Domain operations ─────────────────────────────────────────────────────

    /// Transitions the account to `Active` status.
    ///
    /// This represents the platform confirming that the account's credentials
    /// are valid. Called after a successful SSH connection test or PAT verification.
    /// Suspended accounts cannot be activated — they require platform-side action.
    pub fn activate(&mut self) -> Result<(), AccountError> {
        self.status = self.status.clone().transition_to(
            AccountStatus::Active,
            self.uuid,
        )?;
        Ok(())
    }

    /// Transitions the account to `Inactive`.
    /// The user explicitly wants to stop using this account without deleting it.
    pub fn deactivate(&mut self) -> Result<(), AccountError> {
        self.status = self.status.clone().transition_to(
            AccountStatus::Inactive,
            self.uuid,
        )?;
        Ok(())
    }

    /// Transitions the account to `TokenExpired`.
    /// Called by the adapter when a platform API call returns HTTP 401.
    pub fn mark_token_expired(&mut self) -> Result<(), AccountError> {
        self.status = self.status.clone().transition_to(
            AccountStatus::TokenExpired,
            self.uuid,
        )?;
        Ok(())
    }

    /// Transitions the account to `Suspended`.
    /// Called by the adapter when a platform API call returns HTTP 403.
    pub fn mark_suspended(&mut self) -> Result<(), AccountError> {
        self.status = self.status.clone().transition_to(
            AccountStatus::Suspended,
            self.uuid,
        )?;
        Ok(())
    }

    /// Sets this account as the default for its platform.
    /// The caller (AccountService) is responsible for ensuring that no other
    /// account on the same platform has `is_default = true` before calling this.
    /// That invariant requires a database query, which is a port concern, not
    /// an entity concern.
    pub fn set_as_default(&mut self) {
        self.is_default = true;
    }

    /// Clears the default flag. Called on the previously-default account when
    /// another account is set as default.
    pub fn unset_default(&mut self) {
        self.is_default = false;
    }

    /// Associates an SSH host alias with this account.
    /// The alias is written to `~/.ssh/config` as a `Host` block and embedded
    /// in git remote URLs so the correct SSH key is selected per account.
    pub fn set_ssh_host_alias(&mut self, alias: String) {
        self.ssh_host_alias = Some(alias);
    }

    /// Updates the display name from the platform's user profile API response.
    pub fn set_display_name(&mut self, name: Option<String>) {
        self.display_name = name;
    }

    /// Records the current time as the last time this account was used.
    /// Called by the adapter after any git operation completes successfully.
    pub fn record_usage(&mut self) {
        self.last_used_at = Some(Utc::now());
    }

    // ── Getters ───────────────────────────────────────────────────────────────
    // Fields are private; callers access them through these methods.
    // This preserves the option to add validation or side effects to reads
    // without changing the call sites.

    pub fn uuid(&self)           -> Uuid                    { self.uuid }
    pub fn platform_id(&self)    -> Uuid                    { self.platform_id }
    pub fn alias(&self)          -> &str                    { &self.alias }
    pub fn username(&self)       -> &str                    { &self.username }
    pub fn email(&self)          -> &str                    { &self.email }
    pub fn display_name(&self)   -> Option<&str>            { self.display_name.as_deref() }
    pub fn auth_method(&self)    -> &AuthMethod             { &self.auth_method }
    pub fn ssh_host_alias(&self) -> Option<&str>            { self.ssh_host_alias.as_deref() }
    pub fn status(&self)         -> &AccountStatus          { &self.status }
    pub fn is_default(&self)     -> bool                    { self.is_default }
    pub fn last_used_at(&self)   -> Option<DateTime<Utc>>   { self.last_used_at }
    pub fn created_at(&self)     -> DateTime<Utc>           { self.created_at }

    /// Converts this entity into a DTO suitable for crossing the crate boundary.
    /// The DTO is serialisable and contains no domain logic or private state.
    pub fn to_dto(&self) -> AccountDto {
        AccountDto {
            uuid:             self.uuid,
            platform_id:      self.platform_id,
            alias:            self.alias.clone(),
            username:         self.username.clone(),
            email:            self.email.clone(),
            display_name:     self.display_name.clone(),
            avatar_url:       None, // populated by the provider plugin, not stored in domain
            auth_method:      self.auth_method.to_shared(),
            ssh_host_alias:   self.ssh_host_alias.clone(),
            status:           self.status.to_shared(),
            is_default:       self.is_default,
            last_used_at:     self.last_used_at,
            last_verified_at: None,
            created_at:       self.created_at,
            // Platform display name is denormalised by the service or adapter layer
            // that JOINs the platforms table. The domain entity only stores the UUID.
            platform_name:    None,
        }
    }
}

#[allow(dead_code)]
fn is_valid_alias(alias: &str) -> bool {
    validate_alias(alias).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::value_objects::AuthMethod;

    fn make_account() -> Account {
        Account::new(
            "work".to_string(),
            Uuid::new_v4(),
            "shakamoses".to_string(),
            "shaka@work.dev".to_string(),
            AuthMethod::Ssh,
        ).unwrap()
    }

    #[test]
    fn new_account_starts_unverified() {
        let account = make_account();
        assert_eq!(account.status(), &AccountStatus::Unverified);
        assert!(!account.is_default());
    }

    #[test]
    fn new_account_trims_and_lowercases_email() {
        let account = Account::new(
            "work".to_string(),
            Uuid::new_v4(),
            "Shaka".to_string(),
            "  SHAKA@Work.Dev  ".to_string(),
            AuthMethod::Ssh,
        ).unwrap();
        assert_eq!(account.email(), "shaka@work.dev");
    }

    #[test]
    fn rejects_invalid_alias() {
        let result = Account::new(
            "Work".to_string(), // uppercase not allowed
            Uuid::new_v4(),
            "user".to_string(),
            "user@example.com".to_string(),
            AuthMethod::Ssh,
        );
        assert!(matches!(result, Err(AccountError::InvalidAlias { .. })));
    }

    #[test]
    fn rejects_empty_username() {
        let result = Account::new(
            "work".to_string(),
            Uuid::new_v4(),
            "   ".to_string(), // whitespace-only
            "user@example.com".to_string(),
            AuthMethod::Ssh,
        );
        assert!(matches!(result, Err(AccountError::EmptyUsername)));
    }

    #[test]
    fn activate_transitions_from_unverified() {
        let mut account = make_account();
        assert!(account.activate().is_ok());
        assert_eq!(account.status(), &AccountStatus::Active);
    }

    #[test]
    fn cannot_activate_suspended_account() {
        let mut account = make_account();
        account.status = AccountStatus::Active;
        account.mark_suspended().unwrap();
        assert!(account.activate().is_err());
    }

    #[test]
    fn set_and_unset_default() {
        let mut account = make_account();
        account.set_as_default();
        assert!(account.is_default());
        account.unset_default();
        assert!(!account.is_default());
    }

    #[test]
    fn to_dto_reflects_current_state() {
        let mut account = make_account();
        account.set_as_default();
        account.activate().unwrap();
        let dto = account.to_dto();
        assert_eq!(dto.alias, "work");
        assert!(dto.is_default);
        assert_eq!(dto.status, gm_shared::models::account::AccountStatus::Active);
    }
}