use reqwest::Client;
use serde::Deserialize;
use gm_shared::errors::GitManagerError;
use gm_ports::outbound::repository_provider::RemoteRepositoryInfo;

/// Gitea-compatible API response for a repository.
#[derive(Debug, Deserialize)]
pub struct GiteaRepoResponse {
    pub id:              u64,
    pub name:            String,
    pub full_name:       String,
    pub description:     Option<String>,
    pub clone_url:       String,
    pub ssh_url:         Option<String>,
    pub default_branch:  Option<String>,
    pub private:         bool,
    pub fork:            bool,
    pub archived:        bool,
    pub stargazers_count: Option<u32>,
    pub forks_count:     Option<u32>,
    pub open_issues_count: Option<u32>,
    pub language:        Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GiteaVersionResponse {
    version: String,
}

#[derive(Debug, Clone)]
pub struct CustomClient {
    http: Client,
}

impl CustomClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent(concat!("git-zyrix/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest client construction failed");
        Self { http }
    }

    /// Tests connection to a self-hosted Git server using the Gitea API.
    /// Tries GET /api/v1/version on the given host.
    pub async fn test_connection(&self, host: &str) -> bool {
        let url = format!("https://{}/api/v1/version", host);
        match self.http.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Lists repositories for the authenticated user on a Gitea-compatible server.
    pub async fn list_repositories(
        &self,
        _base_url: &str,
        _token: &str,
        _page: u32,
        _per_page: u32,
    ) -> Result<Vec<GiteaRepoResponse>, GitManagerError> {
        // Requires custom per-host configuration.
        // Returns empty list — self-hosted servers need explicit URL entry.
        Ok(vec![])
    }

    /// Fetches a single repository from a Gitea-compatible server.
    pub async fn get_repository(
        &self,
        base_url: &str,
        _token: &str,
        owner: &str,
        repo: &str,
    ) -> Result<Option<GiteaRepoResponse>, GitManagerError> {
        let url = format!("{}/api/v1/repos/{}/{}", base_url, owner, repo);
        let resp = self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| GitManagerError::Other(format!("self-hosted get repo failed: {e}")))?;

        if resp.status().is_success() {
            resp.json::<GiteaRepoResponse>()
                .await
                .map(Some)
                .map_err(|e| GitManagerError::Other(format!("parse failed: {e}")))
        } else {
            Ok(None)
        }
    }
}

impl Default for CustomClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GiteaRepoResponse {
    pub fn to_remote_info(&self, host: &str) -> RemoteRepositoryInfo {
        let ssh_url = self.ssh_url.clone()
            .unwrap_or_else(|| format!("git@{}:{}.git", host, self.full_name));
        let https_url = format!("https://{}/{}", host, self.full_name);
        RemoteRepositoryInfo {
            full_name:        self.full_name.clone(),
            name:             self.name.clone(),
            description:      self.description.clone(),
            clone_url_ssh:    ssh_url,
            clone_url_https:  https_url,
            default_branch:   self.default_branch.clone().unwrap_or_else(|| "main".to_string()),
            is_private:       self.private,
            is_forked:        self.fork,
            is_archived:      self.archived,
            stargazers:       self.stargazers_count.unwrap_or(0),
            forks:            self.forks_count.unwrap_or(0),
            open_issues:      self.open_issues_count.unwrap_or(0),
            primary_language: self.language.clone(),
        }
    }
}
