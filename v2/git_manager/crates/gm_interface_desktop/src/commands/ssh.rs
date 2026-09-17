// crates/gm_interface_desktop/src/commands/ssh.rs
//
// Tauri command handlers for SSH key management.
//
// SSH operations are security-sensitive: key generation writes to ~/.ssh,
// the test connection dials the real SSH endpoint, and add-to-agent modifies
// the running SSH agent state. Errors are always propagated as descriptive
// strings so the Svelte frontend can display them in a notification.

use tauri::State;
use uuid::Uuid;
use gm_ports::inbound::commands::{GenerateSshKeyCommand, TestSshConnectionCommand};
use gm_shared::models::ssh_key::SshKeyDto;

use crate::services::{AppState, DesktopSshTestResult};

/// Lists all SSH keys for a specific account.
///
/// Returns keys in reverse chronological order (newest first).
///
/// # JavaScript
/// ```ts
/// const keys = await invoke<SshKeyDto[]>('list_ssh_keys', { accountUuid: '...' });
/// ```
#[tauri::command]
pub async fn list_ssh_keys(
    state:        State<'_, AppState>,
    account_uuid: String,
) -> Result<Vec<SshKeyDto>, String> {
    let id = Uuid::parse_str(&account_uuid).map_err(|e| format!("invalid account UUID: {e}"))?;
    state.services().list_ssh_keys(id).await.map_err(|e| e.to_string())
}

/// Generates a new SSH key pair for an account and writes it to ~/.ssh.
///
/// This operation:
///   1. Deactivates the account's current active key (if any)
///   2. Runs ssh-keygen via the Zig SSH layer
///   3. Writes a Host block to ~/.ssh/config
///   4. Optionally adds the key to the SSH agent
///
/// The private key is written to ~/.ssh/id_{type}_{platform}_{alias}.
///
/// # Parameters
/// - `cmd.keyType`    — "ed25519" (recommended) or "rsa"
/// - `cmd.addToAgent` — whether to automatically add to the SSH agent
///
/// # JavaScript
/// ```ts
/// const key = await invoke<SshKeyDto>('generate_ssh_key', {
///   cmd: { accountUuid: '...', keyType: 'ed25519', addToAgent: true, passphrase: null, comment: null }
/// });
/// ```
#[tauri::command]
pub async fn generate_ssh_key(
    state: State<'_, AppState>,
    cmd:   GenerateSshKeyCommand,
) -> Result<SshKeyDto, String> {
    state.services().generate_ssh_key(cmd).await.map_err(|e| e.to_string())
}

/// Tests the SSH connection for an account's active key.
///
/// Attempts an actual SSH connection to the platform's SSH endpoint to verify
/// the key is accepted. Returns whether the connection succeeded, and if so,
/// the username confirmed by the remote server (e.g. "Hi shakamoses!").
///
/// # JavaScript
/// ```ts
/// const result = await invoke<SshTestResult>('test_ssh_connection', {
///   cmd: { accountUuid: '...', timeoutMs: 10000 }
/// });
/// ```
#[tauri::command]
pub async fn test_ssh_connection(
    state: State<'_, AppState>,
    cmd:   TestSshConnectionCommand,
) -> Result<DesktopSshTestResult, String> {
    state.services().test_ssh_connection(cmd).await.map_err(|e| e.to_string())
}

/// Adds the account's active SSH key to the running SSH agent.
///
/// Requires the SSH agent socket to be available (SSH_AUTH_SOCK must be set).
/// The key is added without a passphrase for passwordless operation. If the
/// key was generated with a passphrase, it must be provided when adding.
///
/// # JavaScript
/// ```ts
/// await invoke('add_key_to_agent', { accountUuid: '...' });
/// ```
#[tauri::command]
pub async fn add_key_to_agent(
    state:        State<'_, AppState>,
    account_uuid: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&account_uuid).map_err(|e| format!("invalid account UUID: {e}"))?;
    state.services().add_key_to_agent(id).await.map_err(|e| e.to_string())
}

/// Returns the public key content for an account's active SSH key.
///
/// This is the string the user needs to paste into their platform's
/// SSH key settings (e.g. GitHub Settings → SSH and GPG keys → New SSH key).
///
/// # JavaScript
/// ```ts
/// const pubKey = await invoke<string | null>('get_public_key', { accountUuid: '...' });
/// ```
#[tauri::command]
pub async fn get_public_key(
    state:        State<'_, AppState>,
    account_uuid: String,
) -> Result<Option<String>, String> {
    let id   = Uuid::parse_str(&account_uuid).map_err(|e| format!("invalid account UUID: {e}"))?;
    let keys = state.services().list_ssh_keys(id).await.map_err(|e| e.to_string())?;
    // Return the public key of the active key (is_active = true)
    let active_pub_key = keys.into_iter()
        .filter(|k| k.is_active)
        .map(|k| k.public_key)
        .next();
    Ok(active_pub_key)
}