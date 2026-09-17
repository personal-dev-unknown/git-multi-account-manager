//! Terminal UI utilities: colours, tables, progress, prompts, theme.
//!
//! All functions in this module are pure display logic. They accept DTOs
//! (from gm_shared) and format them for the terminal. They never call domain
//! services and they never return errors — display failures are silently
//! degraded (e.g. falling back to plain text when colour is unavailable).

pub mod banner;
pub mod cli_progress;
pub mod colors;
pub mod dag_visualizer;
pub mod event_handler;
pub mod interactive;
pub mod progress;
pub mod prompts;
pub mod shell;
pub mod tables;
pub mod theme;
pub mod theme_engine;