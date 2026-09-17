// crates/gm_kernel/src/contracts/event.rs
//
// Two types live here: DomainEvent (the trait domain services use when producing
// events) and KernelEvent (the envelope the event bus wraps every event in before
// dispatch and persistence).
//
// ── Why the envelope exists ───────────────────────────────────────────────────
// The architecture document requires every kernel event to carry:
//   event_id, event_type, timestamp_utc, schema_version, correlation_id,
//   causation_id
//
// Adding these fields to every domain struct would pollute the domain with
// infrastructure concerns. Instead, the DomainEvent trait requires the event_type
// string; the kernel wraps the serialised payload in KernelEvent which carries
// the infrastructure fields. Handlers receive a &KernelEvent and access the
// typed payload by deserialising KernelEvent::payload.
//
// ── Ordering guarantee ────────────────────────────────────────────────────────
// The KernelEvent::sequence field is populated by the EventBus using an
// AtomicU64 counter that increments by one for every published event. This
// guarantees that events published within a single process have a stable total
// order even when multiple async tasks are publishing concurrently, because
// fetch_add is an atomic operation with Ordering::SeqCst.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The trait domain events must implement.
///
/// Implementing this trait is lightweight — just one method that returns the
/// string constant from `gm_shared::constants::events`. The serde bounds are
/// required so the event bus can serialise the payload into the KernelEvent
/// envelope.
pub trait DomainEvent: serde::Serialize + Send + Sync + 'static {
    /// Returns the event type string, e.g. "AccountAdded".
    /// Always use the constant from `gm_shared::constants::events` to avoid typos.
    fn event_type(&self) -> &'static str;

    /// Optional correlation ID that links this event to a top-level user action.
    /// If None, the event bus assigns one automatically.
    fn correlation_id(&self) -> Option<Uuid> {
        None
    }
}

/// The full kernel event envelope — what handlers and the audit log receive.
///
/// The `payload` field holds the original domain event serialised as a
/// `serde_json::Value`. Handlers that need to read the typed payload
/// deserialise it with `serde_json::from_value::<MyEvent>(event.payload.clone())`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelEvent {
    /// Unique identifier for this event occurrence. Never shared between events.
    pub event_id:       Uuid,

    /// The event type string, e.g. "RepositoryCloned".
    pub event_type:     String,

    /// When the event was published. Populated by the event bus, not the domain.
    pub timestamp_utc:  DateTime<Utc>,

    /// Monotonically increasing sequence number within this process lifetime.
    /// Enables replay and deterministic ordering for audit log queries.
    pub sequence:       u64,

    /// The serialised domain event payload.
    pub payload:        serde_json::Value,

    /// Schema version string from gm_shared::constants::app::EVENT_SCHEMA_VERSION.
    pub schema_version: String,

    /// Links all events that were triggered by the same user action (e.g. "add account"
    /// produces AccountAdded + SshKeyGenerated under the same correlation_id).
    pub correlation_id: Option<Uuid>,

    /// The event_id of the event that caused this event (empty for root events).
    pub causation_id:   Option<Uuid>,
}

impl KernelEvent {
    /// Attempts to deserialise the payload into the requested type.
    /// Returns Err if the payload shape does not match T.
    pub fn as_typed<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }
}