// crates/gm_ports/src/outbound/platform_repository.rs
//
// Outbound port for reading platform (Git hosting service) reference data.
//
// Platforms are seed data seeded once at migration time and are effectively
// read-only from the application's perspective. This port is separated from
// the domain AccountRepository because platform records are infrastructure
// reference data, not first-class domain entities with lifecycle events.
//
// ── Ownership ─────────────────────────────────────────────────────────────────
// The kernel owns the platform seed; adapters implement the read-path.
// The web and CLI interface layers call list_active() to populate form dropdowns
// and enrich DTOs with display names without embedding a UUID→name map in the
// interface layer itself.
//
// ── Dependency direction ──────────────────────────────────────────────────────
// gm_ports  →  gm_shared  (for PlatformDto, GitManagerError)
// gm_adapters → gm_ports  (implements this trait)
// gm_interface_web → gm_ports  (uses the trait via Arc<dyn PlatformRepository>)

use async_trait::async_trait;
use uuid::Uuid;
use gm_shared::{errors::GitManagerError, models::platform::PlatformDto};

/// Read-only access to the platform (Git hosting service) catalogue.
///
/// The implementation queries the `platforms` table which is populated by the
/// seed migration. Callers should treat the result as a stable but potentially
/// evolving list; never hardcode the UUIDs or slugs outside of the migration.
#[async_trait]
pub trait PlatformRepository: Send + Sync {
    /// Returns all platforms where `is_active = true`, ordered by insertion
    /// sequence (i.e. `id ASC`). The order matches the seed migration, giving
    /// a deterministic and human-readable presentation in dropdowns.
    async fn list_active(&self) -> Result<Vec<PlatformDto>, GitManagerError>;

    /// Returns the platform whose `uuid` column matches `uuid`, or `None` if
    /// the UUID is not present in the database.
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<PlatformDto>, GitManagerError>;
}
