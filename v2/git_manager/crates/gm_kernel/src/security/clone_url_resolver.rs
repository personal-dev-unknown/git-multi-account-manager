use std::sync::Arc;
use uuid::Uuid;

use gm_shared::errors::GitManagerError;

use crate::security::{AuthStrategy, CredentialService};

/// Rewrites a git remote URL to embed the authentication mechanism required by
/// the chosen strategy.
///
/// Strategy → URL transformation:
///   - Ssh:         `git@<host>-<alias>:<owner>/<repo>.git`
///   - HttpsPat:    `https://token@host/owner/repo.git`
///   - HttpsPassword: `https://user:password@host/owner/repo.git`
///   - OAuth:       `https://oauth2:token@host/owner/repo.git`
///   - Anonymous:   Strips any existing credentials from the URL
///
/// Also handles bare repository names (e.g. `owner/repo`) by constructing a
/// provider-specific full URL based on the active platform and auth strategy.
#[derive(Debug)]
pub struct CloneUrlResolver {
    cred_service: Arc<CredentialService>,
}

impl CloneUrlResolver {
    pub fn new(cred_service: Arc<CredentialService>) -> Self {
        Self { cred_service }
    }

    /// Resolve a URL-or-owner/repo string into a fully qualified clone URL
    /// for the given auth strategy, platform, and account.
    pub async fn resolve(
        &self,
        url: &str,
        strategy: &AuthStrategy,
        account_uuid: Option<Uuid>,
    ) -> Result<String, GitManagerError> {
        match strategy {
            AuthStrategy::Ssh => Ok(url.to_string()),
            AuthStrategy::HttpsPat | AuthStrategy::HttpsPassword => {
                let credential = if let Some(uuid) = account_uuid {
                    self.cred_service.get_token(uuid).await?.unwrap_or_default()
                } else {
                    String::new()
                };
                if credential.is_empty() {
                    return Ok(self.strip_credentials(url));
                }
                Ok(self.inject_credential(url, &credential))
            }
            AuthStrategy::OAuth => {
                let token = if let Some(uuid) = account_uuid {
                    self.cred_service.get_token(uuid).await?.unwrap_or_default()
                } else {
                    String::new()
                };
                if token.is_empty() {
                    return Ok(self.strip_credentials(url));
                }
                Ok(self.inject_oauth_token(url, &token))
            }
            AuthStrategy::Anonymous => Ok(self.strip_credentials(url)),
        }
    }

    /// Build a fully-qualified clone URL from a bare repo name or URL.
    ///
    /// - **SSH strategy** — HTTPS URLs are converted to SSH with the account
    ///   alias embedded in the host (e.g. `https://github.com/owner/repo.git`
    ///   → `git@github.com-{alias}:owner/repo.git`). Bare names like
    ///   `owner/repo` produce `git@github.com-{alias}:owner/repo.git`.
    ///   Pre-existing SSH URLs (`git@…`) are returned unchanged.
    ///
    /// - **Other strategies** — HTTPS URLs are returned as-is (callers should
    ///   use `resolve()` for credential injection). Bare names produce
    ///   `https://{host}/owner/repo.git`.
    pub fn build_url(
        input: &str,
        strategy: &AuthStrategy,
        platform_hostname: &str,
        account_alias: &str,
    ) -> String {
        // SSH strategy: convert HTTPS URLs to SSH with account alias.
        // The host is extracted from the URL itself so cross-platform
        // collaborator URLs (e.g. a self-hosted GitLab URL with a GitHub
        // account) produce correct SSH host strings.
        if matches!(strategy, AuthStrategy::Ssh)
            && (input.starts_with("https://") || input.starts_with("http://"))
        {
            let rest = input.trim_start_matches("https://")
                .trim_start_matches("http://");
            if let Some(slash) = rest.find('/') {
                let host = &rest[..slash];
                let path = &rest[slash + 1..];
                return format!("git@{}-{}:{}", host, account_alias, path);
            }
        }

        // If it's already a full URL, return as-is.
        if input.contains("://") || input.starts_with("git@") {
            return input.to_string();
        }

        let bare = input.trim_end_matches(".git");

        match strategy {
            AuthStrategy::Ssh => {
                format!("git@{}-{}:{}.git", platform_hostname, account_alias, bare)
            }
            _ => {
                format!("https://{}/{}.git", platform_hostname, bare)
            }
        }
    }

    fn inject_credential(&self, url: &str, credential: &str) -> String {
        if let Some(rest) = url.strip_prefix("https://") {
            format!("https://{credential}@{rest}")
        } else if let Some(rest) = url.strip_prefix("http://") {
            format!("http://{credential}@{rest}")
        } else {
            url.to_string()
        }
    }

    fn inject_oauth_token(&self, url: &str, token: &str) -> String {
        if let Some(rest) = url.strip_prefix("https://") {
            format!("https://oauth2:{token}@{rest}")
        } else {
            url.to_string()
        }
    }

    fn strip_credentials(&self, url: &str) -> String {
        if let Some(rest) = url.strip_prefix("https://") {
            if let Some(at_pos) = rest.find('@') {
                format!("https://{}", &rest[at_pos + 1..])
            } else {
                url.to_string()
            }
        } else if let Some(rest) = url.strip_prefix("http://") {
            if let Some(at_pos) = rest.find('@') {
                format!("http://{}", &rest[at_pos + 1..])
            } else {
                url.to_string()
            }
        } else {
            url.to_string()
        }
    }
}
