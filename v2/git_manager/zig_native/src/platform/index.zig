// zig_native/src/platform/index.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Platform credential storage module. Selects the correct backend at compile
// time using Zig's `comptime` feature. This means the wrong backend is not even
// compiled for a given target — there is zero dead code in the output library.
//
// ── Backend selection logic ───────────────────────────────────────────────────
// Linux  → linux_keyring.zig  (GNOME libsecret via D-Bus)
//           Requires: libsecret-1-dev, a running GNOME keyring daemon
//           Fallback: falls back to fallback_keyring.zig if the daemon is unavailable
//
// macOS  → macos_keychain.zig (macOS Security.framework)
//           Requires: nothing — Security.framework ships with every macOS
//
// Other  → fallback_keyring.zig (AES-256-GCM encrypted file)
//           Works everywhere: no system dependencies, pure Zig crypto
//           Less secure than a proper keychain: keys live in an encrypted
//           file rather than hardware-backed secure storage
//
// ── Why comptime instead of runtime dispatch? ─────────────────────────────────
// Comptime selection means:
//   - No runtime overhead for the dispatch itself
//   - No code for the wrong platform ships in the binary (smaller artifact)
//   - Platform-specific API calls are fully type-checked against the real headers
//   - Build errors for missing libraries are caught at Zig compile time,
//     not at runtime when the first credential operation runs
//
// ── C ABI contract ────────────────────────────────────────────────────────────
// All three backends export the SAME three functions with identical signatures:
//   gm_platform_store_secret
//   gm_platform_retrieve_secret
//   gm_platform_delete_secret
// The Rust adapter does not know which backend is active — it calls through
// the same extern "C" declarations regardless of platform.

const std = @import("std");
const builtin = @import("builtin");

// Comptime backend selection — evaluated at Zig build time, not at runtime
const backend = switch (builtin.os.tag) {
    .linux => @import("linux_keyring.zig"),
    .macos => @import("macos_keychain.zig"),
    else => @import("fallback_keyring.zig"),
};

// Re-export the three credential functions from the selected backend.
// These become the symbols in libgm_native.a that Rust links against.
pub const gm_platform_store_secret = backend.gm_platform_store_secret;
pub const gm_platform_retrieve_secret = backend.gm_platform_retrieve_secret;
pub const gm_platform_delete_secret = backend.gm_platform_delete_secret;

// Shared result types — single source of truth; all backends import these.
pub const FsResult = @import("../filesystem/atomic_write.zig").FsResult;
pub const SecretResult = @import("types.zig").SecretResult;

// ── Compile-time sanity check ─────────────────────────────────────────────────
// Verify that the selected backend actually exports all three required functions.
// This turns a potential link-time error into a clear compile-time error message.
comptime {
    const backend_has_store = @hasDecl(backend, "gm_platform_store_secret");
    const backend_has_retrieve = @hasDecl(backend, "gm_platform_retrieve_secret");
    const backend_has_delete = @hasDecl(backend, "gm_platform_delete_secret");
    const backend_has_result = @hasDecl(backend, "SecretResult");

    if (!backend_has_store or !backend_has_retrieve or !backend_has_delete or !backend_has_result) {
        @compileError("Platform backend is missing required credential storage exports");
    }
}

test "platform backend compiles and exports required symbols" {
    // Verify all three functions are accessible through the index
    try std.testing.expect(@TypeOf(gm_platform_store_secret) != void);
    try std.testing.expect(@TypeOf(gm_platform_retrieve_secret) != void);
    try std.testing.expect(@TypeOf(gm_platform_delete_secret) != void);
}
