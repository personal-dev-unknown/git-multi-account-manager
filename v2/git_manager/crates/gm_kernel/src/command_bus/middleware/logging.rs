// crates/gm_kernel/src/command_bus/middleware/logging.rs
//
// Logs every command dispatch with its type name and outcome. This middleware
// must be FIRST in the chain so it wraps both validation and the handler,
// giving the full wall-clock time including any validation failures.

use async_trait::async_trait;
use gm_shared::errors::GitManagerError;
use super::Middleware;

#[derive(Debug, Default)]
pub struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, command_type: &str) -> Result<(), GitManagerError> {
        tracing::info!(command_type = command_type, "command dispatched");
        Ok(())
    }

    async fn after(&self, command_type: &str, elapsed_us: u64, success: bool) {
        if success {
            tracing::info!(
                command_type = command_type,
                elapsed_us   = elapsed_us,
                "command completed"
            );
        } else {
            tracing::warn!(
                command_type = command_type,
                elapsed_us   = elapsed_us,
                "command failed"
            );
        }
    }
}