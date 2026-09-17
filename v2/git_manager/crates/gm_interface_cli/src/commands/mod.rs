//! Command handler modules — one per user-facing command group.
//!
//! Each module owns the Clap argument structs for that command group AND
//! the async handler function that receives those args plus a kernel reference.
//! The handler does input validation, calls the appropriate domain service via
//! the kernel's service registry, and delegates all terminal output to the
//! `ui` module. Business logic never lives here — this layer is pure dispatch.

pub mod account;
pub mod clone;
pub mod config;
pub mod dag;
pub mod git;
pub mod logs;
pub mod setup;
pub mod ssh;
pub mod theme;