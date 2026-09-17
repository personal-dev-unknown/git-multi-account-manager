pub mod auth;
pub mod auth_provider;
pub mod client;
pub mod plugin;
pub mod repository_provider;

pub use auth_provider::SourceForgeAuthProvider;
pub use plugin::SourceForgePlugin;
pub use repository_provider::SourceForgeRepositoryProvider;

pub fn create_plugin() -> SourceForgePlugin {
    SourceForgePlugin::new()
}
