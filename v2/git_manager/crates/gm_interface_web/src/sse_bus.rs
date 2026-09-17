// crates/gm_interface_web/src/sse_bus.rs
//
// SseBus bridges the kernel's internal EventBus (handler-push model) to the
// SSE endpoint (HTTP long-poll / stream-pull model).
//
// ── Problem being solved ──────────────────────────────────────────────────────
// The kernel EventBus works by calling registered `EventHandler` trait objects
// synchronously in priority order for each published event. HTTP SSE connections
// are long-lived futures that need to _poll_ for new data. These two models
// cannot be directly composed — we need a shared buffer that the handler writes
// into and the SSE stream reads from without blocking.
//
// ── Solution: tokio broadcast channel ────────────────────────────────────────
// A `tokio::sync::broadcast` channel provides a multi-producer, multi-consumer
// ring buffer. Only one writer exists (`SseBroadcastHandler`), but many readers
// can exist simultaneously — one per connected browser tab. Each reader holds
// an independent cursor into the ring buffer, so a slow tab cannot block faster
// tabs or the kernel event bus.
//
// ── Memory model ─────────────────────────────────────────────────────────────
// Events are shared as `Arc<KernelEvent>`. The broadcast channel stores one
// `Arc` per ring-buffer slot; each subscriber receives a clone of that `Arc`
// (8 bytes + atomic reference increment). The underlying `KernelEvent` is
// allocated exactly once per published event regardless of subscriber count.
// With BROADCAST_CAPACITY = 256 and typical event sizes of ~1 KB, the ring
// buffer occupies ≈256 KB of heap, which is negligible.
//
// ── Lag behaviour ────────────────────────────────────────────────────────────
// When a receiver falls too far behind (its lag exceeds BROADCAST_CAPACITY),
// `tokio::sync::broadcast` marks it as lagged. `BroadcastStream` surfaces this
// as `BroadcastStreamRecvError::Lagged(n)`. The SSE handler converts this into
// a synthetic `resync` event so the client knows to re-fetch state via REST API.
// The connection is NOT closed on lag; streaming continues from the current tail.
//
// ── Connection lifecycle ──────────────────────────────────────────────────────
// Each `sse_handler` invocation calls `SseBus::subscribe()` to obtain a fresh
// `broadcast::Receiver`. When the browser closes the connection, Axum drops the
// handler future, which drops the `BroadcastStream`, which drops the `Receiver`,
// which automatically decrements the sender's receiver count. No explicit
// cleanup is required.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::broadcast;

use gm_kernel::event_bus::EventHandler;
use gm_kernel::contracts::event::KernelEvent;
use gm_shared::errors::GitManagerError;

// ── Capacity constant ─────────────────────────────────────────────────────────

/// Ring-buffer capacity for the SSE broadcast channel.
///
/// 256 slots × ~1 KB average event size ≈ 256 KB peak working set.
/// A client must receive at least one event per 256 kernel events to avoid
/// being marked as lagged. Under typical usage (accounts/SSH/repo events) this
/// provides ample buffer even for clients with high round-trip latency.
///
/// This value is a compile-time constant so it can be referenced in diagnostics.
pub const BROADCAST_CAPACITY: usize = 256;

// ── SseBus ────────────────────────────────────────────────────────────────────

/// Cheaply-cloneable handle to the SSE broadcast channel sender.
///
/// `SseBus` is stored in `AppState` and cloned into every Axum handler. The
/// clone is a reference-count increment on the inner `Arc` — no heap allocation
/// and no channel duplication.
///
/// # Creation
/// `SseBus::new()` is called once in `app::run_server()` before the Axum router
/// is built. The same bus instance is handed to `SseBroadcastHandler` (which
/// writes to it) and to `AppState` (which distributes it to SSE handlers that
/// read from it).
///
/// # Thread safety
/// `broadcast::Sender<Arc<KernelEvent>>` is `Send + Sync` because
/// `Arc<KernelEvent>` is `Send + Sync`. The outer `Arc` makes `SseBus` itself
/// `Clone + Send + Sync + 'static`.
#[derive(Clone, Debug)]
pub struct SseBus(Arc<broadcast::Sender<Arc<KernelEvent>>>);

impl SseBus {
    /// Creates a new `SseBus` backed by a fresh broadcast channel.
    ///
    /// The initial receiver returned by `broadcast::channel` is immediately
    /// dropped; the first real receiver is created when the first SSE client
    /// connects via `subscribe()`.
    pub fn new() -> Self {
        let (tx, _initial_rx) = broadcast::channel(BROADCAST_CAPACITY);
        Self(Arc::new(tx))
    }

    /// Creates a `Receiver` positioned at the current tail of the ring buffer.
    ///
    /// Each call returns an **independent** receiver. Events published *before*
    /// this call are not visible to the new receiver; only events published
    /// *after* this call will be delivered. This matches SSE semantics: the
    /// client starts receiving events from the moment it connects.
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<KernelEvent>> {
        self.0.subscribe()
    }

    /// Returns the number of active receivers (connected SSE clients).
    ///
    /// Includes any receivers that have lagged but not yet been garbage-
    /// collected. Used for connection-count diagnostics and metrics.
    pub fn receiver_count(&self) -> usize {
        self.0.receiver_count()
    }

    /// Returns a clone of the inner `Arc<Sender>` for injection into handlers.
    ///
    /// Called by `SseBroadcastHandler::new()`. The clone shares the same
    /// underlying channel — writes via the returned `Sender` are visible to all
    /// receivers created from this `SseBus`.
    pub(crate) fn sender(&self) -> Arc<broadcast::Sender<Arc<KernelEvent>>> {
        Arc::clone(&self.0)
    }
}

impl Default for SseBus {
    fn default() -> Self {
        Self::new()
    }
}

// ── SseBroadcastHandler ───────────────────────────────────────────────────────

/// Kernel `EventHandler` that forwards every `KernelEvent` to the SSE bus.
///
/// This handler is registered with `kernel.event_bus.subscribe_all()` in
/// `app::run_server()` before the Axum server starts. After registration, every
/// event published anywhere in the kernel — account operations, SSH key events,
/// repository clones, sync sessions, workflow lifecycle — is automatically
/// forwarded to all connected SSE clients.
///
/// # Failure semantics
/// If no SSE clients are currently connected, `broadcast::Sender::send()` returns
/// `Err(SendError)`. This is **expected and non-fatal** — the event is simply not
/// forwarded. The kernel event bus is never blocked or errored by the absence of
/// SSE subscribers.
///
/// # Arc<KernelEvent> allocation strategy
/// The event is cloned into an `Arc` before sending. The `Arc` itself is the
/// value stored in the broadcast ring buffer; each subscriber receives a cheap
/// reference-count clone. Only one `KernelEvent` heap allocation occurs per
/// published event regardless of subscriber count.
///
/// # Ordering guarantee
/// `broadcast::Sender::send()` is not async and completes without yielding.
/// Combined with the kernel event bus's sequential per-type dispatch, the order
/// of events in the broadcast channel matches the order they were dispatched
/// to this handler.
#[derive(Debug)]
pub struct SseBroadcastHandler {
    tx: Arc<broadcast::Sender<Arc<KernelEvent>>>,
}

impl SseBroadcastHandler {
    /// Constructs a new handler wired to the given `SseBus`.
    pub fn new(bus: &SseBus) -> Self {
        Self { tx: bus.sender() }
    }
}

#[async_trait]
impl EventHandler for SseBroadcastHandler {
    /// Forwards the `KernelEvent` to all active SSE receivers.
    ///
    /// This method is called synchronously (no `.await` internally) by the
    /// event dispatcher. It completes in O(1) time: one `Arc` clone and one
    /// lock-free ring-buffer write.
    async fn handle(&self, event: &KernelEvent) -> Result<(), GitManagerError> {
        // Wrap in Arc so the heap allocation is shared across all receivers.
        let arc_event = Arc::new(event.clone());

        match self.tx.send(arc_event) {
            Ok(receiver_count) => {
                tracing::trace!(
                    event_id   = %event.event_id,
                    event_type = %event.event_type,
                    sequence   = event.sequence,
                    receivers  = receiver_count,
                    "sse_bus: event broadcast to active SSE clients"
                );
            }
            Err(_) => {
                // `SendError` means zero active receivers. This is normal when
                // no browser tabs are connected. Trace-level to avoid log noise.
                tracing::trace!(
                    event_type = %event.event_type,
                    sequence   = event.sequence,
                    "sse_bus: no active SSE receivers — event discarded"
                );
            }
        }

        // Always return Ok. A missing SSE client is not a kernel-level error.
        Ok(())
    }
}
