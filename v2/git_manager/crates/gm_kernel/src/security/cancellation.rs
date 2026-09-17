use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A registered function that kills the currently running child subprocess.
/// Set once during binary startup (main.rs) and called from Ctrl+C handlers.
/// Uses a function pointer (no allocation, Send+Sync) for simplicity.
static KILL_CHILD: std::sync::OnceLock<fn() -> bool> = std::sync::OnceLock::new();

/// A shared cancellation flag that can be triggered by a Ctrl+C handler
/// and checked by long-running operations (clone, pull, push).
#[derive(Debug, Clone, Default)]
pub struct Cancelled {
    flag: Arc<AtomicBool>,
}

impl Cancelled {
    pub fn new() -> Self {
        Self { flag: Arc::new(AtomicBool::new(false)) }
    }

    /// Returns true if a cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }

    /// Signal cancellation.
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Release);
    }

    /// Reset to uncancelled.
    pub fn reset(&self) {
        self.flag.store(false, Ordering::Release);
    }
}

/// Register a function that can kill the currently running child subprocess.
/// Called once during binary startup with a closure that invokes the FFI
/// `gm_git_kill_process(0)` function from the Zig native layer.
/// The `0` argument tells Zig to kill whatever child it is currently tracking.
pub fn register_child_killer(killer: fn() -> bool) {
    KILL_CHILD.set(killer).ok();
}

/// Kill the currently running child subprocess (if any).
/// Returns true if a kill signal was sent, false if nothing to kill.
/// Safe to call even when no child process is running (returns false).
pub fn kill_current_child() -> bool {
    KILL_CHILD.get().is_some_and(|k| k())
}

/// Installs a global Ctrl+C handler that sets the given `Cancelled` flag
/// on the first SIGINT, and exits the process on a second SIGINT.
pub fn install_signal_handler(cancelled: Cancelled) {
    let sig_cancelled = cancelled.clone();
    tokio::spawn(async move {
        loop {
            tokio::signal::ctrl_c().await.expect("failed to listen for SIGINT");
            if sig_cancelled.is_cancelled() {
                // Second Ctrl+C — force exit.
                eprintln!("\nForcing exit…");
                std::process::exit(130);
            }
            sig_cancelled.cancel();
            eprintln!("\nCancelling… (press Ctrl+C again to force exit)");
        }
    });
}
