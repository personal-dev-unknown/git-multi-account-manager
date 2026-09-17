//! Account lifecycle status — a domain value object with state transition logic.
//!
//! The `AccountStatus` type here is the domain's own representation, separate
//! from the identically-named DTO in `gm_shared::models::account`. The domain
//! version carries business logic (which transitions are allowed); the shared
//! version is a plain serialisable enum used at the system's edges.
//!
//! State machine:
//!
//!   Unverified ──→ Active         (SSH test passes / PAT verified)
//!   Active     ──→ Inactive       (user manually deactivates)
//!   Active     ──→ TokenExpired   (platform returns 401 during an operation)
//!   Active     ──→ Suspended      (platform returns 403; human intervention needed)
//!   Inactive   ──→ Active         (user reactivates)
//!   TokenExpired → Active         (user provides a fresh credential)
//!   Suspended  ──→ (terminal)     (no automated recovery possible)

use gm_shared::errors::AccountError;
use uuid::Uuid;

/// The lifecycle status of a Git hosting account within Git Manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountStatus {
    /// The account exists in the database but its SSH key or PAT has not yet
    /// been verified against the platform. New accounts start here.
    Unverified,
    /// Fully functional. All git operations are permitted.
    Active,
    /// Manually deactivated by the user. Git operations are refused until
    /// the user explicitly reactivates the account.
    Inactive,
    /// The OAuth token or PAT has expired. The user must update the credential
    /// before operations can resume. This is auto-detected when a platform API
    /// call returns HTTP 401.
    TokenExpired,
    /// The platform has suspended or banned this account. This is auto-detected
    /// when a platform API call returns HTTP 403. No automated recovery —
    /// the user must resolve the issue with the platform first.
    Suspended,
}

impl AccountStatus {
    /// Returns true if this account can perform git operations (clone, push, pull).
    /// Active is the only status that permits git I/O.
    pub fn can_perform_git_operations(&self) -> bool {
        matches!(self, AccountStatus::Active)
    }

    /// Returns true if this status is a permanent terminal state that the
    /// system cannot recover from automatically.
    pub fn is_terminal(&self) -> bool {
        matches!(self, AccountStatus::Suspended)
    }

    /// Validates whether transitioning from `self` to `target` is permitted.
    /// Returns the target status on success, or an `AccountError` describing
    /// the invalid transition.
    pub fn transition_to(
        &self,
        target: AccountStatus,
        account_uuid: Uuid,
    ) -> Result<AccountStatus, AccountError> {
        let allowed = match (&self, &target) {
            // From Unverified — activation is the only forward path
            (AccountStatus::Unverified,   AccountStatus::Active)     => true,

            // From Active — can deactivate, expire, or be suspended
            (AccountStatus::Active,       AccountStatus::Inactive)   => true,
            (AccountStatus::Active,       AccountStatus::TokenExpired) => true,
            (AccountStatus::Active,       AccountStatus::Suspended)  => true,

            // From Inactive — user reactivation only
            (AccountStatus::Inactive,     AccountStatus::Active)     => true,

            // From TokenExpired — fresh credential restores to Active
            (AccountStatus::TokenExpired, AccountStatus::Active)     => true,

            // Suspended is terminal — no automated transitions allowed
            (AccountStatus::Suspended,    _)                         => false,

            // All other transitions are invalid
            _ => false,
        };

        if allowed {
            Ok(target)
        } else {
            Err(AccountError::InvalidStateTransition {
                uuid:           account_uuid,
                current_status: self.to_string(),
            })
        }
    }

    /// Converts this domain status to the corresponding shared DTO status.
    /// Called from `Account::to_dto()` when preparing data for the interface layers.
    pub fn to_shared(&self) -> gm_shared::models::account::AccountStatus {
        use gm_shared::models::account::AccountStatus as Shared;
        match self {
            AccountStatus::Unverified   => Shared::Unverified,
            AccountStatus::Active       => Shared::Active,
            AccountStatus::Inactive     => Shared::Inactive,
            AccountStatus::TokenExpired => Shared::TokenExpired,
            AccountStatus::Suspended    => Shared::Suspended,
        }
    }

    /// Reconstructs a domain status from a shared DTO status.
    /// Called by the persistence adapter when rehydrating an Account from the database.
    pub fn from_shared(shared: &gm_shared::models::account::AccountStatus) -> Self {
        use gm_shared::models::account::AccountStatus as Shared;
        match shared {
            Shared::Unverified   => AccountStatus::Unverified,
            Shared::Active       => AccountStatus::Active,
            Shared::Inactive     => AccountStatus::Inactive,
            Shared::TokenExpired => AccountStatus::TokenExpired,
            Shared::Suspended    => AccountStatus::Suspended,
        }
    }
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStatus::Unverified   => write!(f, "unverified"),
            AccountStatus::Active       => write!(f, "active"),
            AccountStatus::Inactive     => write!(f, "inactive"),
            AccountStatus::TokenExpired => write!(f, "token_expired"),
            AccountStatus::Suspended    => write!(f, "suspended"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_can_perform_git_operations() {
        assert!(AccountStatus::Active.can_perform_git_operations());
        assert!(!AccountStatus::Unverified.can_perform_git_operations());
        assert!(!AccountStatus::Suspended.can_perform_git_operations());
    }

    #[test]
    fn suspended_is_terminal() {
        assert!(AccountStatus::Suspended.is_terminal());
        assert!(!AccountStatus::Active.is_terminal());
    }

    #[test]
    fn valid_transition_unverified_to_active() {
        let uuid = Uuid::new_v4();
        let result = AccountStatus::Unverified.transition_to(AccountStatus::Active, uuid);
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_transition_suspended_to_active() {
        let uuid = Uuid::new_v4();
        let result = AccountStatus::Suspended.transition_to(AccountStatus::Active, uuid);
        assert!(matches!(result, Err(AccountError::InvalidStateTransition { .. })));
    }

    #[test]
    fn round_trip_through_shared() {
        let domain = AccountStatus::TokenExpired;
        let shared = domain.to_shared();
        let back   = AccountStatus::from_shared(&shared);
        assert_eq!(domain, back);
    }
}