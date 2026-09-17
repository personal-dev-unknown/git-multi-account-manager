// crates/gm_interface_desktop/src/app.rs
//
// Tauri application builder — the entry point for the desktop interface.
//
// `run_tauri_app()` is called from `apps/desktop/src/main.rs` after all
// domain services are constructed and wrapped in `AppState`. This function:
//   1. Registers `AppState` as Tauri managed state
//   2. Registers all Tauri command handlers via `generate_handler!`
//   3. Calls `tauri::Builder::default().run()` which blocks the main thread
//
// ── Why services are created outside Tauri ───────────────────────────────────
// The database pool creation is async and requires a Tokio runtime that exists
// before Tauri starts. The binary (`apps/desktop/main.rs`) creates a runtime,
// blocks on service creation, then passes the ready services here. Tauri
// creates its own internal Tokio runtime for async command handlers, but the
// initialization state is already done by the time Tauri starts.
//
// ── Async command handlers ────────────────────────────────────────────────────
// Tauri 1.x runs async commands on its internal runtime automatically. All
// command handlers annotated with `#[tauri::command]` and `async fn` are
// scheduled on Tauri's runtime without additional setup.

use crate::commands::{accounts, git_ops, repositories, ssh};
use crate::services::AppState;

/// Starts the Tauri event loop with all command handlers registered.
///
/// This function blocks the calling thread until the Tauri window is closed.
/// It is called once from `apps/desktop/src/main.rs` as the final step after
/// all domain services are initialized and wrapped in `AppState`.
///
/// # Parameters
/// - `app_state` — the fully initialized application services, ready to serve
///   Tauri command invocations from the Svelte frontend.
///
/// # Panics
/// Panics if the Tauri event loop cannot start (e.g. missing tauri.conf.json,
/// invalid icon path, or unsupported platform). All startup errors are fatal.
pub fn run_tauri_app(app_state: AppState) {
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Account commands
            accounts::list_accounts,
            accounts::get_account,
            accounts::add_account,
            accounts::remove_account,
            accounts::set_default_account,
            accounts::store_account_token,
            accounts::list_platforms,
            // SSH key commands
            ssh::list_ssh_keys,
            ssh::generate_ssh_key,
            ssh::test_ssh_connection,
            ssh::add_key_to_agent,
            ssh::get_public_key,
            // Repository commands
            repositories::list_repositories,
            repositories::clone_repository,
            repositories::list_remote_repositories,
            repositories::get_repository,
            repositories::get_repository_local_path,
            // Git operation commands
            git_ops::git_pull,
            git_ops::git_push,
            git_ops::git_status,
            git_ops::sync_repository,
        ])
        .run(tauri::generate_context!())
        .expect(
            "Git Manager desktop failed to start. \
             Check that tauri.conf.json is present and all required icons exist."
        );
}