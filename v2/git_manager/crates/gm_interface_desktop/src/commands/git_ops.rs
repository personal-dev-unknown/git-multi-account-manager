// crates/gm_interface_desktop/src/commands/git_ops.rs
//
// Tauri command handlers for git operations (pull, push, status, sync).
//
// These are the most user-visible commands in the desktop app — the SyncView
// page calls them when the user triggers a sync. Each command is async and
// may take several seconds for large repositories. The Svelte store shows a
// ProgressBar component while these commands are in flight.

use tauri::State;
use uuid::Uuid;
use gm_ports::inbound::commands::{PullRepositoryCommand, PushRepositoryCommand};

use crate::services::{AppState, DesktopGitOpResult, DesktopGitStatusEntry};

/// Pulls changes from the remote into a local repository.
///
/// Uses the account's active SSH key for authentication. If the pull results
/// in a merge conflict, `hadConflicts` is true in the result and the local
/// repository is left with conflict markers for the user to resolve.
///
/// # JavaScript
/// ```ts
/// const result = await invoke<GitOpResult>('git_pull', {
///   cmd: { repositoryUuid: '...', accountUuid: '...', branch: null, rebase: false }
/// });
/// ```
#[tauri::command]
pub async fn git_pull(
    state: State<'_, AppState>,
    cmd:   PullRepositoryCommand,
) -> Result<DesktopGitOpResult, String> {
    state.services().git_pull(cmd).await.map_err(|e| e.to_string())
}

/// Stages all changes, commits with the provided message, and pushes to the remote.
///
/// This is a three-step atomic sequence (from the user's perspective):
///   1. `git add -A`
///   2. `git commit -m "{commitMessage}"`
///   3. `git push {remote} {branch}`
///
/// If any step fails, subsequent steps are skipped. The user sees the
/// exact error from the Zig git executor.
///
/// # JavaScript
/// ```ts
/// const result = await invoke<GitOpResult>('git_push', {
///   cmd: {
///     repositoryUuid: '...', accountUuid: '...',
///     commitMessage: 'feat: add dark mode', branch: 'main', force: false
///   }
/// });
/// ```
#[tauri::command]
pub async fn git_push(
    state: State<'_, AppState>,
    cmd:   PushRepositoryCommand,
) -> Result<DesktopGitOpResult, String> {
    state.services().git_push(cmd).await.map_err(|e| e.to_string())
}

/// Returns the working directory status of a cloned repository.
///
/// Lists modified, untracked, staged, and conflicted files.
/// Used by the SyncView page to show pending changes before push.
///
/// # JavaScript
/// ```ts
/// const status = await invoke<GitStatusEntry[]>('git_status', { repositoryUuid: '...' });
/// ```
#[tauri::command]
pub async fn git_status(
    state:           State<'_, AppState>,
    repository_uuid: String,
) -> Result<Vec<DesktopGitStatusEntry>, String> {
    let id = Uuid::parse_str(&repository_uuid)
        .map_err(|e| format!("invalid repository UUID: {e}"))?;
    state.services().git_status(id).await.map_err(|e| e.to_string())
}

/// Performs a full sync: pull then push if the pull succeeded cleanly.
///
/// Used by the "Sync" button on the Repositories page. Equivalent to:
///   1. `git pull --rebase` (to integrate remote changes)
///   2. `git push` (to send local commits)
///
/// If the pull fails or has conflicts, the push step is skipped.
///
/// # JavaScript
/// ```ts
/// const result = await invoke<SyncResult>('sync_repository', {
///   repositoryUuid: '...', accountUuid: '...',
///   commitMessage: 'chore: sync', branch: 'main'
/// });
/// ```
#[tauri::command]
pub async fn sync_repository(
    state:           State<'_, AppState>,
    repository_uuid: String,
    account_uuid:    String,
    commit_message:  String,
    branch:          String,
) -> Result<serde_json::Value, String> {
    let repo_id = Uuid::parse_str(&repository_uuid).map_err(|e| format!("invalid repo UUID: {e}"))?;
    let acc_id  = Uuid::parse_str(&account_uuid).map_err(|e| format!("invalid account UUID: {e}"))?;

    // Step 1: Pull (rebase)
    let pull_cmd = PullRepositoryCommand {
        repository_uuid: repo_id,
        account_uuid:    acc_id,
        branch:          Some(branch.clone()),
        rebase:          true,
        dry_run:         false,
    };
    let pull_result = state.services().git_pull(pull_cmd).await.map_err(|e| e.to_string())?;

    if pull_result.had_conflicts {
        return Ok(serde_json::json!({
            "pull": pull_result,
            "push": null,
            "had_conflicts": true,
            "message": "Pull succeeded but merge conflicts were detected. Resolve conflicts before pushing."
        }));
    }

    // Step 2: Push
    let push_cmd = PushRepositoryCommand {
        repository_uuid: repo_id,
        account_uuid:    acc_id,
        commit_message,
        branch,
        force:  false,
        dry_run: false,
    };
    let push_result = state.services().git_push(push_cmd).await.map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "pull": pull_result,
        "push": push_result,
        "had_conflicts": false,
        "message": format!("Synced successfully. Pulled {} commits, pushed {} commits.",
            pull_result.commits_transferred, push_result.commits_transferred)
    }))
}