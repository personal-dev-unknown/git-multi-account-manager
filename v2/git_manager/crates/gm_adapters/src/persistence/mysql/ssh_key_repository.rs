// crates/gm_adapters/src/persistence/mysql/ssh_key_repository.rs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use gm_domain::ssh::{
    entities::SshKey,
    ports::SshKeyRepository,
    value_objects::{KeyType, TestStatus},
};
use gm_shared::errors::SshError;

#[derive(Debug)]
pub struct MySqlSshKeyRepository {
    pool: MySqlPool,
}

impl MySqlSshKeyRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct SshKeyRow {
    uuid:              String,
    account_uuid:      Option<String>,
    name:              String,
    key_type:          String,
    key_size_bits:     Option<i16>,
    public_key:        String,
    fingerprint:       String,
    private_key_path:  String,
    comment_email:     Option<String>,
    is_active:         bool,
    is_added_to_agent: bool,
    last_tested_at:    Option<DateTime<Utc>>,
    last_test_status:  String,
    last_test_error:   Option<String>,
    created_at:        DateTime<Utc>,
}

fn key_type_from_db(s: &str, size: Option<i16>) -> KeyType {
    match s {
        "rsa"   => KeyType::Rsa(size.unwrap_or(4096) as u32),
        "ecdsa" => KeyType::Ecdsa,
        _       => KeyType::Ed25519,
    }
}

fn key_type_to_db(kt: &KeyType) -> &'static str {
    match kt {
        KeyType::Ed25519  => "ed25519",
        KeyType::Rsa(_)   => "rsa",
        KeyType::Ecdsa    => "ecdsa",
    }
}

fn test_status_from_row(row: &SshKeyRow) -> TestStatus {
    match row.last_test_status.as_str() {
        "success" => TestStatus::Success {
            username:  String::new(), // username not stored separately
            tested_at: row.last_tested_at.unwrap_or_else(Utc::now),
        },
        "failed" => TestStatus::Failed {
            reason:    row.last_test_error.clone().unwrap_or_default(),
            tested_at: row.last_tested_at.unwrap_or_else(Utc::now),
        },
        _ => TestStatus::Untested,
    }
}

fn row_to_ssh_key(row: SshKeyRow) -> Result<SshKey, SshError> {
    let uuid       = Uuid::parse_str(&row.uuid)
        .map_err(|e| SshError::StorageFailed(e.to_string()))?;
    let account_id = row.account_uuid.as_deref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|e| SshError::StorageFailed(e.to_string()))?
        .unwrap_or_else(Uuid::nil);

    let key_type    = key_type_from_db(&row.key_type, row.key_size_bits);
    let test_status = test_status_from_row(&row);

    Ok(SshKey::rehydrate(
        uuid,
        account_id,
        row.name,
        key_type,
        row.public_key,
        row.fingerprint,
        row.private_key_path,
        test_status,
        row.is_active,
        row.created_at,
    ))
}

const SELECT_KEYS: &str = r#"
    SELECT
        k.uuid,
        a.uuid AS account_uuid,
        k.name,
        k.key_type,
        k.key_size_bits,
        k.public_key,
        k.fingerprint,
        k.private_key_path,
        k.comment_email,
        k.is_active,
        k.is_added_to_agent,
        k.last_tested_at,
        k.last_test_status,
        k.last_test_error,
        k.created_at
    FROM ssh_keys k
    LEFT JOIN accounts a ON k.account_id = a.id
    WHERE 1=1
"#;

#[async_trait]
impl SshKeyRepository for MySqlSshKeyRepository {
    async fn save(&self, key: &SshKey) -> Result<(), SshError> {
        let key_type_str = key_type_to_db(&key.key_type());
        let key_size     = key.key_type().rsa_bits().map(|b| b as i16);
        let (test_status_str, test_error) = match key.test_status() {
            TestStatus::Untested => ("untested", None),
            TestStatus::Success { .. } => ("success", None),
            TestStatus::Failed { reason, .. } => ("failed", Some(reason.as_str())),
        };
        let last_tested = key.test_status().last_tested_at();

        sqlx::query(r#"
            INSERT INTO ssh_keys
                (uuid, account_id, name, key_type, key_size_bits, public_key, fingerprint,
                 private_key_path, is_active, last_test_status, last_tested_at, last_test_error)
            VALUES (?,
                    CASE WHEN ? IS NULL THEN NULL ELSE (SELECT id FROM accounts WHERE uuid = ?) END,
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                name             = VALUES(name),
                is_active        = VALUES(is_active),
                last_test_status = VALUES(last_test_status),
                last_tested_at   = VALUES(last_tested_at),
                last_test_error  = VALUES(last_test_error),
                updated_at       = NOW()
        "#)
        .bind(key.uuid().to_string())
        .bind(if key.account_id().is_nil() { None } else { Some(key.account_id().to_string()) })
        .bind(if key.account_id().is_nil() { None } else { Some(key.account_id().to_string()) })
        .bind(key.name())
        .bind(key_type_str)
        .bind(key_size)
        .bind(key.public_key())
        .bind(key.fingerprint())
        .bind(key.private_key_path())
        .bind(key.is_active())
        .bind(test_status_str)
        .bind(last_tested)
        .bind(test_error)
        .execute(&self.pool)
        .await
        .map_err(|e| SshError::StorageFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<SshKey>, SshError> {
        let sql = format!("{SELECT_KEYS} AND k.uuid = ?");
        sqlx::query_as::<_, SshKeyRow>(&sql)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SshError::StorageFailed(e.to_string()))?
            .map(row_to_ssh_key)
            .transpose()
    }

    async fn find_active_for_account(&self, account_id: Uuid) -> Result<Option<SshKey>, SshError> {
        let sql = format!("{SELECT_KEYS} AND a.uuid = ? AND k.is_active = TRUE LIMIT 1");
        sqlx::query_as::<_, SshKeyRow>(&sql)
            .bind(account_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SshError::StorageFailed(e.to_string()))?
            .map(row_to_ssh_key)
            .transpose()
    }

    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<SshKey>, SshError> {
        let sql = format!("{SELECT_KEYS} AND a.uuid = ? ORDER BY k.created_at DESC");
        sqlx::query_as::<_, SshKeyRow>(&sql)
            .bind(account_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SshError::StorageFailed(e.to_string()))?
            .into_iter()
            .map(row_to_ssh_key)
            .collect()
    }

    async fn find_by_fingerprint(&self, fingerprint: &str) -> Result<Option<SshKey>, SshError> {
        let sql = format!("{SELECT_KEYS} AND k.fingerprint = ?");
        sqlx::query_as::<_, SshKeyRow>(&sql)
            .bind(fingerprint)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SshError::StorageFailed(e.to_string()))?
            .map(row_to_ssh_key)
            .transpose()
    }

    async fn deactivate_all_for_account(&self, account_id: Uuid) -> Result<(), SshError> {
        sqlx::query(
            "UPDATE ssh_keys SET is_active = FALSE
             WHERE account_id = (SELECT id FROM accounts WHERE uuid = ?)"
        )
        .bind(account_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| SshError::StorageFailed(e.to_string()))?;
        Ok(())
    }
}

impl MySqlSshKeyRepository {
    pub async fn delete(&self, id: Uuid) -> Result<(), SshError> {
        sqlx::query("DELETE FROM ssh_keys WHERE uuid = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| SshError::StorageFailed(e.to_string()))?;
        Ok(())
    }
}