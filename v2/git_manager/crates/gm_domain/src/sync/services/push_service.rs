// crates/gm_domain/src/sync/services/push_service.rs
//
// The PushService orchestrates the push workflow for a single repository:
//   1. Check the working tree status for uncommitted changes
//   2. Stage all changes (git add -A)
//   3. Create a commit with the provided message
//   4. Push the commit to the remote tracking branch
//
// Each step records its progress into the SyncSession so that the session
// log reflects the full operation even if it fails midway. This makes
// post-mortem debugging tractable for users who see an error message.

use std::path::PathBuf;
use std::sync::Arc;

use crate::git::ports::git_executor::{
    CommitOptions, GitExecutor, PushOptions,
};
use crate::sync::{
    entities::{SyncSession, sync_session::SessionType},
    events::{SyncCompleted, SyncStarted},
    ports::SyncSessionRepository,
};
use gm_shared::errors::GitManagerError;
use uuid::Uuid;

#[derive(Debug)]
pub struct PushInput {
    pub repository_uuid: Uuid,
    pub account_id:      Uuid,
    pub repo_path:       PathBuf,
    pub ssh_key_path:    PathBuf,
    pub branch:          String,
    pub commit_message:  String,
    pub author_name:     String,
    pub author_email:    String,
    pub force:           bool,
}

#[derive(Debug)]
pub struct PushOutput {
    pub session:  SyncSession,
    pub started:  SyncStarted,
    pub completed: SyncCompleted,
}

pub struct PushService<GE: GitExecutor, SR: SyncSessionRepository> {
    executor:    Arc<GE>,
    session_repo: Arc<SR>,
}

impl<GE: GitExecutor, SR: SyncSessionRepository> std::fmt::Debug for PushService<GE, SR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PushService").finish_non_exhaustive()
    }
}

impl<GE: GitExecutor, SR: SyncSessionRepository> PushService<GE, SR> {
    pub fn new(executor: Arc<GE>, session_repo: Arc<SR>) -> Self {
        Self { executor, session_repo }
    }

    pub async fn execute(&self, input: PushInput) -> Result<PushOutput, GitManagerError> {
        // Create and persist the session immediately so it's visible even if we crash.
        let mut session = SyncSession::new(
            input.repository_uuid,
            input.account_id,
            SessionType::Push,
            input.branch.clone(),
        );
        session.set_commit_message(input.commit_message.clone());
        self.session_repo.save(&session).await?;

        let started = SyncStarted {
            session_uuid:    session.uuid(),
            repository_uuid: input.repository_uuid,
            account_id:      input.account_id,
            session_type:    "push".to_string(),
        };

        // Step 1: Check status — are there any changes to stage?
        let status = self.executor.status(&input.repo_path).await
            .map_err(|e| GitManagerError::Git(e))?;

        if status.has_conflicts {
            session.fail_with_conflicts(status.conflicted_files.clone());
            self.session_repo.save(&session).await?;
            return Err(GitManagerError::Git(
                gm_shared::errors::GitError::ConflictedState {
                    path: input.repo_path.to_string_lossy().to_string(),
                }
            ));
        }

        let files_staged = status.entries.len() as u32;

        // Step 2: Stage all changes.
        session.begin_staging();
        self.session_repo.save(&session).await?;

        if files_staged > 0 {
            self.executor.stage_all(&input.repo_path).await
                .map_err(|e| { let _ = self.fail_session(&mut session, e.to_string()); e })
                .map_err(|e| GitManagerError::Git(e))?;
        }

        // Step 3: Commit.
        session.begin_committing();
        self.session_repo.save(&session).await?;

        let commit_result = self.executor.commit(CommitOptions {
            repo_path:    input.repo_path.clone(),
            message:      input.commit_message.clone(),
            author_name:  input.author_name.clone(),
            author_email: input.author_email.clone(),
            amend:        false,
        }).await.map_err(|e| GitManagerError::Git(e))?;

        // Step 4: Push.
        session.begin_pushing();
        self.session_repo.save(&session).await?;

        let push_result = self.executor.push(PushOptions {
            repo_path:    input.repo_path.clone(),
            ssh_key_path: input.ssh_key_path.clone(),
            remote:       "origin".to_string(),
            branch:       input.branch.clone(),
            force:        input.force,
        }).await.map_err(|e| GitManagerError::Git(e))?;

        // Finalise the session.
        session.complete_successfully(
            Some(commit_result.sha.clone()),
            files_staged,
            push_result.commits_pushed,
            0,
        );
        self.session_repo.save(&session).await?;

        let completed = SyncCompleted {
            session_uuid:    session.uuid(),
            repository_uuid: input.repository_uuid,
            account_id:      input.account_id,
            commit_sha:      commit_result.sha,
            commits_pushed:  push_result.commits_pushed,
            commits_pulled:  0,
            success:         true,
        };

        Ok(PushOutput { session, started, completed })
    }

    fn fail_session(&self, session: &mut SyncSession, msg: String) {
        session.fail_with_error(msg);
    }
}