//! Git operation error variants.
//!
//! `GitError` covers failures from the Zig git execution layer — clone,
//! pull, push, commit, and status operations. Each variant carries exactly
//! the context needed to explain what failed and suggest a remedy.
//!
//! The `is_retryable()` method allows the workflow engine's retry logic to
//! distinguish transient failures (network timeout) from permanent ones
//! (push rejected because of policy — no amount of retrying will fix that).

use thiserror::Error;

/// Errors arising from git subprocess operations.
#[derive(Debug, Error, PartialEq)]
pub enum GitError {
    /// `git clone` exited non-zero. The `reason` field contains the
    /// combined stderr from git and the SSH subprocess.
    #[error("Clone failed: {reason}")]
    CloneFailed { reason: String },

    /// The remote rejected the push. Common causes: the remote has commits
    /// the local branch has not fetched yet (non-fast-forward), branch
    /// protection rules are active, or the user lacks write permission.
    #[error("Push rejected by remote: {reason}")]
    PushRejected { reason: String },

    /// `git pull` succeeded syntactically but produced merge conflicts.
    /// The `conflicting_files` list contains the paths of affected files.
    /// The sync session must be paused until the user resolves the conflicts.
    #[error("Pull conflict in {file_count} file(s): {first_file}...",
        file_count = conflicting_files.len(),
        first_file = conflicting_files.first().map(|s| s.as_str()).unwrap_or("unknown"))]
    PullConflict { conflicting_files: Vec<String> },

    /// `git commit` failed. This happens when there are no staged changes,
    /// when the author email is rejected by a commit-msg hook, or when
    /// the repository's state prevents a commit (e.g. rebase in progress).
    #[error("Commit failed: {reason}")]
    CommitFailed { reason: String },

    /// `git status --porcelain` failed. This is unusual and typically
    /// indicates a corrupted .git directory or a filesystem permission issue.
    #[error("Status check failed: {reason}")]
    StatusFailed { reason: String },

    /// `git add -A` failed. Can happen if files have been deleted with
    /// unusual permissions, or if the index is locked by another git process.
    #[error("Staging files failed: {reason}")]
    StageFailed { reason: String },

    /// The provided path is not a git repository (no .git directory found
    /// at that path or any parent path). The user may have provided the
    /// wrong working directory.
    #[error("Not a git repository: {path}")]
    NotARepository { path: String },

    /// The SSH key was not accepted by the server. This happens when the
    /// public key has not been added to the Git hosting platform account,
    /// or when the wrong host alias is used.
    #[error("SSH authentication failed for host: {host}")]
    AuthenticationFailed { host: String },

    /// The TCP connection to the server could not be established within
    /// the configured timeout. The server may be unreachable or the
    /// network may be down.
    #[error("Connection to '{host}' timed out after {timeout_ms}ms")]
    NetworkTimeout { host: String, timeout_ms: u32 },

    /// The `git` binary was not found on PATH. The user needs to install
    /// git before any git operations can be performed.
    #[error("git binary not found on PATH — please install git")]
    GitNotFound,

    /// The repository is in a conflicted state from a previous failed merge
    /// or rebase. The user must resolve the conflict manually before
    /// automated operations can continue.
    #[error("Repository is in conflicted state at {path} — resolve conflicts first")]
    ConflictedState { path: String },
}

impl GitError {
    /// Returns true if the error is likely transient and the operation may
    /// succeed if retried after a delay. Returns false for permanent failures
    /// where retry would not help.
    ///
    /// The workflow engine's retry logic uses this to decide whether to
    /// wait and retry (transient) or immediately fail the workflow (permanent).
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            GitError::NetworkTimeout { .. }
                | GitError::AuthenticationFailed { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_timeout_is_retryable() {
        let err = GitError::NetworkTimeout {
            host: "github.com".to_string(),
            timeout_ms: 30_000,
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn push_rejected_is_not_retryable() {
        let err = GitError::PushRejected { reason: "non-fast-forward".to_string() };
        assert!(!err.is_retryable());
    }

    #[test]
    fn pull_conflict_display_shows_file_count() {
        let err = GitError::PullConflict {
            conflicting_files: vec!["src/main.rs".to_string(), "README.md".to_string()],
        };
        let msg = err.to_string();
        assert!(msg.contains("2"));
        assert!(msg.contains("src/main.rs"));
    }
}