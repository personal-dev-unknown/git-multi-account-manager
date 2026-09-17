// crates/gm_kernel/src/contracts/provider.rs
//
// The ProviderPlugin marker trait for platform-integration plugins.
// A plugin that implements ProviderPlugin (in addition to Plugin) signals to
// the kernel that it provides a RepositoryProvider and/or AuthProvider for
// a specific Git hosting platform. The kernel uses this marker during the
// clone wizard to enumerate which platforms have active providers.

use gm_shared::models::platform::PlatformType;

/// Marker trait for plugins that implement platform repository/auth providers.
/// All provider plugins implement both Plugin AND ProviderPlugin.
pub trait ProviderPlugin: crate::contracts::plugin::Plugin {
    /// Returns the platform type this plugin handles.
    fn platform_type(&self) -> PlatformType;
}