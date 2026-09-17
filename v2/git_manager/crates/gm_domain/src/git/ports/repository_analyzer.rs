// crates/gm_domain/src/git/ports/repository_analyzer.rs
//
// Outbound port for inspecting a remote repository before cloning.
// The adapter (ZigGitAnalyzer or HttpAnalyzer) talks to the remote via
// `git ls-remote` or the platform API to gather metadata.

use async_trait::async_trait;
use std::path::PathBuf;
use gm_shared::errors::GitError;
use crate::git::entities::repository_analysis::RepositoryAnalysis;
use crate::git::ports::git_executor::CloneOptions;

/// Bandwidth classification based on estimated transfer speed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandwidthClass {
    DialUp,     // < 1 Mbps
    Slow,       // 1–10 Mbps
    Broadband,  // 10–100 Mbps
    Fast,       // > 100 Mbps
}

impl BandwidthClass {
    pub fn is_slow(&self) -> bool {
        matches!(self, BandwidthClass::DialUp | BandwidthClass::Slow)
    }
}

/// Connection quality snapshot gathered before or during clone.
#[derive(Debug, Clone)]
pub struct ConnectionQuality {
    /// Estimated round-trip time to the remote host in milliseconds.
    pub latency_ms:         Option<u64>,
    /// Estimated bandwidth in bytes per second.
    pub bandwidth_bps:      Option<u64>,
    /// Classification derived from bandwidth_bps.
    pub bandwidth_class:    BandwidthClass,
    /// Whether the remote endpoint is reachable at all.
    pub is_reachable:       bool,
    /// Whether authentication is required to list refs.
    pub requires_auth:      bool,
}

impl ConnectionQuality {
    pub fn unknown() -> Self {
        Self {
            latency_ms:      None,
            bandwidth_bps:   None,
            bandwidth_class: BandwidthClass::Broadband,
            is_reachable:    true,
            requires_auth:   false,
        }
    }

    pub fn unreachable() -> Self {
        Self {
            latency_ms:      None,
            bandwidth_bps:   None,
            bandwidth_class: BandwidthClass::Broadband,
            is_reachable:    false,
            requires_auth:   false,
        }
    }

    pub fn from_bandwidth(bps: u64) -> Self {
        let class = if bps < 125_000 {
            BandwidthClass::DialUp
        } else if bps < 1_250_000 {
            BandwidthClass::Slow
        } else if bps < 12_500_000 {
            BandwidthClass::Broadband
        } else {
            BandwidthClass::Fast
        };
        Self {
            latency_ms:      None,
            bandwidth_bps:   Some(bps),
            bandwidth_class: class,
            is_reachable:    true,
            requires_auth:   false,
        }
    }
}

/// Recommended clone strategy based on analysis + connection quality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloneStrategy {
    /// Full clone — all history, all objects.
    Full,
    /// Shallow clone with the given depth.
    Shallow(u32),
    /// Partial clone with the given filter spec (e.g. "blob:none").
    Partial(&'static str),
    /// Sparse checkout with the given paths.
    Sparse(Vec<String>),
    /// Override the user-provided depth with a computed value.
    DepthOverride(u32),
}

/// Port for inspecting remote repositories pre-clone.
///
/// Implementations:
///   - ZigGitAnalyzer (uses `git ls-remote` + `git remote show`)
///   - HttpAnalyzer   (uses platform REST API for known hosts)
#[async_trait]
pub trait RepositoryAnalyzer: Send + Sync {
    /// Analyzes a remote repository and returns structured metadata.
    async fn analyze(&self, url: &str, ssh_key_path: Option<PathBuf>) -> Result<RepositoryAnalysis, GitError>;

    /// Probes connection quality to the remote host.
    async fn probe_connection(&self, url: &str) -> Result<ConnectionQuality, GitError>;

    /// Returns true if the remote URL is reachable via any protocol.
    async fn ping(&self, url: &str) -> bool;
}

/// Selects the optimal clone strategy based on repository analysis,
/// connection quality, and user preferences.
pub trait StrategySelector: Send + Sync {
    /// Given a repository analysis and connection quality, select
    /// the most efficient clone strategy.
    fn select_strategy(&self, analysis: &RepositoryAnalysis, quality: &ConnectionQuality) -> CloneStrategy;

    /// Adjusts the user's CloneOptions based on the analysis to
    /// produce the optimal set of options for this clone.
    fn optimize_options(&self, opts: &CloneOptions, analysis: &RepositoryAnalysis, quality: &ConnectionQuality) -> CloneOptions;
}
