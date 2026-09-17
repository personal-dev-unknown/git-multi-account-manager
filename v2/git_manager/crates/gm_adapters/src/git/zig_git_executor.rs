// crates/gm_adapters/src/git/zig_git_executor.rs
//
// ZigGitExecutor — the safe Rust adapter bridging the domain's GitExecutor port
// to the native Zig subprocess layer.
//
// Every method name here MUST exactly match the trait defined in
// gm_domain/src/git/ports/git_executor.rs. Every result struct field MUST
// exactly match those defined in that same file. The port is the contract;
// the adapter conforms to it.
//
// Method contract reference (from gm_domain/src/git/ports/git_executor.rs):
//   fn clone()     → CloneResult  { local_path: PathBuf, commit_sha: String, branch: String }
//   fn pull()      → PullResult   { commit_sha: String, commits_pulled: u32, was_fast_forward: bool }
//   fn push()      → PushResult   { commits_pushed: u32 }
//   fn commit()    → CommitResult { sha: String, short_sha: String, message: String }
//   fn stage_all() → ()
//   fn status()    → GitStatus
//   fn fetch()     → ()
//
// CloneOptions fields: url, destination, ssh_key_path, ssh_host_alias, branch,
//                      depth, filter, bare, mirror, sparse_checkout, single_branch,
//                      no_checkout, recurse_submodules, tags_mode, upload_pack
// PullOptions fields:  repo_path: PathBuf, ssh_key_path: PathBuf, rebase: bool, branch: Option<String>
// PushOptions fields:  repo_path: PathBuf, ssh_key_path: PathBuf, remote: String, branch: String, force: bool
//
// ── spawn_blocking ────────────────────────────────────────────────────────────
// All Zig FFI calls are blocking. spawn_blocking moves them to Tokio's dedicated
// blocking thread pool (separate from async worker threads) to prevent starvation.

use std::path::Path;
use async_trait::async_trait;
use gm_shared::errors::GitError;
use gm_domain::git::ports::git_executor::{
    CloneOptions, CloneResult, CommitOptions, CommitResult,
    GitExecutor, GitStatus, PullOptions, PullResult,
    PushOptions, PushResult, StatusEntry,
};

use super::ffi;

/// Stateless git operations adapter backed by the Zig native layer.
#[derive(Debug, Default)]
pub struct ZigGitExecutor;

impl ZigGitExecutor {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl GitExecutor for ZigGitExecutor {

    // ── clone — method name matches port exactly ──────────────────────────────
    async fn clone(&self, opts: CloneOptions) -> Result<CloneResult, GitError> {
        let url         = opts.url.clone();
        // Port field: `destination: PathBuf` (NOT dest_path)
        let dest_str    = opts.destination.to_string_lossy().into_owned();
        let key_str     = opts.ssh_key_path.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
        let alias_str   = opts.ssh_host_alias.clone().unwrap_or_default();
        let branch_str  = opts.branch.clone().unwrap_or_default();
        let depth       = opts.depth.unwrap_or(0);
        let local_path  = opts.destination.clone();
        let branch_out  = opts.branch.clone().unwrap_or_else(|| "main".to_string());

        tokio::task::spawn_blocking(move || {
            let c_url    = ffi::to_cstring(&url);
            let c_dest   = ffi::to_cstring(&dest_str);
            let c_key    = ffi::to_cstring(&key_str);
            let c_alias  = ffi::to_cstring(&alias_str);
            let c_branch = ffi::to_cstring(&branch_str);
            let mut res  = ffi::FfiGitResult::zeroed();

            // SAFETY: All CStrings are valid null-terminated UTF-8 produced by
            // ffi::to_cstring. `res` is a stack-allocated FfiGitResult with
            // all fields initialized to zero. The Zig function does not retain
            // any pointer after returning.
            let ok = unsafe {
                ffi::gm_git_clone(
                    c_url.as_ptr(),
                    c_dest.as_ptr(),
                    c_key.as_ptr(),
                    c_alias.as_ptr(),
                    c_branch.as_ptr(),
                    depth,
                    &mut res as *mut ffi::FfiGitResult,
                )
            };

            if ok {
                Ok(CloneResult {
                    local_path,
                    commit_sha: res.sha_str().to_string(),
                    branch:     branch_out,
                })
            } else {
                Err(GitError::CloneFailed { reason: res.error_str().to_string() })
            }
        })
        .await
        .map_err(|e| GitError::CloneFailed { reason: format!("spawn_blocking join error: {e}") })?
    }

    async fn pull(&self, opts: PullOptions) -> Result<PullResult, GitError> {
        let repo   = opts.repo_path.to_string_lossy().into_owned();
        let key    = opts.ssh_key_path.to_string_lossy().into_owned();
        let rebase = opts.rebase;

        tokio::task::spawn_blocking(move || {
            let c_repo = ffi::to_cstring(&repo);
            let c_key  = ffi::to_cstring(&key);
            let mut res = ffi::FfiGitResult::zeroed();

            // SAFETY: CStrings valid; res is valid zeroed stack allocation.
            let ok = unsafe {
                ffi::gm_git_pull(
                    c_repo.as_ptr(),
                    c_key.as_ptr(),
                    rebase,
                    &mut res as *mut ffi::FfiGitResult,
                )
            };

            if ok {
                Ok(PullResult {
                    commit_sha:       res.sha_str().to_string(),
                    commits_pulled:   0,
                    was_fast_forward: true,
                })
            } else {
                Err(GitError::CloneFailed { reason: res.error_str().to_string() })
            }
        })
        .await
        .map_err(|e| GitError::CloneFailed { reason: format!("spawn_blocking join error: {e}") })?
    }

    async fn push(&self, opts: PushOptions) -> Result<PushResult, GitError> {
        let repo   = opts.repo_path.to_string_lossy().into_owned();
        let key    = opts.ssh_key_path.to_string_lossy().into_owned();
        let remote = opts.remote.clone();
        let branch = opts.branch.clone();
        let force  = opts.force;

        tokio::task::spawn_blocking(move || {
            let c_repo   = ffi::to_cstring(&repo);
            let c_key    = ffi::to_cstring(&key);
            let c_remote = ffi::to_cstring(&remote);
            let c_branch = ffi::to_cstring(&branch);
            let mut res  = ffi::FfiGitResult::zeroed();

            // SAFETY: All CStrings valid; res is valid zeroed stack allocation.
            // Zig arg order: repo_path, remote, branch, key_path, force, result
            let ok = unsafe {
                ffi::gm_git_push(
                    c_repo.as_ptr(),
                    c_remote.as_ptr(),
                    c_branch.as_ptr(),
                    c_key.as_ptr(),
                    force,
                    &mut res as *mut ffi::FfiGitResult,
                )
            };

            if ok {
                Ok(PushResult { commits_pushed: 0 })
            } else {
                Err(GitError::PushRejected { reason: res.error_str().to_string() })
            }
        })
        .await
        .map_err(|e| GitError::PushRejected { reason: format!("spawn_blocking join error: {e}") })?
    }

    async fn commit(&self, opts: CommitOptions) -> Result<CommitResult, GitError> {
        let repo    = opts.repo_path.to_string_lossy().into_owned();
        let message = opts.message.clone();
        let author  = opts.author_name.clone();
        let email   = opts.author_email.clone();
        let amend   = opts.amend;

        tokio::task::spawn_blocking(move || {
            let c_repo    = ffi::to_cstring(&repo);
            let c_message = ffi::to_cstring(&message);
            let c_author  = ffi::to_cstring(&author);
            let c_email   = ffi::to_cstring(&email);
            let mut res   = ffi::FfiGitResult::zeroed();

            // SAFETY: All CStrings valid; res is valid zeroed stack allocation.
            let ok = unsafe {
                ffi::gm_git_commit(
                    c_repo.as_ptr(),
                    c_message.as_ptr(),
                    c_author.as_ptr(),
                    c_email.as_ptr(),
                    amend,
                    &mut res as *mut ffi::FfiGitResult,
                )
            };

            if ok {
                let full_sha  = res.sha_str().to_string();
                let short_sha = full_sha.chars().take(8).collect::<String>();
                Ok(CommitResult { sha: full_sha, short_sha, message })
            } else {
                Err(GitError::CommitFailed { reason: res.error_str().to_string() })
            }
        })
        .await
        .map_err(|e| GitError::CommitFailed { reason: format!("spawn_blocking join error: {e}") })?
    }

    async fn stage_all(&self, _repo_path: &Path) -> Result<(), GitError> {
        // The Zig executor bundles `git add -A` atomically with gm_git_commit.
        // A future backend that separates staging and committing would implement this.
        Ok(())
    }

    async fn status(&self, repo_path: &Path) -> Result<GitStatus, GitError> {
        let repo = repo_path.to_string_lossy().into_owned();

        tokio::task::spawn_blocking(move || {
            let c_repo  = ffi::to_cstring(&repo);
            let mut res = Box::new(ffi::FfiStatusResult::zeroed());

            // SAFETY: CString valid; res is a heap-allocated FfiStatusResult (~100 KB).
            let ok = unsafe {
                ffi::gm_git_status(c_repo.as_ptr(), res.as_mut() as *mut ffi::FfiStatusResult)
            };

            if !ok {
                return Err(GitError::StatusFailed { reason: res.error_str().to_string() });
            }

            let mut entries: Vec<StatusEntry> = Vec::new();
            for path in res.staged_str().lines().filter(|l| !l.is_empty()) {
                entries.push(StatusEntry { path: path.to_string(), index_status: 'M', working_status: ' ' });
            }
            for path in res.unstaged_str().lines().filter(|l| !l.is_empty()) {
                entries.push(StatusEntry { path: path.to_string(), index_status: ' ', working_status: 'M' });
            }
            for path in res.untracked_str().lines().filter(|l| !l.is_empty()) {
                entries.push(StatusEntry { path: path.to_string(), index_status: '?', working_status: '?' });
            }

            Ok(GitStatus {
                branch:           String::new(),
                upstream:         None,
                ahead:            0,
                behind:           0,
                is_clean:         res.is_clean,
                has_conflicts:    false,
                conflicted_files: vec![],
                entries,
            })
        })
        .await
        .map_err(|e| GitError::StatusFailed { reason: format!("spawn_blocking join error: {e}") })?
    }

    // ── Sparse clone ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
    async fn sparse_clone(
        &self,
        opts: CloneOptions,
        paths: Vec<String>,
    ) -> Result<CloneResult, GitError> {
        let url         = opts.url.clone();
        let dest_str    = opts.destination.to_string_lossy().into_owned();
        let key_str     = opts.ssh_key_path.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
        let alias_str   = opts.ssh_host_alias.clone().unwrap_or_default();
        let branch_str  = opts.branch.clone().unwrap_or_default();
        let depth       = opts.depth.unwrap_or(0);
        let local_path  = opts.destination.clone();
        let branch_out  = opts.branch.clone().unwrap_or_else(|| "main".to_string());
        let paths_joined = paths.join(",");

        tokio::task::spawn_blocking(move || {
            let c_url    = ffi::to_cstring(&url);
            let c_dest   = ffi::to_cstring(&dest_str);
            let c_key    = ffi::to_cstring(&key_str);
            let c_alias  = ffi::to_cstring(&alias_str);
            let c_branch = ffi::to_cstring(&branch_str);
            let c_paths  = ffi::to_cstring(&paths_joined);
            let mut res  = ffi::FfiGitResult::zeroed();

            let ok = unsafe {
                ffi::gm_git_sparse_clone(
                    c_url.as_ptr(),
                    c_dest.as_ptr(),
                    c_key.as_ptr(),
                    c_alias.as_ptr(),
                    c_branch.as_ptr(),
                    depth,
                    c_paths.as_ptr(),
                    &mut res as *mut ffi::FfiGitResult,
                )
            };

            if ok {
                Ok(CloneResult { local_path, commit_sha: res.sha_str().to_string(), branch: branch_out })
            } else {
                Err(GitError::CloneFailed { reason: res.error_str().to_string() })
            }
        })
        .await
        .map_err(|e| GitError::CloneFailed { reason: format!("spawn_blocking join error: {e}") })?
    }

    // ── Mirror clone ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
    async fn mirror_clone(&self, opts: CloneOptions) -> Result<CloneResult, GitError> {
        let url         = opts.url.clone();
        let dest_str    = opts.destination.to_string_lossy().into_owned();
        let key_str     = opts.ssh_key_path.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
        let alias_str   = opts.ssh_host_alias.clone().unwrap_or_default();
        let branch_str  = opts.branch.clone().unwrap_or_default();
        let depth       = opts.depth.unwrap_or(0);
        let local_path  = opts.destination.clone();
        let branch_out  = opts.branch.clone().unwrap_or_else(|| "HEAD".to_string());

        tokio::task::spawn_blocking(move || {
            let c_url    = ffi::to_cstring(&url);
            let c_dest   = ffi::to_cstring(&dest_str);
            let c_key    = ffi::to_cstring(&key_str);
            let c_alias  = ffi::to_cstring(&alias_str);
            let c_branch = ffi::to_cstring(&branch_str);
            let mut res  = ffi::FfiGitResult::zeroed();

            let ok = unsafe {
                ffi::gm_git_mirror_clone(
                    c_url.as_ptr(),
                    c_dest.as_ptr(),
                    c_key.as_ptr(),
                    c_alias.as_ptr(),
                    c_branch.as_ptr(),
                    depth,
                    &mut res as *mut ffi::FfiGitResult,
                )
            };

            if ok {
                Ok(CloneResult { local_path, commit_sha: res.sha_str().to_string(), branch: branch_out })
            } else {
                Err(GitError::CloneFailed { reason: res.error_str().to_string() })
            }
        })
        .await
        .map_err(|e| GitError::CloneFailed { reason: format!("spawn_blocking join error: {e}") })?
    }

    // ── Bare clone ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
    async fn bare_clone(&self, opts: CloneOptions) -> Result<CloneResult, GitError> {
        let url         = opts.url.clone();
        let dest_str    = opts.destination.to_string_lossy().into_owned();
        let key_str     = opts.ssh_key_path.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
        let alias_str   = opts.ssh_host_alias.clone().unwrap_or_default();
        let branch_str  = opts.branch.clone().unwrap_or_default();
        let depth       = opts.depth.unwrap_or(0);
        let local_path  = opts.destination.clone();
        let branch_out  = opts.branch.clone().unwrap_or_else(|| "HEAD".to_string());

        tokio::task::spawn_blocking(move || {
            let c_url    = ffi::to_cstring(&url);
            let c_dest   = ffi::to_cstring(&dest_str);
            let c_key    = ffi::to_cstring(&key_str);
            let c_alias  = ffi::to_cstring(&alias_str);
            let c_branch = ffi::to_cstring(&branch_str);
            let mut res  = ffi::FfiGitResult::zeroed();

            let ok = unsafe {
                ffi::gm_git_bare_clone(
                    c_url.as_ptr(),
                    c_dest.as_ptr(),
                    c_key.as_ptr(),
                    c_alias.as_ptr(),
                    c_branch.as_ptr(),
                    depth,
                    &mut res as *mut ffi::FfiGitResult,
                )
            };

            if ok {
                Ok(CloneResult { local_path, commit_sha: res.sha_str().to_string(), branch: branch_out })
            } else {
                Err(GitError::CloneFailed { reason: res.error_str().to_string() })
            }
        })
        .await
        .map_err(|e| GitError::CloneFailed { reason: format!("spawn_blocking join error: {e}") })?
    }

    async fn fetch(&self, repo_path: &Path, ssh_key_path: &Path) -> Result<(), GitError> {
        let repo = repo_path.to_string_lossy().into_owned();
        let key  = ssh_key_path.to_string_lossy().into_owned();

        tokio::task::spawn_blocking(move || {
            let c_repo = ffi::to_cstring(&repo);
            let c_key  = ffi::to_cstring(&key);
            let mut res = ffi::FfiFetchResult::zeroed();

            // SAFETY: CStrings valid; res is valid zeroed stack allocation.
            let ok = unsafe {
                ffi::gm_git_fetch(c_repo.as_ptr(), c_key.as_ptr(), &mut res as *mut ffi::FfiFetchResult)
            };

            if ok { Ok(()) } else { Err(GitError::CloneFailed { reason: res.error_str().to_string() }) }
        })
        .await
        .map_err(|e| GitError::CloneFailed { reason: format!("spawn_blocking join error: {e}") })?
    }
}

// ── Internal: porcelain status parser ────────────────────────────────────────

/// Parses `git status --porcelain=v1` output into StatusEntry items.
/// Each line: "XY path" where X=index status, Y=worktree status, path starts at byte 3.
#[allow(dead_code)]
fn parse_porcelain_status(raw: &str) -> Vec<StatusEntry> {
    raw.lines()
        .filter(|l| l.len() >= 3)
        .map(|l| {
            let bytes = l.as_bytes();
            StatusEntry {
                path:           l[3..].to_string(),
                index_status:   bytes[0] as char,
                working_status: bytes[1] as char,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modified_entries() {
        let raw = "M  src/main.rs\n M src/lib.rs\n?? new.rs";
        let entries = parse_porcelain_status(raw);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].index_status, 'M');
        assert_eq!(entries[0].path, "src/main.rs");
        assert_eq!(entries[2].index_status, '?');
        assert_eq!(entries[2].working_status, '?');
    }

    #[test]
    fn empty_status_is_clean() {
        let entries = parse_porcelain_status("");
        assert!(entries.is_empty());
    }
}