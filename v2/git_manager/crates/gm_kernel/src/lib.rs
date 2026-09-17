//! gm_kernel — the microkernel runtime.
//!
//! This crate is the nerve centre of the entire system. It provides the stable
//! infrastructure that every other component depends on for communication,
//! discovery, and lifecycle management. It is deliberately minimal: its only
//! responsibility is *wiring*, not business logic.
//!
//! # Subsystems (internal dependency order)
//!
//! **contracts** → The Rust traits that every plugin, command, and event must
//! implement. Contracts are pure interfaces — no logic, no state.
//!
//! **service_registry** → A type-indexed `Arc<DashMap>` that plugins use to
//! advertise services and the kernel uses to resolve dependencies at runtime.
//!
//! **event_bus** → A typed publish-subscribe dispatcher. Domain services call
//! `kernel.publish(event_type, payload)` after completing an operation. The
//! event bus serialises the payload, wraps it in a `KernelEvent` envelope,
//! and dispatches it to all registered handlers asynchronously.
//!
//! **command_bus** → Routes inbound commands from interface plugins through a
//! middleware chain (logging, validation, auth) to the registered handler.
//!
//! **plugin_system** → Handles the full plugin lifecycle: discovery, topological
//! sort by dependency graph, version compatibility checking, `on_load()` /
//! `on_unload()` calls, and dynamic `.so` loading via `libloading`.
//!
//! **workflow_engine** → Executes multi-step named workflows (e.g.
//! "clone_and_configure") with persistent context, per-step timeouts, retry
//! policies, and rollback on failure.
//!
//! **lifecycle** → Ordered startup/shutdown hooks for subsystems that need to
//! prepare state before the first command is processed (database migration) or
//! flush state before the process exits (connection pool drain).
//!
//! **security** → The `CredentialVault` (AES-256-GCM with machine-derived key)
//! and a no-op `PermissionChecker` stub ready for multi-user expansion.
//!
//! **kernel** → The `Kernel` struct that holds all subsystems behind a single
//! `Arc`. The `dispatch`, `publish`, and `get` convenience methods on `Kernel`
//! eliminate the need for callers to unwrap Arc references everywhere.
//!
//! **bootstrap** → The single `bootstrap(config, static_plugins)` async
//! function that initialises every subsystem in the correct order and returns
//! a ready-to-use `Arc<Kernel>`.

#![allow(clippy::module_name_repetitions)]
#![deny(missing_debug_implementations)]

pub mod bootstrap;
pub mod command_bus;
pub mod contracts;
pub mod config_loader;
pub mod env_loader;
pub mod event_bus;
pub mod kernel;
pub mod lifecycle;
pub mod plugin_system;
pub mod security;
pub mod service_registry;
pub mod workflow_engine;

// Re-export the most commonly used public types so callers can write
// `use gm_kernel::{Kernel, bootstrap}` instead of reaching into submodules.
pub use bootstrap::{bootstrap, AppConfig};
pub use contracts::plugin::Plugin;
pub use kernel::Kernel;
