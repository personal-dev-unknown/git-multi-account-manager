use reqwest::Client;
use serde::Deserialize;
use gm_shared::errors::GitManagerError;
use gm_ports::outbound::repository_provider::RemoteRepositoryInfo;
use gm_ports::outbound::auth_provider::AuthResult;

#[derive(Debug, Deserialize)]
pub struct SourceForgeProjectResponse {
    pub name:        String,
    pub summary:     Option<String>,
    pub created:     Option<String>,
    pub url:         Option<String>,
}

#[derive(Debug, Deserialize)]
struct SourceForgeProjectsResponse {
    projects: Vec<SourceForgeProjectResponse>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SourceForgeClient {
    http:     Client,
    base_url: String,
    ssh_host: String,
}

impl SourceForgeClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent(concat!("git-zyrix/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client construction failed");
        Self {
            http,
            base_url: "https://sourceforge.net/rest".to_string(),
            ssh_host: "git.code.sf.net".to_string(),
        }
    }

    fn ssh_url_for(project: &str) -> String {
        format!("git@git.code.sf.net:p/{project}/git.git")
    }

    fn https_url_for(project: &str) -> String {
        format!("https://git.code.sf.net/p/{project}/git.git")
    }

    pub async fn validate_token(&self, _token: &str) -> Result<AuthResult, GitManagerError> {
        // SourceForge does not have a PAT-based API.
        // Authentication is via SSH keys only.
        Ok(AuthResult {
            is_valid:   true,
            username:   None,
            expires_at: None,
        })
    }

    /// Lists public projects for a SourceForge user.
    /// SourceForge API: GET /rest/projects?username={username}
    pub async fn list_projects(
        &self,
        username: &str,
    ) -> Result<Vec<SourceForgeProjectResponse>, GitManagerError> {
        let url = format!("{}/projects?username={}", self.base_url, username);
        let resp = self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("SourceForge list projects failed: {e}")))?;

        if resp.status().is_success() {
            let body: SourceForgeProjectsResponse = resp.json().await
                .map_err(|e| GitManagerError::Other(format!("SourceForge parse failed: {e}")))?;
            Ok(body.projects)
        } else {
            Err(self.api_error(resp).await)
        }
    }

    /// Fetches details for a single SourceForge project.
    pub async fn get_project(
        &self,
        project_name: &str,
    ) -> Result<SourceForgeProjectResponse, GitManagerError> {
        let url = format!("{}/projects/{}", self.base_url, project_name);
        let resp = self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("SourceForge get project failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<SourceForgeProjectResponse>()
                .await
                .map_err(|e| GitManagerError::Other(format!("SourceForge project parse failed: {e}")))
        } else {
            Err(self.api_error(resp).await)
        }
    }

    async fn api_error(&self, resp: reqwest::Response) -> GitManagerError {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        GitManagerError::Other(format!("SourceForge API {status}: {body}"))
    }
}

impl Default for SourceForgeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceForgeProjectResponse {
    pub fn to_remote_info(&self, username: &str) -> RemoteRepositoryInfo {
        let project_name = &self.name;
        RemoteRepositoryInfo {
            full_name:        format!("{}/{}", username, project_name),
            name:             project_name.clone(),
            description:      self.summary.clone(),
            clone_url_ssh:    SourceForgeClient::ssh_url_for(project_name),
            clone_url_https:  SourceForgeClient::https_url_for(project_name),
            default_branch:   "master".to_string(),
            is_private:       false,
            is_forked:        false,
            is_archived:      false,
            stargazers:       0,
            forks:            0,
            open_issues:      0,
            primary_language: None,
        }
    }
}
