use async_trait::async_trait;
use sqlx::{MySql, Pool};
use uuid::Uuid;

use gm_domain::clone::entities::{CloneOperation, CloneOperationStatus};
use gm_domain::clone::ports::CloneOperationRepository;
use gm_shared::errors::GitManagerError;

/// MySQL-backed `CloneOperationRepository`.
pub struct MySqlCloneOperationRepository {
    pool: Pool<MySql>,
}

impl MySqlCloneOperationRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CloneOperationRepository for MySqlCloneOperationRepository {
    async fn insert(&self, op: &CloneOperation) -> Result<(), GitManagerError> {
        sqlx::query(
            r#"INSERT INTO clone_operations
               (uuid, account_id, repository_id, url, destination, strategy, protocol, status, attempts, duration_ms, error_message, started_at, completed_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(op.uuid.to_string())
        .bind(op.account_id.to_string())
        .bind(op.repository_id.as_ref().map(Uuid::to_string))
        .bind(&op.url)
        .bind(&op.destination)
        .bind(&op.strategy)
        .bind(&op.protocol)
        .bind(status_to_str(&op.status))
        .bind(op.attempts)
        .bind(op.duration_ms.map(|d| d as i64))
        .bind(&op.error_message)
        .bind(op.started_at)
        .bind(op.completed_at)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    async fn update(&self, op: &CloneOperation) -> Result<(), GitManagerError> {
        sqlx::query(
            r#"UPDATE clone_operations
               SET status = ?, attempts = ?, duration_ms = ?, error_message = ?, completed_at = ?
               WHERE uuid = ?"#,
        )
        .bind(status_to_str(&op.status))
        .bind(op.attempts)
        .bind(op.duration_ms.map(|d| d as i64))
        .bind(&op.error_message)
        .bind(op.completed_at)
        .bind(op.uuid.to_string())
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<CloneOperation>, GitManagerError> {
        let row = sqlx::query_as::<_, CloneOperationRow>(
            r#"SELECT * FROM clone_operations WHERE uuid = ?"#,
        )
        .bind(uuid.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        row.map(TryInto::try_into).transpose()
    }

    async fn find_by_account(
        &self,
        account_id: Uuid,
        limit: u32,
    ) -> Result<Vec<CloneOperation>, GitManagerError> {
        let rows = sqlx::query_as::<_, CloneOperationRow>(
            r#"SELECT * FROM clone_operations WHERE account_id = ? ORDER BY started_at DESC LIMIT ?"#,
        )
        .bind(account_id.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}

fn db_err(e: impl std::fmt::Display) -> GitManagerError {
    GitManagerError::Database(e.to_string())
}

// ---- Row struct ----

#[derive(sqlx::FromRow)]
struct CloneOperationRow {
    uuid:           String,
    account_id:     String,
    repository_id:  Option<String>,
    url:            String,
    destination:    String,
    strategy:       String,
    protocol:       String,
    status:         String,
    attempts:       u32,
    duration_ms:    Option<i64>,
    error_message:  Option<String>,
    started_at:     chrono::DateTime<chrono::Utc>,
    completed_at:   Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<CloneOperationRow> for CloneOperation {
    type Error = GitManagerError;

    fn try_from(row: CloneOperationRow) -> Result<Self, Self::Error> {
        let uuid = Uuid::parse_str(&row.uuid)
            .map_err(|e| GitManagerError::Database(format!("invalid uuid: {e}")))?;
        let account_id = Uuid::parse_str(&row.account_id)
            .map_err(|e| GitManagerError::Database(format!("invalid account_id: {e}")))?;
        let repository_id = row
            .repository_id
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|e| GitManagerError::Database(format!("invalid repository_id: {e}")))?;

        Ok(Self {
            uuid,
            account_id,
            repository_id,
            url: row.url,
            destination: row.destination,
            strategy: row.strategy,
            protocol: row.protocol,
            status: str_to_status(&row.status)?,
            attempts: row.attempts,
            duration_ms: row.duration_ms.map(|d| d as u64),
            error_message: row.error_message,
            started_at: row.started_at,
            completed_at: row.completed_at,
        })
    }
}

// ---- Helpers ----

fn status_to_str(s: &CloneOperationStatus) -> &'static str {
    match s {
        CloneOperationStatus::Started => "started",
        CloneOperationStatus::Completed => "completed",
        CloneOperationStatus::Failed => "failed",
    }
}

fn str_to_status(s: &str) -> Result<CloneOperationStatus, GitManagerError> {
    match s {
        "started" => Ok(CloneOperationStatus::Started),
        "completed" => Ok(CloneOperationStatus::Completed),
        "failed" => Ok(CloneOperationStatus::Failed),
        other => Err(GitManagerError::Other(format!(
            "invalid clone operation status: {other}"
        ))),
    }
}
