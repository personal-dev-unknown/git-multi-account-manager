use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use gm_ports::outbound::git_op_repository::{GitOpRepository, GitOperationDto};

#[derive(sqlx::FromRow)]
struct GitOperationRow {
    uuid:            String,
    operation_type:  String,
    status:          String,
    account_alias:   Option<String>,
    repository_name: Option<String>,
    started_at:      chrono::NaiveDateTime,
    duration_ms:     Option<u32>,
    error_message:   Option<String>,
}

#[derive(Debug)]
pub struct SqliteGitOperationRepository {
    pool: SqlitePool,
}

impl SqliteGitOperationRepository {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }
}

#[async_trait]
impl GitOpRepository for SqliteGitOperationRepository {
    async fn list_recent(
        &self,
        account_uuid: Option<Uuid>,
        limit: u32,
    ) -> Result<Vec<GitOperationDto>, gm_shared::errors::GitManagerError> {
        let rows = if let Some(acc_uuid) = account_uuid {
            sqlx::query_as::<_, GitOperationRow>(r#"
                SELECT g.uuid, g.operation_type, g.status,
                       a.alias AS account_alias,
                       r.name  AS repository_name,
                       g.started_at, g.duration_ms, g.error_message
                FROM git_operations g
                LEFT JOIN accounts a ON a.uuid = ?
                LEFT JOIN repositories r ON r.uuid = g.repository_id
                ORDER BY g.started_at DESC
                LIMIT ?
            "#)
            .bind(acc_uuid.to_string())
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| gm_shared::errors::GitManagerError::Database(e.to_string()))?
        } else {
            sqlx::query_as::<_, GitOperationRow>(r#"
                SELECT g.uuid, g.operation_type, g.status,
                       a.alias AS account_alias,
                       r.name  AS repository_name,
                       g.started_at, g.duration_ms, g.error_message
                FROM git_operations g
                LEFT JOIN accounts a ON a.uuid = g.account_id
                LEFT JOIN repositories r ON r.uuid = g.repository_id
                ORDER BY g.started_at DESC
                LIMIT ?
            "#)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| gm_shared::errors::GitManagerError::Database(e.to_string()))?
        };

        rows.into_iter().map(|row| {
            let uuid = Uuid::parse_str(&row.uuid)
                .map_err(|e| gm_shared::errors::GitManagerError::Database(
                    format!("invalid uuid '{}': {}", row.uuid, e)))?;
            Ok(GitOperationDto {
                uuid,
                op_type:        row.operation_type,
                status:         row.status,
                account_alias:  row.account_alias,
                repository_name: row.repository_name,
                started_at:     DateTime::from_naive_utc_and_offset(row.started_at, Utc),
                duration_ms:    row.duration_ms,
                error_message:  row.error_message,
            })
        }).collect()
    }

    async fn record(
        &self,
        repository_id: Uuid,
        account_id:    Uuid,
        op_type:       &str,
        status:        &str,
        commit_sha:    Option<&str>,
        error_message: Option<&str>,
        duration_ms:   Option<u32>,
    ) -> Result<(), gm_shared::errors::GitManagerError> {
        sqlx::query(r#"
            INSERT INTO git_operations
                (uuid, repository_id, account_id, operation_type, status,
                 commit_sha, error_message, duration_ms, started_at, completed_at)
            VALUES (?,
                    (SELECT id FROM repositories WHERE uuid = ?),
                    (SELECT id FROM accounts     WHERE uuid = ?),
                    ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#)
        .bind(Uuid::new_v4().to_string())
        .bind(repository_id.to_string())
        .bind(account_id.to_string())
        .bind(op_type)
        .bind(status)
        .bind(commit_sha)
        .bind(error_message)
        .bind(duration_ms)
        .execute(&self.pool)
        .await
        .map_err(|e| gm_shared::errors::GitManagerError::Database(e.to_string()))?;
        Ok(())
    }
}
