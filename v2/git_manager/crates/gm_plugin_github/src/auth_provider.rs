// crates/gm_plugin_github/src/auth_provider.rs
//
// Implements the gm_ports::outbound::AuthProvider trait for GitHub.
// The implementation validates PATs by calling GET /user with the provided
// credential and confirming that GitHub returns a 200 OK. GitHub PATs do not
// support programmatic token refresh, so refresh_token returns an error.

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::auth_provider::{AuthProvider, AuthResult, TokenRefreshResult};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::GitHubClient;

#[allow(dead_code)]
#[derive(Debug)]
pub struct GitHubAuthProvider {
    client:  Arc<GitHubClient>,
    creds:   Arc<CredentialService>,
}

impl GitHubAuthProvider {
    pub fn new(client: Arc<GitHubClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }
}

#[async_trait]
impl AuthProvider for GitHubAuthProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::GitHub
    }

    async fn validate_credential(
        &self,
        _account_uuid: Uuid,
        credential:   &str,
    ) -> Result<AuthResult, GitManagerError> {
        // The caller passes the raw PAT as `credential`. We validate it
        // directly against the GitHub API and let the vault handle storage.
        self.client.validate_token(credential).await
    }

    async fn refresh_token(
        &self,
        _account_uuid:  Uuid,
        _refresh_token: &str,
    ) -> Result<TokenRefreshResult, GitManagerError> {
        // GitHub PATs cannot be refreshed programmatically.
        // Fine-grained tokens expire but GitHub does not provide a refresh API.
        // The user must regenerate their token on github.com.
        Err(GitManagerError::Other(
            "GitHub PATs cannot be refreshed programmatically. \
             Please generate a new token at https://github.com/settings/tokens".to_string()
        ))
    }
}