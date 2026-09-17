// crates/gm_kernel/src/event_bus/event_bus.rs
//
// The EventBus is the primary publish/subscribe mechanism in the kernel. It wraps
// the EventDispatcher and adds the KernelEvent envelope construction logic —
// filling in the event_id (a new UUID), timestamp_utc (Utc::now()), sequence
// (AtomicU64), and schema_version.
//
// ── Atomicity of the sequence counter ────────────────────────────────────────
// The sequence counter uses `std::sync::atomic::AtomicU64` with
// `Ordering::SeqCst` (sequentially consistent). This guarantees that if
// two async tasks publish events concurrently, they each get a unique,
// monotonically increasing sequence number with no gaps from the perspective
// of a serial observer. SeqCst is chosen over AcqRel because the sequence
// number is used for replay ordering — we need all CPUs to agree on the
// total order of all stores, not just a per-thread release/acquire pairing.
//
// ── publish is async but sequence assignment is synchronous ──────────────────
// The atomic increment happens before the async dispatch begins. This means the
// sequence number reflects the order in which publish() was *called*, not the
// order in which handlers *completed*. For audit replay purposes, call order
// is the correct semantic — it reflects the user's intent order.

use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use gm_shared::constants::app::EVENT_SCHEMA_VERSION;
use crate::contracts::event::{DomainEvent, KernelEvent};
use crate::event_bus::dispatcher::{EventDispatcher, EventHandler};
use uuid::Uuid;
use chrono::Utc;

/// The kernel's event publication and subscription hub.
#[derive(Debug)]
pub struct EventBus {
    dispatcher: EventDispatcher,
    sequence:   AtomicU64,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            dispatcher: EventDispatcher::new(),
            sequence:   AtomicU64::new(0),
        }
    }

    /// Publishes a domain event.
    ///
    /// Steps:
    ///   1. Serialise the typed event into a `serde_json::Value` payload.
    ///   2. Atomically fetch-and-increment the sequence counter.
    ///   3. Wrap the payload in a `KernelEvent` envelope.
    ///   4. Dispatch the envelope to all subscribers.
    ///
    /// The serialisation step is infallible for well-formed domain events —
    /// if it fails (which would indicate a programming error, not a runtime
    /// condition) the event is dropped and an error is logged.
    pub async fn publish<E: DomainEvent>(&self, event: E, correlation_id: Option<Uuid>) {
        let payload = match serde_json::to_value(&event) {
            Ok(v)  => v,
            Err(e) => {
                tracing::error!(
                    event_type = event.event_type(),
                    error      = %e,
                    "failed to serialise domain event — event dropped"
                );
                return;
            }
        };

        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);

        let kernel_event = KernelEvent {
            event_id:       Uuid::new_v4(),
            event_type:     event.event_type().to_string(),
            timestamp_utc:  Utc::now(),
            sequence:       seq,
            payload,
            schema_version: EVENT_SCHEMA_VERSION.to_string(),
            correlation_id: correlation_id.or_else(|| event.correlation_id()),
            causation_id:   None,
        };

        tracing::debug!(
            event_id   = %kernel_event.event_id,
            event_type = %kernel_event.event_type,
            sequence   = seq,
            "dispatching kernel event"
        );

        self.dispatcher.dispatch(&kernel_event).await;
    }

    /// Registers a handler for a specific event type.
    pub fn subscribe(&self, event_type: String, handler: Arc<dyn EventHandler>, priority: u16) {
        self.dispatcher.subscribe(event_type, handler, priority);
    }

    /// Registers a handler that receives ALL events (e.g. the audit handler).
    pub fn subscribe_all(&self, handler: Arc<dyn EventHandler>) {
        self.dispatcher.subscribe_all(handler);
    }

    /// Returns the current sequence counter value.
    /// Used in diagnostics and test assertions.
    pub fn current_sequence(&self) -> u64 {
        self.sequence.load(Ordering::SeqCst)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}