// crates/gm_interface_desktop/src/lib.rs
//
// Library root for the `gm_interface_desktop` crate.
//
// This crate is built as both a `cdylib` (for Tauri's dynamic loading) and
// an `rlib` (for the binary to link against). The `apps/desktop/main.rs`
// binary imports this crate to call `run_tauri_app()` with the constructed
// `AppState`.
//
// ── Public API ────────────────────────────────────────────────────────────────
// - `run_tauri_app(AppState)` — starts the Tauri event loop (in app.rs)
// - `AppState`                — wraps Arc<dyn DesktopServices> for managed state
// - `DesktopServices`         — the service facade trait (in services.rs)
// - `DesktopPlugin`           — the kernel Plugin implementation (in plugin.rs)

pub mod app;
pub mod banner;
pub mod commands;
pub mod plugin;
pub mod services;

pub use app::run_tauri_app;
pub use plugin::DesktopPlugin;
pub use services::{AppState, DesktopServices, DesktopGitOpResult, DesktopSshTestResult, DesktopGitStatusEntry};