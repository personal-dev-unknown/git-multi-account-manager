use std::sync::Arc;
use uuid::Uuid;

use gm_ports::outbound::platform_repository::PlatformRepository;
use gm_shared::errors::GitManagerError;

use crate::security::CredentialService;

/// The authentication strategies available for git operations.
/// Ordered from most secure / preferred to least.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthStrategy {
    /// SSH key pair with `GIT_SSH_COMMAND` isolation.
    Ssh,
    /// HTTPS with Personal Access Token stored in the credential vault.
    HttpsPat,
    /// HTTPS with username + password stored in the credential vault.
    HttpsPassword,
    /// OAuth 2.0 access / refresh tokens.
    OAuth,
    /// No credentials — public repositories only (clone-only).
    Anonymous,
}

impl AuthStrategy {
    /// Returns true if this strategy can be used to push to a remote.
    pub fn supports_push(&self) -> bool {
        !matches!(self, AuthStrategy::Anonymous)
    }

    /// Returns true if this strategy is a credential-based HTTPS method.
    pub fn uses_https_auth(&self) -> bool {
        matches!(self, AuthStrategy::HttpsPat | AuthStrategy::HttpsPassword)
    }

    /// Returns true if this strategy requires an SSH key on disk.
    pub fn requires_ssh_key(&self) -> bool {
        matches!(self, AuthStrategy::Ssh)
    }
}

impl std::fmt::Display for AuthStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthStrategy::Ssh => write!(f, "SSH"),
            AuthStrategy::HttpsPat => write!(f, "HTTPS PAT"),
            AuthStrategy::HttpsPassword => write!(f, "HTTPS Password"),
            AuthStrategy::OAuth => write!(f, "OAuth"),
            AuthStrategy::Anonymous => write!(f, "Anonymous"),
        }
    }
}

/// The type of operation being authenticated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    Clone,
    Pull,
    Push,
    Fetch,
    ApiRead,
    ApiWrite,
}

/// A resolved authentication strategy with ordered fallback options.
#[derive(Debug, Clone)]
pub struct ResolvedStrategy {
    /// The primary strategy to attempt first.
    pub primary: AuthStrategy,
    /// Ordered fallback strategies if the primary fails (SSH→PAT→Password→Anonymous).
    pub fallbacks: Vec<AuthStrategy>,
}

impl ResolvedStrategy {
    /// Returns all strategies in priority order.
    pub fn all(&self) -> Vec<AuthStrategy> {
        let mut all = vec![self.primary.clone()];
        all.extend(self.fallbacks.clone());
        all
    }
}

/// Resolves the optimal authentication strategy for a given account and operation.
///
/// Selection logic (in priority order):
///   1. SSH — if the account has an active SSH key
///   2. HTTPS PAT — if a credential is stored and the platform supports PATs
///   3. HTTPS Password — if a credential is stored and the platform supports HTTPS
///   4. OAuth — if OAuth tokens are available
///   5. Anonymous — no credentials needed (public clone only)
///
/// Fallback chain: SSH → PAT → Password → Anonymous
pub struct AuthStrategyResolver {
    platform_repo: Arc<dyn PlatformRepository>,
    cred_service:  Arc<CredentialService>,
}

impl std::fmt::Debug for AuthStrategyResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthStrategyResolver")
            .field("platform_repo", &"<PlatformRepository>")
            .field("cred_service", &self.cred_service)
            .finish()
    }
}

impl AuthStrategyResolver {
    pub fn new(
        platform_repo: Arc<dyn PlatformRepository>,
        cred_service:  Arc<CredentialService>,
    ) -> Self {
        Self { platform_repo, cred_service }
    }

    /// Resolves the best authentication strategy for the given account and operation.
    ///
    /// If `has_ssh_key` is true but no credential is stored, SSH is preferred.
    /// If `has_ssh_key` is false but a credential is stored, HTTPS PAT is preferred.
    /// If neither is available, Anonymous is used (clone-only).
    pub async fn resolve(
        &self,
        platform_id:   Uuid,
        has_ssh_key:   bool,
        has_credential: bool,
        operation:     OperationKind,
    ) -> Result<ResolvedStrategy, GitManagerError> {
        let platform = self.platform_repo.find_by_uuid(platform_id).await?
            .ok_or_else(|| GitManagerError::Other(format!("Platform {platform_id} not found")))?;

        let primary = self.resolve_primary(has_ssh_key, has_credential, &platform, operation);

        let fallbacks = self.build_fallbacks(
            &primary, has_ssh_key, has_credential, &platform, operation,
        );

        Ok(ResolvedStrategy { primary, fallbacks })
    }

    fn resolve_primary(
        &self,
        has_ssh_key: bool,
        has_credential: bool,
        platform: &gm_shared::models::platform::PlatformDto,
        _operation: OperationKind,
    ) -> AuthStrategy {
        if has_ssh_key {
            return AuthStrategy::Ssh;
        }
        if has_credential && platform.supports_pat {
            return AuthStrategy::HttpsPat;
        }
        if has_credential && platform.supports_https {
            return AuthStrategy::HttpsPassword;
        }
        if has_credential {
            return AuthStrategy::HttpsPat;
        }
        AuthStrategy::Anonymous
    }

    fn build_fallbacks(
        &self,
        primary: &AuthStrategy,
        has_ssh_key: bool,
        has_credential: bool,
        platform: &gm_shared::models::platform::PlatformDto,
        operation: OperationKind,
    ) -> Vec<AuthStrategy> {
        let mut fallbacks = Vec::new();

        if *primary != AuthStrategy::Ssh && has_ssh_key {
            fallbacks.push(AuthStrategy::Ssh);
        }
        if *primary != AuthStrategy::HttpsPat && has_credential && platform.supports_pat {
            fallbacks.push(AuthStrategy::HttpsPat);
        }
        if *primary != AuthStrategy::HttpsPassword && has_credential && platform.supports_https {
            fallbacks.push(AuthStrategy::HttpsPassword);
        }
        if *primary != AuthStrategy::Anonymous && operation == OperationKind::Clone {
            fallbacks.push(AuthStrategy::Anonymous);
        }

        fallbacks
    }
}
