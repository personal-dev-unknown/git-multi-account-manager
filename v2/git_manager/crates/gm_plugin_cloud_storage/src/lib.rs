pub mod auth_provider;
pub mod plugin;
pub mod repository_provider;
pub mod url_utils;

pub use auth_provider::CloudStorageAuthProvider;
pub use plugin::CloudStoragePlugin;
pub use repository_provider::CloudStorageRepositoryProvider;

pub fn create_plugin() -> CloudStoragePlugin {
    CloudStoragePlugin::new()
}
