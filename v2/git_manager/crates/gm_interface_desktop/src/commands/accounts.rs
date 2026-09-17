// crates/gm_interface_desktop/src/commands/accounts.rs
//
// Tauri command handlers for account management operations.
//
// Each function is a Tauri command handler: it receives `tauri::State<'_, AppState>`
// plus typed parameters deserialized from JavaScript JSON, and returns a
// `Result<T, String>` where T implements `serde::Serialize`.
//
// ── UUID handling ─────────────────────────────────────────────────────────────
// JavaScript/JSON represents UUIDs as strings. Parameters that should be UUIDs
// are received as `Option<String>` or `String` and parsed inside the handler
// with a clean error message on failure. The trait API uses native `Uuid` for
// type safety throughout all lower layers.
//
// ── Error propagation ─────────────────────────────────────────────────────────
// Errors become `Err(String)` which Tauri serializes as a rejected Promise in
// JavaScript. The Svelte store wraps this in a user-visible notification.

use tauri::State;
use uuid::Uuid;
use gm_ports::inbound::commands::AddAccountCommand;
use gm_shared::models::account::AccountDto;

use crate::services::AppState;

/// Lists all accounts, optionally filtered by platform UUID.
///
/// # Parameters
/// - `platform_id` — optional platform UUID string; null/undefined = all platforms
///
/// # JavaScript
/// ```ts
/// const accounts = await invoke<AccountDto[]>('list_accounts', { platformId: null });
/// ```
#[tauri::command]
pub async fn list_accounts(
    state:       State<'_, AppState>,
    platform_id: Option<String>,
) -> Result<Vec<AccountDto>, String> {
    let pid = platform_id
        .map(|s| Uuid::parse_str(&s).map_err(|e| format!("invalid platform_id UUID: {e}")))
        .transpose()?;

    state.services().list_accounts(pid).await.map_err(|e| e.to_string())
}

/// Retrieves a single account by UUID.
///
/// Returns `null` to JavaScript if the account does not exist.
///
/// # JavaScript
/// ```ts
/// const account = await invoke<AccountDto | null>('get_account', { uuid: '...' });
/// ```
#[tauri::command]
pub async fn get_account(
    state: State<'_, AppState>,
    uuid:  String,
) -> Result<Option<AccountDto>, String> {
    let id = Uuid::parse_str(&uuid).map_err(|e| format!("invalid account UUID: {e}"))?;
    state.services().get_account(id).await.map_err(|e| e.to_string())
}

/// Creates a new account.
///
/// # JavaScript
/// ```ts
/// const account = await invoke<AccountDto>('add_account', {
///   cmd: { alias: 'work', platformId: '...', username: 'shaka', email: 'shaka@example.com', authMethod: 'ssh' }
/// });
/// ```
#[tauri::command]
pub async fn add_account(
    state: State<'_, AppState>,
    cmd:   AddAccountCommand,
) -> Result<AccountDto, String> {
    state.services().add_account(cmd).await.map_err(|e| e.to_string())
}

/// Removes an account and its associated SSH keys and credentials.
///
/// # JavaScript
/// ```ts
/// await invoke('remove_account', { uuid: '...' });
/// ```
#[tauri::command]
pub async fn remove_account(
    state: State<'_, AppState>,
    uuid:  String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&uuid).map_err(|e| format!("invalid account UUID: {e}"))?;
    state.services().remove_account(id).await.map_err(|e| e.to_string())
}

/// Sets the default account for a platform.
/// Only one account per platform can be the default at a time.
///
/// # JavaScript
/// ```ts
/// await invoke('set_default_account', { accountUuid: '...', platformId: '...' });
/// ```
#[tauri::command]
pub async fn set_default_account(
    state:        State<'_, AppState>,
    account_uuid: String,
    platform_id:  String,
) -> Result<(), String> {
    let aid = Uuid::parse_str(&account_uuid).map_err(|e| format!("invalid account UUID: {e}"))?;
    let pid = Uuid::parse_str(&platform_id).map_err(|e| format!("invalid platform UUID: {e}"))?;
    state.services().set_default_account(aid, pid).await.map_err(|e| e.to_string())
}

/// Stores or replaces the authentication token (PAT) for an account.
///
/// # Security
/// The token is encrypted by the CredentialVault before being stored.
/// It is never written to disk in plaintext. The vault uses the machine's
/// hardware identity to derive the encryption key.
///
/// # JavaScript
/// ```ts
/// await invoke('store_account_token', { accountUuid: '...', token: 'ghp_...' });
/// ```
#[tauri::command]
pub async fn store_account_token(
    state:        State<'_, AppState>,
    account_uuid: String,
    token:        String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&account_uuid).map_err(|e| format!("invalid account UUID: {e}"))?;
    state.services().store_account_token(id, token).await.map_err(|e| e.to_string())
}

/// Returns the list of all known platform UUIDs and display names.
/// Used by the "add account" form to populate the platform dropdown.
///
/// # JavaScript
/// ```ts
/// const platforms = await invoke<Platform[]>('list_platforms');
/// ```
#[tauri::command]
pub async fn list_platforms() -> Result<Vec<serde_json::Value>, String> {
    use serde_json::json;
    Ok(vec![
        json!({ "uuid": "00000000-0001-0000-0000-000000000001", "name": "github",       "display_name": "GitHub",       "ssh_host": "github.com" }),
        json!({ "uuid": "00000000-0002-0000-0000-000000000001", "name": "gitlab",       "display_name": "GitLab",       "ssh_host": "gitlab.com" }),
        json!({ "uuid": "00000000-0003-0000-0000-000000000001", "name": "bitbucket",    "display_name": "Bitbucket",    "ssh_host": "bitbucket.org" }),
        json!({ "uuid": "00000000-0004-0000-0000-000000000001", "name": "azure_devops", "display_name": "Azure DevOps", "ssh_host": "ssh.dev.azure.com" }),
        json!({ "uuid": "00000000-0005-0000-0000-000000000001", "name": "sourceforge",  "display_name": "SourceForge",  "ssh_host": "git.code.sf.net" }),
    ])
}