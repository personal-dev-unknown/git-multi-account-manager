//! gm_domain — Pure business logic for Git Manager.
//!
//! This crate is the domain layer of the hexagonal architecture. It contains
//! every business rule that defines what Git Manager does, expressed as Rust
//! structs, traits, and methods — with no awareness of databases, HTTP, SSH
//! binaries, or any other infrastructure.
//!
//! The Rust compiler enforces this isolation automatically: `gm_domain/Cargo.toml`
//! does not list `sqlx`, `reqwest`, `tokio`, or any infrastructure crate. Any
//! attempt to call a SQL query from within this crate produces a compile error.
//!
//! # Sub-domains
//!
//! The domain is divided into six bounded contexts, each owning its own
//! entities, value objects, ports, services, and events.
//!
//! - **accounts** — the central concept: a Git hosting account belonging to a user.
//! - **repositories** — Git repositories, both discovered and cloned locally.
//! - **ssh** — SSH key pairs, host configurations, and connection state.
//! - **git** — the git execution model: branches, commits, executor port.
//! - **sync** — the staging → commit → push/pull cycle and its session tracking.
//! - **configuration** — application-level settings persisted across restarts.

#![allow(clippy::module_name_repetitions)]
#![deny(missing_debug_implementations)]

pub mod accounts;
pub mod clone;
pub mod configuration;
pub mod git;
pub mod repositories;
pub mod ssh;
pub mod sync;