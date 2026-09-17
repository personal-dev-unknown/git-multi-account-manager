// crates/gm_adapters/src/filesystem/ffi.rs
//
// FFI contract for the Zig filesystem operations: atomic writes, permission
// setting, and path expansion. These three primitives are used throughout the
// adapter layer wherever we need to write config files, set SSH key permissions,
// or expand ~/.ssh paths.

// ─────────────────────────────────────────────────────────────────────────────
// Path expansion result
// ─────────────────────────────────────────────────────────────────────────────

/// Result of `gm_fs_expand_path`.
#[repr(C)]
pub struct FfiPathResult {
    pub path:      [u8; 4096],
    pub path_len:  usize,
    pub error_message: [u8; 512],
    pub error_len: usize,
}

impl FfiPathResult {
    pub fn zeroed() -> Self {
        // SAFETY: u8 arrays and usize only; all-zero valid.
        unsafe { std::mem::zeroed() }
    }
    pub fn path_str(&self)  -> &str { str_from_buf(&self.path, self.path_len) }
    pub fn error_str(&self) -> &str { str_from_buf(&self.error_message, self.error_len) }
}

// ─────────────────────────────────────────────────────────────────────────────
// extern "C" declarations
// ─────────────────────────────────────────────────────────────────────────────

extern "C" {
    /// Writes `data_ptr[..data_len]` to `path` atomically (temp file + rename).
    /// Returns true on success.
    pub fn gm_fs_atomic_write(
        path:     *const std::os::raw::c_char,
        data_ptr: *const u8,
        data_len: usize,
    ) -> bool;

    /// Sets the Unix permission mode on `path` to `mode` (e.g. 0o600 for SSH keys).
    /// Returns true on success.
    pub fn gm_fs_set_permissions(
        path: *const std::os::raw::c_char,
        mode: u32,
    ) -> bool;

    /// Expands a path that may start with `~` to an absolute path using $HOME.
    /// Writes the result into `result`. Returns true on success.
    pub fn gm_fs_expand_path(
        path:   *const std::os::raw::c_char,
        result: *mut FfiPathResult,
    ) -> bool;
}

fn str_from_buf(buf: &[u8], len: usize) -> &str {
    let clamped = len.min(buf.len());
    std::str::from_utf8(&buf[..clamped]).unwrap_or("")
}

pub fn to_cstring(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s)
        .unwrap_or_else(|_| panic!("Filesystem FFI: null byte in string: {s:?}"))
}