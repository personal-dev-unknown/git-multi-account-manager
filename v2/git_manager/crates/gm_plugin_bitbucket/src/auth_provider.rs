// crates/gm_plugin_bitbucket/src/auth_provider.rs
//
// Implements gm_ports::outbound::AuthProvider for Bitbucket Cloud.
//
// ── Bitbucket authentication model ───────────────────────────────────────────
// Bitbucket Cloud does not support Personal Access Tokens in the same sense as
// GitHub or GitLab. Instead it uses "App Passwords" — account-scoped secrets
// that are combined with the Bitbucket username for HTTP Basic Auth:
//
//     Authorization: Basic base64("username:app_password")
//
// The credential stored in our CredentialService for a Bitbucket account is
// therefore the combined string "username:app_password" rather than a bare
// token. The `auth::split_credential()` helper separates them before use.
//
// ── Why not OAuth? ────────────────────────────────────────────────────────────
// Bitbucket does support OAuth 2.0, but implementing the PKCE flow for a CLI
// tool requires a local HTTP callback server and browser interaction, which is
// a significant UX and security complexity overhead. App Passwords achieve the
// same access scope with a simpler setup flow: create a password in the Bitbucket
// UI, paste it into git-manager, done. We will add OAuth as an optional auth
// method in a future version.
//
// ── Validation strategy ───────────────────────────────────────────────────────
// We call `GET https://api.bitbucket.org/2.0/user` using HTTP Basic Auth with the
// split credential. A 200 response confirms validity and returns the username.
// A 401 means the credential is incorrect or the app password has been revoked.

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::auth_provider::{AuthProvider, AuthResult, TokenRefreshResult};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::BitbucketClient;

/// Validates Bitbucket App Password credentials by calling the Bitbucket REST
/// API with HTTP Basic Auth. The stored credential has the format "username:app_password".
#[allow(dead_code)]
#[derive(Debug)]
pub struct BitbucketAuthProvider {
    client: Arc<BitbucketClient>,
    creds:  Arc<CredentialService>,
}

impl BitbucketAuthProvider {
    pub fn new(client: Arc<BitbucketClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }
}

#[async_trait]
impl AuthProvider for BitbucketAuthProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::Bitbucket
    }

    /// Validates a "username:app_password" credential against the Bitbucket API.
    ///
    /// The `credential` parameter must be in the format "username:app_password".
    /// This matches the format stored by the CredentialService for Bitbucket
    /// accounts. If the colon separator is missing the entire string is treated
    /// as the password and the username is treated as empty — which will fail
    /// validation with a descriptive error rather than a silent 401.
    async fn validate_credential(
        &self,
        _account_uuid: Uuid,
        credential:    &str,
    ) -> Result<AuthResult, GitManagerError> {
        self.client.validate_token(credential).await
    }

    /// App Passwords cannot be refreshed — they are static credentials.
    ///
    /// When a Bitbucket App Password is revoked (e.g. by the user rotating
    /// security credentials), the account transitions to TokenExpired status
    /// and the user must create a new App Password in the Bitbucket UI and
     /// update it with: git-zyrix account update-token --alias <alias>
    async fn refresh_token(
        &self,
        _account_uuid:  Uuid,
        _refresh_token: &str,
    ) -> Result<TokenRefreshResult, GitManagerError> {
        Err(GitManagerError::Other(
            "Bitbucket App Passwords cannot be refreshed programmatically. \
             Please create a new App Password at \
             https://bitbucket.org/account/settings/app-passwords/ \
             and update it with: git-zyrix account update-token --alias <alias>".to_string(),
        ))
    }
}