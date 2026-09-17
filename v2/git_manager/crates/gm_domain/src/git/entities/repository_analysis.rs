// crates/gm_domain/src/git/entities/repository_analysis.rs
//
// RepositoryAnalysis captures the result of inspecting a remote repository
// before cloning. It feeds the StrategySelector (Phase 4) so the system can
// choose the optimal clone method.

/// Estimated size classification for a remote repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoSizeClass {
    Tiny,    // < 1 MiB
    Small,   // 1–10 MiB
    Medium,  // 10–100 MiB
    Large,   // 100 MiB–1 GiB
    Huge,    // > 1 GiB
}

/// Result of analyzing a remote repository before clone.
#[derive(Debug, Clone)]
pub struct RepositoryAnalysis {
    /// The remote URL that was analyzed.
    pub url:              String,
    /// Estimated repository size in bytes (from remote API or `git ls-remote`).
    pub estimated_size:   Option<u64>,
    /// Size classification based on estimated_size.
    pub size_class:       RepoSizeClass,
    /// Number of branches visible on the remote.
    pub branch_count:     Option<u32>,
    /// Number of tags visible on the remote.
    pub tag_count:        Option<u32>,
    /// Whether the remote has a valid .git or is a recognized git host.
    pub is_reachable:     bool,
    /// Whether the remote is accessible anonymously.
    pub is_public:        bool,
    /// Whether authentication is required to list refs.
    pub requires_auth:    bool,
    /// Detected default branch (e.g. "main", "master").
    pub default_branch:   Option<String>,
    /// Human-readable summary of the analysis.
    pub summary:          String,
}

impl RepositoryAnalysis {
    /// Estimate how long a full clone might take based on size class.
    pub fn estimated_clone_seconds(&self) -> u64 {
        match self.size_class {
            RepoSizeClass::Tiny   => 2,
            RepoSizeClass::Small  => 10,
            RepoSizeClass::Medium => 45,
            RepoSizeClass::Large  => 180,
            RepoSizeClass::Huge   => 600,
        }
    }

    /// Returns true if a shallow clone (depth=1) would save significant time.
    pub fn shallow_recommended(&self) -> bool {
        matches!(self.size_class, RepoSizeClass::Large | RepoSizeClass::Huge)
    }

    /// Returns true if a partial clone (filter) would be beneficial.
    pub fn partial_clone_recommended(&self) -> bool {
        matches!(self.size_class, RepoSizeClass::Medium | RepoSizeClass::Large | RepoSizeClass::Huge)
    }
}

impl RepoSizeClass {
    pub fn from_bytes(bytes: u64) -> Self {
        match bytes {
            0..=1_000_000             => RepoSizeClass::Tiny,
            1_000_001..=10_000_000    => RepoSizeClass::Small,
            10_000_001..=100_000_000  => RepoSizeClass::Medium,
            100_000_001..=1_000_000_000 => RepoSizeClass::Large,
            _                         => RepoSizeClass::Huge,
        }
    }
}
