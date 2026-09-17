// crates/gm_plugin_github/src/client.rs
//
// The GitHub HTTP client. This struct wraps a reqwest::Client and provides
// typed, authenticated methods for the specific GitHub REST API v3 endpoints
// the plugin needs. It is stateless regarding credentials — the caller passes
// the PAT on each call. This design means the same client instance can serve
// multiple accounts without any state mutation between calls.
//
// ── Rate limiting awareness ───────────────────────────────────────────────────
// GitHub's primary REST API allows 5 000 requests/hour for authenticated users.
// The client reads the X-RateLimit-Remaining header and logs a warning when
// it falls below 100. No automatic retry or back-off is implemented in v1;
// the workflow engine's step retry logic handles transient 429 responses.
//
// ── Error handling ────────────────────────────────────────────────────────────
// HTTP-level errors (network, TLS) become GitManagerError::Other.
// GitHub API errors (non-200 status with a JSON body) are converted to
// descriptive GitManagerError::Other messages that include the status code
// and the API's "message" field.

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use gm_shared::errors::GitManagerError;
use gm_ports::outbound::repository_provider::RemoteRepositoryInfo;
use gm_ports::outbound::auth_provider::AuthResult;

/// Serialised shape of a GitHub repository as returned by the REST API.
#[derive(Debug, Deserialize)]
pub struct GitHubRepoResponse {
    pub id:               u64,
    pub full_name:        String,
    pub name:             String,
    pub description:      Option<String>,
    pub clone_url:        String,
    pub ssh_url:          String,
    pub default_branch:   String,
    pub private:          bool,
    pub fork:             bool,
    pub archived:         bool,
    pub stargazers_count: u32,
    pub forks_count:      u32,
    pub open_issues_count: u32,
    pub language:         Option<String>,
}

impl GitHubRepoResponse {
    pub fn to_remote_info(&self) -> RemoteRepositoryInfo {
        RemoteRepositoryInfo {
            full_name:        self.full_name.clone(),
            name:             self.name.clone(),
            description:      self.description.clone(),
            clone_url_ssh:    self.ssh_url.clone(),
            clone_url_https:  self.clone_url.clone(),
            default_branch:   self.default_branch.clone(),
            is_private:       self.private,
            is_forked:        self.fork,
            is_archived:      self.archived,
            stargazers:       self.stargazers_count,
            forks:            self.forks_count,
            open_issues:      self.open_issues_count,
            primary_language: self.language.clone(),
        }
    }
}

/// Serialised shape of the GitHub user object from GET /user.
#[derive(Debug, Deserialize)]
pub struct GitHubUserResponse {
    pub login: String,
    pub name:  Option<String>,
    pub email: Option<String>,
}

/// GitHub API error response body shape.
#[derive(Debug, Deserialize)]
struct GitHubErrorResponse {
    message:           String,
    #[allow(dead_code)]
    documentation_url: Option<String>,
}

/// The GitHub REST API v3 HTTP client.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    http:     Client,
    base_url: String,
}

impl GitHubClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent(concat!("git-zyrix/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client construction failed — this is a bug");

        Self {
            http,
            base_url: "https://api.github.com".to_string(),
        }
    }

    // ── Authenticated request helpers ─────────────────────────────────────────

    fn auth_header(token: &str) -> String {
        format!("token {token}")
    }

    /// Validates a PAT by calling GET /user and confirming a 200 response.
    pub async fn validate_token(&self, token: &str) -> Result<AuthResult, GitManagerError> {
        let resp = self.http
            .get(format!("{}/user", self.base_url))
            .header("Authorization", Self::auth_header(token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitHub /user request failed: {e}")))?;

        let status = resp.status();
        if status == StatusCode::OK {
            let body: GitHubUserResponse = resp
                .json()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitHub /user parse failed: {e}")))?;

            Ok(AuthResult {
                is_valid:   true,
                username:   Some(body.login),
                expires_at: None, // GitHub PATs don't declare expiry in the API response
            })
        } else if status == StatusCode::UNAUTHORIZED {
            Ok(AuthResult { is_valid: false, username: None, expires_at: None })
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Lists repositories for the authenticated user, paginated.
    pub async fn list_repositories(
        &self,
        token:    &str,
        page:     u32,
        per_page: u32,
    ) -> Result<Vec<GitHubRepoResponse>, GitManagerError> {
        let url = format!(
            "{}/user/repos?type=all&sort=updated&per_page={per_page}&page={page}",
            self.base_url
        );
        let resp = self.http
            .get(&url)
            .header("Authorization", Self::auth_header(token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitHub list repos failed: {e}")))?;

        self.check_rate_limit(&resp);
        if resp.status().is_success() {
            resp.json::<Vec<GitHubRepoResponse>>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitHub repo parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Fetches a single repository by its full name (owner/repo).
    pub async fn get_repository(
        &self,
        token:     &str,
        full_name: &str,
    ) -> Result<GitHubRepoResponse, GitManagerError> {
        let url = format!("{}/repos/{full_name}", self.base_url);
        let resp = self.http
            .get(&url)
            .header("Authorization", Self::auth_header(token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitHub get repo failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<GitHubRepoResponse>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitHub repo parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Forks a repository to the authenticated user's account or a target organization.
    pub async fn fork_repository(
        &self,
        token: &str,
        full_name: &str,
        organization: Option<&str>,
    ) -> Result<GitHubRepoResponse, GitManagerError> {
        let url = format!("{}/repos/{full_name}/forks", self.base_url);
        let mut req = self.http
            .post(&url)
            .header("Authorization", Self::auth_header(token))
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(org) = organization {
            #[derive(serde::Serialize)]
            struct ForkBody<'a> {
                organization: &'a str,
            }
            req = req.json(&ForkBody { organization: org });
        }

        let resp = req
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitHub fork failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<GitHubRepoResponse>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitHub fork parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Lists repositories for a specific organization.
    pub async fn list_organization_repositories(
        &self,
        token: &str,
        org: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<GitHubRepoResponse>, GitManagerError> {
        let url = format!(
            "{}/orgs/{org}/repos?type=all&sort=updated&per_page={per_page}&page={page}",
            self.base_url
        );
        let resp = self.http
            .get(&url)
            .header("Authorization", Self::auth_header(token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitHub list org repos failed: {e}")))?;

        self.check_rate_limit(&resp);
        if resp.status().is_success() {
            resp.json::<Vec<GitHubRepoResponse>>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitHub org repo parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Lists repositories the authenticated user has starred.
    pub async fn list_starred_repositories(
        &self,
        token: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<GitHubRepoResponse>, GitManagerError> {
        let url = format!(
            "{}/user/starred?sort=updated&direction=desc&per_page={per_page}&page={page}",
            self.base_url
        );
        let resp = self.http
            .get(&url)
            .header("Authorization", Self::auth_header(token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitHub list starred failed: {e}")))?;

        self.check_rate_limit(&resp);
        if resp.status().is_success() {
            resp.json::<Vec<GitHubRepoResponse>>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitHub starred parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Searches repositories using the GitHub search API.
    pub async fn search_repositories(
        &self,
        token: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<GitHubRepoResponse>, GitManagerError> {
        #[derive(Deserialize)]
        struct SearchResult {
            items: Vec<GitHubRepoResponse>,
        }
        let q    = urlencoding::encode(query);
        let url  = format!("{}/search/repositories?q={q}&per_page={limit}", self.base_url);
        let resp = self.http
            .get(&url)
            .header("Authorization", Self::auth_header(token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitHub search failed: {e}")))?;

        if resp.status().is_success() {
            let result = resp.json::<SearchResult>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitHub search parse: {e}")))?;
            Ok(result.items)
        } else {
            Err(self.api_error(resp).await)
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn check_rate_limit(&self, resp: &reqwest::Response) {
        if let Some(remaining) = resp
            .headers()
            .get("X-RateLimit-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            if remaining < 100 {
                tracing::warn!(remaining, "GitHub API rate limit running low");
            }
        }
    }

    async fn api_error(&self, resp: reqwest::Response) -> GitManagerError {
        let status  = resp.status();
        let message = resp
            .json::<GitHubErrorResponse>()
            .await
            .map(|e| e.message)
            .unwrap_or_else(|_| "unknown GitHub API error".to_string());
        GitManagerError::Other(format!("GitHub API {status}: {message}"))
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

pub use gm_shared::utilities::url_encode as urlencoding;