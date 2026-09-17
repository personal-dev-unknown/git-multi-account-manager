//! gm_shared — Shared primitives for the Git Manager workspace
//!
//! This crate is the bottom of the dependency graph. Every other crate in the
//! workspace imports from here. The key discipline: this crate has zero
//! infrastructure dependencies. It contains only types, errors, constants,
//! validators, and pure utility functions.
//!
//! # Module structure
//!
//! - `models` — Serializable data transfer objects (DTOs) that cross crate
//!   boundaries without carrying domain logic or database concerns.
//! - `errors` — The complete error hierarchy for the system, using `thiserror`
//!   for compile-time-generated `Display` and `std::error::Error` implementations.
//! - `constants` — Compile-time constants: platform slugs, default values,
//!   event type strings that prevent typos via IDE auto-complete.
//! - `validation` — Pure validation functions and the `GitUrl` parser that
//!   breaks SSH and HTTPS remote URLs into their components.
//! - `utilities` — Platform helpers, path utilities, and color support detection.

// ── Lint configuration ────────────────────────────────────────────────────────
// These allow declarations apply workspace-wide only when compiling gm_shared.
// They suppress lints that are false positives for a shared-types crate:
//   module_name_repetitions: "AccountDto" in module "account" is idiomatic
//   missing_errors_doc: not every helper needs a documented error section
#![allow(clippy::module_name_repetitions)]
#![deny(missing_debug_implementations)]
#![allow(missing_docs)]

pub mod constants;
pub mod errors;
pub mod models;
pub mod utilities;
pub mod validation;

// Re-export the most commonly used types at the crate root so that callers
// can write `use gm_shared::GitManagerError` instead of
// `use gm_shared::errors::base::GitManagerError`.
pub use errors::base::GitManagerError;
pub use errors::account::AccountError;
pub use errors::git::GitError;
pub use errors::ssh::SshError;
pub use errors::plugin::PluginError;

pub use models::account::{AccountDto, AccountStatus, AuthMethod};
pub use models::platform::{PlatformDto, PlatformType};
pub use models::repository::RepositoryDto;
pub use models::ssh_key::{SshKeyDto, KeyType, TestStatus};

pub use validation::url_parser::{GitUrl, GitProtocol, parse_git_url};