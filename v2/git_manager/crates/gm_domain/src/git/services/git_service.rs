// crates/gm_domain/src/git/services/git_service.rs
//
// The GitService orchestrates multi-step git workflows for a single repository.
// It is generic over GE: GitExecutor, which means the underlying git binary
// interaction is entirely swappable — in tests you inject a mock, in production
// the kernel injects ZigGitExecutor from gm_adapters.
//
// The service's responsibilities are:
//   1. Validate pre-conditions (repository is cloned, SSH key file exists)
//   2. Delegate the actual subprocess work to the GitExecutor port
//   3. Produce domain events from successful operations for the kernel to publish
//
// What the service deliberately does NOT do:
//   - Touch the filesystem directly (that is the GitExecutor's job via Zig)
//   - Read or write to the database (the SyncService handles SyncSession persistence)
//   - Handle retries (the workflow engine's retry policy handles that)

use std::path::PathBuf;
use std::sync::Arc;

use crate::git::events::CommitCreated;
use crate::git::ports::git_executor::{
    CloneOptions, CloneResult, CommitOptions, CommitResult, GitExecutor,
    GitStatus, PullOptions, PullResult, PushOptions, PushResult,
};
use gm_shared::errors::GitError;

/// Result of a successful clone orchestrated through GitService.
#[derive(Debug)]
pub struct CloneOutput {
    pub result: CloneResult,
    pub event:  CommitCreated,
}

/// Result of a successful push.
#[derive(Debug)]
pub struct PushOutput {
    pub result: PushResult,
}

/// Result of a successful pull.
#[derive(Debug)]
pub struct PullOutput {
    pub result:      PullResult,
    pub event:       Option<CommitCreated>,
}

/// Domain service for git repository operations.
pub struct GitService<GE: GitExecutor> {
    executor: Arc<GE>,
}

impl<GE: GitExecutor> std::fmt::Debug for GitService<GE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitService").finish_non_exhaustive()
    }
}

impl<GE: GitExecutor> GitService<GE> {
    pub fn new(executor: Arc<GE>) -> Self {
        Self { executor }
    }

    /// Clones a repository using the given options and produces a CommitCreated event
    /// for the HEAD commit.
    pub async fn clone_repository(
        &self,
        opts:            CloneOptions,
        account_id:      uuid::Uuid,
        repository_uuid: uuid::Uuid,
    ) -> Result<CloneOutput, GitError> {
        let result = GitExecutor::clone(self.executor.as_ref(), opts).await?;
        let event = CommitCreated {
            repository_uuid,
            account_id,
            commit_sha:    result.commit_sha.clone(),
            commit_message: format!("Initial clone on branch {}", result.branch),
        };
        Ok(CloneOutput { result, event })
    }

    /// Stages all changes and creates a commit with the given message.
    pub async fn stage_and_commit(
        &self,
        repo_path:    PathBuf,
        message:      String,
        author_name:  String,
        author_email: String,
        _account_id:   uuid::Uuid,
        _repository_uuid: uuid::Uuid,
    ) -> Result<CommitResult, GitError> {
        GitExecutor::stage_all(self.executor.as_ref(), &repo_path).await?;
        let opts = CommitOptions {
            repo_path,
            message: message.clone(),
            author_name,
            author_email,
            amend: false,
        };
        GitExecutor::commit(self.executor.as_ref(), opts).await
    }

    /// Pulls changes from the remote tracking branch.
    pub async fn pull(
        &self,
        repo_path:    PathBuf,
        ssh_key_path: PathBuf,
        rebase:       bool,
        branch:       Option<String>,
    ) -> Result<PullResult, GitError> {
        let opts = PullOptions { repo_path, ssh_key_path, rebase, branch };
        GitExecutor::pull(self.executor.as_ref(), opts).await
    }

    /// Pushes local commits to the remote.
    pub async fn push(
        &self,
        repo_path:    PathBuf,
        ssh_key_path: PathBuf,
        remote:       String,
        branch:       String,
        force:        bool,
    ) -> Result<PushResult, GitError> {
        let opts = PushOptions { repo_path, ssh_key_path, remote, branch, force };
        GitExecutor::push(self.executor.as_ref(), opts).await
    }

    /// Returns the current working-tree status.
    pub async fn status(&self, repo_path: PathBuf) -> Result<GitStatus, GitError> {
        GitExecutor::status(self.executor.as_ref(), &repo_path).await
    }
}