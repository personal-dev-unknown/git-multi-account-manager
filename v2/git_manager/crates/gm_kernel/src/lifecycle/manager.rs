// crates/gm_kernel/src/lifecycle/manager.rs
//
// The LifecycleManager coordinates the ordered startup and shutdown of subsystems
// that need to prepare state before the first command arrives (startup hooks) or
// flush state before the process exits (shutdown hooks).
//
// ── Why ordered hooks matter ──────────────────────────────────────────────────
// Two subsystems that both need startup preparation may have an ordering
// requirement: the logging subsystem must be ready before the plugin loader
// runs, for example, because the loader emits tracing events during load.
// The priority field enforces this: lower values run first on startup and last
// on shutdown (LIFO — the last thing started is the first thing stopped,
// mirroring how reverse-dependency teardown works in any service mesh).
//
// ── Hook failure policy ───────────────────────────────────────────────────────
// If a startup hook fails, startup() returns the error immediately without
// calling remaining hooks. This is the correct behaviour: if a critical
// pre-condition fails (e.g. can't acquire a lock file), starting the remaining
// subsystems would put the process in a partially-initialised state that is
// harder to reason about than a clean failure.
//
// For shutdown hooks, errors are logged but never propagate — we attempt all
// cleanup regardless of individual failures, because at shutdown time there is
// no way to "undo" the stop of other subsystems. The process is ending; the
// goal is best-effort resource release.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use gm_shared::errors::GitManagerError;

/// A single lifecycle hook with its name and priority.
struct Hook {
    /// Human-readable name used in tracing output and error messages.
    name:     String,
    /// Lower values run first on startup; higher values run first on shutdown.
    priority: u16,
    handler:  Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), GitManagerError>> + Send>> + Send + Sync>,
}

/// Ordered startup and shutdown hook registry.
///
/// Register hooks early (during bootstrap or plugin on_load()) and call
/// `startup()` once all registrations are complete. Call `shutdown()` before
/// the process exits to give all subsystems a clean teardown opportunity.
#[derive(Default)]
pub struct LifecycleManager {
    startup_hooks:  RwLock<Vec<Hook>>,
    shutdown_hooks: RwLock<Vec<Hook>>,
}

impl std::fmt::Debug for LifecycleManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let startup_count  = self.startup_hooks.read()
            .map(|h| h.len()).unwrap_or(0);
        let shutdown_count = self.shutdown_hooks.read()
            .map(|h| h.len()).unwrap_or(0);
        write!(f, "LifecycleManager {{ startup_hooks: {startup_count}, shutdown_hooks: {shutdown_count} }}")
    }
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a startup hook.
    ///
    /// Hooks with lower priority values run first. The same priority value on
    /// two hooks means their relative order is insertion order (stable sort).
    ///
    /// The handler is an async closure that returns `Result<(), GitManagerError>`.
    /// Use the boxed future signature to work around Rust's trait object limitations
    /// with async closures.
    ///
    /// Example:
    /// ```ignore
    /// lifecycle.register_startup("log-flush", 0, || Box::pin(async {
    ///     tracing::info!("logging subsystem ready");
    ///     Ok(())
    /// }));
    /// ```
    pub fn register_startup<F, Fut>(&self, name: impl Into<String>, priority: u16, handler: F)
    where
        F:   Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), GitManagerError>> + Send + 'static,
    {
        let name    = name.into();
        let wrapped = Arc::new(move || -> Pin<Box<dyn Future<Output = Result<(), GitManagerError>> + Send>> {
            Box::pin(handler())
        });
        self.startup_hooks.write()
            .expect("lifecycle: startup_hooks RwLock poisoned")
            .push(Hook { name, priority, handler: wrapped });
    }

    /// Registers a shutdown hook (mirror of register_startup).
    pub fn register_shutdown<F, Fut>(&self, name: impl Into<String>, priority: u16, handler: F)
    where
        F:   Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), GitManagerError>> + Send + 'static,
    {
        let name    = name.into();
        let wrapped = Arc::new(move || -> Pin<Box<dyn Future<Output = Result<(), GitManagerError>> + Send>> {
            Box::pin(handler())
        });
        self.shutdown_hooks.write()
            .expect("lifecycle: shutdown_hooks RwLock poisoned")
            .push(Hook { name, priority, handler: wrapped });
    }

    /// Executes all startup hooks in ascending priority order.
    ///
    /// Returns immediately on the first hook failure, leaving subsequent hooks
    /// unrun. The caller (bootstrap.rs) treats any startup failure as fatal.
    pub async fn startup(&self) -> Result<(), GitManagerError> {
        let mut hooks: Vec<_> = self.startup_hooks.read()
            .expect("lifecycle: startup_hooks RwLock poisoned")
            .iter()
            .map(|h| (h.name.clone(), h.priority, Arc::clone(&h.handler)))
            .collect();

        // Stable sort: ties in priority keep their registration order.
        hooks.sort_by_key(|(_, p, _)| *p);

        for (name, priority, handler) in hooks {
            tracing::debug!(hook = %name, priority = priority, "running startup hook");
            handler().await.map_err(|e| {
                tracing::error!(hook = %name, error = %e, "startup hook failed");
                e
            })?;
            tracing::debug!(hook = %name, "startup hook completed");
        }

        Ok(())
    }

    /// Executes all shutdown hooks in descending priority order (LIFO semantics).
    ///
    /// Errors from individual hooks are logged but do not abort the remaining
    /// hooks — we attempt every teardown regardless of individual failures.
    pub async fn shutdown(&self) {
        let mut hooks: Vec<_> = self.shutdown_hooks.read()
            .expect("lifecycle: shutdown_hooks RwLock poisoned")
            .iter()
            .map(|h| (h.name.clone(), h.priority, Arc::clone(&h.handler)))
            .collect();

        // Descending priority: highest runs first (the last-started stops first).
        hooks.sort_by_key(|(_, p, _)| std::cmp::Reverse(*p));

        for (name, priority, handler) in hooks {
            tracing::debug!(hook = %name, priority = priority, "running shutdown hook");
            if let Err(e) = handler().await {
                tracing::error!(
                    hook  = %name,
                    error = %e,
                    "shutdown hook failed — continuing teardown"
                );
            }
        }
    }
}

// Bring Reverse into scope for the sort
