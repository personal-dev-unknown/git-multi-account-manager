// crates/gm_adapters/src/persistence/mysql/event_repository.rs
//
// SqlxEventStore implements the EventStore trait from gm_kernel. Its sole
// purpose is to write every KernelEvent to the events table so the application
// has a complete, immutable audit trail of everything that happened.
//
// ── Why this is append-only ───────────────────────────────────────────────────
// The events table is an immutable log. No UPDATE or DELETE is ever issued here.
// If an event is corrupted or needs to be superseded, a compensating event is
// emitted instead. This is the standard event-sourcing discipline: the log is
// the source of truth, and history is never rewritten.
//
// ── Field mapping ─────────────────────────────────────────────────────────────
// KernelEvent has a `sequence` field (monotonic process counter) and
// `correlation_id` / `causation_id` that the schema stores in the `metadata`
// JSON column rather than as dedicated columns. This keeps the schema stable
// even as the kernel event envelope gains new fields — adding fields to
// metadata JSON is backward compatible.

use async_trait::async_trait;
use sqlx::MySqlPool;
use serde_json::json;

use gm_kernel::event_bus::EventStore;
use gm_kernel::contracts::event::KernelEvent;
use gm_shared::errors::GitManagerError;

#[derive(Debug)]
pub struct SqlxEventStore {
    pool: MySqlPool,
}

impl SqlxEventStore {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for SqlxEventStore {
    async fn append(&self, event: &KernelEvent) -> Result<(), GitManagerError> {
        // The metadata column carries fields that don't have dedicated columns.
        let metadata = json!({
            "sequence":       event.sequence,
            "correlation_id": event.correlation_id,
            "causation_id":   event.causation_id,
        });

        sqlx::query(
            r#"
            INSERT INTO events
                (uuid, event_type, event_version, payload, metadata, published_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(event.event_id.to_string())
        .bind(&event.event_type)
        .bind(&event.schema_version)
        .bind(serde_json::to_string(&event.payload)
            .map_err(|e| GitManagerError::Other(format!("event payload serialisation: {e}")))?)
        .bind(metadata.to_string())
        .bind(event.timestamp_utc)
        .execute(&self.pool)
        .await
        .map_err(|e| GitManagerError::Database(format!("event append failed: {e}")))?;

        Ok(())
    }
}