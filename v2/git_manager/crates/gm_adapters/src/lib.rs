//! gm_adapters — the infrastructure adapter layer.
//!
//! This crate is the meeting point of two worlds: the pure Rust domain/kernel
//! above and the real-world infrastructure below (Zig native layer, MySQL,
//! SQLite, OS keychain). It contains no business logic. Its job is translation:
//! converting between the domain's abstract port interfaces and the concrete
//! calls that actually move data or spawn processes.
//!
//! # Module map
//!
//! **ssh** — Safe Rust wrappers around the Zig SSH operations (keygen, agent,
//! connection test, config write). The `ffi.rs` sub-module holds the raw
//! `extern "C"` declarations and `#[repr(C)]` structs. The `zig_ssh_provider.rs`
//! sub-module wraps those calls behind the `SshProvider` and `SshOperations` traits.
//!
//! **git** — Wraps the Zig git subprocess executor behind `GitExecutor`.
//!
//! **filesystem** — Wraps atomic write, permission setting, and path expansion.
//!
//! **platform** — Wraps the OS credential store (libsecret, Keychain, WinCred).
//!
//! **persistence** — SQLx-based repositories for MySQL (production) and SQLite
//! (development / testing). Includes SQL migration files under `migrations/`.
//!
//! **cache** — In-memory TTL cache for repository list results and SSH key
//! queries that would otherwise hit the database on every command invocation.
//!
//! **logging** — A structured logger adapter that bridges the `tracing` spans
//! produced throughout the system into a format suitable for file output.

#![allow(clippy::module_name_repetitions)]

pub mod cache;
pub mod filesystem;
pub mod git;
pub mod logging;
pub mod persistence;
pub mod platform;
pub mod ssh;