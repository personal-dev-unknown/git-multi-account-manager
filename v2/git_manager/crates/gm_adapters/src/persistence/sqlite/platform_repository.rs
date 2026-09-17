use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;
use gm_shared::{errors::GitManagerError, models::platform::PlatformDto};
use gm_ports::outbound::PlatformRepository;

#[derive(Debug, Clone)]
pub struct SqlitePlatformRepository {
    pool: SqlitePool,
}

impl SqlitePlatformRepository {
    pub fn new(pool: SqlitePool) -> Self { Self { pool } }
}

#[derive(sqlx::FromRow)]
struct PlatformRow {
    uuid:            String,
    name:            String,
    display_name:    String,
    api_base_url:    Option<String>,
    ssh_host:        Option<String>,
    supports_oauth:  bool,
    supports_pat:    bool,
    supports_ssh:    bool,
    supports_https:  bool,
    is_active:       bool,
    is_self_hosted:  bool,
}

fn row_to_dto(row: PlatformRow) -> Result<PlatformDto, GitManagerError> {
    let uuid = Uuid::parse_str(&row.uuid)
        .map_err(|e| GitManagerError::Database(format!("invalid platform uuid '{}': {}", row.uuid, e)))?;
    Ok(PlatformDto {
        uuid,
        name:            row.name,
        display_name:    row.display_name,
        api_base_url:    row.api_base_url,
        ssh_host:        row.ssh_host,
        supports_oauth:  row.supports_oauth,
        supports_pat:    row.supports_pat,
        supports_ssh:    row.supports_ssh,
        supports_https:  row.supports_https,
        is_active:       row.is_active,
        is_self_hosted:  row.is_self_hosted,
    })
}

const SELECT_PLATFORMS: &str = r#"
    SELECT uuid, name, display_name, api_base_url, ssh_host,
           supports_oauth, supports_pat, supports_ssh, supports_https,
           is_active, is_self_hosted
    FROM platforms
"#;

#[async_trait]
impl PlatformRepository for SqlitePlatformRepository {
    async fn list_active(&self) -> Result<Vec<PlatformDto>, GitManagerError> {
        let sql = format!("{SELECT_PLATFORMS} WHERE is_active = 1 ORDER BY rowid ASC");
        sqlx::query_as::<_, PlatformRow>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| GitManagerError::Database(e.to_string()))?
            .into_iter()
            .map(row_to_dto)
            .collect()
    }

    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<PlatformDto>, GitManagerError> {
        let sql = format!("{SELECT_PLATFORMS} WHERE uuid = ?");
        sqlx::query_as::<_, PlatformRow>(&sql)
            .bind(uuid.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| GitManagerError::Database(e.to_string()))?
            .map(row_to_dto)
            .transpose()
    }
}
