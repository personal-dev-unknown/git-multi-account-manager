// crates/gm_plugin_bitbucket/src/client.rs
//
// Bitbucket REST API 2.0 HTTP client.
//
// ── Authentication ────────────────────────────────────────────────────────────
// Bitbucket uses HTTP Basic authentication with the user's Bitbucket username
// and an "App Password" (not the account password). The credential stored in
// the vault is "username:app_password" — colon-separated — which this client
// splits and encodes into the Authorization header.
//
// App Passwords are created at https://bitbucket.org/account/settings/app-passwords/
// with the required permissions: Repositories:Read, Account:Read.
//
// ── Pagination ────────────────────────────────────────────────────────────────
// Bitbucket uses cursor-based pagination via the `next` URL in the response body.
// This client uses page number + pagelen for simplicity in v1.

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use gm_shared::errors::GitManagerError;
use gm_ports::outbound::repository_provider::RemoteRepositoryInfo;
use gm_ports::outbound::auth_provider::AuthResult;

// ── Response types ────────────────────────────────────────────────────────────

/// Bitbucket repository object from the 2.0 API.
#[derive(Debug, Deserialize)]
pub struct BitbucketRepoResponse {
    pub full_name:   String,
    pub name:        Option<String>,
    pub description: Option<String>,
    pub is_private:  bool,
    pub language:    Option<String>,
    #[serde(rename = "mainbranch")]
    pub mainbranch:  Option<BitbucketBranch>,
    pub links:       BitbucketRepoLinks,
    #[serde(rename = "parent")]
    pub parent:      Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketBranch {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketRepoLinks {
    pub clone: Option<Vec<BitbucketCloneLink>>,
}

#[derive(Debug, Deserialize)]
pub struct BitbucketCloneLink {
    pub href: String,
    pub name: String,  // "https" or "ssh"
}

impl BitbucketRepoResponse {
    pub fn to_remote_info(&self) -> RemoteRepositoryInfo {
        let slug      = self.full_name.split('/').next_back().unwrap_or(&self.full_name);
        let name      = self.name.clone().unwrap_or_else(|| slug.to_string());
        let branch    = self.mainbranch.as_ref().map(|b| b.name.clone()).unwrap_or_else(|| "main".to_string());
        let is_forked = self.parent.is_some();

        let (ssh_url, https_url) = self.links.clone
            .as_ref()
            .map(|links| {
                let ssh   = links.iter().find(|l| l.name == "ssh").map(|l| l.href.clone()).unwrap_or_default();
                let https = links.iter().find(|l| l.name == "https").map(|l| l.href.clone()).unwrap_or_default();
                (ssh, https)
            })
            .unwrap_or_default();

        RemoteRepositoryInfo {
            full_name:        self.full_name.clone(),
            name,
            description:      self.description.clone(),
            clone_url_ssh:    ssh_url,
            clone_url_https:  https_url,
            default_branch:   branch,
            is_private:       self.is_private,
            is_forked,
            is_archived:      false, // Bitbucket doesn't have an archived flag in v1
            stargazers:       0,     // Bitbucket doesn't expose star counts in the list API
            forks:            0,
            open_issues:      0,
            primary_language: self.language.clone(),
        }
    }
}

/// Bitbucket paginated response wrapper.
#[derive(Debug, Deserialize)]
pub struct BitbucketPageResponse {
    pub values:  Vec<BitbucketRepoResponse>,
    pub next:    Option<String>,
    pub size:    Option<u32>,
}

/// Bitbucket user object from GET /user.
#[derive(Debug, Deserialize)]
pub struct BitbucketUserResponse {
    pub username:     Option<String>,
    pub nickname:     Option<String>,
    pub account_id:   Option<String>,
    pub display_name: Option<String>,
}

impl BitbucketUserResponse {
    pub fn effective_username(&self) -> String {
        self.username.clone()
            .or_else(|| self.nickname.clone())
            .or_else(|| self.display_name.clone())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

// ── The client ────────────────────────────────────────────────────────────────

/// Bitbucket REST API 2.0 HTTP client.
#[derive(Debug, Clone)]
pub struct BitbucketClient {
    http:     Client,
    base_url: String,
}

impl BitbucketClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent(concat!("git-zyrix/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client construction failed");
        Self {
            http,
            base_url: "https://api.bitbucket.org/2.0".to_string(),
        }
    }

    // ── Auth helpers ──────────────────────────────────────────────────────────

    /// Splits a "username:app_password" credential into its components.
    /// Returns an error if the credential is not in the expected format.
    fn split_credential(credential: &str) -> Result<(&str, &str), GitManagerError> {
        credential.split_once(':').ok_or_else(|| {
            GitManagerError::Other(
                "Bitbucket credential must be in 'username:app_password' format. \
                 Create an App Password at https://bitbucket.org/account/settings/app-passwords/"
                    .to_string()
            )
        })
    }

    // ── API methods ───────────────────────────────────────────────────────────

    /// Validates the credential by calling GET /user with Basic auth.
    pub async fn validate_token(&self, credential: &str) -> Result<AuthResult, GitManagerError> {
        let (username, password) = Self::split_credential(credential)?;

        let resp = self.http
            .get(format!("{}/user", self.base_url))
            .basic_auth(username, Some(password))
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Bitbucket /user request failed: {e}")))?;

        match resp.status() {
            StatusCode::OK => {
                let body: BitbucketUserResponse = resp.json().await
                    .map_err(|e| GitManagerError::Other(format!("Bitbucket /user parse failed: {e}")))?;
                Ok(AuthResult {
                    is_valid:   true,
                    username:   Some(body.effective_username()),
                    expires_at: None,
                })
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Ok(AuthResult { is_valid: false, username: None, expires_at: None })
            }
            _ => Err(self.api_error(resp).await),
        }
    }

    /// Lists repositories for the authenticated user.
    /// Uses the workspace derived from the username portion of the credential.
    pub async fn list_repositories(
        &self,
        credential: &str,
        page:       u32,
        per_page:   u32,
    ) -> Result<BitbucketPageResponse, GitManagerError> {
        let (username, password) = Self::split_credential(credential)?;
        let url = format!(
            "{}/repositories/{username}?role=member&pagelen={per_page}&page={page}",
            self.base_url
        );
        let resp = self.http
            .get(&url)
            .basic_auth(username, Some(password))
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Bitbucket list repos failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<BitbucketPageResponse>()
                .await
                .map_err(|e| GitManagerError::Other(format!("Bitbucket repo list parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Fetches a single repository by workspace and slug.
    /// `full_name` is in "workspace/slug" format.
    pub async fn get_repository(
        &self,
        credential: &str,
        full_name:  &str,
    ) -> Result<BitbucketRepoResponse, GitManagerError> {
        let (username, password) = Self::split_credential(credential)?;
        let url = format!("{}/repositories/{full_name}", self.base_url);
        let resp = self.http
            .get(&url)
            .basic_auth(username, Some(password))
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Bitbucket get repo failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<BitbucketRepoResponse>()
                .await
                .map_err(|e| GitManagerError::Other(format!("Bitbucket repo parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Forks a repository to the authenticated user's workspace or a target workspace.
    pub async fn fork_repository(
        &self,
        credential: &str,
        full_name:  &str,
        target_workspace: Option<&str>,
    ) -> Result<BitbucketRepoResponse, GitManagerError> {
        let (username, password) = Self::split_credential(credential)?;
        let url = format!("{}/repositories/{full_name}/forks", self.base_url);

        let mut req = self.http
            .post(&url)
            .basic_auth(username, Some(password));

        if let Some(workspace) = target_workspace {
            let body = serde_json::json!({
                "workspace": { "slug": workspace }
            });
            req = req.json(&body);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Bitbucket fork failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<BitbucketRepoResponse>()
                .await
                .map_err(|e| GitManagerError::Other(format!("Bitbucket fork parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Lists repositories for a specific team/workspace.
    pub async fn list_team_repositories(
        &self,
        credential: &str,
        team:       &str,
        page:       u32,
        per_page:   u32,
    ) -> Result<BitbucketPageResponse, GitManagerError> {
        let (username, password) = Self::split_credential(credential)?;
        let url = format!(
            "{}/repositories/{team}?pagelen={per_page}&page={page}",
            self.base_url
        );
        let resp = self.http
            .get(&url)
            .basic_auth(username, Some(password))
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Bitbucket list team repos failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<BitbucketPageResponse>()
                .await
                .map_err(|e| GitManagerError::Other(format!("Bitbucket team repo list parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Searches repositories by name within the user's workspaces.
    pub async fn search_repositories(
        &self,
        credential: &str,
        query:      &str,
        limit:      u32,
    ) -> Result<Vec<BitbucketRepoResponse>, GitManagerError> {
        let (username, password) = Self::split_credential(credential)?;
        let q   = simple_encode(query);
        let url = format!(
            "{}/repositories/{username}?q=name~\"{q}\"&pagelen={limit}",
            self.base_url
        );
        let resp = self.http
            .get(&url)
            .basic_auth(username, Some(password))
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Bitbucket search failed: {e}")))?;

        if resp.status().is_success() {
            let page: BitbucketPageResponse = resp.json().await
                .map_err(|e| GitManagerError::Other(format!("Bitbucket search parse failed: {e}")))?;
            Ok(page.values)
        } else {
            Err(self.api_error(resp).await)
        }
    }

    async fn api_error(&self, resp: reqwest::Response) -> GitManagerError {
        let status = resp.status();
        let body   = resp.text().await.unwrap_or_else(|_| "unknown error".to_string());
        GitManagerError::Other(format!("Bitbucket API {status}: {body}"))
    }
}

impl Default for BitbucketClient {
    fn default() -> Self {
        Self::new()
    }
}

pub use gm_shared::utilities::url_encode::encode as simple_encode;