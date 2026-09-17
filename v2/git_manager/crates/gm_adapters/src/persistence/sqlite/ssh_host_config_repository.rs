use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use gm_domain::ssh::{
    entities::SshHostConfig,
    ports::SshHostConfigRepository,
};
use gm_shared::errors::SshError;

fn ssh_err(ctx: &'static str, e: impl std::fmt::Display) -> SshError {
    SshError::ConfigWriteFailed { reason: format!("{ctx}: {e}") }
}

fn parse_uuid(s: &str, ctx: &'static str) -> Result<Uuid, SshError> {
    Uuid::parse_str(s).map_err(|e| ssh_err(ctx, e))
}

#[derive(sqlx::FromRow)]
struct SshHostConfigRow {
    uuid:            String,
    account_uuid:    String,
    ssh_key_uuid:    String,
    host_alias:      String,
    hostname:        String,
    username:        String,
    identity_file:   String,
    port:            u16,
    identities_only: bool,
    created_at:      DateTime<Utc>,
}

fn row_to_config(row: SshHostConfigRow) -> Result<SshHostConfig, SshError> {
    Ok(SshHostConfig::rehydrate(
        parse_uuid(&row.uuid,         "ssh_host_configs.uuid")?,
        parse_uuid(&row.account_uuid, "ssh_host_configs.account_uuid")?,
        parse_uuid(&row.ssh_key_uuid, "ssh_host_configs.ssh_key_uuid")?,
        row.host_alias,
        row.hostname,
        row.username,
        row.identity_file,
        row.port,
        row.identities_only,
        true,
        row.created_at,
    ))
}

const SELECT_CONFIGS: &str = r#"
    SELECT
        shc.uuid,
        a.uuid   AS account_uuid,
        sk.uuid  AS ssh_key_uuid,
        shc.host_alias,
        shc.hostname,
        shc."user"         AS username,
        shc.identity_file,
        shc.port,
        shc.identity_only  AS identities_only,
        shc.created_at
    FROM ssh_host_configs shc
    JOIN accounts  a  ON shc.account_id = a.id
    JOIN ssh_keys  sk ON shc.ssh_key_id  = sk.id
    WHERE 1=1
"#;

#[derive(Debug, Clone)]
pub struct SqliteSshHostConfigRepository {
    pool: SqlitePool,
}

impl SqliteSshHostConfigRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SshHostConfigRepository for SqliteSshHostConfigRepository {
    async fn save(&self, config: &SshHostConfig) -> Result<(), SshError> {
        sqlx::query(r#"
            INSERT INTO ssh_host_configs
                (uuid, account_id, ssh_key_id, host_alias, hostname,
                 identity_file, port, "user", identity_only, created_at)
            VALUES (?,
                    (SELECT id FROM accounts  WHERE uuid = ?),
                    (SELECT id FROM ssh_keys  WHERE uuid = ?),
                    ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(uuid) DO UPDATE SET
                host_alias      = excluded.host_alias,
                hostname        = excluded.hostname,
                identity_file   = excluded.identity_file,
                port            = excluded.port,
                "user"          = excluded."user",
                identity_only   = excluded.identity_only
        "#)
        .bind(config.uuid().to_string())
        .bind(config.account_id().to_string())
        .bind(config.ssh_key_id().to_string())
        .bind(config.host_alias())
        .bind(config.hostname())
        .bind(config.identity_file())
        .bind(config.port())
        .bind(config.username())
        .bind(config.identities_only())
        .bind(config.created_at())
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| ssh_err("save", e))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<SshHostConfig>, SshError> {
        let sql = format!("{SELECT_CONFIGS} AND shc.uuid = ?");
        sqlx::query_as::<_, SshHostConfigRow>(&sql)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ssh_err("find_by_id", e))?
            .map(row_to_config)
            .transpose()
    }

    async fn find_by_host_alias(&self, alias: &str) -> Result<Option<SshHostConfig>, SshError> {
        let sql = format!("{SELECT_CONFIGS} AND shc.host_alias = ?");
        sqlx::query_as::<_, SshHostConfigRow>(&sql)
            .bind(alias)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ssh_err("find_by_host_alias", e))?
            .map(row_to_config)
            .transpose()
    }

    async fn find_by_account(&self, account_id: Uuid) -> Result<Option<SshHostConfig>, SshError> {
        let sql = format!("{SELECT_CONFIGS} AND a.uuid = ? ORDER BY shc.created_at DESC LIMIT 1");
        sqlx::query_as::<_, SshHostConfigRow>(&sql)
            .bind(account_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ssh_err("find_by_account", e))?
            .map(row_to_config)
            .transpose()
    }

    async fn list_all_active(&self) -> Result<Vec<SshHostConfig>, SshError> {
        let sql = format!("{SELECT_CONFIGS} ORDER BY shc.created_at DESC");
        sqlx::query_as::<_, SshHostConfigRow>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ssh_err("list_all_active", e))?
            .into_iter()
            .map(row_to_config)
            .collect()
    }

    async fn deactivate_for_account(&self, account_id: Uuid) -> Result<(), SshError> {
        sqlx::query(r#"
            DELETE FROM ssh_host_configs
            WHERE account_id = (SELECT id FROM accounts WHERE uuid = ?)
        "#)
        .bind(account_id.to_string())
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| ssh_err("deactivate_for_account", e))
    }
}
