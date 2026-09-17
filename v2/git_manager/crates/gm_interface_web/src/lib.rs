//! gm_interface_web — the Axum HTTP interface plugin.
pub mod app;
pub mod banner;
pub mod layout;
pub mod plugin;
pub mod routes;
pub mod services;
pub mod session;
pub mod sse_bus;
pub mod state;

pub use plugin::WebPlugin;
pub fn create_plugin() -> WebPlugin { WebPlugin::new() }