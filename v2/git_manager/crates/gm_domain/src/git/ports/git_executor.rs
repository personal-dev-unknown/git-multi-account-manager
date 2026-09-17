// crates/gm_domain/src/git/ports/git_executor.rs
//
// The GitExecutor port is the domain's clean abstraction over git subprocess
// operations. The domain's GitService and SyncService call this trait; the
// ZigGitExecutor in gm_adapters implements it by crossing the Rust→Zig FFI
// boundary and running the actual git binary with precise environment control.
//
// Every struct here is plain data — no trait objects, no async, no I/O.
// The async happens inside the adapter implementation, hidden behind the
// #[async_trait] boundary.
//
// Design note — why not just use git2-rs?
// The git2 crate (libgit2) is excellent for programmatic git access but it
// bypasses the system's SSH agent, ignores GIT_SSH_COMMAND, and does not
// respect per-account identity isolation. We need the system git binary so
// that the precise SSH environment we construct via Zig is honoured.

use std::path::{Path, PathBuf};
use async_trait::async_trait;
use gm_shared::errors::GitError;

// ── Input types ───────────────────────────────────────────────────────────────

/// Controls how tags are handled during clone/fetch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagsMode {
    /// Do not fetch any tags (--no-tags).
    None,
    /// Only fetch tags reachable from the cloned branch (default — no flag).
    Reachable,
    /// Fetch all tags from the remote (--tags).
    All,
}

/// Options for a git clone operation.
#[derive(Debug, Clone)]
pub struct CloneOptions {
    /// The SSH or HTTPS remote URL to clone from.
    pub url:                String,
    /// Absolute path on disk where the repository should be cloned.
    pub destination:        PathBuf,
    /// Absolute path to the SSH private key for this account (None for anonymous HTTPS).
    pub ssh_key_path:       Option<PathBuf>,
    /// The SSH host alias (e.g. "github.com-work") for ~/.ssh/config lookup.
    pub ssh_host_alias:     Option<String>,
    /// If Some, clone only this specific branch. If None, clone the default branch.
    pub branch:             Option<String>,
    /// Clone depth: None = full history, Some(1) = shallow, Some(N) = limited history.
    pub depth:              Option<u32>,
    /// Partial clone filter (e.g. "blob:none", "tree:0", "blob:limit=1m").
    pub filter:             Option<String>,
    /// If true, create a bare repository (no working directory).
    pub bare:               bool,
    /// If true, create a mirror repository (bare + all refs as-is).
    pub mirror:             bool,
    /// If set, initialize sparse checkout with only the specified paths.
    pub sparse_checkout:    Option<Vec<String>>,
    /// If true, clone only the tip of the requested branch (--single-branch).
    pub single_branch:      bool,
    /// If true, do not checkout HEAD after clone (--no-checkout).
    pub no_checkout:        bool,
    /// If true, initialize and clone submodules recursively.
    pub recurse_submodules: bool,
    /// How to handle tags during clone.
    pub tags_mode:          TagsMode,
    /// Custom upload pack executable (--upload-pack).
    pub upload_pack:          Option<String>,
    /// Estimated repository size in bytes (0 = unknown).
    /// Used for disk space validation and memory protection.
    pub estimated_size_bytes: u64,
}

impl CloneOptions {
    pub fn is_shallow(&self) -> bool {
        self.depth.is_some()
    }
}

impl Default for CloneOptions {
    fn default() -> Self {
        Self {
            url:                String::new(),
            destination:        PathBuf::new(),
            ssh_key_path:       None,
            ssh_host_alias:     None,
            branch:             None,
            depth:              None,
            filter:             None,
            bare:               false,
            mirror:             false,
            sparse_checkout:    None,
            single_branch:      false,
            no_checkout:        false,
            recurse_submodules: false,
            tags_mode:          TagsMode::Reachable,
            upload_pack:        None,
            estimated_size_bytes: 0,
        }
    }
}

/// Options for a git pull operation.
#[derive(Debug)]
pub struct PullOptions {
    /// Absolute path to the working repository directory.
    pub repo_path:    PathBuf,
    pub ssh_key_path: PathBuf,
    /// If true, use --rebase instead of the default merge strategy.
    pub rebase:       bool,
    pub branch:       Option<String>,
}

/// Options for a git push operation.
#[derive(Debug)]
pub struct PushOptions {
    pub repo_path:    PathBuf,
    pub ssh_key_path: PathBuf,
    pub remote:       String,
    pub branch:       String,
    /// If true, pass --force-with-lease (safer than --force).
    pub force:        bool,
}

/// Options for creating a git commit.
#[derive(Debug)]
pub struct CommitOptions {
    pub repo_path:    PathBuf,
    pub message:      String,
    pub author_name:  String,
    pub author_email: String,
    /// If true, amend the most recent commit rather than creating a new one.
    pub amend:        bool,
}

// ── Output types ─────────────────────────────────────────────────────────────

/// Result of a successful git clone.
#[derive(Debug)]
pub struct CloneResult {
    /// Absolute path of the cloned repository directory.
    pub local_path:  PathBuf,
    /// The HEAD commit SHA immediately after cloning.
    pub commit_sha:  String,
    /// The default branch of the cloned repository.
    pub branch:      String,
}

/// Result of a successful git pull.
#[derive(Debug)]
pub struct PullResult {
    /// The HEAD commit SHA after the pull completed.
    pub commit_sha:     String,
    /// Number of new commits fetched from the remote.
    pub commits_pulled: u32,
    /// True if the pull was a fast-forward; false if a merge commit was created.
    pub was_fast_forward: bool,
}

/// Result of a successful git push.
#[derive(Debug)]
pub struct PushResult {
    /// Number of commits pushed to the remote.
    pub commits_pushed: u32,
}

/// Result of a successful git commit.
#[derive(Debug)]
pub struct CommitResult {
    pub sha:          String,
    pub short_sha:    String,
    pub message:      String,
}

/// A file as reported by `git status --porcelain`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    pub path:           String,
    pub index_status:   char,
    pub working_status: char,
}

/// The complete working directory status of a repository.
#[derive(Debug)]
pub struct GitStatus {
    pub branch:           String,
    pub upstream:         Option<String>,
    pub ahead:            u32,
    pub behind:           u32,
    pub entries:          Vec<StatusEntry>,
    pub is_clean:         bool,
    pub has_conflicts:    bool,
    pub conflicted_files: Vec<String>,
}

// ── The port trait ────────────────────────────────────────────────────────────

/// The domain's outbound port for git subprocess operations.
///
/// This is the primary abstraction point between the pure domain and the
/// OS-level git execution. Tests mock this trait; production wires in
/// ZigGitExecutor from gm_adapters.
///
/// `Send + Sync` are required because the kernel stores this as an
/// `Arc<dyn GitExecutor>` and calls it from multiple async tasks concurrently.
#[async_trait]
pub trait GitExecutor: Send + Sync {
    /// Clones a remote repository to a local directory using the standard method.
    /// Supports all CloneOptions fields including depth, filter, bare, mirror, etc.
    async fn clone(&self, opts: CloneOptions) -> Result<CloneResult, GitError>;

    /// Clones a repository with sparse checkout — only the specified paths are
    /// fetched into the working tree. Uses `--filter=blob:none` + `sparse-checkout init --cone`.
    async fn sparse_clone(&self, opts: CloneOptions, paths: Vec<String>)
        -> Result<CloneResult, GitError>;

    /// Creates a mirror clone: a bare clone with all remote refs mirrored as-is.
    async fn mirror_clone(&self, opts: CloneOptions) -> Result<CloneResult, GitError>;

    /// Creates a bare clone (no working directory, only .git data).
    async fn bare_clone(&self, opts: CloneOptions) -> Result<CloneResult, GitError>;

    /// Fetches and integrates changes from the remote tracking branch.
    async fn pull(&self, opts: PullOptions) -> Result<PullResult, GitError>;

    /// Pushes local commits to the remote.
    async fn push(&self, opts: PushOptions) -> Result<PushResult, GitError>;

    /// Creates a new commit from the currently staged changes.
    async fn commit(&self, opts: CommitOptions) -> Result<CommitResult, GitError>;

    /// Stages all modified, untracked, and deleted files. Equivalent to `git add -A`.
    async fn stage_all(&self, repo_path: &Path) -> Result<(), GitError>;

    /// Returns the current working directory status.
    async fn status(&self, repo_path: &Path) -> Result<GitStatus, GitError>;

    /// Fetches updates from the remote without merging.
    async fn fetch(&self, repo_path: &Path, ssh_key_path: &Path) -> Result<(), GitError>;
}