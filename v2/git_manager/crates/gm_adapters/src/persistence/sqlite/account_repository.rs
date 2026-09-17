// crates/gm_adapters/src/persistence/sqlite/account_repository.rs
//
// The SQLite implementation of AccountRepository, intended for development
// and integration testing without a MySQL server. The queries are written
// to be compatible with SQLite's SQL dialect, which differs from MySQL in
// a few important ways:
//
//   - No ON DUPLICATE KEY UPDATE: SQLite uses INSERT OR REPLACE or UPSERT syntax.
//   - TIMESTAMP columns are stored as TEXT in SQLite (ISO 8601 format).
//   - ENUM constraints don't exist: SQLite accepts any string; constraints are
//     application-level.
//   - Subqueries in INSERT ... VALUES are supported but the JOIN syntax for
//     platform_id resolution is the same.
//
// The schema applied to SQLite is the same file as MySQL (SQLite is permissive
// about column types and ignores ENGINE=InnoDB), so the same migration files
// work for both databases. SQLx handles the connection pool type difference
// via AnyPool.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use gm_domain::accounts::{
    entities::Account,
    ports::AccountRepository,
    value_objects::{AccountStatus, AuthMethod},
};
use gm_shared::errors::AccountError;

#[derive(Debug)]
pub struct SqliteAccountRepository {
    pool: SqlitePool,
}

impl SqliteAccountRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AccountRow {
    uuid:           String,
    platform_uuid:  String,
    alias:          String,
    username:       String,
    email:          String,
    display_name:   Option<String>,
    auth_method:    String,
    ssh_host_alias: Option<String>,
    status:         String,
    is_default:     bool,
    last_used_at:   Option<DateTime<Utc>>,
    created_at:     DateTime<Utc>,
}

fn auth_method_from_db(s: &str) -> AuthMethod {
    match s {
        "https_pat"      => AuthMethod::HttpsPat,
        "https_password" => AuthMethod::HttpsPassword,
        "oauth"          => AuthMethod::OAuth,
        "anonymous"      => AuthMethod::Anonymous,
        _                => AuthMethod::Ssh,
    }
}

fn auth_method_to_db(m: &AuthMethod) -> &'static str {
    match m {
        AuthMethod::Ssh           => "ssh",
        AuthMethod::HttpsPat      => "https_pat",
        AuthMethod::HttpsPassword => "https_password",
        AuthMethod::OAuth         => "oauth",
        AuthMethod::Anonymous     => "anonymous",
    }
}

fn status_from_db(s: &str) -> AccountStatus {
    match s {
        "inactive"      => AccountStatus::Inactive,
        "suspended"     => AccountStatus::Suspended,
        "token_expired" => AccountStatus::TokenExpired,
        "unverified"    => AccountStatus::Unverified,
        _               => AccountStatus::Active,
    }
}

fn status_to_db(s: &AccountStatus) -> &'static str {
    match s {
        AccountStatus::Active       => "active",
        AccountStatus::Inactive     => "inactive",
        AccountStatus::Suspended    => "suspended",
        AccountStatus::TokenExpired => "token_expired",
        AccountStatus::Unverified   => "unverified",
    }
}

fn row_to_account(row: AccountRow) -> Result<Account, AccountError> {
    let uuid = Uuid::parse_str(&row.uuid)
        .map_err(|e| AccountError::StorageFailed(e.to_string()))?;
    let platform_id = Uuid::parse_str(&row.platform_uuid)
        .map_err(|e| AccountError::StorageFailed(e.to_string()))?;

    Ok(Account::rehydrate(
        uuid,
        platform_id,
        row.alias,
        row.username,
        row.email,
        row.display_name,
        auth_method_from_db(&row.auth_method),
        row.ssh_host_alias,
        status_from_db(&row.status),
        row.is_default,
        row.last_used_at,
        row.created_at,
    ))
}

const SELECT_ACCOUNTS: &str = r#"
    SELECT
        a.uuid,
        p.uuid AS platform_uuid,
        a.alias,
        a.username,
        a.email,
        a.display_name,
        a.auth_method,
        a.ssh_host_alias,
        a.status,
        a.is_default,
        a.last_used_at,
        a.created_at
    FROM accounts a
    JOIN platforms p ON a.platform_id = p.id
    WHERE a.deleted_at IS NULL
"#;

#[async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn save(&self, account: &Account) -> Result<(), AccountError> {
        let auth_method = auth_method_to_db(account.auth_method());
        let status      = status_to_db(account.status());

        sqlx::query(r#"
            INSERT INTO accounts
                (uuid, platform_id, alias, username, email, display_name,
                 auth_method, ssh_host_alias, status, is_default, last_used_at)
            VALUES (?, (SELECT id FROM platforms WHERE uuid = ?), ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(uuid) DO UPDATE SET
                alias          = excluded.alias,
                username       = excluded.username,
                email          = excluded.email,
                display_name   = excluded.display_name,
                auth_method    = excluded.auth_method,
                ssh_host_alias = excluded.ssh_host_alias,
                status         = excluded.status,
                is_default     = excluded.is_default,
                last_used_at   = excluded.last_used_at
        "#)
        .bind(account.uuid().to_string())
        .bind(account.platform_id().to_string())
        .bind(account.alias())
        .bind(account.username())
        .bind(account.email())
        .bind(account.display_name())
        .bind(auth_method)
        .bind(account.ssh_host_alias())
        .bind(status)
        .bind(account.is_default())
        .bind(account.last_used_at())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("UNIQUE constraint failed") && msg.contains("alias") {
                AccountError::AliasAlreadyExists {
                    alias:       account.alias().to_string(),
                    platform_id: account.platform_id(),
                }
            } else {
                AccountError::StorageFailed(msg)
            }
        })?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, AccountError> {
        let sql = format!("{SELECT_ACCOUNTS} AND a.uuid = ?");
        sqlx::query_as::<_, AccountRow>(&sql)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AccountError::StorageFailed(e.to_string()))?
            .map(row_to_account)
            .transpose()
    }

    async fn find_by_alias(&self, alias: &str, platform_id: Uuid) -> Result<Option<Account>, AccountError> {
        let sql = format!("{SELECT_ACCOUNTS} AND a.alias = ? AND p.uuid = ?");
        sqlx::query_as::<_, AccountRow>(&sql)
            .bind(alias)
            .bind(platform_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AccountError::StorageFailed(e.to_string()))?
            .map(row_to_account)
            .transpose()
    }

    async fn list_all(&self) -> Result<Vec<Account>, AccountError> {
        let sql = format!("{SELECT_ACCOUNTS} ORDER BY a.alias ASC");
        sqlx::query_as::<_, AccountRow>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AccountError::StorageFailed(e.to_string()))?
            .into_iter()
            .map(row_to_account)
            .collect()
    }

    async fn list_by_platform(&self, platform_id: Uuid) -> Result<Vec<Account>, AccountError> {
        let sql = format!("{SELECT_ACCOUNTS} AND p.uuid = ? ORDER BY a.is_default DESC, a.alias ASC");
        sqlx::query_as::<_, AccountRow>(&sql)
            .bind(platform_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AccountError::StorageFailed(e.to_string()))?
            .into_iter()
            .map(row_to_account)
            .collect()
    }

    async fn delete(&self, id: Uuid) -> Result<(), AccountError> {
        sqlx::query("UPDATE accounts SET deleted_at = CURRENT_TIMESTAMP WHERE uuid = ? AND deleted_at IS NULL")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| AccountError::StorageFailed(e.to_string()))?;
        Ok(())
    }

    async fn set_default(&self, account_id: Uuid, platform_id: Uuid) -> Result<(), AccountError> {
        sqlx::query(
            "UPDATE accounts SET is_default = 0
             WHERE platform_id = (SELECT id FROM platforms WHERE uuid = ?) AND deleted_at IS NULL",
        )
        .bind(platform_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AccountError::StorageFailed(e.to_string()))?;

        sqlx::query("UPDATE accounts SET is_default = 1 WHERE uuid = ?")
            .bind(account_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| AccountError::StorageFailed(e.to_string()))?;

        Ok(())
    }

    async fn count_by_platform(&self, platform_id: Uuid) -> Result<u64, AccountError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM accounts a JOIN platforms p ON a.platform_id = p.id
             WHERE p.uuid = ? AND a.deleted_at IS NULL",
        )
        .bind(platform_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AccountError::StorageFailed(e.to_string()))?;

        Ok(row.0 as u64)
    }
}