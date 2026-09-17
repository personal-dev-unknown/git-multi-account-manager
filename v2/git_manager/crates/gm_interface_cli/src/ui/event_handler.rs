// crates/gm_interface_cli/src/ui/event_handler.rs
//
// CliEventHandler — the terminal notification layer for domain events.
//
// This handler is registered as a global subscriber on the kernel event bus
// in CliPlugin::run() before the interactive session starts.  It receives
// every KernelEvent that flows through the system and prints a compact,
// colour-coded one-liner to stdout so the user sees real-time feedback for
// operations that happen in the background or as side-effects of a command.
//
// ── Design decisions ──────────────────────────────────────────────────────────
// • subscribe_all()  — one registration, all events, no gap if new event types
//   are added to the domain without a corresponding subscription entry here.
// • get_event_subscriptions() still lists every event individually so the
//   kernel's plugin lifecycle logger can show a clean subscription manifest
//   in the debug trace on boot.
// • Notifications are written directly to stdout with println!.  This is safe
//   because Rust's println! is line-buffered and each call is atomic within a
//   single thread.  Interleaving with dialoguer prompts is acceptable: the
//   notification appears above the prompt on the next render cycle.
// • Errors in payload deserialisation are logged at debug level and swallowed —
//   an unknown or malformed payload must never crash the terminal session.

use std::sync::Arc;

use async_trait::async_trait;
use gm_kernel::{contracts::event::KernelEvent, event_bus::EventHandler};
use gm_shared::{constants::events::*, errors::GitManagerError};

use crate::ui::colors;

/// Global terminal notification handler.
///
/// Registered once with `kernel.event_bus.subscribe_all()` in CliPlugin::run().
#[derive(Debug)]
pub struct CliEventHandler;

impl CliEventHandler {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait]
impl EventHandler for CliEventHandler {
    async fn handle(&self, event: &KernelEvent) -> Result<(), GitManagerError> {
        let msg = format_event(event);
        if let Some(line) = msg {
            println!("{line}");
        }
        Ok(())
    }
}

/// Maps a KernelEvent to a human-readable notification line.
/// Returns None for event types that do not need terminal output.
fn format_event(event: &KernelEvent) -> Option<String> {
    let p = &event.payload;

    match event.event_type.as_str() {
        // ── Account ───────────────────────────────────────────────────────────
        ACCOUNT_ADDED => {
            let alias    = p["alias"].as_str().unwrap_or("?");
            let platform = p["platform_id"].as_str().unwrap_or("?");
            Some(format!(
                "\n{} Account added  {} {}",
                colors::success_prefix(),
                colors::bold(alias),
                colors::dim(&format!("(platform: {platform})"))
            ))
        }
        ACCOUNT_REMOVED => {
            let alias = p["alias"].as_str().unwrap_or("?");
            Some(format!(
                "\n{} Account removed  {}",
                colors::warn_prefix(),
                colors::bold(alias)
            ))
        }
        ACCOUNT_STATUS_CHANGED => {
            let prev = p["previous_status"].as_str().unwrap_or("?");
            let next = p["new_status"].as_str().unwrap_or("?");
            Some(format!(
                "\n{} Account status  {} → {}",
                colors::info_prefix(),
                colors::dim(prev),
                colors::bold(next)
            ))
        }

        // ── SSH keys ──────────────────────────────────────────────────────────
        SSH_KEY_GENERATED => {
            let fp  = p["fingerprint"].as_str().unwrap_or("?");
            let typ = p["key_type_str"].as_str().unwrap_or("?");
            Some(format!(
                "\n{} SSH key generated  {} {}",
                colors::success_prefix(),
                colors::bold(typ),
                colors::dim(fp)
            ))
        }
        SSH_KEY_TESTED => {
            let ok  = p["success"].as_bool().unwrap_or(false);
            let who = p["username"].as_str().unwrap_or("");
            let err = p["error"].as_str().unwrap_or("");
            if ok {
                Some(format!(
                    "\n{} SSH test passed  authenticated as {}",
                    colors::success_prefix(),
                    colors::bold(who)
                ))
            } else {
                Some(format!(
                    "\n{} SSH test failed  {}",
                    colors::error_prefix(),
                    colors::dim(err)
                ))
            }
        }
        SSH_KEY_ADDED_TO_AGENT => {
            Some(format!(
                "\n{} SSH key loaded into agent",
                colors::success_prefix()
            ))
        }

        // ── Repositories ──────────────────────────────────────────────────────
        REPOSITORY_DISCOVERED => {
            let name = p["full_name"].as_str().unwrap_or("?");
            Some(format!(
                "\n{} Repository discovered  {}",
                colors::info_prefix(),
                colors::bold(name)
            ))
        }
        REPOSITORY_CLONED => {
            let name = p["full_name"].as_str().unwrap_or("?");
            let path = p["local_path"].as_str().unwrap_or("?");
            Some(format!(
                "\n{} Cloned {}  →  {}",
                colors::success_prefix(),
                colors::bold(name),
                colors::dim(path)
            ))
        }
        REPOSITORY_SYNCED => {
            let sha = p["commit_sha"].as_str().unwrap_or("?");
            let short = &sha[..sha.len().min(8)];
            Some(format!(
                "\n{} Repository synced  {}",
                colors::success_prefix(),
                colors::dim(short)
            ))
        }

        // ── Git operations ────────────────────────────────────────────────────
        COMMIT_CREATED => {
            let msg = p["commit_message"].as_str().unwrap_or("?");
            let sha = p["commit_sha"].as_str().unwrap_or("?");
            let short = &sha[..sha.len().min(8)];
            Some(format!(
                "\n{} Committed  {} {}",
                colors::success_prefix(),
                colors::bold(msg),
                colors::dim(&format!("({short})"))
            ))
        }
        BRANCH_CREATED => {
            let branch = p["branch_name"].as_str().unwrap_or("?");
            let base   = p["base_commit_sha"].as_str().unwrap_or("?");
            let short  = &base[..base.len().min(8)];
            Some(format!(
                "\n{} Branch created  {} at {}",
                colors::success_prefix(),
                colors::bold(branch),
                colors::dim(short)
            ))
        }
        MERGE_CONFLICT_DETECTED => {
            let files: Vec<&str> = p["conflicted_files"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();
            let list = files.join(", ");
            Some(format!(
                "\n{} Merge conflicts detected  {}",
                colors::error_prefix(),
                colors::dim(&list)
            ))
        }

        // ── Sync sessions ─────────────────────────────────────────────────────
        SYNC_STARTED => {
            let kind = p["session_type"].as_str().unwrap_or("sync");
            Some(format!(
                "\n{} {} session started",
                colors::info_prefix(),
                colors::bold(kind)
            ))
        }
        SYNC_COMPLETED => {
            let ok      = p["success"].as_bool().unwrap_or(false);
            let pushed  = p["commits_pushed"].as_u64().unwrap_or(0);
            let pulled  = p["commits_pulled"].as_u64().unwrap_or(0);
            if ok {
                Some(format!(
                    "\n{} Sync completed  {} pushed, {} pulled",
                    colors::success_prefix(),
                    colors::bold(&pushed.to_string()),
                    colors::bold(&pulled.to_string())
                ))
            } else {
                Some(format!(
                    "\n{} Sync failed",
                    colors::error_prefix()
                ))
            }
        }
        CONFLICT_DETECTED => {
            let files: Vec<&str> = p["conflicted_files"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();
            let list = files.join(", ");
            Some(format!(
                "\n{} Sync conflict  {}",
                colors::warn_prefix(),
                colors::dim(&list)
            ))
        }

        // ── Workflows ─────────────────────────────────────────────────────────
        WORKFLOW_STARTED => {
            let name = p["workflow_name"].as_str().unwrap_or("workflow");
            Some(format!(
                "\n{} Workflow started  {}",
                colors::info_prefix(),
                colors::dim(name)
            ))
        }
        WORKFLOW_COMPLETED => {
            let name = p["workflow_name"].as_str().unwrap_or("workflow");
            Some(format!(
                "\n{} Workflow completed  {}",
                colors::success_prefix(),
                colors::dim(name)
            ))
        }
        WORKFLOW_FAILED => {
            let name  = p["workflow_name"].as_str().unwrap_or("workflow");
            let error = p["error"].as_str().unwrap_or("unknown error");
            Some(format!(
                "\n{} Workflow failed  {}  — {}",
                colors::error_prefix(),
                colors::dim(name),
                colors::dim(error)
            ))
        }

        // Unknown / internal events — no terminal output.
        _ => None,
    }
}
