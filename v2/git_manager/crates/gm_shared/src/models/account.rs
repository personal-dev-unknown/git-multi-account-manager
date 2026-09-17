//! Account-related data transfer objects and enumerations.
//!
//! These types are the "postal form" of an Account entity — they carry all the
//! data needed to display or transmit an account, but they carry none of the
//! domain rules. The `Account` entity in `gm_domain` enforces invariants at
//! construction time. The `AccountDto` here is a plain struct that simply holds
//! values without any validation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A serializable snapshot of an account, safe to pass across crate boundaries.
///
/// Created by `Account::to_dto()` in the domain layer. Consumed by interface
/// plugins (CLI, Web, Desktop) to render account information to users, and by
/// the workflow engine to pass account context between steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountDto {
    /// UUID primary key — stable across renames and platform changes
    pub uuid: Uuid,

    /// UUID of the platform this account belongs to
    pub platform_id: Uuid,

    /// Human-readable name chosen by the user: "work", "personal", "client-acme"
    /// Must be lowercase letters, hyphens, and digits only. 2–50 characters.
    pub alias: String,

    /// The username on the hosting platform: "shakamoses" on GitHub
    pub username: String,

    /// Email address associated with this account on the platform
    pub email: String,

    /// Optional display name from the platform profile (may be null for new accounts)
    pub display_name: Option<String>,

    /// URL to the user's avatar image on the platform
    pub avatar_url: Option<String>,

    /// How this account authenticates: SSH key, PAT, OAuth token, etc.
    pub auth_method: AuthMethod,

    /// The SSH host alias used in git remote URLs and ~/.ssh/config.
    /// For a GitHub work account this would be "github.com-work".
    /// None until the user runs SSH key setup.
    pub ssh_host_alias: Option<String>,

    /// Current lifecycle status of the account
    pub status: AccountStatus,

    /// Whether this is the default account for its platform.
    /// When running `git-zyrix clone` without specifying an account,
    /// the default account for the detected platform is used.
    pub is_default: bool,

    /// When this account was last used for a git operation
    pub last_used_at: Option<DateTime<Utc>>,

    /// When this account was last verified with the platform API
    pub last_verified_at: Option<DateTime<Utc>>,

    /// When the account was first created in the system
    pub created_at: DateTime<Utc>,

    /// Display name of the platform (e.g. "GitHub"), denormalised by the
    /// service or adapter layer. The domain entity only stores `platform_id`;
    /// this field is filled in by the binary's wiring layer and is `None`
    /// until then.
    pub platform_name: Option<String>,
}

/// The lifecycle status of an account.
///
/// The state machine for account status is:
///   Unverified → Active         (after SSH test passes or PAT is verified)
///   Active → Inactive           (user manually deactivates)
///   Active → TokenExpired       (OAuth/PAT expiry detected during an operation)
///   Active → Suspended          (platform suspended the user — detected via API 403)
///   Inactive → Active           (user reactivates)
///   TokenExpired → Active       (after the user updates the credential)
///   Suspended → (terminal)      (cannot recover without platform action)
///
/// The domain service enforces valid transitions; invalid state changes return
/// an `AccountError::InvalidStateTransition`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    /// Just created; SSH key has not been tested yet or PAT not yet verified.
    Unverified,
    /// Fully functional — SSH auth or token verified.
    Active,
    /// Manually deactivated by the user. Git operations are refused.
    Inactive,
    /// OAuth token or PAT has expired. The user must update the credential.
    TokenExpired,
    /// The platform has suspended or banned this account. Human intervention required.
    Suspended,
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

/// How an account authenticates with its hosting platform.
///
/// The auth method determines which credentials are stored and which
/// GIT_SSH_COMMAND or HTTPS token is used for git operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    /// SSH key pair. The private key path is in the associated SshKey record.
    /// This is the recommended method for most use cases.
    Ssh,
    /// HTTPS with a Personal Access Token. Used for platforms where SSH is
    /// not available or for CI environments. The PAT is stored encrypted.
    HttpsPat,
    /// HTTPS with a username and password. Less secure than PAT; only for
    /// platforms that do not support PATs.
    HttpsPassword,
    /// OAuth 2.0 PKCE flow. Access and refresh tokens are stored encrypted.
    /// Used when the platform's OAuth scope is needed (e.g., creating repos).
    OAuth,
    /// No authentication. Only usable for public read-only repositories.
    Anonymous,
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::Ssh           => write!(f, "ssh"),
            AuthMethod::HttpsPat      => write!(f, "https_pat"),
            AuthMethod::HttpsPassword => write!(f, "https_password"),
            AuthMethod::OAuth         => write!(f, "oauth"),
            AuthMethod::Anonymous     => write!(f, "anonymous"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_status_serializes_snake_case() {
        // Verify that serde produces lowercase snake_case strings — this is what
        // the database stores and what the web API returns.
        let json = serde_json::to_string(&AccountStatus::TokenExpired).unwrap();
        assert_eq!(json, "\"token_expired\"");
    }

    #[test]
    fn auth_method_deserializes_from_snake_case() {
        let method: AuthMethod = serde_json::from_str("\"https_pat\"").unwrap();
        assert_eq!(method, AuthMethod::HttpsPat);
    }

    #[test]
    fn account_dto_round_trips_through_json() {
        let dto = AccountDto {
            uuid:              Uuid::new_v4(),
            platform_id:       Uuid::new_v4(),
            alias:             "work".to_string(),
            username:          "shakamoses".to_string(),
            email:             "shaka@work.dev".to_string(),
            display_name:      None,
            avatar_url:        None,
            auth_method:       AuthMethod::Ssh,
            ssh_host_alias:    Some("github.com-work".to_string()),
            status:            AccountStatus::Active,
            is_default:        true,
            last_used_at:      None,
            last_verified_at:  None,
            created_at:        Utc::now(),
            platform_name:     Some("GitHub".to_string()),
        };

        let json    = serde_json::to_string(&dto).unwrap();
        let decoded: AccountDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.alias, decoded.alias);
        assert_eq!(dto.auth_method, decoded.auth_method);
        assert_eq!(dto.is_default, decoded.is_default);
    }
}