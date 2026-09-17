// crates/gm_kernel/src/event_bus/store.rs
//
// The EventStore port defines how kernel events are persisted for audit and
// replay. The concrete implementation (in gm_adapters) writes to the `events`
// table in MySQL or SQLite using SQLx. Keeping the store as a trait lets tests
// inject a no-op or in-memory implementation without hitting the database.

use async_trait::async_trait;
use gm_shared::errors::GitManagerError;
use crate::contracts::event::KernelEvent;

/// Persistence contract for the kernel event log.
///
/// Every event that flows through the EventBus is offered to the registered
/// EventStore before being dispatched to subscribers. If the store write fails,
/// the error is logged but dispatch continues — event loss is preferable to
/// blocking the entire command pipeline on a database outage.
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Appends one event to the persistent log.
    async fn append(&self, event: &KernelEvent) -> Result<(), GitManagerError>;
}

/// A no-op EventStore used during testing and for the period between kernel
/// construction and database connection setup during bootstrap.
#[derive(Debug, Default)]
pub struct NoOpEventStore;

#[async_trait]
impl EventStore for NoOpEventStore {
    async fn append(&self, _event: &KernelEvent) -> Result<(), GitManagerError> {
        Ok(())
    }
}