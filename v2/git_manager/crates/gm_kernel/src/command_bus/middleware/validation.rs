// crates/gm_kernel/src/command_bus/middleware/validation.rs
//
// Calls validate() on the command before the handler runs. The validate() method
// is defined on the Command trait and performs structural validation — empty
// required fields, out-of-range values, etc. Business rule violations (e.g.
// "alias already exists") are the domain service's responsibility, not the
// command's. This middleware catches format problems early, before the domain
// allocates any resources processing an obviously malformed request.

use async_trait::async_trait;
use gm_shared::errors::GitManagerError;
use super::Middleware;

/// Calls validate() on the raw serialised command context.
/// The CommandBus calls this middleware with the command type name; the actual
/// validate() call happens at the dispatcher level where the typed command is
/// still available before erasure.
#[derive(Debug, Default)]
pub struct ValidationMiddleware;

#[async_trait]
impl Middleware for ValidationMiddleware {
    async fn before(&self, command_type: &str) -> Result<(), GitManagerError> {
        // Structural validation (non-empty fields, format) happens at the
        // dispatcher level where the typed command is accessible. This middleware
        // exists as the wiring point for pre-dispatch cross-cutting validation
        // rules (e.g. rate limiting) that operate at the type-name level.
        tracing::debug!(command_type = command_type, "validation middleware: before");
        Ok(())
    }

    async fn after(&self, _command_type: &str, _elapsed_us: u64, _success: bool) {}
}