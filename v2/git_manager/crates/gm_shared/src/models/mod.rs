//! Data Transfer Objects (DTOs) for cross-crate communication.
//!
//! DTOs are the postal system of the architecture. When domain entities need
//! to cross a crate boundary — from the domain layer to an interface plugin,
//! or from an adapter back to the domain — they travel as DTOs. A DTO carries
//! the data but carries none of the domain logic, database knowledge, or
//! infrastructure concerns of the entity it represents.
//!
//! All DTOs in this module derive `Serialize` and `Deserialize` so they can
//! be sent over the web interface as JSON, stored in workflow context blobs,
//! and transmitted through Tauri's IPC system to the Svelte frontend.

pub mod account;
pub mod platform;
pub mod repository;
pub mod ssh_key;