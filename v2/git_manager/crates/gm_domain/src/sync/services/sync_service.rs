// crates/gm_domain/src/sync/services/sync_service.rs
//
// The SyncService coordinates the bidirectional sync: pull first to integrate
// remote changes, then push local commits. It delegates to PullService and
// PushService for each half, combining their events into a single SyncCompleted.
//
// The pull-then-push order is deliberate: pushing first on a branch that is
// behind its remote would be rejected by the server (non-fast-forward). By
// pulling first, we ensure the local branch includes all remote changes before
// we attempt to push, which succeeds as a fast-forward in the common case.

use std::path::PathBuf;
use std::sync::Arc;

use crate::git::ports::git_executor::GitExecutor;
use crate::sync::{
    events::{SyncCompleted, SyncStarted, ConflictDetected},
    ports::SyncSessionRepository,
    services::{PullService, PullInput, PushService, PushInput},
};
use gm_shared::errors::GitManagerError;
use uuid::Uuid;

#[derive(Debug)]
pub struct SyncInput {
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    pub repo_path:       PathBuf,
    pub ssh_key_path:    PathBuf,
    pub branch:          String,
    pub commit_message:  String,
    pub author_name:     String,
    pub author_email:    String,
}

#[derive(Debug)]
pub struct SyncOutput {
    pub started:   SyncStarted,
    pub completed: SyncCompleted,
    pub conflict:  Option<ConflictDetected>,
}

pub struct SyncService<GE: GitExecutor, SR: SyncSessionRepository> {
    pull_service: PullService<GE, SR>,
    push_service: PushService<GE, SR>,
}

impl<GE: GitExecutor, SR: SyncSessionRepository> std::fmt::Debug for SyncService<GE, SR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncService").finish_non_exhaustive()
    }
}

impl<GE: GitExecutor, SR: SyncSessionRepository> SyncService<GE, SR> {
    pub fn new(executor: Arc<GE>, session_repo: Arc<SR>) -> Self {
        Self {
            pull_service: PullService::new(executor.clone(), session_repo.clone()),
            push_service: PushService::new(executor, session_repo),
        }
    }

    /// Executes a full bidirectional sync: pull then push.
    /// Returns early (with a ConflictDetected event) if the pull step
    /// produces merge conflicts — in that case the push step is skipped.
    pub async fn execute(&self, input: SyncInput) -> Result<SyncOutput, GitManagerError> {
        // Phase 1: Pull
        let pull_out = self.pull_service.execute(PullInput {
            repository_uuid: input.repository_uuid,
            account_id:      input.account_id,
            repo_path:       input.repo_path.clone(),
            ssh_key_path:    input.ssh_key_path.clone(),
            branch:          Some(input.branch.clone()),
            rebase:          false,
        }).await?;

        // If the pull detected conflicts, stop here.
        if let Some(conflict) = pull_out.conflict {
            return Ok(SyncOutput {
                started:   pull_out.started,
                completed: pull_out.completed,
                conflict:  Some(conflict),
            });
        }

        // Phase 2: Push
        let push_out = self.push_service.execute(PushInput {
            repository_uuid: input.repository_uuid,
            account_id:      input.account_id,
            repo_path:       input.repo_path,
            ssh_key_path:    input.ssh_key_path,
            branch:          input.branch,
            commit_message:  input.commit_message,
            author_name:     input.author_name,
            author_email:    input.author_email,
            force:           false,
        }).await?;

        // Combine the metrics from both phases into a single SyncCompleted event.
        let combined = SyncCompleted {
            session_uuid:    push_out.session.uuid(),
            repository_uuid: input.repository_uuid,
            account_id:      input.account_id,
            commit_sha:      push_out.completed.commit_sha,
            commits_pushed:  push_out.completed.commits_pushed,
            commits_pulled:  pull_out.completed.commits_pulled,
            success:         true,
        };

        Ok(SyncOutput { started: pull_out.started, completed: combined, conflict: None })
    }
}