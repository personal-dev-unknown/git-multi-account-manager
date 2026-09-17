// crates/gm_adapters/src/git/repository_analyzer.rs
//
// RepositoryAnalyzer implementation that probes remote repos via `git ls-remote`.
// Runs in spawn_blocking to avoid starving Tokio.

use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Instant;

use gm_domain::git::entities::repository_analysis::{RepoSizeClass, RepositoryAnalysis};
use gm_domain::git::ports::repository_analyzer::{BandwidthClass, ConnectionQuality, RepositoryAnalyzer};
use gm_shared::errors::GitError;

/// Analyzer that uses `git ls-remote` to inspect remote repositories.
/// Does not require SSH isolation since it only reads refs.
#[derive(Debug, Default)]
pub struct GitRepositoryAnalyzer;

impl GitRepositoryAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RepositoryAnalyzer for GitRepositoryAnalyzer {
    async fn analyze(&self, url: &str, _ssh_key_path: Option<PathBuf>) -> Result<RepositoryAnalysis, GitError> {
        let url = url.to_string();
        tokio::task::spawn_blocking(move || -> Result<RepositoryAnalysis, GitError> {
            let output = std::process::Command::new("git")
                .args(["ls-remote", "--heads", "--tags", &url])
                .output()
                .map_err(|e| GitError::CloneFailed {
                    reason: format!("failed to run git ls-remote: {e}"),
                })?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !output.status.success() {
                return Ok(RepositoryAnalysis {
                    url: url.clone(),
                    estimated_size: None,
                    size_class: RepoSizeClass::Tiny,
                    branch_count: None,
                    tag_count: None,
                    is_reachable: false,
                    is_public: true,
                    requires_auth: stderr.contains("Authentication failed")
                        || stderr.contains("Permission denied")
                        || stderr.contains("Repository not found"),
                    default_branch: None,
                    summary: format!("remote unreachable: {}", stderr.trim()),
                });
            }

            let mut branches = Vec::new();
            let mut tags = Vec::new();

            for line in stdout.lines() {
                let line = line.trim();
                if let Some(ref_path) = line.split_whitespace().nth(1) {
                    if ref_path.starts_with("refs/heads/") {
                        branches.push(ref_path.trim_start_matches("refs/heads/").to_string());
                    } else if ref_path.starts_with("refs/tags/") {
                        tags.push(ref_path.trim_start_matches("refs/tags/").to_string());
                    }
                }
            }

            let default_branch = branches.iter().find(|b| *b == "main" || *b == "master")
                .or_else(|| branches.first())
                .cloned();

            let branch_count = Some(branches.len() as u32);
            let tag_count = Some(tags.len() as u32);

            // Estimate size from ref count (rough heuristic)
            let estimated_size = Some((branches.len() as u64 + tags.len() as u64) * 50_000);
            let size_class = RepoSizeClass::from_bytes(estimated_size.unwrap_or(0));

            Ok(RepositoryAnalysis {
                url: url.clone(),
                estimated_size,
                size_class,
                branch_count,
                tag_count,
                is_reachable: true,
                is_public: stderr.is_empty(),
                requires_auth: false,
                default_branch,
                summary: format!(
                    "{} branches, {} tags, reachable={}",
                    branches.len(),
                    tags.len(),
                    true,
                ),
            })
        })
        .await
        .map_err(|e| GitError::CloneFailed { reason: format!("spawn_blocking join error: {e}") })?
    }

    async fn probe_connection(&self, url: &str) -> Result<ConnectionQuality, GitError> {
        let url = url.to_string();
        let host = extract_host(&url).unwrap_or_else(|| "unknown".to_string());

        tokio::task::spawn_blocking(move || -> Result<ConnectionQuality, GitError> {
            let start = Instant::now();

            // Quick TCP ping to estimate latency (try port 22 for SSH, 443 for HTTPS)
            let addr = format!("{}:{}", host, 22);
            let tcp_result = addr.parse::<std::net::SocketAddr>()
                .ok()
                .and_then(|sa| std::net::TcpStream::connect_timeout(&sa, std::time::Duration::from_secs(5)).ok())
                .or_else(|| {
                    let addr = format!("{}:{}", host, 443);
                    addr.parse::<std::net::SocketAddr>().ok().and_then(|sa| {
                        std::net::TcpStream::connect_timeout(&sa, std::time::Duration::from_secs(5)).ok()
                    })
                });

            let latency_ms = tcp_result.map(|_| start.elapsed().as_millis() as u64);

            // Probe via git ls-remote to get a data point for bandwidth estimation
            let probe_result = std::process::Command::new("git")
                .args(["ls-remote", "--heads", &url])
                .output();

            match probe_result {
                Ok(output) if output.status.success() => {
                    let elapsed = start.elapsed();
                    let data_bytes = output.stdout.len() + output.stderr.len();
                    let bandwidth_bps = if elapsed.as_secs() > 0 {
                        Some(data_bytes as u64 / std::cmp::max(1, elapsed.as_secs() as u64))
                    } else {
                        None
                    };
                    Ok(ConnectionQuality::from_bandwidth(bandwidth_bps.unwrap_or(1_000_000)))
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // Even on auth failure, connection quality may be good
                    Ok(ConnectionQuality {
                        latency_ms,
                        bandwidth_bps: None,
                        bandwidth_class: BandwidthClass::Broadband,
                        is_reachable: true,
                        requires_auth: stderr.contains("Authentication failed")
                            || stderr.contains("Permission denied"),
                    })
                }
                Err(e) => {
                    tracing::warn!("connection probe failed for {url}: {e}");
                    Ok(ConnectionQuality::unreachable())
                }
            }
        })
        .await
        .map_err(|e| GitError::CloneFailed { reason: format!("spawn_blocking join error: {e}") })?
    }

    async fn ping(&self, url: &str) -> bool {
        self.probe_connection(url).await
            .map(|q| q.is_reachable)
            .unwrap_or(false)
    }
}

fn extract_host(url: &str) -> Option<String> {
    if let Some(rest) = url.strip_prefix("https://") {
        rest.split('/').next().map(|s| s.to_string())
    } else if let Some(rest) = url.strip_prefix("http://") {
        rest.split('/').next().map(|s| s.to_string())
    } else if url.starts_with("git@") {
        url.split('@').nth(1)
            .and_then(|s| s.split(':').next())
            .map(|s| s.to_string())
    } else {
        None
    }
}
