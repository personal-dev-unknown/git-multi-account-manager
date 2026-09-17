// crates/gm_kernel/src/event_bus/handlers/audit_handler.rs
//
// The AuditEventHandler is the kernel's built-in global event subscriber.
// It is registered via `event_bus.subscribe_all()` during bootstrap, which means
// it receives EVERY event that flows through the system regardless of type.
// Its sole responsibility is appending each event to the persistent event store
// so there is an immutable audit trail of everything that happened.
//
// ── Why this is a kernel concern, not a plugin ────────────────────────────────
// Audit logging must happen for ALL events, including events from plugins that
// have not loaded yet and events from the kernel itself (plugin load/unload).
// Placing this in a plugin would mean a window at boot where events escape the
// audit trail. By registering it before plugins load, the kernel guarantees
// 100% event coverage.
//
// ── Error handling ────────────────────────────────────────────────────────────
// If the event store write fails (database unavailable), the error is logged at
// ERROR level with the full event details. This allows operators to reconstruct
// the missed window from application logs. Dispatch continues regardless —
// a database hiccup must not block git operations from completing.

use std::sync::Arc;
use async_trait::async_trait;
use gm_shared::errors::GitManagerError;
use crate::{
    contracts::event::KernelEvent,
    event_bus::{dispatcher::EventHandler, store::EventStore},
};

/// The kernel-level audit trail handler.
/// Registered as a global subscriber before any plugin loads.
pub struct AuditEventHandler {
    store: Arc<dyn EventStore>,
}

impl std::fmt::Debug for AuditEventHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuditEventHandler").field("store", &"<EventStore>").finish()
    }
}

impl AuditEventHandler {
    pub fn new(store: Arc<dyn EventStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl EventHandler for AuditEventHandler {
    async fn handle(&self, event: &KernelEvent) -> Result<(), GitManagerError> {
        if let Err(e) = self.store.append(event).await {
            // Log the failure but do not propagate — audit failures must not
            // interrupt the normal command→event→subscriber flow.
            tracing::error!(
                event_id   = %event.event_id,
                event_type = %event.event_type,
                sequence   = event.sequence,
                error      = %e,
                "audit event store write failed — event may be missing from audit log"
            );
        }
        Ok(())
    }
}