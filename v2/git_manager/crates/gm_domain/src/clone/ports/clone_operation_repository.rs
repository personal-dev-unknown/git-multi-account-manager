use async_trait::async_trait;
use uuid::Uuid;

use crate::clone::entities::CloneOperation;
use gm_shared::errors::GitManagerError;

/// Repository port for persisting clone operations.
///
/// Implementations store `CloneOperation` records (SQL, in-memory, etc.)
/// for tracking and observability.
#[async_trait]
pub trait CloneOperationRepository: Send + Sync {
    /// Persist a new clone operation.
    async fn insert(&self, operation: &CloneOperation) -> Result<(), GitManagerError>;

    /// Update an existing clone operation (e.g. mark completed/failed).
    async fn update(&self, operation: &CloneOperation) -> Result<(), GitManagerError>;

    /// Retrieve a clone operation by its UUID.
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<CloneOperation>, GitManagerError>;

    /// Retrieve all clone operations for a given account, most recent first.
    async fn find_by_account(&self, account_id: Uuid, limit: u32) -> Result<Vec<CloneOperation>, GitManagerError>;
}
