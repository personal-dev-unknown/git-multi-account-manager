// crates/gm_adapters/src/persistence/mysql/repository_repository.rs
//
// MySQL adapter for the `RepositoryRepository` port. All queries use
// `sqlx::query` / `sqlx::query_as` (non-macro forms) to avoid requiring
// a live DATABASE_URL at compile time, while remaining fully parameterised
// and injection-safe at runtime.
//
// UUID-to-BIGINT mapping follows the schema convention: the `repositories`
// table stores its own CHAR(36) uuid, but FK columns (`account_id`,
// `platform_id`) reference the BIGINT PKs of `accounts` and `platforms`.
// Reads JOIN both tables to reconstruct the domain UUIDs.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use gm_domain::repositories::{
    entities::{repository::Visibility, Repository},
    ports::RepositoryRepository,
    value_objects::RepositoryUrl,
};
use gm_shared::errors::GitManagerError;

// ── Error helper ──────────────────────────────────────────────────────────────

fn storage_err(e: impl ToString) -> GitManagerError {
    GitManagerError::Database(e.to_string())
}

// ── Repository struct ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct MySqlRepositoryRepository {
    pool: MySqlPool,
}

impl MySqlRepositoryRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

// ── Row type ──────────────────────────────────────────────────────────────────
// Mirrors the columns selected by SELECT_REPOS. All UUID FK columns are
// resolved to CHAR(36) strings via JOINs so the domain never sees raw BIGINTs.

#[derive(sqlx::FromRow)]
struct RepositoryRow {
    uuid:            String,
    account_uuid:    String,
    platform_uuid:   String,
    name:            String,
    full_name:       String,
    description:     Option<String>,
    local_path:      Option<String>,
    remote_url:      String,
    clone_url_ssh:   Option<String>,
    clone_url_https: Option<String>,
    default_branch:  String,
    current_branch:  Option<String>,
    visibility:      String,
    is_cloned:       bool,
    is_archived:     bool,
    is_forked:       bool,
    last_commit_sha: Option<String>,
    last_synced_at:  Option<DateTime<Utc>>,
    created_at:      DateTime<Utc>,
}

// ── Row → entity ──────────────────────────────────────────────────────────────

fn row_to_repository(row: RepositoryRow) -> Result<Repository, GitManagerError> {
    let uuid        = Uuid::parse_str(&row.uuid).map_err(storage_err)?;
    let account_id  = Uuid::parse_str(&row.account_uuid).map_err(storage_err)?;
    let platform_id = Uuid::parse_str(&row.platform_uuid).map_err(storage_err)?;

    // Prefer the explicit SSH URL; fall back to the generic remote_url.
    let ssh_url  = row.clone_url_ssh.unwrap_or_else(|| row.remote_url.clone());
    let repo_url = RepositoryUrl::new(ssh_url).map_err(|e| storage_err(e.to_string()))?;

    Ok(Repository::rehydrate(
        uuid,
        account_id,
        platform_id,
        row.name,
        row.full_name,
        row.description,
        row.local_path,
        repo_url,
        // clone_url_ssh already consumed above; store HTTPS clone URL directly.
        None,
        row.clone_url_https,
        row.default_branch,
        row.current_branch,
        Visibility::from_str(&row.visibility),
        row.is_cloned,
        row.is_archived,
        row.is_forked,
        row.last_commit_sha,
        row.last_synced_at,
        row.created_at,
    ))
}

// ── Base SELECT fragment ──────────────────────────────────────────────────────
// All query methods append their WHERE clause to this fragment using
// `format!("{SELECT_REPOS} AND ...")` to keep the projection consistent.

const SELECT_REPOS: &str = r#"
    SELECT
        r.uuid,
        a.uuid  AS account_uuid,
        p.uuid  AS platform_uuid,
        r.name,
        r.full_name,
        r.description,
        r.local_path,
        r.remote_url,
        r.clone_url_ssh,
        r.clone_url_https,
        r.default_branch,
        r.current_branch,
        r.visibility,
        r.is_cloned,
        r.is_archived,
        r.is_forked,
        r.last_commit_sha,
        r.last_synced_at,
        r.created_at
    FROM repositories r
    JOIN accounts  a ON r.account_id  = a.id
    JOIN platforms p ON r.platform_id = p.id
    WHERE 1=1
"#;

// ── Visibility helper ─────────────────────────────────────────────────────────

fn visibility_str(v: &Visibility) -> &'static str {
    match v {
        Visibility::Public   => "public",
        Visibility::Private  => "private",
        Visibility::Internal => "internal",
    }
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[async_trait]
impl RepositoryRepository for MySqlRepositoryRepository {
    async fn save(&self, repo: &Repository) -> Result<(), GitManagerError> {
        sqlx::query(r#"
            INSERT INTO repositories
                (uuid, account_id, platform_id, name, full_name, description,
                 remote_url, clone_url_ssh, clone_url_https, local_path,
                 default_branch, current_branch, visibility,
                 is_cloned, is_archived, is_forked,
                 last_commit_sha, last_synced_at)
            VALUES (?,
                    (SELECT id FROM accounts  WHERE uuid = ?),
                    (SELECT id FROM platforms WHERE uuid = ?),
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                local_path      = VALUES(local_path),
                current_branch  = VALUES(current_branch),
                is_cloned       = VALUES(is_cloned),
                default_branch  = VALUES(default_branch),
                last_synced_at  = VALUES(last_synced_at),
                last_commit_sha = VALUES(last_commit_sha),
                updated_at      = NOW()
        "#)
        .bind(repo.uuid().to_string())
        .bind(repo.account_id().to_string())
        .bind(repo.platform_id().to_string())
        .bind(repo.name())
        .bind(repo.full_name())
        .bind(repo.description())
        .bind(repo.remote_url().as_str())
        .bind(repo.remote_url().as_str())   // clone_url_ssh falls back to remote_url
        .bind(Option::<&str>::None)          // clone_url_https not available on entity
        .bind(repo.local_path())
        .bind(repo.default_branch())
        .bind(repo.current_branch())
        .bind(visibility_str(repo.visibility()))
        .bind(repo.is_cloned())
        .bind(repo.is_archived())
        .bind(repo.is_forked())
        .bind(repo.last_commit_sha())
        .bind(repo.last_synced_at())
        .execute(&self.pool)
        .await
        .map_err(storage_err)?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Repository>, GitManagerError> {
        let sql = format!("{SELECT_REPOS} AND r.uuid = ?");
        sqlx::query_as::<_, RepositoryRow>(&sql)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_err)?
            .map(row_to_repository)
            .transpose()
    }

    async fn find_by_local_path(
        &self,
        path: &str,
    ) -> Result<Option<Repository>, GitManagerError> {
        let sql = format!("{SELECT_REPOS} AND r.local_path = ?");
        sqlx::query_as::<_, RepositoryRow>(&sql)
            .bind(path)
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_err)?
            .map(row_to_repository)
            .transpose()
    }

    async fn find_by_full_name(
        &self,
        full_name: &str,
        account_id: Uuid,
    ) -> Result<Option<Repository>, GitManagerError> {
        let sql = format!("{SELECT_REPOS} AND r.full_name = ? AND a.uuid = ?");
        sqlx::query_as::<_, RepositoryRow>(&sql)
            .bind(full_name)
            .bind(account_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_err)?
            .map(row_to_repository)
            .transpose()
    }

    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Repository>, GitManagerError> {
        let sql = format!("{SELECT_REPOS} AND a.uuid = ? ORDER BY r.full_name ASC");
        sqlx::query_as::<_, RepositoryRow>(&sql)
            .bind(account_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_err)?
            .into_iter()
            .map(row_to_repository)
            .collect()
    }

    async fn list_cloned(&self) -> Result<Vec<Repository>, GitManagerError> {
        let sql = format!("{SELECT_REPOS} AND r.is_cloned = TRUE ORDER BY r.full_name ASC");
        sqlx::query_as::<_, RepositoryRow>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(storage_err)?
            .into_iter()
            .map(row_to_repository)
            .collect()
    }

    async fn delete(&self, id: Uuid) -> Result<(), GitManagerError> {
        sqlx::query("DELETE FROM repositories WHERE uuid = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_err)?;
        Ok(())
    }
}