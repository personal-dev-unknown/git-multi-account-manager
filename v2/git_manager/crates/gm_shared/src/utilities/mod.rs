//! Shared utility functions used across multiple crates.
//!
//! These are pure functions with no external crate dependencies — only the
//! standard library. Each module handles one specific concern: filesystem
//! path manipulation, OS platform detection, terminal color support, and
//! URL encoding. None of these functions carry domain knowledge; they are
//! infrastructure helpers that are useful to every layer of the system.

pub mod colors;
pub mod filesystem;
pub mod platform;
pub mod url_encode;