// crates/gm_adapters/src/persistence/mysql/account_repository.rs
//
// The MySQL implementation of the AccountRepository domain port. This is where
// the pure domain model meets the real database. The code in this file has no
// business logic — it only translates between the Account entity and the
// accounts / platforms rows in MySQL.
//
// ── Schema bridge ─────────────────────────────────────────────────────────────
// The database schema uses BIGINT surrogate keys for all foreign keys (for join
// performance) but exposes UUID columns as the stable public identifiers. The
// domain only ever sees UUIDs, so every query that involves a foreign key must
// do a subquery or JOIN to translate between the two representations.
//
// For example, inserting an account requires knowing the platform's BIGINT id
// from its UUID:
//   INSERT INTO accounts (..., platform_id, ...) VALUES (..., (SELECT id FROM platforms WHERE uuid = ?), ...)
//
// ── Error mapping ─────────────────────────────────────────────────────────────
// sqlx errors are mapped to AccountError::StorageFailed which carries the
// underlying message as a string. Infrastructure errors (disk full, connection
// lost, deadlock timeout) do not map neatly to domain errors — they are
// genuinely exceptional conditions that the caller can log and surface to the
// user as "operation failed, please retry".
//
// Specific errors like "duplicate key" for alias conflicts ARE mapped to the
// typed domain error (AccountError::AliasAlreadyExists) so the UI can display
// a helpful message instead of a raw SQL error.
//
// ── Soft deletes ─────────────────────────────────────────────────────────────
// The accounts table has a deleted_at column. All queries append
// `AND deleted_at IS NULL` to exclude soft-deleted rows. The delete() method
// sets deleted_at rather than issuing a DELETE statement, preserving the row
// for foreign key integrity (sync sessions, git operations still reference it).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use gm_domain::accounts::{
    entities::Account,
    ports::AccountRepository,
    value_objects::{AccountStatus, AuthMethod},
};
use gm_shared::errors::AccountError;

// ─────────────────────────────────────────────────────────────────────────────
// The adapter struct
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct MySqlAccountRepository {
    pool: MySqlPool,
}

impl MySqlAccountRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Row type — maps DB columns to a flat Rust struct via sqlx::FromRow
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Helpers — string↔enum conversions
// ─────────────────────────────────────────────────────────────────────────────

fn auth_method_to_db(m: &AuthMethod) -> &'static str {
    match m {
        AuthMethod::Ssh           => "ssh",
        AuthMethod::HttpsPat      => "https_pat",
        AuthMethod::HttpsPassword => "https_password",
        AuthMethod::OAuth         => "oauth",
        AuthMethod::Anonymous     => "anonymous",
    }
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

fn status_to_db(s: &AccountStatus) -> &'static str {
    match s {
        AccountStatus::Active       => "active",
        AccountStatus::Inactive     => "inactive",
        AccountStatus::Suspended    => "suspended",
        AccountStatus::TokenExpired => "token_expired",
        AccountStatus::Unverified   => "unverified",
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

fn row_to_account(row: AccountRow) -> Result<Account, AccountError> {
    let uuid = Uuid::parse_str(&row.uuid)
        .map_err(|e| AccountError::StorageFailed(format!("invalid uuid: {e}")))?;
    let platform_id = Uuid::parse_str(&row.platform_uuid)
        .map_err(|e| AccountError::StorageFailed(format!("invalid platform_uuid: {e}")))?;

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

/// The SELECT list and JOIN used by every read query. Defined once to ensure
/// all query paths return the same columns in the same order.
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

// ─────────────────────────────────────────────────────────────────────────────
// impl AccountRepository
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl AccountRepository for MySqlAccountRepository {
    async fn save(&self, account: &Account) -> Result<(), AccountError> {
        let auth_method  = auth_method_to_db(account.auth_method());
        let status       = status_to_db(account.status());

        // INSERT … ON DUPLICATE KEY UPDATE handles both create and update cases.
        // platform_id is resolved from the platform UUID via a subquery.
        sqlx::query(
            r#"
            INSERT INTO accounts
                (uuid, platform_id, alias, username, email, display_name,
                 auth_method, ssh_host_alias, status, is_default, last_used_at)
            VALUES (?, (SELECT id FROM platforms WHERE uuid = ?), ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                alias          = VALUES(alias),
                username       = VALUES(username),
                email          = VALUES(email),
                display_name   = VALUES(display_name),
                auth_method    = VALUES(auth_method),
                ssh_host_alias = VALUES(ssh_host_alias),
                status         = VALUES(status),
                is_default     = VALUES(is_default),
                last_used_at   = VALUES(last_used_at),
                updated_at     = NOW()
            "#,
        )
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
            if msg.contains("Duplicate entry") && msg.contains("uq_accounts_alias_platform") {
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

    async fn find_by_alias(
        &self,
        alias:       &str,
        platform_id: Uuid,
    ) -> Result<Option<Account>, AccountError> {
        let sql = format!(
            "{SELECT_ACCOUNTS} AND a.alias = ? AND p.uuid = ?"
        );
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
        let sql = format!(
            "{SELECT_ACCOUNTS} AND p.uuid = ? ORDER BY a.is_default DESC, a.alias ASC"
        );
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
        // Soft delete: set deleted_at rather than removing the row.
        // Sync sessions and git operations still reference this account via FK.
        sqlx::query("UPDATE accounts SET deleted_at = NOW() WHERE uuid = ? AND deleted_at IS NULL")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| AccountError::StorageFailed(e.to_string()))?;
        Ok(())
    }

    async fn set_default(
        &self,
        account_id:  Uuid,
        platform_id: Uuid,
    ) -> Result<(), AccountError> {
        // Execute as a transaction to guarantee atomicity: clear + set happen
        // together. If either query fails the transaction rolls back, leaving
        // the previous default account unchanged.
        let mut tx = self.pool.begin().await
            .map_err(|e| AccountError::StorageFailed(format!("begin transaction: {e}")))?;

        // Clear the default flag for all accounts on this platform.
        sqlx::query(
            "UPDATE accounts SET is_default = FALSE
             WHERE platform_id = (SELECT id FROM platforms WHERE uuid = ?)
               AND deleted_at IS NULL",
        )
        .bind(platform_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| AccountError::StorageFailed(format!("clear defaults: {e}")))?;

        // Set the specific account as default.
        sqlx::query("UPDATE accounts SET is_default = TRUE WHERE uuid = ?")
            .bind(account_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| AccountError::StorageFailed(format!("set default: {e}")))?;

        tx.commit().await
            .map_err(|e| AccountError::StorageFailed(format!("commit: {e}")))?;

        Ok(())
    }

    async fn count_by_platform(&self, platform_id: Uuid) -> Result<u64, AccountError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM accounts a
             JOIN platforms p ON a.platform_id = p.id
             WHERE p.uuid = ? AND a.deleted_at IS NULL",
        )
        .bind(platform_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AccountError::StorageFailed(e.to_string()))?;

        Ok(row.0 as u64)
    }
}