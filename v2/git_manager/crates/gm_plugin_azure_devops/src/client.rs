// crates/gm_plugin_azure_devops/src/client.rs
//
// Azure DevOps REST API 7.0 HTTP client.
//
// ── Authentication ────────────────────────────────────────────────────────────
// Azure DevOps REST API uses HTTP Basic authentication with an empty username
// and the Personal Access Token as the password. The PAT is Base64-encoded in
// the standard Basic auth scheme: base64(":{pat}")
//
// The credential stored in the vault is the PAT only (no username prefix).
// The client constructs the Basic auth header internally.
//
// ── Organisation and Project ─────────────────────────────────────────────────
// Azure DevOps structures resources as:
//   organization → project → repository
//
// The credential in the vault may optionally include the organization and project
// as a prefix: "organization/project:pat". The client parses this if present,
// or the account's `ssh_host_alias` encodes the org/project.
//
// For simplicity in v1, the credential format is "organization:pat" and repository
// listing retrieves all projects within that organisation.
//
// ── API Version ───────────────────────────────────────────────────────────────
// All requests use api-version=7.0 which is the current stable version.

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use gm_shared::errors::GitManagerError;
use gm_ports::outbound::repository_provider::RemoteRepositoryInfo;
use gm_ports::outbound::auth_provider::AuthResult;

// ── Response types ────────────────────────────────────────────────────────────

/// Azure DevOps Git repository object.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzdoRepoResponse {
    pub id:            String,
    pub name:          String,
    pub project:       AzdoProject,
    pub remote_url:    String,
    pub ssh_url:       Option<String>,
    pub default_branch: Option<String>,
    pub is_disabled:   Option<bool>,
    pub is_fork:       Option<bool>,
    pub size:          Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct AzdoProject {
    pub id:    String,
    pub name:  String,
    pub state: Option<String>,
}

impl AzdoRepoResponse {
    pub fn to_remote_info(&self, organization: &str) -> RemoteRepositoryInfo {
        let full_name   = format!("{}/{}/{}", organization, self.project.name, self.name);
        let ssh_url     = self.ssh_url.clone()
            .unwrap_or_else(|| format!(
                "git@ssh.dev.azure.com:v3/{}/{}/{}",
                organization, self.project.name, self.name
            ));

        RemoteRepositoryInfo {
            full_name,
            name:             self.name.clone(),
            description:      None,
            clone_url_ssh:    ssh_url,
            clone_url_https:  self.remote_url.clone(),
            default_branch:   self.default_branch.clone().unwrap_or_else(|| "main".to_string()),
            is_private:       true,  // ADO repos are private by default
            is_forked:        self.is_fork.unwrap_or(false),
            is_archived:      self.is_disabled.unwrap_or(false),
            stargazers:       0,
            forks:            0,
            open_issues:      0,
            primary_language: None,
        }
    }
}

/// ADO response envelope for repository list.
#[derive(Debug, Deserialize)]
pub struct AzdoRepoListResponse {
    pub count: u32,
    pub value: Vec<AzdoRepoResponse>,
}

/// ADO project list response.
#[derive(Debug, Deserialize)]
pub struct AzdoProjectListResponse {
    pub count: u32,
    pub value: Vec<AzdoProject>,
}

/// ADO profile response from /_apis/profile/profiles/me
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzdoProfileResponse {
    pub display_name: Option<String>,
    pub public_alias: Option<String>,
    pub email_address: Option<String>,
}

// ── The client ────────────────────────────────────────────────────────────────

/// Azure DevOps REST API 7.0 HTTP client.
#[derive(Debug, Clone)]
pub struct AzureDevOpsClient {
    http: Client,
}

impl AzureDevOpsClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent(concat!("git-zyrix/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client construction failed");
        Self { http }
    }

    // ── Credential helpers ────────────────────────────────────────────────────

    /// Splits "organization:pat" credential into components.
    /// Returns an error if the format is wrong.
    pub fn split_credential<'a>(credential: &'a str) -> Result<(&'a str, &'a str), GitManagerError> {
        credential.split_once(':').ok_or_else(|| {
            GitManagerError::Other(
                "Azure DevOps credential must be 'organization:pat' format. \
                 Create a PAT at https://dev.azure.com → User Settings → Personal Access Tokens."
                    .to_string()
            )
        })
    }

    // ── API methods ───────────────────────────────────────────────────────────

    /// Validates a PAT by calling the ADO profile API.
    pub async fn validate_token(&self, credential: &str) -> Result<AuthResult, GitManagerError> {
        let (org, pat) = Self::split_credential(credential)?;
        // The profile endpoint is organisation-independent
        let url = format!(
            "https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=7.0"
        );
        let resp = self.http
            .get(&url)
            .basic_auth("", Some(pat))
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Azure DevOps profile request failed: {e}")))?;

        match resp.status() {
            StatusCode::OK => {
                let body: AzdoProfileResponse = resp.json().await.map_err(|e| {
                    GitManagerError::Other(format!("Azure DevOps profile parse failed: {e}"))
                })?;
                let username = body.display_name
                    .or(body.public_alias)
                    .or(body.email_address)
                    .unwrap_or_else(|| format!("{org}_user"));
                Ok(AuthResult { is_valid: true, username: Some(username), expires_at: None })
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Ok(AuthResult { is_valid: false, username: None, expires_at: None })
            }
            _ => Err(self.api_error(resp).await),
        }
    }

    /// Lists all repositories across all projects in the organization.
    pub async fn list_repositories(
        &self,
        credential: &str,
    ) -> Result<Vec<(AzdoRepoResponse, String)>, GitManagerError> {
        let (org, pat) = Self::split_credential(credential)?;

        // First, list all projects in the organization
        let projects_url = format!(
            "https://dev.azure.com/{org}/_apis/projects?api-version=7.0&$top=200"
        );
        let proj_resp = self.http
            .get(&projects_url)
            .basic_auth("", Some(pat))
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Azure DevOps list projects failed: {e}")))?;

        if !proj_resp.status().is_success() {
            return Err(self.api_error(proj_resp).await);
        }

        let projects: AzdoProjectListResponse = proj_resp.json().await
            .map_err(|e| GitManagerError::Other(format!("Azure DevOps projects parse failed: {e}")))?;

        let mut all_repos: Vec<(AzdoRepoResponse, String)> = Vec::new();

        // For each project, list its repositories
        for project in projects.value.iter().filter(|p| p.state.as_deref() == Some("wellFormed")) {
            let repos_url = format!(
                "https://dev.azure.com/{org}/{project}/_apis/git/repositories?api-version=7.0",
                project = project.name
            );
            let repos_resp = self.http
                .get(&repos_url)
                .basic_auth("", Some(pat))
                .send()
                .await
                .map_err(|e| GitManagerError::Other(format!("Azure DevOps list repos failed: {e}")))?;

            if repos_resp.status().is_success() {
                let list: AzdoRepoListResponse = repos_resp.json().await
                    .map_err(|e| GitManagerError::Other(format!("Azure DevOps repos parse failed: {e}")))?;
                for repo in list.value {
                    all_repos.push((repo, org.to_string()));
                }
            }
        }

        Ok(all_repos)
    }

    /// Fetches a single repository by full name "organization/project/repo".
    pub async fn get_repository(
        &self,
        credential: &str,
        full_name:  &str,
    ) -> Result<(AzdoRepoResponse, String), GitManagerError> {
        let (org, pat) = Self::split_credential(credential)?;
        let parts: Vec<&str> = full_name.splitn(3, '/').collect();
        if parts.len() < 3 {
            return Err(GitManagerError::Other(format!(
                "Azure DevOps full_name must be 'org/project/repo', got: {full_name}"
            )));
        }
        let (project, repo) = (parts[1], parts[2]);
        let url = format!(
            "https://dev.azure.com/{org}/{project}/_apis/git/repositories/{repo}?api-version=7.0"
        );
        let resp = self.http
            .get(&url)
            .basic_auth("", Some(pat))
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Azure DevOps get repo failed: {e}")))?;

        if resp.status().is_success() {
            let repo: AzdoRepoResponse = resp.json().await
                .map_err(|e| GitManagerError::Other(format!("Azure DevOps repo parse failed: {e}")))?;
            Ok((repo, org.to_string()))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Forks a repository by creating a new repository with the source as parent.
    /// `full_name` is "org/project/repo". The fork is created in the same project
    /// with a name of "fork-{repo}" unless `target_name` is provided.
    pub async fn fork_repository(
        &self,
        credential:  &str,
        full_name:   &str,
        target_name: Option<&str>,
    ) -> Result<(AzdoRepoResponse, String), GitManagerError> {
        let (org, pat) = Self::split_credential(credential)?;
        let parts: Vec<&str> = full_name.splitn(3, '/').collect();
        if parts.len() < 3 {
            return Err(GitManagerError::Other(format!(
                "Azure DevOps full_name must be 'org/project/repo', got: {full_name}"
            )));
        }
        let (src_project, src_repo) = (parts[1], parts[2]);
        let default_name = format!("fork-{src_repo}");
        let fork_name = target_name.unwrap_or(&default_name);

        // First get the source repo to obtain its ID
        let get_url = format!(
            "https://dev.azure.com/{org}/{src_project}/_apis/git/repositories/{src_repo}?api-version=7.0"
        );
        let get_resp = self.http
            .get(&get_url)
            .basic_auth("", Some(pat))
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Azure DevOps get source repo failed: {e}")))?;

        if !get_resp.status().is_success() {
            return Err(self.api_error(get_resp).await);
        }
        let source_repo: AzdoRepoResponse = get_resp.json().await
            .map_err(|e| GitManagerError::Other(format!("Azure DevOps source repo parse failed: {e}")))?;

        // Create a new repo with the source as parent
        #[derive(serde::Serialize)]
        struct CreateForkRequest {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            parent_repository: Option<ParentRepoRef>,
        }
        #[derive(serde::Serialize)]
        struct ParentRepoRef {
            id: String,
        }
        let body = CreateForkRequest {
            name: fork_name.to_string(),
            parent_repository: Some(ParentRepoRef {
                id: source_repo.id.clone(),
            }),
        };

        let create_url = format!(
            "https://dev.azure.com/{org}/{src_project}/_apis/git/repositories?api-version=7.0"
        );
        let create_resp = self.http
            .post(&create_url)
            .basic_auth("", Some(pat))
            .json(&body)
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("Azure DevOps fork failed: {e}")))?;

        if create_resp.status().is_success() {
            let repo: AzdoRepoResponse = create_resp.json().await
                .map_err(|e| GitManagerError::Other(format!("Azure DevOps fork parse failed: {e}")))?;
            Ok((repo, org.to_string()))
        } else {
            Err(self.api_error(create_resp).await)
        }
    }

    /// Searches repositories by name across all projects.
    pub async fn search_repositories(
        &self,
        credential: &str,
        query:      &str,
        limit:      u32,
    ) -> Result<Vec<(AzdoRepoResponse, String)>, GitManagerError> {
        // ADO doesn't have a simple repo search — we list all and filter locally.
        let all = self.list_repositories(credential).await?;
        let q   = query.to_lowercase();
        Ok(all
            .into_iter()
            .filter(|(repo, _)| repo.name.to_lowercase().contains(&q))
            .take(limit as usize)
            .collect())
    }

    async fn api_error(&self, resp: reqwest::Response) -> GitManagerError {
        let status = resp.status();
        let body   = resp.text().await.unwrap_or_else(|_| "unknown error".to_string());
        GitManagerError::Other(format!("Azure DevOps API {status}: {body}"))
    }
}

impl Default for AzureDevOpsClient {
    fn default() -> Self { Self::new() }
}