// crates/gm_domain/src/sync/services/pull_service.rs
//
// The PullService orchestrates the pull workflow:
//   1. Check for existing uncommitted local changes (warn but do not abort)
//   2. Execute git pull with the configured rebase/merge strategy
//   3. Detect and surface any merge conflicts that arise
//
// Unlike push, pull can succeed even with a dirty working tree as long as
// there are no conflicting changes. The service preserves the full
// stdout/stderr in the session log for diagnostics.

use std::path::PathBuf;
use std::sync::Arc;

use crate::git::ports::git_executor::{GitExecutor, PullOptions};
use crate::sync::{
    entities::{SyncSession, sync_session::SessionType},
    events::{SyncCompleted, SyncStarted, ConflictDetected},
    ports::SyncSessionRepository,
};
use gm_shared::errors::GitManagerError;
use uuid::Uuid;

#[derive(Debug)]
pub struct PullInput {
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    pub repo_path:       PathBuf,
    pub ssh_key_path:    PathBuf,
    pub branch:          Option<String>,
    pub rebase:          bool,
}

#[derive(Debug)]
pub struct PullOutput {
    pub session:    SyncSession,
    pub started:    SyncStarted,
    pub completed:  SyncCompleted,
    /// Set if a conflict was detected rather than resolved automatically.
    pub conflict:   Option<ConflictDetected>,
}

pub struct PullService<GE: GitExecutor, SR: SyncSessionRepository> {
    executor:     Arc<GE>,
    session_repo: Arc<SR>,
}

impl<GE: GitExecutor, SR: SyncSessionRepository> std::fmt::Debug for PullService<GE, SR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PullService").finish_non_exhaustive()
    }
}

impl<GE: GitExecutor, SR: SyncSessionRepository> PullService<GE, SR> {
    pub fn new(executor: Arc<GE>, session_repo: Arc<SR>) -> Self {
        Self { executor, session_repo }
    }

    pub async fn execute(&self, input: PullInput) -> Result<PullOutput, GitManagerError> {
        let mut session = SyncSession::new(
            input.repository_uuid,
            input.account_id,
            SessionType::Pull,
            input.branch.clone().unwrap_or_else(|| "main".to_string()),
        );
        self.session_repo.save(&session).await?;

        let started = SyncStarted {
            session_uuid:    session.uuid(),
            repository_uuid: input.repository_uuid,
            account_id:      input.account_id,
            session_type:    "pull".to_string(),
        };

        session.begin_pulling();
        self.session_repo.save(&session).await?;

        let pull_result = self.executor.pull(PullOptions {
            repo_path:    input.repo_path.clone(),
            ssh_key_path: input.ssh_key_path,
            rebase:       input.rebase,
            branch:       input.branch.clone(),
        }).await;

        match pull_result {
            Ok(result) => {
                session.complete_successfully(
                    Some(result.commit_sha.clone()),
                    0,
                    0,
                    result.commits_pulled,
                );
                self.session_repo.save(&session).await?;

                let completed = SyncCompleted {
                    session_uuid:    session.uuid(),
                    repository_uuid: input.repository_uuid,
                    account_id:      input.account_id,
                    commit_sha:      result.commit_sha,
                    commits_pushed:  0,
                    commits_pulled:  result.commits_pulled,
                    success:         true,
                };

                Ok(PullOutput { session, started, completed, conflict: None })
            }
            Err(gm_shared::errors::GitError::PullConflict { conflicting_files }) => {
                session.fail_with_conflicts(conflicting_files.clone());
                self.session_repo.save(&session).await?;

                let conflict = ConflictDetected {
                    session_uuid:     session.uuid(),
                    repository_uuid:  input.repository_uuid,
                    account_id:       input.account_id,
                    conflicted_files: conflicting_files.clone(),
                };

                let completed = SyncCompleted {
                    session_uuid:    session.uuid(),
                    repository_uuid: input.repository_uuid,
                    account_id:      input.account_id,
                    commit_sha:      String::new(),
                    commits_pushed:  0,
                    commits_pulled:  0,
                    success:         false,
                };

                Ok(PullOutput { session, started, completed, conflict: Some(conflict) })
            }
            Err(e) => {
                session.fail_with_error(e.to_string());
                self.session_repo.save(&session).await?;
                Err(GitManagerError::Git(e))
            }
        }
    }
}