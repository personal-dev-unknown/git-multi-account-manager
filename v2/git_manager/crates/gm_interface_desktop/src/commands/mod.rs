// crates/gm_interface_desktop/src/commands/mod.rs
//
// Tauri command module registry.
//
// All Tauri command functions are grouped by domain area. Each module
// corresponds directly to a section of the Svelte frontend's pages:
//   accounts.rs     → pages/Accounts.svelte + components/AccountCard.svelte
//   ssh.rs          → pages/SSH.svelte + components/SshKeyRow.svelte
//   repositories.rs → pages/Repositories.svelte + pages/CloneWizard.svelte
//   git_ops.rs      → pages/SyncView.svelte
//
// The `invoke_handler!` macro in app.rs lists every public command function.
// Adding a new command requires only: (1) adding the fn here, (2) listing it
// in tauri::generate_handler![...] in app.rs, (3) calling invoke() in Svelte.

pub mod accounts;
pub mod git_ops;
pub mod repositories;
pub mod ssh;