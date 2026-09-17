// crates/gm_kernel/src/contracts/command.rs
//
// The Command and CommandHandler traits are the kernel's type contracts for the
// command-dispatch pattern. Together they implement the Command pattern from DDD:
// a Command is a request to do something (with intent and all necessary data),
// and a CommandHandler is the code that decides how to fulfil the request.
//
// ── Type-level routing ────────────────────────────────────────────────────────
// The CommandBus dispatches commands by TypeId. When you call
// `kernel.dispatch(AddAccountCommand {...})`, the bus looks up the handler
// registered for `TypeId::of::<AddAccountCommand>()`. This gives compile-time
// safety: if no handler is registered the dispatch returns an error at runtime,
// but the types are validated at compile time.
//
// ── Associated Output type ────────────────────────────────────────────────────
// Each Command declares its expected return type as `type Output`. The command
// bus uses type erasure to store and call handlers generically, then downcasts
// the erased `Box<dyn Any>` back to `C::Output` before returning to the caller.
// This means callers get a fully typed result without needing to cast manually.
//
// ── Send + Sync + 'static bounds ─────────────────────────────────────────────
// These bounds are required because:
//   - Commands travel across async task boundaries (dispatch is async)
//   - Commands are stored in `Box<dyn Any + Send + Sync>` during type erasure
//   - The 'static bound prevents commands from capturing references with
//     shorter lifetimes, which would be unsound across await points

use async_trait::async_trait;
use gm_shared::errors::GitManagerError;

/// A command represents a user's intent to change system state.
///
/// Commands are the inbound side of the hexagonal architecture: they arrive from
/// interface plugins (CLI, Web, Desktop) and are routed through the command bus
/// to the appropriate domain service. Every command struct in gm_ports/inbound/
/// should implement this trait.
pub trait Command: Send + Sync + 'static {
    /// The type produced on successful handling. Must be Send + Sync + 'static
    /// because it crosses async boundaries on its way back to the caller.
    type Output: Send + Sync + 'static;

    /// Validates the command's structural integrity before the domain sees it.
    /// This is the command bus's validation middleware call site. Validate only
    /// format constraints here (empty fields, out-of-range values). Business rule
    /// violations are returned as errors from the domain service handler.
    fn validate(&self) -> Result<(), GitManagerError>;
}

/// The trait that handles a specific command type.
///
/// Command handlers are the bridge between the inbound port (the command struct)
/// and the domain service. They live in the kernel's handler registry, not in
/// the domain. A typical handler:
///   1. Resolves domain services from the service registry
///   2. Calls the service method with the command's data
///   3. Publishes any domain events returned by the service
///   4. Returns the typed output to the command bus caller
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, cmd: C) -> Result<C::Output, GitManagerError>;
}

/// A type-erased version of CommandHandler that the command bus's DashMap can
/// store without needing to know the concrete command type at storage time.
///
/// The command bus erases the type at registration and restores it at dispatch.
/// This internal trait is not part of the public API — plugin authors implement
/// `CommandHandler<C>`, not `ErasedCommandHandler`.
#[async_trait]
pub(crate) trait ErasedCommandHandler: Send + Sync {
    /// Handles the command, receiving and returning type-erased `Box<dyn Any>` values.
    /// The command bus downcast both ends before calling and after returning.
    async fn handle_erased(
        &self,
        cmd: Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, GitManagerError>;
}

/// Wraps a typed `CommandHandler<C>` in the erased interface the command bus
/// stores. This is the only place that performs the unsafe-free type erasure
/// and restoration using `Any::downcast`.
pub(crate) struct TypedHandlerWrapper<C: Command> {
    inner: std::sync::Arc<dyn CommandHandler<C>>,
}

impl<C: Command> TypedHandlerWrapper<C> {
    pub fn new(handler: std::sync::Arc<dyn CommandHandler<C>>) -> Self {
        Self { inner: handler }
    }
}

#[async_trait]
impl<C: Command + 'static> ErasedCommandHandler for TypedHandlerWrapper<C>
where
    C::Output: 'static,
{
    async fn handle_erased(
        &self,
        cmd: Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, GitManagerError> {
        let typed_cmd = cmd
            .downcast::<C>()
            .map_err(|_| GitManagerError::Other(
                format!("command downcast failed for type {}", std::any::type_name::<C>())
            ))?;
        let output = self.inner.handle(*typed_cmd).await?;
        Ok(Box::new(output) as Box<dyn std::any::Any + Send + Sync>)
    }
}