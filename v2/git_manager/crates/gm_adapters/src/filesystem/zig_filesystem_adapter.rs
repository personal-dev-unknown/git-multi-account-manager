// crates/gm_adapters/src/filesystem/zig_filesystem_adapter.rs
//
// Safe wrappers around the Zig filesystem primitives. These three functions
// are used throughout the adapter layer rather than Rust's std::fs where
// atomic write semantics are required (SSH keys, SSH config, application config).

use std::path::{Path, PathBuf};
use gm_shared::errors::GitManagerError;
use super::ffi;

/// Writes `data` to `path` atomically using Zig's temp-file + rename pattern.
///
/// If the write fails midway, the original file at `path` is unchanged. The
/// temp file is created in the same directory as `path` so that the rename
/// is guaranteed to be atomic within a single filesystem (no cross-device move).
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<(), GitManagerError> {
    let c_path = ffi::to_cstring(path.to_str().ok_or_else(|| {
        GitManagerError::Other("path contains non-UTF-8 bytes".to_string())
    })?);

    // SAFETY: c_path is a valid null-terminated string. data_ptr/data_len
    // form a valid slice reference. The Zig function reads but does not
    // retain a reference to the data after return.
    let ok = unsafe {
        ffi::gm_fs_atomic_write(c_path.as_ptr(), data.as_ptr(), data.len())
    };

    if ok {
        Ok(())
    } else {
        Err(GitManagerError::Other(
            format!("atomic write failed: {}", path.display())
        ))
    }
}

/// Sets Unix permissions on `path` to `mode` (e.g. `0o600` for SSH private keys).
///
/// On non-Unix platforms this is a no-op returning Ok(()); the Zig layer
/// handles platform detection internally.
pub fn set_permissions(path: &Path, mode: u32) -> Result<(), GitManagerError> {
    let c_path = ffi::to_cstring(path.to_str().ok_or_else(|| {
        GitManagerError::Other("path contains non-UTF-8 bytes".to_string())
    })?);

    // SAFETY: c_path is valid null-terminated. mode is a plain u32.
    let ok = unsafe { ffi::gm_fs_set_permissions(c_path.as_ptr(), mode) };

    if ok {
        Ok(())
    } else {
        Err(GitManagerError::Other(
            format!("set_permissions({mode:o}) failed: {}", path.display())
        ))
    }
}

/// Expands a path starting with `~` to an absolute path using the $HOME variable.
///
/// Returns the input path unchanged if it does not start with `~`.
pub fn expand_path(path: &str) -> Result<PathBuf, GitManagerError> {
    let c_path = ffi::to_cstring(path);
    let mut result = ffi::FfiPathResult::zeroed();

    // SAFETY: c_path is valid null-terminated. result is a valid stack allocation.
    let ok = unsafe {
        ffi::gm_fs_expand_path(c_path.as_ptr(), &mut result as *mut ffi::FfiPathResult)
    };

    if ok {
        Ok(PathBuf::from(result.path_str()))
    } else {
        Err(GitManagerError::Other(
            format!("expand_path failed for '{path}': {}", result.error_str())
        ))
    }
}