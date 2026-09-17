//! Event type string constants used by the event bus and event store.
//!
//! These constants serve a concrete purpose beyond convenience: they prevent
//! typos in event type strings that would only fail at runtime. Every place
//! in the codebase that publishes or subscribes to an event uses these constants
//! instead of inline string literals. If you mistype `ACCOUNT_ADDEDD` the
//! compiler catches it; if you mistype `"AccountAddedd"` in a string it silently
//! routes to nothing.
//!
//! Naming convention: `DOMAIN_VERB` in SCREAMING_SNAKE_CASE for constants,
//! mapped to `DomainVerb` PascalCase in the event payload. The two must match
//! because the event store records and replays events by their string type.

// ── Account events ────────────────────────────────────────────────────────────
/// Published after a new account is created and its initial record is persisted.
pub const ACCOUNT_ADDED: &str = "AccountAdded";

/// Published after an account is permanently removed from the system.
pub const ACCOUNT_REMOVED: &str = "AccountRemoved";

/// Published when an account's lifecycle status changes (e.g. Unverified → Active).
pub const ACCOUNT_STATUS_CHANGED: &str = "AccountStatusChanged";

// ── SSH key events ────────────────────────────────────────────────────────────
/// Published after ssh-keygen successfully writes the key pair to disk and
/// the public key + fingerprint are recorded in the database.
pub const SSH_KEY_GENERATED: &str = "SshKeyGenerated";

/// Published after a connection test completes, whether successful or not.
/// The event payload includes the test status so listeners can react to both
/// success (update the account to Active) and failure (notify the user).
pub const SSH_KEY_TESTED: &str = "SshKeyTested";

/// Published after the key is successfully loaded into the SSH agent via ssh-add.
pub const SSH_KEY_ADDED_TO_AGENT: &str = "SshKeyAddedToAgent";

// ── Repository events ─────────────────────────────────────────────────────────
/// Published after a repository is found via the platform API and its metadata
/// record is written to the database (before it is cloned locally).
pub const REPOSITORY_DISCOVERED: &str = "RepositoryDiscovered";

/// Published after git clone completes successfully and is_cloned is set to true.
pub const REPOSITORY_CLONED: &str = "RepositoryCloned";

/// Published after a push or pull operation updates the repository's last_synced_at
/// and last_commit_sha fields.
pub const REPOSITORY_SYNCED: &str = "RepositorySynced";

// ── Git operation events ──────────────────────────────────────────────────────
/// Published after a git commit is created by the sync service.
pub const COMMIT_CREATED: &str = "CommitCreated";

/// Published after a new branch is created in a managed repository.
pub const BRANCH_CREATED: &str = "BranchCreated";

/// Published when a pull operation encounters merge conflicts that require
/// manual resolution before the operation can continue.
pub const MERGE_CONFLICT_DETECTED: &str = "MergeConflictDetected";

// ── Sync session events ───────────────────────────────────────────────────────
/// Published when a sync session transitions from Pending to its first
/// active state (Staging, Pushing, or Pulling).
pub const SYNC_STARTED: &str = "SyncStarted";

/// Published when a sync session reaches a terminal Success state.
pub const SYNC_COMPLETED: &str = "SyncCompleted";

/// Published within a sync session when a conflict is detected mid-operation.
/// Distinct from MERGE_CONFLICT_DETECTED because it carries sync session context.
pub const CONFLICT_DETECTED: &str = "ConflictDetected";

// ── Workflow events ───────────────────────────────────────────────────────────
// ── Clone progress events ─────────────────────────────────────────────────────
/// Published periodically during a git clone to report progress percentage.
pub const CLONE_PROGRESS_UPDATED: &str = "CloneProgressUpdated";

/// Published when the workflow engine begins executing a workflow instance.
pub const WORKFLOW_STARTED: &str = "WorkflowStarted";

/// Published when a workflow instance reaches the Completed terminal state.
pub const WORKFLOW_COMPLETED: &str = "WorkflowCompleted";

/// Published when a workflow instance reaches the Failed terminal state.
pub const WORKFLOW_FAILED: &str = "WorkflowFailed";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_constants_are_pascal_case() {
        // Verify the pattern: constant string value starts with uppercase
        // and contains no underscores (they're camel-case in the string).
        for constant in [
            ACCOUNT_ADDED, ACCOUNT_REMOVED, SSH_KEY_GENERATED,
            REPOSITORY_CLONED, SYNC_COMPLETED, WORKFLOW_FAILED,
        ] {
            assert!(
                constant.chars().next().unwrap().is_uppercase(),
                "Event type '{constant}' must start with uppercase"
            );
            assert!(
                !constant.contains('_'),
                "Event type '{constant}' must not contain underscores"
            );
        }
    }
}