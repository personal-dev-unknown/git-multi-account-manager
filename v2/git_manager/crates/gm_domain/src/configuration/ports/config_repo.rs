// crates/gm_domain/src/configuration/ports/config_repository.rs
//
// Persistence port for the Configuration entity. Unlike the account or
// repository ports — which work with collections — there is exactly one
// Configuration record per installation. The port therefore uses load/save
// semantics rather than find/list. The adapter returns a fresh default
// configuration if no record exists yet, so the service layer never has
// to handle a None case for the initial boot.

use async_trait::async_trait;
use crate::configuration::entities::Configuration;
use gm_shared::errors::GitManagerError;

/// Persistence contract for the single application configuration record.
#[async_trait]
pub trait ConfigRepository: Send + Sync {
    /// Loads the current configuration. Returns a default-valued Configuration
    /// if no record has been persisted yet (i.e., first boot after install).
    async fn load(&self) -> Result<Configuration, GitManagerError>;

    /// Persists the configuration. Uses an upsert: creates the record if it
    /// does not exist, otherwise updates it in place.
    async fn save(&self, config: &Configuration) -> Result<(), GitManagerError>;
}