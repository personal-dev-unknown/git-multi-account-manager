// crates/gm_kernel/src/command_bus/dispatcher.rs
//
// The CommandDispatcher holds the map from command TypeId to handler and
// performs the type erasure → dispatch → type restoration cycle. It is the
// inner engine that the CommandBus wraps with the middleware pipeline.
//
// ── Type erasure pattern ──────────────────────────────────────────────────────
// Rust's type system makes storing heterogeneous typed handlers in a single map
// require type erasure. The pattern used here:
//
//   1. Registration: wrap a typed `Arc<dyn CommandHandler<C>>` in a
//      `TypedHandlerWrapper<C>` which implements `ErasedCommandHandler`.
//      Store the wrapper as `Arc<dyn ErasedCommandHandler>` under TypeId::of::<C>().
//
//   2. Dispatch: box the command as `Box<dyn Any + Send + Sync>`.
//      Look up the erased handler by TypeId.
//      Call handle_erased(boxed_cmd) → Result<Box<dyn Any + Send + Sync>>.
//      Downcast the boxed result to C::Output.
//
// Both downcasts are guaranteed safe because we inserted under TypeId::of::<C>()
// and return C::Output from the handler — if the types don't match, it is a
// programming error that should panic in tests rather than fail silently at
// runtime. The downcast failure path returns a diagnostic error rather than
// panicking in production.
//
// ── validate() call site ──────────────────────────────────────────────────────
// The Command::validate() method is called inside dispatch(), AFTER type erasure
// is reversed (we have the typed command at hand). This is the correct place
// because validate() operates on the concrete command type, not on Box<dyn Any>.

use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;
use gm_shared::errors::GitManagerError;
use crate::contracts::command::{Command, CommandHandler, ErasedCommandHandler, TypedHandlerWrapper};

#[derive(Default)]
pub struct CommandDispatcher {
    handlers: DashMap<TypeId, Arc<dyn ErasedCommandHandler>>,
}

impl std::fmt::Debug for CommandDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandDispatcher")
            .field("handlers", &format_args!("[{} handler(s)]", self.handlers.len()))
            .finish()
    }
}

impl CommandDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a handler for command type C.
    /// Panics in debug builds if a handler for C is already registered,
    /// to catch accidental double-registration during development.
    pub fn register<C: Command + 'static>(
        &self,
        handler: Arc<dyn CommandHandler<C>>,
    ) {
        let wrapper = Arc::new(TypedHandlerWrapper::new(handler)) as Arc<dyn ErasedCommandHandler>;
        let key = TypeId::of::<C>();
        debug_assert!(
            !self.handlers.contains_key(&key),
            "duplicate handler registration for {}",
            std::any::type_name::<C>()
        );
        self.handlers.insert(key, wrapper);
    }

    /// Dispatches a typed command through the registered handler.
    ///
    /// Steps:
    ///   1. Validate the command (structural rules via Command::validate()).
    ///   2. Look up the handler by TypeId.
    ///   3. Type-erase the command to Box<dyn Any>.
    ///   4. Call the erased handler.
    ///   5. Downcast the erased result to C::Output.
    pub async fn dispatch<C: Command + 'static>(
        &self,
        cmd: C,
    ) -> Result<C::Output, GitManagerError> {
        // Step 1: structural validation
        cmd.validate()?;

        // Step 2: look up handler
        let type_id = TypeId::of::<C>();
        let handler = self.handlers.get(&type_id).ok_or_else(|| {
            GitManagerError::Other(format!(
                "no handler registered for command type {}",
                std::any::type_name::<C>()
            ))
        })?;

        // Step 3 + 4: type-erase and dispatch
        let boxed_cmd: Box<dyn Any + Send + Sync> = Box::new(cmd);
        let boxed_result = handler.handle_erased(boxed_cmd).await?;

        // Step 5: downcast result
        boxed_result
            .downcast::<C::Output>()
            .map(|b| *b)
            .map_err(|_| GitManagerError::Other(format!(
                "handler for {} returned an unexpected output type",
                std::any::type_name::<C>()
            )))
    }

    /// Returns true if a handler is registered for command type C.
    pub fn has_handler<C: Command + 'static>(&self) -> bool {
        self.handlers.contains_key(&TypeId::of::<C>())
    }
}