// crates/gm_kernel/src/kernel.rs
//
// The Kernel is the central runtime object that every interface plugin and
// command handler holds. Its role is purely one of coordination: it provides
// type-safe, ergonomic access to every subsystem without exposing the raw Arc
// handles that each subsystem would otherwise require callers to unwrap directly.
//
// ── Why a facade over the subsystems ─────────────────────────────────────────
// Interface plugins (CLI, Web, Desktop) need to dispatch commands, publish
// events, resolve services, and run workflows. If each plugin held individual
// Arcs for each subsystem, the plugin API surface would be:
//
//   fn run(&self, command_bus: Arc<CommandBus>, event_bus: Arc<EventBus>,
//          service_registry: Arc<ServiceRegistry>, ...) { ... }
//
// — which is fragile because adding a new subsystem would require changing
// every InterfacePlugin::run() signature. The Kernel facade solves this:
// plugins hold a single Arc<Kernel> and call self.dispatch(), self.publish(), etc.
// New subsystems are added to Kernel without touching plugin signatures.
//
// ── Thread safety ─────────────────────────────────────────────────────────────
// All methods on Kernel are either `&self` (immutable reference) calling into
// subsystems that use interior mutability via DashMap, AtomicU64, or RwLock,
// or `async` methods that spawn tasks which hold their own Arc references.
// Arc<Kernel> is therefore `Send + Sync` and can be freely shared across
// async task boundaries and between threads.
//
// ── What Kernel deliberately does NOT contain ─────────────────────────────────
// - Database connection pools (those are in the adapter layer, accessed via the
//   service registry or passed directly to domain services).
// - Domain service instances (those are in the service registry; callers fetch
//   them via kernel.get::<MyService>() rather than hard-coding field accesses).
// - HTTP servers or CLI parsers (those are in interface plugins).
// - Any mutable shared state without synchronisation primitives.

use std::any::Any;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    command_bus::CommandBus,
    contracts::command::Command,
    contracts::event::DomainEvent,
    event_bus::EventBus,
    lifecycle::LifecycleManager,
    plugin_system::PluginRegistry,
    security::CredentialVault,
    service_registry::ServiceRegistry,
    workflow_engine::WorkflowRunner,
};
use gm_shared::errors::GitManagerError;

/// The kernel runtime — held as `Arc<Kernel>` by every component that needs
/// to communicate with the rest of the system.
pub struct Kernel {
    /// The registry of all loaded plugins and their metadata.
    pub plugin_registry:  Arc<PluginRegistry>,
    /// Routes inbound commands through middleware to the registered handler.
    pub command_bus:      Arc<CommandBus>,
    /// Dispatches domain events to all registered subscribers.
    pub event_bus:        Arc<EventBus>,
    /// Executes named multi-step workflows with retry and rollback.
    pub workflow_runner:  Arc<WorkflowRunner>,
    /// Type-indexed dependency injection container for plugin-registered services.
    pub service_registry: Arc<ServiceRegistry>,
    /// Ordered startup and shutdown hooks.
    pub lifecycle:        Arc<LifecycleManager>,
    /// AES-256-GCM encryption for secrets persisted to the database.
    pub credential_vault: Arc<CredentialVault>,
}

impl std::fmt::Debug for Kernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Kernel {{ plugins_loaded: {}, services_registered: {} }}",
            self.plugin_registry.count(),
            self.service_registry.len(),
        )
    }
}

impl Kernel {
    // ── Command dispatch ──────────────────────────────────────────────────────

    /// Dispatches a command through the middleware pipeline (logging, validation,
    /// auth) to the registered handler and returns the typed output.
    ///
    /// The command type `C` must implement `Command` and have a registered handler
    /// (registered via `kernel.command_bus.register::<C>(handler)`). If no handler
    /// is registered, dispatch returns `Err(GitManagerError::Other(...))`.
    pub async fn dispatch<C: Command + 'static>(&self, cmd: C) -> Result<C::Output, GitManagerError> {
        self.command_bus.dispatch(cmd).await
    }

    // ── Event publication ─────────────────────────────────────────────────────

    /// Publishes a domain event to all registered subscribers.
    ///
    /// The event is serialised into a `KernelEvent` envelope with a fresh UUID,
    /// the current timestamp, and a monotonically increasing sequence number.
    /// All global handlers (audit) and type-specific handlers receive the event
    /// asynchronously. Errors from individual handlers are logged, not propagated.
    pub async fn publish<E: DomainEvent>(&self, event: E) {
        self.event_bus.publish(event, None).await;
    }

    /// Same as `publish` but attaches a `correlation_id` that links this event
    /// to a higher-level user action. All events produced by a single command
    /// should share the same correlation ID for audit trail reconstruction.
    pub async fn publish_correlated<E: DomainEvent>(&self, event: E, correlation_id: Uuid) {
        self.event_bus.publish(event, Some(correlation_id)).await;
    }

    // ── Service registry ──────────────────────────────────────────────────────

    /// Retrieves a service by its concrete type `T` from the service registry.
    ///
    /// Returns `Some(Arc<T>)` if the service is registered, `None` if it is not.
    /// Services are registered by plugins during their `on_load()` call.
    ///
    /// Generic constraint: `T: Any + Send + Sync` is required because services
    /// are stored as type-erased `Arc<dyn Any + Send + Sync>` and downcast here.
    pub fn get<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.service_registry.get::<T>()
    }

    /// Registers a service in the service registry.
    ///
    /// Typically called from plugin `on_load()` implementations. Overwrites any
    /// previously registered service of the same type — last registration wins.
    pub fn register<T: Any + Send + Sync>(&self, service: Arc<T>) {
        self.service_registry.register::<T>(service);
    }

    // ── Workflow execution ────────────────────────────────────────────────────

    /// Runs a named workflow with the provided initial context, optionally
    /// linking all produced events under a single correlation ID.
    ///
    /// Returns the accumulated context after all steps complete, or an error
    /// if any step exhausts its retries and the workflow's failure policy is Abort.
    pub async fn run_workflow(
        &self,
        name:           &str,
        initial_context: serde_json::Value,
        correlation_id:  Option<Uuid>,
    ) -> Result<serde_json::Value, GitManagerError> {
        self.workflow_runner.run(name, initial_context, correlation_id).await
    }

    // ── Credential vault ──────────────────────────────────────────────────────

    /// Returns a reference to the credential vault for encrypting and decrypting
    /// secrets before they cross the domain→adapter boundary.
    pub fn vault(&self) -> &CredentialVault {
        &self.credential_vault
    }

    // ── Plugin registry ───────────────────────────────────────────────────────

    /// Returns the number of successfully loaded plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugin_registry.count()
    }

    /// Returns the names of all loaded plugins, sorted alphabetically.
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugin_registry.names()
    }

    // ── Graceful shutdown ─────────────────────────────────────────────────────

    /// Runs all registered shutdown hooks in reverse priority order.
    ///
    /// Call this before the process exits to give subsystems a chance to flush
    /// buffers, close connections, and release locks. Hook errors are logged
    /// but do not prevent other hooks from running.
    pub async fn shutdown(&self) {
        tracing::info!("kernel shutdown initiated");
        self.lifecycle.shutdown().await;
        tracing::info!("kernel shutdown complete");
    }
}