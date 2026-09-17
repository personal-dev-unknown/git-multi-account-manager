use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::auth_provider::{AuthProvider, AuthResult, TokenRefreshResult};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::client::CustomClient;

#[allow(dead_code)]
#[derive(Debug)]
pub struct CustomAuthProvider {
    client: Arc<CustomClient>,
    creds:  Arc<CredentialService>,
}

impl CustomAuthProvider {
    pub fn new(client: Arc<CustomClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }
}

#[async_trait]
impl AuthProvider for CustomAuthProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::SelfHosted("custom".to_string())
    }

    async fn validate_credential(
        &self,
        _account_uuid: Uuid,
        credential: &str,
    ) -> Result<AuthResult, GitManagerError> {
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
            "Self-hosted platforms do not support token refresh. \
             Use SSH key authentication.".to_string()
        ))
    }
}
