use std::collections::HashMap;
use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;
use gm_shared::errors::GitManagerError;
use gm_domain::configuration::{
    Configuration,
    ConfigRepository,
};

#[derive(Debug, Clone)]
pub struct SqliteConfigRepository {
    pool: SqlitePool,
}

impl SqliteConfigRepository {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }
}

#[derive(sqlx::FromRow)]
struct ConfigRow {
    config_key:   String,
    config_value: Option<String>,
}

#[async_trait]
impl ConfigRepository for SqliteConfigRepository {
    async fn load(&self) -> Result<Configuration, GitManagerError> {
        let rows = sqlx::query_as::<_, ConfigRow>(
            "SELECT config_key, config_value FROM app_configurations"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GitManagerError::Database(e.to_string()))?;

        if rows.is_empty() {
            return Ok(Configuration::new_with_defaults());
        }

        let mut entries = HashMap::new();
        for row in rows {
            if let Some(val) = row.config_value {
                entries.insert(row.config_key, val);
            }
        }

        Ok(Configuration::rehydrate(
            Uuid::new_v4(),
            entries,
            Utc::now(),
            Utc::now(),
        ))
    }

    async fn save(&self, config: &Configuration) -> Result<(), GitManagerError> {
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| GitManagerError::Database(e.to_string()))?;

        for (key, value) in config.all() {
            sqlx::query(
                r#"INSERT INTO app_configurations (config_key, config_value, data_type)
                   VALUES (?, ?, 'string')
                   ON CONFLICT(config_key) DO UPDATE SET config_value = excluded.config_value, updated_at = CURRENT_TIMESTAMP"#
            )
            .bind(key)
            .bind(value)
            .execute(&mut *tx)
            .await
            .map_err(|e| GitManagerError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| GitManagerError::Database(e.to_string()))?;

        Ok(())
    }
}
