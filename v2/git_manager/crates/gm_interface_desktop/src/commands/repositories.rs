// crates/gm_interface_desktop/src/commands/repositories.rs
//
// Tauri command handlers for repository management operations.

use tauri::State;
use uuid::Uuid;
use gm_ports::inbound::commands::CloneRepositoryCommand;
use gm_shared::models::repository::RepositoryDto;

use crate::services::AppState;

/// Lists repositories, optionally filtered by account.
///
/// Returns cloned repositories when `accountUuid` is None (shows the dashboard
/// view with all locally-cloned repos). When `accountUuid` is provided, lists
/// all repos for that specific account including uncloned remote ones.
///
/// # JavaScript
/// ```ts
/// const repos = await invoke<RepositoryDto[]>('list_repositories', { accountUuid: null });
/// ```
#[tauri::command]
pub async fn list_repositories(
    state:        State<'_, AppState>,
    account_uuid: Option<String>,
) -> Result<Vec<RepositoryDto>, String> {
    let id = account_uuid
        .map(|s| Uuid::parse_str(&s).map_err(|e| format!("invalid account UUID: {e}")))
        .transpose()?;
    state.services().list_repositories(id).await.map_err(|e| e.to_string())
}

/// Clones a remote repository using an account's SSH credentials.
///
/// Performs the full clone workflow:
///   1. Retrieves the account's active SSH key
///   2. Invokes the Zig git executor to run `git clone` with the correct
///      GIT_SSH_COMMAND pointing to the account's private key
///   3. Creates a Repository entity in the database
///   4. Returns the persisted RepositoryDto with the local path set
///
/// # Parameters
/// - `cmd.url`         — remote SSH or HTTPS URL (e.g. "git@github.com:owner/repo.git")
/// - `cmd.accountUuid` — which account's SSH key to use
/// - `cmd.destination` — optional local path; inferred from URL if null
/// - `cmd.branch`      — optional branch; clones default branch if null
/// - `cmd.depth`       — 0 = full history, > 0 = shallow clone with that many commits
///
/// # JavaScript
/// ```ts
/// const repo = await invoke<RepositoryDto>('clone_repository', {
///   cmd: {
///     url: 'git@github.com:owner/repo.git',
///     accountUuid: '...',
///     destination: null,
///     branch: null,
///     depth: 0,
///   }
/// });
/// ```
#[tauri::command]
pub async fn clone_repository(
    state: State<'_, AppState>,
    cmd:   CloneRepositoryCommand,
) -> Result<RepositoryDto, String> {
    state.services().clone_repository(cmd).await.map_err(|e| e.to_string())
}

/// Retrieves a single repository by UUID.
///
/// Returns null if the repository is not found in the local database.
///
/// # JavaScript
/// ```ts
/// const repo = await invoke<RepositoryDto | null>('get_repository', { uuid: '...' });
/// ```
#[tauri::command]
pub async fn get_repository(
    state: State<'_, AppState>,
    uuid:  String,
) -> Result<Option<RepositoryDto>, String> {
    let id   = Uuid::parse_str(&uuid).map_err(|e| format!("invalid repository UUID: {e}"))?;
    let repos = state.services().list_repositories(None).await.map_err(|e| e.to_string())?;
    Ok(repos.into_iter().find(|r| r.uuid == id))
}

/// Lists repositories from the platform API (paginated), using the account's PAT.
///
/// # JavaScript
/// ```ts
/// const repos = await invoke<string[]>('list_remote_repositories', {
///   accountUuid: '...',
///   page: 1,
///   perPage: 15,
/// });
/// ```
#[tauri::command]
pub async fn list_remote_repositories(
    state:        State<'_, AppState>,
    account_uuid: String,
    page:         Option<u32>,
    per_page:     Option<u32>,
) -> Result<Vec<String>, String> {
    let id   = Uuid::parse_str(&account_uuid).map_err(|e| format!("invalid account UUID: {e}"))?;
    let p    = page.unwrap_or(1);
    let pp   = per_page.unwrap_or(15);
    state.services().list_remote_repositories(id, p, pp).await.map_err(|e| e.to_string())
}

/// Opens the repository's local directory in the system's file manager.
///
/// Uses Tauri's shell API via the frontend rather than a Rust command.
/// This command exists as a stub that returns the local path for the
/// Svelte frontend to open via `open()` from the @tauri-apps/api shell module.
///
/// # JavaScript
/// ```ts
/// const path = await invoke<string | null>('get_repository_local_path', { uuid: '...' });
/// if (path) { await open(path); }
/// ```
#[tauri::command]
pub async fn get_repository_local_path(
    state: State<'_, AppState>,
    uuid:  String,
) -> Result<Option<String>, String> {
    let id    = Uuid::parse_str(&uuid).map_err(|e| format!("invalid repository UUID: {e}"))?;
    let repos = state.services().list_repositories(None).await.map_err(|e| e.to_string())?;
    Ok(repos.into_iter().find(|r| r.uuid == id).and_then(|r| r.local_path))
}