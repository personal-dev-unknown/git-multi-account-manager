// crates/gm_plugin_gitlab/src/auth_provider.rs
//
// GitLab implementation of the `AuthProvider` outbound port.
//
// The `AuthProvider` trait is defined in gm_ports and provides two operations:
//   1. `validate_credential` — tests a PAT against the GitLab API
//   2. `refresh_token`       — refreshes an OAuth token (not supported in v1 PAT flow)
//
// The provider receives the PAT as a plaintext string. The kernel's CredentialService
// handles all encryption/decryption; by the time validate_credential() is called,
// the caller has already decrypted the stored credential.

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::auth_provider::{AuthProvider, AuthResult, TokenRefreshResult};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::GitLabClient;

/// GitLab implementation of `AuthProvider`.
/// Created during `on_load()` and registered in the service registry.
#[allow(dead_code)]
pub struct GitLabAuthProvider {
    client: Arc<GitLabClient>,
    creds:  Arc<CredentialService>,
}

impl GitLabAuthProvider {
    pub fn new(client: Arc<GitLabClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }
}

#[async_trait]
impl AuthProvider for GitLabAuthProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::GitLab
    }

    /// Validates a GitLab credential against the REST API v4.
    ///
    /// Supports two credential formats:
    ///   - Personal Access Token (PAT): a plain token string (no colon).
    ///     Validated via the `PRIVATE-TOKEN` header.
    ///   - Password auth: "username:password" format (contains a colon).
    ///     Validated via HTTP Basic auth.
    ///
    /// The `credential` parameter is the plaintext credential as entered by the user
    /// or decrypted from the credential store by the caller. This method never stores
    /// credentials — that responsibility stays with the kernel's CredentialService.
    ///
    /// A 401/403 response is returned as `is_valid: false` rather than an Err, so the
    /// command handler can show a clean "invalid credential" message without treating
    /// it as a system failure.
    async fn validate_credential(
        &self,
        _account_uuid: Uuid,
        credential:    &str,
    ) -> Result<AuthResult, GitManagerError> {
        // Detect credential type: "username:password" format contains a colon
        if credential.contains(':') {
            tracing::debug!("validating GitLab password credential via Basic auth");
            self.client.validate_password(credential).await
        } else {
            tracing::debug!("validating GitLab PAT via /user endpoint");
            self.client.validate_token(credential).await
        }
    }

    /// GitLab OAuth token refresh.
    ///
    /// PAT-based accounts never expire (unless set by an admin policy) and do not
    /// have refresh tokens. OAuth-based accounts use a refresh token to obtain a
    /// new access token when the current one expires.
    ///
    /// The refresh flow calls GitLab's OAuth token endpoint:
    ///   POST https://gitlab.com/oauth/token
    ///   grant_type=refresh_token&refresh_token={token}&client_id={id}&client_secret={secret}
    ///
    /// OAuth app credentials (client_id, client_secret) are not stored per-account;
    /// they belong to the Git Manager application registration on GitLab. For v1
    /// only PAT auth is supported, so this returns an error directing users to
    /// generate a new PAT if their token is revoked.
    async fn refresh_token(
        &self,
        _account_uuid:  Uuid,
        _refresh_token: &str,
    ) -> Result<TokenRefreshResult, GitManagerError> {
        // PAT tokens cannot be refreshed — the user must generate a new one.
        // OAuth refresh support is planned for a future release and will require
        // an OAuth application registered on GitLab with a configured callback URL.
        Err(GitManagerError::Other(
            "GitLab PAT tokens cannot be refreshed automatically. \
             Please generate a new Personal Access Token at \
             https://gitlab.com/-/user_settings/personal_access_tokens \
             and update your account credentials.".to_string()
        ))
    }
}