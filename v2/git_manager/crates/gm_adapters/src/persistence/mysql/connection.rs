// crates/gm_adapters/src/persistence/mysql/connection.rs
//
// The MySQL connection pool factory. This module's sole purpose is to create
// a configured sqlx::MySqlPool that all repository implementations share.
// Using a pool (rather than a single connection) is critical for async Rust
// because multiple tokio tasks may be executing concurrent database queries
// simultaneously — a pool gives each task its own connection from the pre-allocated
// set rather than serialising all queries through a single connection.
//
// ── Pool configuration ────────────────────────────────────────────────────────
// The min_connections(1) ensures one connection is always pre-established and
// ready, eliminating connection latency on the first query after idle periods.
// max_connections(10) is generous for a single-user local tool but leaves room
// for the web interface, CLI, and background sync tasks to run concurrently
// without exhausting the MySQL server's connection limit.
//
// ── Migration execution ───────────────────────────────────────────────────────
// migrations_dir should point to the migrations/ directory within this adapter
// crate. SQLx tracks which migrations have been applied in the _sqlx_migrations
// table, so running migrate() on every boot is safe and idempotent.

use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::time::Duration;
use gm_shared::errors::GitManagerError;

/// Creates a MySQL connection pool and returns it.
/// Does NOT run migrations — that is the binary entry point's responsibility
/// so that migration failures are surfaced clearly at startup, not buried inside
/// adapter construction.
pub async fn create_mysql_pool(database_url: &str) -> Result<MySqlPool, GitManagerError> {
    MySqlPoolOptions::new()
        .min_connections(1)
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .connect(database_url)
        .await
        .map_err(|e| GitManagerError::Database(format!("MySQL pool creation failed: {e}")))
}