pub mod auth_provider;
pub mod client;
pub mod plugin;
pub mod repository_provider;

pub use auth_provider::CustomAuthProvider;
pub use plugin::CustomPlugin;
pub use repository_provider::CustomRepositoryProvider;

pub fn create_plugin() -> CustomPlugin {
    CustomPlugin::new()
}
