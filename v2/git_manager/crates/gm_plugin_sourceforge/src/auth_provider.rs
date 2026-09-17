use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::auth_provider::{AuthProvider, AuthResult, TokenRefreshResult};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::SourceForgeClient;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SourceForgeAuthProvider {
    client: Arc<SourceForgeClient>,
    creds:  Arc<CredentialService>,
}

impl SourceForgeAuthProvider {
    pub fn new(client: Arc<SourceForgeClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }
}

#[async_trait]
impl AuthProvider for SourceForgeAuthProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::SourceForge
    }

    async fn validate_credential(
        &self,
        _account_uuid: Uuid,
        credential: &str,
    ) -> Result<AuthResult, GitManagerError> {
        // SourceForge validates via SSH key, not PAT. If a credential
        // was stored (an SSH key fingerprint) we consider it valid.
        if credential.is_empty() {
            return Ok(AuthResult {
                is_valid: false,
                username: None,
                expires_at: None,
            });
        }
        Ok(AuthResult {
            is_valid: true,
            username: None,
            expires_at: None,
        })
    }

    async fn refresh_token(
        &self,
        _account_uuid: Uuid,
        _refresh_token: &str,
    ) -> Result<TokenRefreshResult, GitManagerError> {
        Err(GitManagerError::Other(
            "SourceForge does not support token refresh. \
             Use SSH key authentication instead.".to_string()
        ))
    }
}
