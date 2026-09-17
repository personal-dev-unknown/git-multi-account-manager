use async_trait::async_trait;
use uuid::Uuid;
use gm_shared::errors::GitManagerError;

#[derive(Debug, Clone)]
pub struct GitOperationDto {
    pub uuid:            Uuid,
    pub op_type:         String,
    pub status:          String,
    pub account_alias:   Option<String>,
    pub repository_name: Option<String>,
    pub started_at:      chrono::DateTime<chrono::Utc>,
    pub duration_ms:     Option<u32>,
    pub error_message:   Option<String>,
}

#[async_trait]
pub trait GitOpRepository: Send + Sync {
    async fn list_recent(
        &self,
        account_uuid: Option<Uuid>,
        limit: u32,
    ) -> Result<Vec<GitOperationDto>, GitManagerError>;

    async fn record(
        &self,
        repository_id: Uuid,
        account_id:    Uuid,
        op_type:       &str,
        status:        &str,
        commit_sha:    Option<&str>,
        error_message: Option<&str>,
        duration_ms:   Option<u32>,
    ) -> Result<(), GitManagerError>;
}
