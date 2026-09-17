//! The complete error hierarchy for Git Manager.
//!
//! The error design follows a two-level pattern. The `base` module defines
//! `GitManagerError` — the root error type that every interface plugin receives
//! when a command fails. It wraps the domain-specific error types (AccountError,
//! GitError, SshError, PluginError) using thiserror's `#[from]` attribute,
//! which makes the `?` operator convert sub-errors automatically.
//!
//! The sub-error types are kept in separate modules because they carry
//! domain-specific context: `AccountError::AliasAlreadyExists` carries the
//! alias and platform_id that caused the conflict, while `GitError::PushRejected`
//! carries the reason string from the remote. Collapsing everything into one
//! giant enum would lose this precision.

pub mod account;
pub mod base;
pub mod git;
pub mod plugin;
pub mod ssh;

pub use account::AccountError;
pub use base::GitManagerError;
pub use git::GitError;
pub use plugin::PluginError;
pub use ssh::SshError;