// crates/gm_kernel/src/command_bus/middleware/mod.rs
//
// The command bus middleware pipeline. Each middleware wraps the command
// dispatch with a before/after behaviour. Middlewares run in registration order,
// forming a chain of responsibility:
//
//   logging → validation → auth → handler → result
//
// If any middleware returns Err, the chain is aborted and the error propagates
// back to the caller without reaching subsequent middlewares or the handler.
// This means logging always runs (observability), validation always runs before
// auth (no point authorising an invalid command), and the handler only sees
// valid, authorised commands.

use async_trait::async_trait;
use gm_shared::errors::GitManagerError;

/// A step in the command bus middleware pipeline.
///
/// Implementations should keep their before() logic fast and non-blocking.
/// Logging and validation are synchronous; auth checks a token, which is
/// fast because it is a local claim check, not a network call.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Called before the command handler runs.
    /// Return Err to abort the pipeline and return the error to the caller.
    async fn before(&self, command_type: &str) -> Result<(), GitManagerError>;

    /// Called after the command handler completes, whether it succeeded or failed.
    /// Receives the elapsed time in microseconds for metrics recording.
    async fn after(&self, command_type: &str, elapsed_us: u64, success: bool);
}

pub mod auth;
pub mod logging;
pub mod validation;

pub use auth::AuthMiddleware;
pub use logging::LoggingMiddleware;
pub use validation::ValidationMiddleware;