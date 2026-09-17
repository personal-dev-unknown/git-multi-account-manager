// crates/gm_plugin_azure_devops/src/auth_provider.rs
//
// Implements gm_ports::outbound::AuthProvider for Azure DevOps.
//
// ── Why Azure DevOps authentication is different ──────────────────────────────
// Azure DevOps PATs use a peculiar HTTP Basic Auth convention: the username
// field is intentionally left empty and the entire PAT is placed in the
// password field. This means the Base64-encoded Authorization header looks like:
//
//     base64(":" + pat)
//
// This is exactly the pattern our BitbucketClient.basic_auth() helper implements,
// except Bitbucket uses the account's real username as the first component.
// For Azure DevOps the credential stored in CredentialService is "org:pat"
// where the org is the Azure DevOps organisation slug.
//
// ── Validation endpoint ───────────────────────────────────────────────────────
// Azure DevOps REST API profile endpoint (VSSPS — Visual Studio Services
// Profile Service) validates the PAT:
//
//     GET https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=6.0
//
// A 200 response with a profile JSON object confirms the PAT is valid. A 401
// means invalid or expired. A 203 (Non-Authoritative Information) means the
// request was redirected for Microsoft account sign-in, which indicates the
// organisation requires interactive authentication — this is treated as failure.
//
// ── Token refresh ─────────────────────────────────────────────────────────────
// Azure DevOps PATs can be given a maximum lifetime of 1 year but cannot be
// refreshed via the REST API. Users must generate new PATs in the Azure DevOps
// portal. We surface this via a descriptive error message that links to the
// correct settings page.

use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use gm_kernel::security::CredentialService;
use gm_ports::outbound::auth_provider::{AuthProvider, AuthResult, TokenRefreshResult};
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;

use crate::auth::split_azure_credential;
use crate::client::AzureDevOpsClient;

/// Validates Azure DevOps PAT credentials by calling the VSSPS profile API.
/// Credentials are stored in the format "org:pat".
#[allow(dead_code)]
#[derive(Debug)]
pub struct AzureDevOpsAuthProvider {
    client: Arc<AzureDevOpsClient>,
    creds:  Arc<CredentialService>,
}

impl AzureDevOpsAuthProvider {
    pub fn new(client: Arc<AzureDevOpsClient>, creds: Arc<CredentialService>) -> Self {
        Self { client, creds }
    }
}

#[async_trait]
impl AuthProvider for AzureDevOpsAuthProvider {
    fn platform_type(&self) -> PlatformType {
        PlatformType::AzureDevOps
    }

    /// Validates an "org:pat" credential against the Azure DevOps profile API.
    ///
    /// The `credential` parameter must be in "org:pat" format. The organisation
    /// slug is used when calling the repository listing API; the PAT alone is
    /// sufficient for the profile validation endpoint.
    async fn validate_credential(
        &self,
        _account_uuid: Uuid,
        credential:    &str,
    ) -> Result<AuthResult, GitManagerError> {
        let (org, _pat) = split_azure_credential(credential);

        if org.is_empty() {
            return Err(GitManagerError::Other(
                "Azure DevOps credential must be in 'organisation:pat' format. \
                 Example: mycompany:abcd1234...".to_string(),
            ));
        }

        // The client splits the credential internally to construct the Basic auth header.
        self.client.validate_token(credential).await
    }

    /// Azure DevOps PATs cannot be refreshed programmatically.
    ///
    /// The Azure DevOps REST API has no token-refresh endpoint for PATs.
    /// When a PAT expires, the account transitions to TokenExpired status
    /// and the user must generate a new PAT via the Azure DevOps portal.
    async fn refresh_token(
        &self,
        _account_uuid:  Uuid,
        _refresh_token: &str,
    ) -> Result<TokenRefreshResult, GitManagerError> {
        Err(GitManagerError::Other(
            "Azure DevOps PATs cannot be refreshed programmatically. \
             Please generate a new PAT at https://dev.azure.com/<org>/_usersSettings/tokens \
             and update it with: git-zyrix account update-token --alias <alias>".to_string(),
        ))
    }
}