// crates/gm_kernel/src/event_bus/dispatcher.rs
//
// The EventDispatcher is the internal engine of the event bus. It owns the
// subscriber lists and calls each handler when an event is published.
//
// ── Handler model ─────────────────────────────────────────────────────────────
// Handlers are stored as `Arc<dyn EventHandler>` — a single trait object that
// receives a &KernelEvent. This design was chosen over the alternative of
// storing generic `EventHandler<E>` implementations because:
//
//   1. The dispatcher must store handlers for different event types in the same
//      data structure. Generics cannot do this without type erasure.
//   2. Handlers need the KernelEvent envelope (not just the payload) so they
//      can access correlation_id, sequence number, and schema_version.
//   3. If a handler needs the typed payload, it calls `event.as_typed::<E>()`.
//
// ── Concurrency model ─────────────────────────────────────────────────────────
// The DashMap holds `Vec<Arc<dyn EventHandler>>` per event type. Reads are
// concurrent and lock-free within a shard. Writes (subscribe) are rare (only
// during boot) so the per-shard lock contention is negligible.
//
// Handlers are called sequentially in priority order (high → low) within a
// single tokio task. This provides ordering guarantees within a single publish
// call. Independent publish calls from different tasks execute concurrently;
// the sequence number on KernelEvent provides a total order for audit replay.

use std::sync::Arc;
use async_trait::async_trait;
use dashmap::DashMap;
use gm_shared::errors::GitManagerError;
use crate::contracts::event::KernelEvent;

/// The trait that all event handlers must implement.
///
/// A handler receives the full KernelEvent envelope, which includes the
/// serialised payload, correlation ID, sequence number, and schema version.
/// Handlers that need the typed payload call `event.as_typed::<MyEventType>()`.
///
/// Handlers MUST NOT panic. Any error they encounter should be logged and
/// swallowed — the event bus does not propagate handler errors to the publisher.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Called for each event this handler is subscribed to.
    async fn handle(&self, event: &KernelEvent) -> Result<(), GitManagerError>;
}

/// A handler paired with its registered priority for stable ordering.
struct HandlerEntry {
    handler:  Arc<dyn EventHandler>,
    priority: u16,
}

/// Stores event handlers and dispatches events to them.
#[derive(Default)]
pub struct EventDispatcher {
    /// event_type → handlers, sorted by priority descending.
    /// DashMap gives concurrent reads without a global lock.
    handlers:        DashMap<String, Vec<HandlerEntry>>,
    /// Handlers that receive ALL events regardless of type.
    /// The audit handler is always registered here.
    global_handlers: std::sync::RwLock<Vec<Arc<dyn EventHandler>>>,
}

impl std::fmt::Debug for EventDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let global_count = self.global_handlers.read().map(|g| g.len()).unwrap_or(0);
        f.debug_struct("EventDispatcher")
            .field("handlers", &format_args!("[{} event type(s)]", self.handlers.len()))
            .field("global_handlers", &format_args!("[{} global handler(s)]", global_count))
            .finish()
    }
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a handler for a specific event type.
    /// Handlers with higher priority values are called before lower-priority handlers.
    pub fn subscribe(&self, event_type: String, handler: Arc<dyn EventHandler>, priority: u16) {
        let mut entry = self.handlers.entry(event_type).or_default();
        entry.push(HandlerEntry { handler, priority });
        // Sort descending so index 0 is highest priority.
        // This is O(n log n) per subscription but subscriptions only happen at boot.
        entry.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| {
            Arc::as_ptr(&a.handler).cast::<()>().cmp(&Arc::as_ptr(&b.handler).cast::<()>())
        }));
    }

    /// Registers a handler that receives every event, regardless of type.
    pub fn subscribe_all(&self, handler: Arc<dyn EventHandler>) {
        self.global_handlers.write()
            .expect("dispatcher: global_handlers RwLock poisoned")
            .push(handler);
    }

    /// Dispatches a `KernelEvent` to all matching handlers.
    ///
    /// Handler errors are logged with `tracing::error!` but do NOT abort
    /// dispatch to remaining handlers. A single misbehaving handler cannot
    /// prevent other subscribers from receiving the event.
    pub async fn dispatch(&self, event: &KernelEvent) {
        // Global handlers (e.g. audit) run first.
        let globals = self.global_handlers.read()
            .expect("dispatcher: global_handlers RwLock poisoned")
            .clone();
        for handler in &globals {
            if let Err(e) = handler.handle(event).await {
                tracing::error!(
                    event_type = %event.event_type,
                    event_id   = %event.event_id,
                    error      = %e,
                    "global event handler returned an error"
                );
            }
        }

        // Type-specific handlers run after global handlers.
        if let Some(typed_handlers) = self.handlers.get(&event.event_type) {
            for entry in typed_handlers.iter() {
                let handler = &entry.handler;
                if let Err(e) = handler.handle(event).await {
                    tracing::error!(
                        event_type = %event.event_type,
                        event_id   = %event.event_id,
                        error      = %e,
                        "typed event handler returned an error"
                    );
                }
            }
        }
    }
}