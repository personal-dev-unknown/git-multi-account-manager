use std::path::Path;
use gm_domain::git::ports::DiskSpaceChecker;
use super::ffi;

/// Checks available disk space using the Zig native statvfs FFI call.
/// Uses the POSIX statvfs syscall (not `df` subprocess).
#[derive(Debug, Default)]
pub struct ZigDiskSpaceChecker;

impl DiskSpaceChecker for ZigDiskSpaceChecker {
    fn available_bytes(&self, path: &Path) -> Result<u64, String> {
        let c_path = ffi::to_cstring(&path.to_string_lossy());
        // SAFETY: c_path is a valid null-terminated C string.
        // The Zig function reads it, performs a statvfs syscall, and returns
        // immediately. No pointers are retained after the call returns.
        let bytes = unsafe { ffi::gm_fs_available_disk_space(c_path.as_ptr()) };
        if bytes == 0 {
            return Err(format!(
                "failed to query disk space for '{}': path may not exist or statvfs failed",
                path.display()
            ));
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disk_space_works_for_root_via_zig() {
        let checker = ZigDiskSpaceChecker;
        let bytes = checker.available_bytes(Path::new("/")).unwrap();
        assert!(bytes > 0, "root should have some disk space");
    }
}
