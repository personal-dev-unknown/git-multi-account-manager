// zig_native/src/platform/types.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Shared FFI types for the platform credential storage layer.
// Defining these in one place ensures that linux_keyring.zig and
// fallback_keyring.zig both use the *same* Zig type when passing pointers
// across the module boundary. Zig treats structurally identical types in
// different modules as distinct — so a *linux_keyring.SecretResult and a
// *fallback_keyring.SecretResult are incompatible even if their fields match.

const std = @import("std");

/// Result buffer for secret retrieval operations.
/// Matches GmSecretResult in gm_native.h and FfiSecretResult in Rust.
pub const SecretResult = extern struct {
    secret:        [4096]u8 = std.mem.zeroes([4096]u8),
    secret_len:    usize    = 0,
    error_message: [512]u8  = std.mem.zeroes([512]u8),
    error_len:     usize    = 0,
};
