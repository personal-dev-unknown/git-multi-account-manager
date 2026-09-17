use async_trait::async_trait;
use sqlx::SqlitePool;
use serde_json::json;

use gm_kernel::event_bus::EventStore;
use gm_kernel::contracts::event::KernelEvent;
use gm_shared::errors::GitManagerError;

#[derive(Debug)]
pub struct SqliteEventStore {
    pool: SqlitePool,
}

impl SqliteEventStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn append(&self, event: &KernelEvent) -> Result<(), GitManagerError> {
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
