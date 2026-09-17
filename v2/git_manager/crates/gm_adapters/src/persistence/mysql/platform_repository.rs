// crates/gm_adapters/src/persistence/mysql/platform_repository.rs
//
// MySQL implementation of the PlatformRepository outbound port.
//
// Platforms are seed data inserted once by the migration and treated as
// effectively read-only by the application. This adapter exposes them to the
// web and CLI interface layers so that form dropdowns and account enrichment
// can be driven by the real database rather than hardcoded UUID→name maps.
//
// ── Pattern ───────────────────────────────────────────────────────────────────
// Follows the same conventions as MySqlAccountRepository:
//   1. A private `#[derive(sqlx::FromRow)]` row struct maps DB columns 1:1.
//   2. A `row_to_dto` free function converts the raw row into the shared DTO.
//   3. All queries use `sqlx::query_as::<_, PlatformRow>` so the mapping is
//      compile-time checked against the struct field names and types.
//   4. Errors are mapped to `GitManagerError::Database`.
//
// ── Boolean columns ───────────────────────────────────────────────────────────
// MySQL/MariaDB stores BOOLEAN as TINYINT(1). SQLx's MySQL driver maps
// TINYINT(1) to `bool` when the Rust field is typed `bool`, so no manual
// conversion is needed here.

use async_trait::async_trait;
use sqlx::MySqlPool;
use uuid::Uuid;
use gm_shared::{errors::GitManagerError, models::platform::PlatformDto};
use gm_ports::outbound::PlatformRepository;

// ─────────────────────────────────────────────────────────────────────────────
// Adapter struct
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MySqlPlatformRepository {
    pool: MySqlPool,
}

impl MySqlPlatformRepository {
    pub fn new(pool: MySqlPool) -> Self { Self { pool } }
}

// ─────────────────────────────────────────────────────────────────────────────
// Row type — 1:1 column mapping; SQLx populates this via FromRow derive
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Row → DTO conversion
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// SELECT fragment reused by both read queries
// ─────────────────────────────────────────────────────────────────────────────

const SELECT_PLATFORMS: &str = r#"
    SELECT uuid, name, display_name, api_base_url, ssh_host,
           supports_oauth, supports_pat, supports_ssh, supports_https,
           is_active, is_self_hosted
    FROM platforms
"#;

// ─────────────────────────────────────────────────────────────────────────────
// impl PlatformRepository
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl PlatformRepository for MySqlPlatformRepository {
    async fn list_active(&self) -> Result<Vec<PlatformDto>, GitManagerError> {
        let sql = format!("{SELECT_PLATFORMS} WHERE is_active = 1 ORDER BY id ASC");
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
