// crates/gm_ports/src/outbound/auth_provider.rs
//
// The AuthProvider outbound port — how a platform plugin authenticates an
// account. Each platform supports different auth flows (PAT, OAuth PKCE, SSH).
// The provider handles the flow and returns a credential that the kernel stores
// in the CredentialVault.

use async_trait::async_trait;
use uuid::Uuid;
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

/// The outcome of a successful authentication attempt.
#[derive(Debug, Clone)]
pub struct AuthResult {
    /// The encrypted credential value — produced by the kernel vault before calling authenticate.
    /// The provider validates the plaintext credential; the kernel stores the ciphertext.
    pub is_valid:       bool,
    /// The platform-confirmed username (e.g. "shakamoses").
    pub username:       Option<String>,
    /// When the credential expires. None for non-expiring credentials (static PATs).
    pub expires_at:     Option<chrono::DateTime<chrono::Utc>>,
}

/// The result of refreshing an OAuth token.
#[derive(Debug, Clone)]
pub struct TokenRefreshResult {
    pub new_access_token:  String,
    pub new_refresh_token: Option<String>,
    pub expires_at:        chrono::DateTime<chrono::Utc>,
}

/// Platform-specific authentication operations.
/// Implemented by each provider plugin.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    fn platform_type(&self) -> PlatformType;

    /// Validates whether the provided credential (PAT or password) is accepted
    /// by the platform API. Returns the platform-confirmed username on success.
    async fn validate_credential(
        &self,
        account_uuid: Uuid,
        credential:   &str,
    ) -> Result<AuthResult, GitManagerError>;

    /// Refreshes an expired OAuth access token using a stored refresh token.
    /// Returns the new tokens so the kernel can re-encrypt and store them.
    async fn refresh_token(
        &self,
        account_uuid:  Uuid,
        refresh_token: &str,
    ) -> Result<TokenRefreshResult, GitManagerError>;
}