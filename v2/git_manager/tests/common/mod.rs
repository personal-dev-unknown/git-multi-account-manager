use std::sync::Arc;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use uuid::Uuid;

use gm_adapters::persistence::sqlite::{SqliteAccountRepository, setup_sqlite_schema};
use gm_domain::accounts::ports::AccountRepository;
use gm_domain::accounts::services::account_service::AccountService;

/// Known platform UUID for GitHub (matching seed migration)
pub const GITHUB_PLATFORM_ID: Uuid = uuid::Uuid::from_u128(0x00000000_0001_0000_0000_000000000001);

/// Creates an in-memory SQLite database, creates the schema,
/// and returns the pool.
pub async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite pool");

    setup_sqlite_schema(&pool)
        .await
        .expect("Failed to set up SQLite schema");

    pool
}

/// Creates an AccountService with an in-memory SQLite repository for testing.
pub fn create_account_service(pool: SqlitePool) -> AccountService {
    let repo = Arc::new(SqliteAccountRepository::new(pool)) as Arc<dyn AccountRepository>;
    AccountService::new(repo)
}
