pub mod auth_provider;
pub mod plugin;
pub mod repository_provider;
pub mod url_utils;

pub use auth_provider::LocalPathAuthProvider;
pub use plugin::LocalPathPlugin;
pub use repository_provider::LocalPathRepositoryProvider;

pub fn create_plugin() -> LocalPathPlugin {
    LocalPathPlugin::new()
}
