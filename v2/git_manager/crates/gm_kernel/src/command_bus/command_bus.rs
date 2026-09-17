// crates/gm_kernel/src/command_bus/command_bus.rs
//
// The CommandBus is the public API through which interface plugins dispatch
// user intent into the domain. It wraps the CommandDispatcher with a
// middleware pipeline that runs before and after every dispatch.
//
// ── Middleware execution order ────────────────────────────────────────────────
// Middlewares are stored in a Vec and called in index order for before(),
// and in reverse index order for after(). This means middleware[0] is the
// outermost wrapper — its before() runs first and its after() runs last.
// The registered order is: [LoggingMiddleware, ValidationMiddleware, AuthMiddleware].
// The execution trace for a successful command is therefore:
//
//   LoggingMiddleware::before()        → emits tracing::info!("command dispatched")
//   ValidationMiddleware::before()     → no-op in v1
//   AuthMiddleware::before()           → calls PermissionChecker::check_command()
//   CommandDispatcher::dispatch(cmd)   → validate() + handler
//   AuthMiddleware::after()            → no-op
//   ValidationMiddleware::after()      → no-op
//   LoggingMiddleware::after()         → emits tracing::info!("command completed")
//
// ── Timing ────────────────────────────────────────────────────────────────────
// The elapsed time passed to after() is measured from the start of before()
// calls to the end of the handler, not including after() itself. This gives
// a useful "time to response" metric that includes serialisation and validation.
//
// ── Why `impl Command` not `dyn Command` ─────────────────────────────────────
// The dispatch() method is generic over C: Command. This gives the compiler
// full type information to optimise the call path (no vtable lookup for the
// handler) and allows C::Output to be a concrete type at the call site. The
// caller does not need to box or cast the result.

use std::sync::Arc;
use std::time::Instant;
use gm_shared::errors::GitManagerError;
use crate::contracts::command::{Command, CommandHandler};
use crate::command_bus::dispatcher::CommandDispatcher;
use crate::command_bus::middleware::Middleware;

/// The public command dispatch facade with middleware support.
pub struct CommandBus {
    dispatcher:  CommandDispatcher,
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl std::fmt::Debug for CommandBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandBus")
            .field("dispatcher", &self.dispatcher)
            .field("middlewares", &format_args!("[{} middleware(s)]", self.middlewares.len()))
            .finish()
    }
}

impl CommandBus {
    pub fn new(middlewares: Vec<Arc<dyn Middleware>>) -> Self {
        Self {
            dispatcher: CommandDispatcher::new(),
            middlewares,
        }
    }

    /// Registers a handler for command type C.
    pub fn register<C: Command + 'static>(&self, handler: Arc<dyn CommandHandler<C>>) {
        self.dispatcher.register::<C>(handler);
    }

    /// Dispatches a command through the middleware chain and then to the handler.
    ///
    /// The middleware before() calls run in order. If any returns Err, dispatch
    /// is aborted and the error is returned immediately. On success, the handler
    /// runs. After the handler returns (success or failure), after() is called on
    /// all middlewares in reverse order.
    pub async fn dispatch<C: Command + 'static>(
        &self,
        cmd: C,
    ) -> Result<C::Output, GitManagerError> {
        let command_type = std::any::type_name::<C>();
        let start        = Instant::now();

        // Run before() hooks in order. Abort on first failure.
        for mw in &self.middlewares {
            if let Err(e) = mw.before(command_type).await {
                // Still run after() in reverse for cleanup
                let elapsed = start.elapsed().as_micros() as u64;
                for mw_after in self.middlewares.iter().rev() {
                    mw_after.after(command_type, elapsed, false).await;
                }
                return Err(e);
            }
        }

        // Run the actual dispatch
        let result = self.dispatcher.dispatch(cmd).await;

        let elapsed = start.elapsed().as_micros() as u64;
        let success = result.is_ok();

        // Run after() hooks in reverse order
        for mw in self.middlewares.iter().rev() {
            mw.after(command_type, elapsed, success).await;
        }

        result
    }
}