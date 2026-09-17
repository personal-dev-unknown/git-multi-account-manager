//! Authentication method — how an account authenticates with its platform.

use gm_shared::models::account::AuthMethod as SharedAuthMethod;

/// The authentication mechanism used by an account to prove identity to
/// the Git hosting platform. This drives which credentials are stored and
/// which transport flags are set when executing git commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthMethod {
    /// SSH key pair. The private key path is stored in the linked SshKey record.
    /// git operations use GIT_SSH_COMMAND with IdentitiesOnly=yes to select
    /// the correct key and prevent identity leakage to the SSH agent.
    Ssh,

    /// HTTPS with a Personal Access Token. The PAT is stored encrypted via
    /// the platform credential store. Git operations inject it as the password
    /// in the HTTPS URL (or via the GIT_ASKPASS mechanism).
    HttpsPat,

    /// HTTPS with a username and password. Legacy; PAT is strongly preferred
    /// because platforms are deprecating password authentication for git operations.
    HttpsPassword,

    /// OAuth 2.0 PKCE flow. Access and refresh tokens are stored encrypted.
    /// Enables creating repositories and managing team memberships through
    /// the platform API in addition to git operations.
    OAuth,

    /// No authentication. Only usable for cloning public repositories.
    /// Push operations will always fail for Anonymous accounts.
    Anonymous,
}

impl AuthMethod {
    /// Returns true if this auth method relies on SSH keys for git operations.
    /// When true, the SshKey and SshHostConfig records are required.
    pub fn uses_ssh(&self) -> bool {
        matches!(self, AuthMethod::Ssh)
    }

    /// Returns true if this auth method requires a stored credential
    /// (PAT, password, or OAuth token) in the platform credential store.
    pub fn requires_stored_credential(&self) -> bool {
        matches!(
            self,
            AuthMethod::HttpsPat | AuthMethod::HttpsPassword | AuthMethod::OAuth
        )
    }

    /// Converts to the shared DTO variant for transport to interface layers.
    pub fn to_shared(&self) -> SharedAuthMethod {
        match self {
            AuthMethod::Ssh           => SharedAuthMethod::Ssh,
            AuthMethod::HttpsPat      => SharedAuthMethod::HttpsPat,
            AuthMethod::HttpsPassword => SharedAuthMethod::HttpsPassword,
            AuthMethod::OAuth         => SharedAuthMethod::OAuth,
            AuthMethod::Anonymous     => SharedAuthMethod::Anonymous,
        }
    }

    /// Reconstructs from the shared DTO variant — called during account rehydration.
    pub fn from_shared(shared: &SharedAuthMethod) -> Self {
        match shared {
            SharedAuthMethod::Ssh           => AuthMethod::Ssh,
            SharedAuthMethod::HttpsPat      => AuthMethod::HttpsPat,
            SharedAuthMethod::HttpsPassword => AuthMethod::HttpsPassword,
            SharedAuthMethod::OAuth         => AuthMethod::OAuth,
            SharedAuthMethod::Anonymous     => AuthMethod::Anonymous,
        }
    }

    /// Returns the string slug stored in the database `auth_method` column.
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthMethod::Ssh           => "ssh",
            AuthMethod::HttpsPat      => "https_pat",
            AuthMethod::HttpsPassword => "https_password",
            AuthMethod::OAuth         => "oauth",
            AuthMethod::Anonymous     => "anonymous",
        }
    }

    /// Parses the database slug back to a domain AuthMethod.
    /// Returns `None` for unrecognised values rather than panicking —
    /// unknown values in the database indicate a forward-migration situation
    /// where the database was written by a newer version of the application.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ssh"            => Some(AuthMethod::Ssh),
            "https_pat"      => Some(AuthMethod::HttpsPat),
            "https_password" => Some(AuthMethod::HttpsPassword),
            "oauth"          => Some(AuthMethod::OAuth),
            "anonymous"      => Some(AuthMethod::Anonymous),
            _                => None,
        }
    }
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}