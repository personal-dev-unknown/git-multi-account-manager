//! Compile-time constants shared across all crates.
//!
//! The most important thing about this module is what it prevents: typos in
//! string literals that would only fail at runtime. By declaring event type
//! strings as `pub const EVENT_NAME: &str = "EventName"`, the compiler catches
//! any misspelling at the usage site. IDEs also provide auto-complete on
//! constants but not on inline string literals.

pub mod app;
pub mod events;