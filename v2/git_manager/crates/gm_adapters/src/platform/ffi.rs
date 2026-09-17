// crates/gm_adapters/src/platform/ffi.rs
// FFI contract for the OS credential store (GNOME libsecret, macOS Keychain, WinCred).

/// Result of `gm_platform_retrieve_secret`.
/// Must match `zig_native/src/platform/types.zig :: SecretResult` EXACTLY.
///
/// WARNING: Do NOT add a `found` field here — Zig's SecretResult does not have
/// one. Any diverging field causes struct misalignment and silent data corruption.
/// Use `secret_len > 0` on the Rust side to detect whether a credential was found.
#[repr(C)]
pub struct FfiSecretResult {
    pub secret:        [u8; 4096],
    pub secret_len:    usize,
    pub error_message: [u8; 512],
    pub error_len:     usize,
}

impl FfiSecretResult {
    pub fn zeroed() -> Self {
        // SAFETY: u8 arrays + usize; all-zero bytes are valid.
        unsafe { std::mem::zeroed() }
    }

    pub fn secret_bytes(&self) -> &[u8] {
        &self.secret[..self.secret_len.min(self.secret.len())]
    }

    pub fn error_str(&self) -> &str {
        let len = self.error_len.min(self.error_message.len());
        std::str::from_utf8(&self.error_message[..len]).unwrap_or("unknown error")
    }
}

extern "C" {
    /// Stores `secret_ptr[..secret_len]` in the OS keychain under the
    /// (label, username) composite key. Overwrites any existing entry.
    pub fn gm_platform_store_secret(
        label:      *const std::os::raw::c_char,
        username:   *const std::os::raw::c_char,
        secret_ptr: *const u8,
        secret_len: usize,
    ) -> bool;

    /// Retrieves the secret stored under (label, username).
    /// Sets result.secret_len > 0 (not an error) if a credential was found.
    /// Returns false on actual errors; sets result.error_message.
    pub fn gm_platform_retrieve_secret(
        label:    *const std::os::raw::c_char,
        username: *const std::os::raw::c_char,
        result:   *mut FfiSecretResult,
    ) -> bool;

    /// Deletes the credential for (label, username). Returns true even if
    /// no credential existed (idempotent — safe to call during cleanup).
    pub fn gm_platform_delete_secret(
        label:    *const std::os::raw::c_char,
        username: *const std::os::raw::c_char,
    ) -> bool;
}

pub fn to_cstring(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s)
        .unwrap_or_else(|_| panic!("Platform FFI: null byte in string: {s:?}"))
}