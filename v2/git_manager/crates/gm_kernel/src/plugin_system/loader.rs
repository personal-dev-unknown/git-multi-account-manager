// crates/gm_kernel/src/plugin_system/loader.rs
//
// The PluginLoader handles both static plugins (compiled into the binary,
// passed in as Box<dyn Plugin>) and dynamic plugins (loaded from .so/.dylib
// files at runtime via libloading).
//
// ── Static loading ────────────────────────────────────────────────────────────
// Static plugins are the primary mechanism for the built-in provider and
// interface plugins. They are compiled into the same binary as the kernel,
// so they share the same Rust vtable layout — no ABI concerns. The bootstrap
// function receives them as `Vec<Box<dyn Plugin>>` from the binary entry point
// (apps/cli/main.rs), which means the kernel itself has no compile-time
// dependency on specific plugin crates.
//
// ── Dynamic loading ───────────────────────────────────────────────────────────
// Dynamic plugins are .so files in a user-specified directory. Each .so must
// export two C ABI functions:
//
//   extern "C" fn _gm_plugin_create() -> *mut std::ffi::c_void
//   extern "C" fn _gm_plugin_destroy(ptr: *mut std::ffi::c_void)
//
// The create function allocates a `Box<dyn Plugin>` on the heap, double-boxes it,
// and returns a raw void pointer. The destroy function accepts that pointer,
// casts it back to `*mut Box<dyn Plugin>`, and drops it. This double-boxing
// is necessary because `dyn Plugin` is a fat pointer (data + vtable) and cannot
// itself be converted to a thin `*mut c_void`.
//
// ── Safety of dynamic loading ─────────────────────────────────────────────────
// The `unsafe` code here is unavoidable — libloading calls are inherently unsafe
// because the .so's memory layout must match the kernel's expectations. The safety
// invariants are:
//   1. The .so was compiled with the same Rust toolchain version as the kernel.
//   2. The .so implements the `Plugin` trait correctly (this is the author's
//      responsibility, not something the kernel can verify at load time).
//   3. The `_gm_plugin_create` function allocates with the same allocator as the
//      kernel. In practice, both use the global allocator (jemalloc or system).
//   4. The `_gm_plugin_destroy` function is called exactly once, during unload.
//
// These constraints mean that dynamic plugins in the wild must ship with version
// metadata and the kernel must reject plugins compiled against incompatible ABI
// versions (checked via PluginValidator::check_version).

use std::path::Path;
use std::sync::Arc;
use gm_shared::errors::PluginError;
use crate::contracts::plugin::Plugin;
use crate::plugin_system::registry::PluginRegistry;
use crate::plugin_system::validator::PluginValidator;
use crate::service_registry::ServiceRegistry;

#[derive(Debug)]
pub struct PluginLoader {
    plugin_registry:  Arc<PluginRegistry>,
    service_registry: Arc<ServiceRegistry>,
}

impl PluginLoader {
    pub fn new(
        plugin_registry:  Arc<PluginRegistry>,
        service_registry: Arc<ServiceRegistry>,
    ) -> Self {
        Self { plugin_registry, service_registry }
    }

    /// Loads a static plugin (compiled into the binary).
    ///
    /// Steps:
    ///   1. Check version compatibility.
    ///   2. Verify the plugin is not already registered.
    ///   3. Call on_load() with the service registry reference.
    ///   4. Insert the plugin into the plugin registry.
    pub fn load_static(&self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let metadata = plugin.metadata();

        // tracing::info!(
        //     plugin  = %metadata.name,
        //     version = %metadata.version,
        //     "loading static plugin"
        // );

        // Version check
        PluginValidator::check_version(metadata)?;

        // Duplicate check
        if self.plugin_registry.contains(&metadata.name) {
            return Err(PluginError::Duplicate { name: metadata.name.clone() });
        }

        // Dependency check — all declared dependencies must be loaded already.
        for dep in &metadata.dependencies {
            if !self.plugin_registry.contains(dep) {
                return Err(PluginError::DependencyMissing {
                    plugin:   metadata.name.clone(),
                    required: dep.clone(),
                });
            }
        }

        // Lifecycle hook
        plugin.on_load(Arc::clone(&self.service_registry))
            .map_err(|e| PluginError::RegistrationFailed {
                name:   metadata.name.clone(),
                reason: e.to_string(),
            })?;

        // Clone the name before moving plugin into Arc (metadata borrows plugin).
        let plugin_name = metadata.name.clone();
        let _ = metadata;

        // Register in the plugin registry
        self.plugin_registry.insert(Arc::from(plugin))
            .map_err(|e| {
                tracing::error!(plugin = %plugin_name, error = %e, "plugin registry insert failed");
                e
            })?;

        // tracing::info!(plugin = %plugin_name, "static plugin loaded successfully");
        Ok(())
    }

    /// Scans `dir` for .so/.dylib files and loads each one as a dynamic plugin.
    ///
    /// Load failures for individual plugins are logged and skipped rather than
    /// aborting the boot sequence — one broken external plugin should not prevent
    /// the rest of the system from starting. The caller can check the plugin
    /// registry count after this call to verify expected plugins loaded.
    pub fn load_directory(&self, dir: &str) -> Result<(), PluginError> {
        let path = Path::new(dir);

        let extension = if cfg!(target_os = "windows") { "dll" }
                        else if cfg!(target_os = "macos") { "dylib" }
                        else { "so" };

        let entries = std::fs::read_dir(path).map_err(|e| PluginError::LoadFailed {
            name:   dir.to_string(),
            reason: e.to_string(),
        })?;

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.extension().and_then(|e| e.to_str()) == Some(extension) {
                let name = entry_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                if let Err(e) = self.load_dynamic_plugin(&entry_path, &name) {
                    tracing::error!(
                        plugin = %name,
                        path   = %entry_path.display(),
                        error  = %e,
                        "dynamic plugin load failed — skipping"
                    );
                }
            }
        }

        Ok(())
    }

    /// Loads a single dynamic plugin from a .so/.dylib file.
    fn load_dynamic_plugin(&self, path: &Path, hint_name: &str) -> Result<(), PluginError> {
        tracing::info!(path = %path.display(), "loading dynamic plugin");

        // SAFETY: We are loading an arbitrary .so file. The safety requirements
        // are documented in the module-level comment above. In production, .so
        // files in the plugin directory should be audited or signed.
        let lib = unsafe {
            libloading::Library::new(path).map_err(|e| PluginError::LoadFailed {
                name:   hint_name.to_string(),
                reason: e.to_string(),
            })?
        };

        // The .so must export a C ABI factory function.
        // SAFETY: We trust that _gm_plugin_create follows the contract documented
        // in the module comment. The void pointer it returns is a double-boxed
        // Box<dyn Plugin> allocated by the plugin's crate.
        let create_fn: libloading::Symbol<unsafe extern "C" fn() -> *mut std::ffi::c_void> =
            unsafe {
                lib.get(b"_gm_plugin_create\0").map_err(|e| PluginError::LoadFailed {
                    name:   hint_name.to_string(),
                    reason: format!("missing _gm_plugin_create symbol: {e}"),
                })?
            };

        let raw_ptr = unsafe { create_fn() };
        if raw_ptr.is_null() {
            return Err(PluginError::LoadFailed {
                name:   hint_name.to_string(),
                reason: "_gm_plugin_create returned null".to_string(),
            });
        }

        // SAFETY: We trust that the pointer is a valid *mut Box<dyn Plugin> as
        // required by the plugin contract. We take ownership by converting to Box
        // and immediately dereferencing. The .so stays in scope for the process
        // lifetime (via Arc<Library> if we implement unloading), ensuring the
        // vtable pointers inside the plugin remain valid.
        let plugin: Box<dyn Plugin> = unsafe {
            let double_box = Box::from_raw(raw_ptr as *mut Box<dyn Plugin>);
            *double_box
        };

        // The Library must not be dropped — dropping it unmaps the .so from
        // memory, invalidating all vtable pointers in the plugin. Leak the
        // library handle intentionally. If dynamic unloading is added in the
        // future, the Library handle must be kept alive until after on_unload()
        // returns and the plugin Arc is dropped.
        std::mem::forget(lib);

        self.load_static(plugin)
    }
}