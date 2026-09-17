// crates/gm_domain/src/sync/entities/sync_session.rs
//
// A SyncSession tracks the complete lifecycle of one git operation cycle:
// staging, committing, pushing, or pulling. It is the domain's audit record
// for every meaningful git event — the kernel creates a session before the
// operation begins and the sync services update it as the operation progresses.
//
// Sessions can be replayed for diagnostics (the stdout/stderr of each step
// is preserved) and are the source of truth for the operation history display
// in the CLI and web UI. The session's `status` field drives the state machine
// that prevents concurrent conflicting operations on the same repository.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// The type of sync operation being tracked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionType {
    /// Combined stage-all + commit + push workflow.
    Push,
    /// Pull (fetch + merge or rebase) workflow.
    Pull,
    /// The bidirectional sync: pull first, then push.
    Sync,
}

impl SessionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionType::Push  => "push",
            SessionType::Pull  => "pull",
            SessionType::Sync  => "sync",
        }
    }
}

/// The lifecycle state of a sync session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    /// The session has been created but work has not yet started.
    Pending,
    /// The `git add -A` step is in progress.
    Staging,
    /// The `git commit` step is in progress.
    Committing,
    /// The `git push` step is in progress.
    Pushing,
    /// The `git pull` step is in progress.
    Pulling,
    /// All steps completed without error.
    Success,
    /// An error occurred in one of the steps. See `error_message`.
    Failed,
    /// The operation detected merge conflicts. The user must intervene.
    Conflicted,
}

impl SessionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, SessionStatus::Success | SessionStatus::Failed | SessionStatus::Conflicted)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStatus::Pending     => "pending",
            SessionStatus::Staging     => "staging",
            SessionStatus::Committing  => "committing",
            SessionStatus::Pushing     => "pushing",
            SessionStatus::Pulling     => "pulling",
            SessionStatus::Success     => "success",
            SessionStatus::Failed      => "failed",
            SessionStatus::Conflicted  => "conflicted",
        }
    }
}

/// A single tracked git operation cycle.
#[derive(Debug, Clone)]
pub struct SyncSession {
    uuid:             Uuid,
    repository_uuid:  Uuid,
    account_id:       Uuid,
    session_type:     SessionType,
    status:           SessionStatus,
    branch_name:      String,
    commit_message:   Option<String>,
    commit_sha:       Option<String>,
    files_staged:     u32,
    commits_pushed:   u32,
    commits_pulled:   u32,
    conflicts_count:  u32,
    /// Combined stdout and stderr from all git subprocess calls.
    stdout_log:       String,
    stderr_log:       String,
    error_message:    Option<String>,
    started_at:       DateTime<Utc>,
    completed_at:     Option<DateTime<Utc>>,
}

impl SyncSession {
    /// Creates a new SyncSession in Pending state.
    pub fn new(
        repository_uuid: Uuid,
        account_id:      Uuid,
        session_type:    SessionType,
        branch_name:     String,
    ) -> Self {
        Self {
            uuid:            Uuid::new_v4(),
            repository_uuid,
            account_id,
            session_type,
            status:          SessionStatus::Pending,
            branch_name,
            commit_message:  None,
            commit_sha:      None,
            files_staged:    0,
            commits_pushed:  0,
            commits_pulled:  0,
            conflicts_count: 0,
            stdout_log:      String::new(),
            stderr_log:      String::new(),
            error_message:   None,
            started_at:      Utc::now(),
            completed_at:    None,
        }
    }

    /// Rehydrates from the database.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        uuid:             Uuid,
        repository_uuid:  Uuid,
        account_id:       Uuid,
        session_type:     SessionType,
        status:           SessionStatus,
        branch_name:      String,
        commit_message:   Option<String>,
        commit_sha:       Option<String>,
        files_staged:     u32,
        commits_pushed:   u32,
        commits_pulled:   u32,
        conflicts_count:  u32,
        stdout_log:       String,
        stderr_log:       String,
        error_message:    Option<String>,
        started_at:       DateTime<Utc>,
        completed_at:     Option<DateTime<Utc>>,
    ) -> Self {
        Self { uuid, repository_uuid, account_id, session_type, status, branch_name,
               commit_message, commit_sha, files_staged, commits_pushed, commits_pulled,
               conflicts_count, stdout_log, stderr_log, error_message, started_at, completed_at }
    }

    // ── Domain state transitions ───────────────────────────────────────────────

    pub fn begin_staging(&mut self)    { self.status = SessionStatus::Staging; }
    pub fn begin_committing(&mut self) { self.status = SessionStatus::Committing; }
    pub fn begin_pushing(&mut self)    { self.status = SessionStatus::Pushing; }
    pub fn begin_pulling(&mut self)    { self.status = SessionStatus::Pulling; }

    pub fn complete_successfully(
        &mut self,
        commit_sha:     Option<String>,
        files_staged:   u32,
        commits_pushed: u32,
        commits_pulled: u32,
    ) {
        self.status         = SessionStatus::Success;
        self.commit_sha     = commit_sha;
        self.files_staged   = files_staged;
        self.commits_pushed = commits_pushed;
        self.commits_pulled = commits_pulled;
        self.completed_at   = Some(Utc::now());
    }

    pub fn fail_with_error(&mut self, error: String) {
        self.status        = SessionStatus::Failed;
        self.error_message = Some(error);
        self.completed_at  = Some(Utc::now());
    }

    pub fn fail_with_conflicts(&mut self, conflicted_files: Vec<String>) {
        self.status          = SessionStatus::Conflicted;
        self.conflicts_count = conflicted_files.len() as u32;
        self.error_message   = Some(format!(
            "Merge conflict in {} file(s): {}",
            conflicted_files.len(),
            conflicted_files.join(", ")
        ));
        self.completed_at = Some(Utc::now());
    }

    pub fn append_stdout(&mut self, line: &str) {
        if !self.stdout_log.is_empty() { self.stdout_log.push('\n'); }
        self.stdout_log.push_str(line);
    }

    pub fn append_stderr(&mut self, line: &str) {
        if !self.stderr_log.is_empty() { self.stderr_log.push('\n'); }
        self.stderr_log.push_str(line);
    }

    pub fn set_commit_message(&mut self, msg: String) { self.commit_message = Some(msg); }

    // ── Getters ───────────────────────────────────────────────────────────────
    pub fn uuid(&self)             -> Uuid                    { self.uuid }
    pub fn repository_uuid(&self)  -> Uuid                    { self.repository_uuid }
    pub fn account_id(&self)       -> Uuid                    { self.account_id }
    pub fn session_type(&self)     -> &SessionType            { &self.session_type }
    pub fn status(&self)           -> &SessionStatus          { &self.status }
    pub fn branch_name(&self)      -> &str                    { &self.branch_name }
    pub fn commit_sha(&self)       -> Option<&str>            { self.commit_sha.as_deref() }
    pub fn commit_message(&self)   -> Option<&str>            { self.commit_message.as_deref() }
    pub fn files_staged(&self)     -> u32                     { self.files_staged }
    pub fn commits_pushed(&self)   -> u32                     { self.commits_pushed }
    pub fn commits_pulled(&self)   -> u32                     { self.commits_pulled }
    pub fn conflicts_count(&self)  -> u32                     { self.conflicts_count }
    pub fn stdout_log(&self)       -> &str                    { &self.stdout_log }
    pub fn stderr_log(&self)       -> &str                    { &self.stderr_log }
    pub fn error_message(&self)    -> Option<&str>            { self.error_message.as_deref() }
    pub fn started_at(&self)       -> DateTime<Utc>           { self.started_at }
    pub fn completed_at(&self)     -> Option<DateTime<Utc>>   { self.completed_at }
}