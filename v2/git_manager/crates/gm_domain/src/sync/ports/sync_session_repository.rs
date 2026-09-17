// crates/gm_domain/src/sync/ports/sync_session_repository.rs
//
// Persistence port for SyncSession entities. Sessions are written frequently
// (status update on every step transition) so the adapter implementation must
// use efficient upserts rather than full-row rewrites where possible.

use async_trait::async_trait;
use uuid::Uuid;

use crate::sync::entities::SyncSession;
use gm_shared::errors::GitManagerError;

#[async_trait]
pub trait SyncSessionRepository: Send + Sync {
    async fn save(&self, session: &SyncSession) -> Result<(), GitManagerError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SyncSession>, GitManagerError>;
    /// Returns all sessions for a repository, ordered by started_at descending.
    /// The `limit` parameter caps the result to avoid loading unbounded history.
    async fn list_by_repository(
        &self,
        repository_uuid: Uuid,
        limit:           u32,
    ) -> Result<Vec<SyncSession>, GitManagerError>;
    /// Returns non-terminal sessions for a repository, used to detect whether
    /// an operation is already running before starting a new one.
    async fn find_active_for_repository(
        &self,
        repository_uuid: Uuid,
    ) -> Result<Option<SyncSession>, GitManagerError>;
}