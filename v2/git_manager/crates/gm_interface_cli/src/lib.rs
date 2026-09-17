//! gm_interface_cli — the terminal interface plugin.
//!
//! This crate contains the complete CLI interface for Git Multi-Account Manager.
//! It is structured as a plugin that implements the kernel's `InterfacePlugin`
//! trait, meaning the binary entry point creates it, passes it to bootstrap,
//! and the kernel calls `run()` after all provider plugins have loaded.
//!
//! # Module layout
//!
//! The `plugin` module holds the `CliPlugin` struct and its trait implementations.
//! The `app` module holds the Clap argument structs and the top-level router that
//! dispatches to individual command handlers. The `commands` module contains one
//! sub-module per user-facing command group (account, ssh, clone, git, config,
//! logs). The `ui` module contains all terminal rendering utilities (tables,
//! progress bars, prompts, colour theme) — all pure display logic, no business
//! rules, no service calls.

#![allow(clippy::module_name_repetitions)]

pub mod app;
pub mod commands;
pub mod plugin;
pub mod services;   // ← application service facade; retrieved via CliServicesHandle
pub mod ui;

pub use plugin::CliPlugin;
pub use services::{CliServices, CliServicesHandle};

/// Creates a new CLI plugin instance ready to be passed to bootstrap.
pub fn create_plugin() -> CliPlugin {
    CliPlugin::new()
}