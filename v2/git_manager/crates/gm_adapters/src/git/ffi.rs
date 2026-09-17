// crates/gm_adapters/src/git/ffi.rs
//
// FFI contract between Rust and the Zig git execution layer.
// CRITICAL: field layout must exactly match the Zig extern structs in
// zig_native/src/git_exec/executor.zig. Every field order, size, and
// padding must be identical on both sides.

use std::os::raw::c_char;

// ─────────────────────────────────────────────────────────────────────────────
// Shared git operation result
// ─────────────────────────────────────────────────────────────────────────────

/// Mirrors `GitResult` in executor.zig. Used for clone, pull, push, commit.
#[repr(C)]
pub struct FfiGitResult {
    pub success:     bool,
    pub commit_sha:  [u8; 64],
    pub sha_len:     usize,
    pub stdout_data: [u8; 65536],
    pub stdout_len:  usize,
    pub stderr_data: [u8; 4096],
    pub stderr_len:  usize,
    pub exit_code:   i32,
    /// PID of the most recently spawned git subprocess (populated by runChild in Zig).
    /// -1 means no child process is currently tracked. Used by `gm_git_kill_process`.
    pub child_pid:   i32,
}

impl FfiGitResult {
    pub fn zeroed() -> Self { unsafe { std::mem::zeroed() } }
    pub fn sha_str(&self)    -> &str { str_from_buf(&self.commit_sha,  self.sha_len) }
    pub fn error_str(&self)  -> &str { str_from_buf(&self.stderr_data, self.stderr_len) }
    pub fn output_str(&self) -> &str { str_from_buf(&self.stdout_data, self.stdout_len) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Status result
// ─────────────────────────────────────────────────────────────────────────────

/// Mirrors `GitStatus` in executor.zig.
#[repr(C)]
pub struct FfiStatusResult {
    pub staged_files:    [u8; 32768],
    pub staged_len:      usize,
    pub staged_count:    u32,
    pub unstaged_files:  [u8; 32768],
    pub unstaged_len:    usize,
    pub unstaged_count:  u32,
    pub untracked_files: [u8; 32768],
    pub untracked_len:   usize,
    pub untracked_count: u32,
    pub is_clean:        bool,
    pub error_message:   [u8; 512],
    pub error_len:       usize,
}

impl FfiStatusResult {
    pub fn zeroed() -> Self { unsafe { std::mem::zeroed() } }
    pub fn error_str(&self)     -> &str { str_from_buf(&self.error_message,   self.error_len) }
    pub fn staged_str(&self)    -> &str { str_from_buf(&self.staged_files,    self.staged_len) }
    pub fn unstaged_str(&self)  -> &str { str_from_buf(&self.unstaged_files,  self.unstaged_len) }
    pub fn untracked_str(&self) -> &str { str_from_buf(&self.untracked_files, self.untracked_len) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Fetch
// ─────────────────────────────────────────────────────────────────────────────

/// Result of `gm_git_fetch`. Mirrors `FetchResult` in executor.zig.
#[repr(C)]
pub struct FfiFetchResult {
    pub success:    bool,
    pub error_data: [u8; 512],
    pub error_len:  usize,
}

impl FfiFetchResult {
    pub fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }
    pub fn error_str(&self) -> &str { str_from_buf(&self.error_data, self.error_len) }
}

// ─────────────────────────────────────────────────────────────────────────────
// extern "C" declarations
// ─────────────────────────────────────────────────────────────────────────────

extern "C" {
    /// Returns available bytes on the filesystem containing `path`.
    /// Uses the POSIX statvfs syscall. Returns 0 on error.
    pub fn gm_fs_available_disk_space(path: *const c_char) -> u64;

    /// Checks whether `required_bytes` are available on the filesystem.
    /// Returns 0 = sufficient, -1 = insufficient, -2 = error.
    pub fn gm_fs_ensure_disk_space(path: *const c_char, required_bytes: u64) -> i32;

    /// Kills a git subprocess by PID. Pass 0 to kill the currently-tracked child.
    /// Returns true if a kill signal was sent, false if there was no process to kill.
    pub fn gm_git_kill_process(pid: i32) -> bool;

    /// Clones `url` into `dest`. `host_alias` is the SSH config alias already
    /// embedded in the URL — accepted but not used separately by the Zig layer.
    /// `branch` selects a non-default branch; empty string = remote default.
    /// `depth` > 0 produces a shallow clone.
    pub fn gm_git_clone(
        url:        *const c_char,
        dest:       *const c_char,
        key_path:   *const c_char,
        host_alias: *const c_char,
        branch:     *const c_char,
        depth:      u32,
        result:     *mut FfiGitResult,
    ) -> bool;

    /// Pulls from the remote tracking branch.
    pub fn gm_git_pull(
        repo_path: *const c_char,
        key_path:  *const c_char,
        rebase:    bool,
        result:    *mut FfiGitResult,
    ) -> bool;

    /// Pushes `branch` to `remote`. Parameter order matches the Zig export.
    pub fn gm_git_push(
        repo_path: *const c_char,
        remote:    *const c_char,
        branch:    *const c_char,
        key_path:  *const c_char,
        force:     bool,
        result:    *mut FfiGitResult,
    ) -> bool;

    /// Commits staged changes. Passes `amend` for `--amend` commits.
    pub fn gm_git_commit(
        repo_path:    *const c_char,
        message:      *const c_char,
        author_name:  *const c_char,
        author_email: *const c_char,
        amend:        bool,
        result:       *mut FfiGitResult,
    ) -> bool;

    /// Runs `git status --porcelain=v1` in `repo_path`.
    pub fn gm_git_status(
        repo_path: *const c_char,
        result:    *mut FfiStatusResult,
    ) -> bool;

    /// Runs `git fetch --all` in `repo_path`.
    pub fn gm_git_fetch(
        repo_path: *const c_char,
        key_path:  *const c_char,
        result:    *mut FfiFetchResult,
    ) -> bool;

    // ── Clone variants ────────────────────────────────────────────────────────

    /// Sparse clone: partial clone + sparse-checkout init with the given paths.
    /// `paths` is a comma-separated list of directory/file patterns to include.
    pub fn gm_git_sparse_clone(
        url:        *const c_char,
        dest:       *const c_char,
        key_path:   *const c_char,
        host_alias: *const c_char,
        branch:     *const c_char,
        depth:      u32,
        paths:      *const c_char,
        result:     *mut FfiGitResult,
    ) -> bool;

    /// Mirror clone: `git clone --mirror` — bare clone with all refs mirrored.
    pub fn gm_git_mirror_clone(
        url:        *const c_char,
        dest:       *const c_char,
        key_path:   *const c_char,
        host_alias: *const c_char,
        branch:     *const c_char,
        depth:      u32,
        result:     *mut FfiGitResult,
    ) -> bool;

    /// Bare clone: `git clone --bare` — no working directory.
    pub fn gm_git_bare_clone(
        url:        *const c_char,
        dest:       *const c_char,
        key_path:   *const c_char,
        host_alias: *const c_char,
        branch:     *const c_char,
        depth:      u32,
        result:     *mut FfiGitResult,
    ) -> bool;

    /// Fetches additional objects from a partial clone's promisor remote.
    pub fn gm_git_partial_clone_fetch(
        repo_path: *const c_char,
        result:    *mut FfiGitResult,
    ) -> bool;

    /// Reconfigures an existing sparse checkout with new paths (cone mode).
    pub fn gm_git_sparse_init(
        repo_path: *const c_char,
        paths:     *const c_char,
        result:    *mut FfiGitResult,
    ) -> bool;
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Interprets `buf[..len]` as a UTF-8 string. Returns empty string on invalid UTF-8.
fn str_from_buf(buf: &[u8], len: usize) -> &str {
    let clamped = len.min(buf.len());
    std::str::from_utf8(&buf[..clamped]).unwrap_or("")
}

pub fn to_cstring(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s)
        .unwrap_or_else(|_| panic!("Git FFI: string contains null byte: {s:?}"))
}