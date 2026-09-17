// crates/gm_kernel/src/command_bus/middleware/auth.rs
//
// The AuthMiddleware is the command bus gatekeeper for permission checking.
// In v1 this is a permissive stub — Git Manager is a single-user local tool,
// so every command is permitted. The middleware exists so that multi-user
// or enterprise editions can inject a real policy engine (RBAC, ABAC, OAuth
// scope checking) without changing any calling code.
//
// ── How a real implementation would work ─────────────────────────────────────
// The PermissionChecker in security/ would hold a session token (set by the web
// interface at login time) or derive permissions from the OS user identity. The
// before() method would call checker.is_permitted(command_type) and return
// Err(GitManagerError::Other("unauthorised")) if denied.
//
// ── Why this stub is still meaningful ────────────────────────────────────────
// Even as a no-op, the AuthMiddleware wires the correct call sequence:
// logging → validation → auth → handler. Future expansion replaces only the
// struct's implementation, not the wiring.

use async_trait::async_trait;
use gm_shared::errors::GitManagerError;
use super::Middleware;
use crate::security::PermissionChecker;
use std::sync::Arc;

/// Checks whether the current session is permitted to execute a command.
/// The v1 implementation always permits all commands.
#[derive(Debug)]
pub struct AuthMiddleware {
    checker: Arc<PermissionChecker>,
}

impl AuthMiddleware {
    pub fn new(checker: Arc<PermissionChecker>) -> Self {
        Self { checker }
    }
}

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn before(&self, command_type: &str) -> Result<(), GitManagerError> {
        self.checker.check_command(command_type)?;
        Ok(())
    }

    async fn after(&self, _command_type: &str, _elapsed_us: u64, _success: bool) {}
}