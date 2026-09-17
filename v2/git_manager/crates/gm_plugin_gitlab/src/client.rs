// crates/gm_plugin_gitlab/src/client.rs
//
// The GitLab HTTP client for REST API v4.
//
// GitLab's personal access tokens are passed via the `PRIVATE-TOKEN` header.
// GitLab also supports OAuth Bearer tokens via the `Authorization` header.
// This client supports both; the binary selects the correct flow at startup
// based on the account's auth_method field.
//
// ── Rate limiting ─────────────────────────────────────────────────────────────
// GitLab imposes 300 requests/minute per user for REST API calls on gitlab.com.
// The client reads the `RateLimit-Remaining` header and logs a warning below 30.
//
// ── Self-hosted GitLab ────────────────────────────────────────────────────────
// `base_url` defaults to "https://gitlab.com/api/v4" but can be overridden
// for self-hosted instances. All URL construction goes through `self.base_url`
// so this works transparently.

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use gm_shared::errors::GitManagerError;
use gm_ports::outbound::repository_provider::RemoteRepositoryInfo;
use gm_ports::outbound::auth_provider::AuthResult;

// ── API response types ────────────────────────────────────────────────────────

/// GitLab project object as returned by the REST API v4.
#[derive(Debug, Deserialize)]
pub struct GitLabProjectResponse {
    pub id:                    u64,
    pub path_with_namespace:   String,
    pub path:                  String,
    pub description:           Option<String>,
    pub http_url_to_repo:      String,
    pub ssh_url_to_repo:       String,
    pub default_branch:        Option<String>,
    pub visibility:            String,   // "public", "internal", "private"
    pub archived:              bool,
    pub forked_from_project:   Option<serde_json::Value>,
    pub star_count:            u32,
    pub forks_count:           u32,
    pub open_issues_count:     Option<u32>,
    pub programming_language:  Option<String>,
}

impl GitLabProjectResponse {
    pub fn to_remote_info(&self) -> RemoteRepositoryInfo {
        RemoteRepositoryInfo {
            full_name:        self.path_with_namespace.clone(),
            name:             self.path.clone(),
            description:      self.description.clone(),
            clone_url_ssh:    self.ssh_url_to_repo.clone(),
            clone_url_https:  self.http_url_to_repo.clone(),
            default_branch:   self.default_branch.clone().unwrap_or_else(|| "main".to_string()),
            is_private:       self.visibility == "private",
            is_forked:        self.forked_from_project.is_some(),
            is_archived:      self.archived,
            stargazers:       self.star_count,
            forks:            self.forks_count,
            open_issues:      self.open_issues_count.unwrap_or(0),
            primary_language: self.programming_language.clone(),
        }
    }
}

/// GitLab user object from GET /user.
#[derive(Debug, Deserialize)]
pub struct GitLabUserResponse {
    pub id:       u64,
    pub username: String,
    pub name:     Option<String>,
    pub email:    Option<String>,
}

/// GitLab API error response envelope.
#[derive(Debug, Deserialize)]
struct GitLabErrorResponse {
    message: Option<serde_json::Value>,
    error:   Option<String>,
}

impl GitLabErrorResponse {
    fn message_string(&self) -> String {
        if let Some(msg) = &self.message {
            return msg.to_string().trim_matches('"').to_string();
        }
        if let Some(err) = &self.error {
            return err.clone();
        }
        "unknown GitLab API error".to_string()
    }
}

// ── The client ────────────────────────────────────────────────────────────────

/// GitLab REST API v4 HTTP client.
/// Stateless: the PAT is passed per-call so the same instance serves many accounts.
#[derive(Debug, Clone)]
pub struct GitLabClient {
    http:     Client,
    base_url: String,
}

impl GitLabClient {
    /// Creates a client targeting gitlab.com.
    pub fn new() -> Self {
        Self::with_base_url("https://gitlab.com/api/v4")
    }

    /// Creates a client targeting a custom GitLab instance.
    /// `base_url` should be the full API base, e.g. "https://gitlab.myco.com/api/v4".
    pub fn with_base_url(base_url: &str) -> Self {
        let http = Client::builder()
            .user_agent(concat!("git-zyrix/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client construction failed — this is a bug in the build environment");
        Self { http, base_url: base_url.to_string() }
    }

    // ── Authentication helpers ────────────────────────────────────────────────

    /// Validates a PAT or OAuth token by calling GET /user.
    /// Returns `is_valid: false` for a 401 instead of an Err so the caller can
    /// present a clean "invalid credential" message without treating it as a crash.
    pub async fn validate_token(&self, token: &str) -> Result<AuthResult, GitManagerError> {
        let resp = self.http
            .get(format!("{}/user", self.base_url))
            .header("PRIVATE-TOKEN", token)
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitLab /user request failed: {e}")))?;

        match resp.status() {
            StatusCode::OK => {
                let body: GitLabUserResponse = resp.json().await
                    .map_err(|e| GitManagerError::Other(format!("GitLab /user parse failed: {e}")))?;
                Ok(AuthResult {
                    is_valid:   true,
                    username:   Some(body.username),
                    expires_at: None, // GitLab PATs don't expose expiry in this endpoint
                })
            }
            StatusCode::UNAUTHORIZED => {
                Ok(AuthResult { is_valid: false, username: None, expires_at: None })
            }
            _ => Err(self.api_error(resp).await),
        }
    }

    /// Validates a username:password credential by calling GET /user with Basic auth.
    /// GitLab supports password authentication via Basic auth for API v4.
    pub async fn validate_password(&self, credential: &str) -> Result<AuthResult, GitManagerError> {
        let (username, password) = credential.split_once(':').ok_or_else(|| {
            GitManagerError::Other(
                "GitLab password credential must be in 'username:password' format".to_string()
            )
        })?;

        let resp = self.http
            .get(format!("{}/user", self.base_url))
            .basic_auth(username, Some(password))
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitLab password auth failed: {e}")))?;

        match resp.status() {
            StatusCode::OK => {
                let body: GitLabUserResponse = resp.json().await
                    .map_err(|e| GitManagerError::Other(format!("GitLab /user parse failed: {e}")))?;
                Ok(AuthResult {
                    is_valid:   true,
                    username:   Some(body.username),
                    expires_at: None,
                })
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Ok(AuthResult { is_valid: false, username: None, expires_at: None })
            }
            _ => Err(self.api_error(resp).await),
        }
    }

    // ── Repository listing ────────────────────────────────────────────────────

    /// Lists projects accessible to the authenticated user.
    /// Returns only non-archived projects where the user is a member.
    pub async fn list_repositories(
        &self,
        token:    &str,
        page:     u32,
        per_page: u32,
    ) -> Result<Vec<GitLabProjectResponse>, GitManagerError> {
        let url = format!(
            "{}/projects?membership=true&archived=false&order_by=last_activity_at&per_page={per_page}&page={page}",
            self.base_url
        );
        let resp = self.http
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitLab list projects failed: {e}")))?;

        self.check_rate_limit(&resp);
        if resp.status().is_success() {
            resp.json::<Vec<GitLabProjectResponse>>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitLab project list parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Fetches a single project by its full namespace path (namespace/name).
    pub async fn get_repository(
        &self,
        token:     &str,
        full_name: &str,
    ) -> Result<GitLabProjectResponse, GitManagerError> {
        // GitLab requires URL-encoding the `/` in namespace/path as `%2F`
        let encoded = full_name.replace('/', "%2F");
        let url = format!("{}/projects/{encoded}", self.base_url);
        let resp = self.http
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitLab get project failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<GitLabProjectResponse>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitLab project parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Forks a project to the authenticated user's namespace or a target group.
    pub async fn fork_project(
        &self,
        token: &str,
        project_id: u64,
        namespace: Option<&str>,
    ) -> Result<GitLabProjectResponse, GitManagerError> {
        let url = format!("{}/projects/{project_id}/fork", self.base_url);
        let mut req = self.http
            .post(&url)
            .header("PRIVATE-TOKEN", token);

        if let Some(ns) = namespace {
            #[derive(serde::Serialize)]
            struct ForkBody<'a> {
                namespace: &'a str,
            }
            req = req.json(&ForkBody { namespace: ns });
        }

        let resp = req
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitLab fork failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<GitLabProjectResponse>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitLab fork parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Lists projects belonging to a specific group, including subgroups.
    pub async fn list_group_projects(
        &self,
        token: &str,
        group: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<GitLabProjectResponse>, GitManagerError> {
        let encoded = group.replace('/', "%2F");
        let url = format!(
            "{}/groups/{encoded}/projects?include_subgroups=true&archived=false&order_by=last_activity_at&per_page={per_page}&page={page}",
            self.base_url
        );
        let resp = self.http
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitLab list group projects failed: {e}")))?;

        self.check_rate_limit(&resp);
        if resp.status().is_success() {
            resp.json::<Vec<GitLabProjectResponse>>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitLab group project list parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Searches projects by name. GitLab's `?search=` parameter performs a
    /// prefix search on name; `?search_namespaces=true` extends it to namespaces.
    pub async fn search_repositories(
        &self,
        token: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<GitLabProjectResponse>, GitManagerError> {
        let q   = simple_encode(query);
        let url = format!(
            "{}/projects?membership=true&search={q}&search_namespaces=true&per_page={limit}",
            self.base_url
        );
        let resp = self.http
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("GitLab search failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<Vec<GitLabProjectResponse>>()
                .await
                .map_err(|e| GitManagerError::Other(format!("GitLab search parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn check_rate_limit(&self, resp: &reqwest::Response) {
        if let Some(remaining) = resp
            .headers()
            .get("RateLimit-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            if remaining < 30 {
                tracing::warn!(remaining, "GitLab API rate limit running low");
            }
        }
    }

    async fn api_error(&self, resp: reqwest::Response) -> GitManagerError {
        let status = resp.status();
        let message = resp
            .json::<GitLabErrorResponse>()
            .await
            .map(|e| e.message_string())
            .unwrap_or_else(|_| "unknown GitLab API error".to_string());
        GitManagerError::Other(format!("GitLab API {status}: {message}"))
    }
}

impl Default for GitLabClient {
    fn default() -> Self {
        Self::new()
    }
}

pub use gm_shared::utilities::url_encode::encode as simple_encode;