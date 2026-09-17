// zig_native/src/platform/macos_keychain.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// macOS Keychain credential storage using the Security framework API.
// Selected by platform/index.zig via comptime switch on builtin.os.tag.
//
// ── Why the @cImport is inside a comptime conditional ─────────────────────────
// Zig's @cImport runs the translate-c engine (libclang) to convert C headers
// into a Zig namespace. ZLS (the Zig language server) runs translate-c on every
// @cImport it encounters while analysing the workspace — including files that the
// build system would never compile for the current target.
//
// On Linux, index.zig routes to linux_keyring.zig and this file is never compiled.
// But ZLS still scans it, reaches the original top-level @cImport, and errors
// because Security/Security.h does not exist on Linux.
//
// The fix: wrap BOTH the @cImport AND the implementation in a comptime
// `if (builtin.os.tag == .macos)` block. When Zig evaluates a comptime
// conditional, only the matching branch is analysed. On Linux, the macOS
// branch is completely dead — translate-c is never invoked, no Apple
// header is sought, no error fires in ZLS.
//
// The exported functions have an early comptime return at the top that fires on
// non-macOS. This means their bodies (which reference the Security types) are
// in dead code that Zig does not type-check on Linux.
//
// ── Security.framework API used ───────────────────────────────────────────────
//   SecKeychainAddGenericPassword      — store a new password
//   SecKeychainFindGenericPassword     — retrieve a stored password
//   SecKeychainItemDelete              — delete a stored item
//   SecKeychainItemFreeContent         — free memory returned by Find
//   SecKeychainItemModifyAttributesAndData — update existing item
//
// ── Memory ownership ──────────────────────────────────────────────────────────
// SecKeychainFindGenericPassword allocates a buffer for the returned password.
// This buffer MUST be freed with SecKeychainItemFreeContent — it is NOT a
// standard malloc() allocation and cannot be freed with free() or allocator.free().
// The code below handles this correctly using defer.

const std     = @import("std");
const builtin = @import("builtin");

const FsResult = @import("../filesystem/atomic_write.zig").FsResult;

/// Result buffer for secret retrieval.
/// Matches GmSecretResult in gm_native.h and FfiSecretResult in Rust.
pub const SecretResult = extern struct {
    secret:        [4096]u8 = std.mem.zeroes([4096]u8),
    secret_len:    usize    = 0,
    error_message: [512]u8  = std.mem.zeroes([512]u8),
    error_len:     usize    = 0,
};

// ── Platform-conditional C header import ─────────────────────────────────────
//
// This is the core fix for the "Security/Security.h file not found" ZLS error.
//
// The `if (builtin.os.tag == .macos)` expression is comptime-evaluated.
// When Zig (or ZLS) processes this file on Linux, the .macos branch is dead
// code — translate-c is never invoked for that branch. ZLS and the compiler
// see the `else struct {}` branch instead: an empty struct that compiles
// everywhere.
//
// On macOS, the .macos branch is taken, translate-c runs, Security.framework
// types are available, and the function implementations below use them.
const c = if (builtin.os.tag == .macos)
    @cImport({
        @cInclude("Security/Security.h");
    })
else
    // Empty stub namespace — only here so the file parses on non-macOS.
    // The exported functions return immediately on non-macOS (see below),
    // so no code that uses `c.xxx` types is ever reached or type-checked here.
    struct {};

// ─────────────────────────────────────────────────────────────────────────────
// Exported credential storage functions
//
// Each function begins with:
//   if (comptime builtin.os.tag != .macos) { ...return false; }
//
// This comptime guard makes the remainder of the function body dead code on
// non-macOS targets. Dead code in comptime branches is NOT type-checked by
// Zig, so the `c.SecKeychainItemRef`, `c.errSecSuccess`, etc. references
// below are invisible to the compiler and ZLS when building for Linux.
// ─────────────────────────────────────────────────────────────────────────────

/// Stores a credential in the macOS Keychain using upsert semantics.
/// If the (label, username) pair already exists, the password is updated.
/// If it does not exist, a new Keychain item is created.
pub export fn gm_platform_store_secret(
    label:      [*:0]const u8,
    username:   [*:0]const u8,
    secret:     [*]const u8,
    secret_len: usize,
    result:     *FsResult,
) bool {
    // Dead code on non-macOS — the body below uses Security.framework types
    // that only exist when c == @cImport({ @cInclude("Security/Security.h"); }).
    if (comptime builtin.os.tag != .macos) {
        writeError(result, "macOS Keychain is unavailable on this platform. " ++
            "This code path should never be reached — platform/index.zig routes " ++
            "to the correct backend at compile time.");
        return false;
    }

    const label_slice    = std.mem.span(label);
    const username_slice = std.mem.span(username);

    // Upsert: find existing item first. If found, update it.
    // SecKeychainAddGenericPassword fails on duplicate, so we check first.
    var item_ref: c.SecKeychainItemRef = null;
    var existing_len: u32              = 0;
    var existing_ptr: ?*anyopaque      = null;

    const find_status = c.SecKeychainFindGenericPassword(
        null,                                             // default keychain
        @intCast(label_slice.len),    label_slice.ptr,   // serviceName
        @intCast(username_slice.len), username_slice.ptr, // accountName
        &existing_len, &existing_ptr,                    // output (discarded)
        &item_ref,                                       // output: item reference
    );

    if (existing_ptr != null) {
        // Free the password buffer returned by Find before we proceed.
        // SecKeychainItemFreeContent is NOT free() — it uses the Security
        // framework's own deallocation path. Never use allocator.free() here.
        _ = c.SecKeychainItemFreeContent(null, existing_ptr);
    }

    if (find_status == c.errSecSuccess) {
        // Item exists — update it in-place.
        defer if (item_ref != null) c.CFRelease(item_ref);

        const update_status = c.SecKeychainItemModifyAttributesAndData(
            item_ref,
            null,                     // no attribute changes
            @intCast(secret_len),
            secret,
        );
        if (update_status != c.errSecSuccess) {
            var gpa = std.heap.GeneralPurposeAllocator(.{}){};
            defer _ = gpa.deinit();
            const msg = std.fmt.allocPrint(
                gpa.allocator(),
                "Keychain update failed: OSStatus {d}",
                .{update_status},
            ) catch {
                writeError(result, "Keychain update failed (alloc error)");
                return false;
            };
            defer gpa.allocator().free(msg);
            writeError(result, msg);
            return false;
        }
        return true;
    }

    if (find_status != c.errSecItemNotFound) {
        // Unexpected error during lookup.
        var gpa = std.heap.GeneralPurposeAllocator(.{}){};
        defer _ = gpa.deinit();
        const msg = std.fmt.allocPrint(
            gpa.allocator(),
            "Keychain lookup failed: OSStatus {d}",
            .{find_status},
        ) catch {
            writeError(result, "Keychain lookup failed");
            return false;
        };
        defer gpa.allocator().free(msg);
        writeError(result, msg);
        return false;
    }

    // Item does not exist — add new.
    const add_status = c.SecKeychainAddGenericPassword(
        null,                                              // default keychain
        @intCast(label_slice.len),    label_slice.ptr,    // serviceName
        @intCast(username_slice.len), username_slice.ptr, // accountName
        @intCast(secret_len), secret,                     // password data
        null,                                             // item ref not needed
    );

    if (add_status != c.errSecSuccess) {
        var gpa = std.heap.GeneralPurposeAllocator(.{}){};
        defer _ = gpa.deinit();
        const msg = std.fmt.allocPrint(
            gpa.allocator(),
            "Keychain add failed: OSStatus {d}",
            .{add_status},
        ) catch {
            writeError(result, "Keychain add failed (alloc error)");
            return false;
        };
        defer gpa.allocator().free(msg);
        writeError(result, msg);
        return false;
    }

    return true;
}

/// Retrieves a credential from the macOS Keychain.
/// Returns false (not an error) when the credential is not found.
pub export fn gm_platform_retrieve_secret(
    label:    [*:0]const u8,
    username: [*:0]const u8,
    result:   *SecretResult,
) bool {
    if (comptime builtin.os.tag != .macos) {
        writeSecretError(result, "macOS Keychain unavailable on this platform");
        return false;
    }

    const label_slice    = std.mem.span(label);
    const username_slice = std.mem.span(username);

    var password_len: u32          = 0;
    var password_ptr: ?*anyopaque  = null;

    const status = c.SecKeychainFindGenericPassword(
        null,
        @intCast(label_slice.len),    label_slice.ptr,
        @intCast(username_slice.len), username_slice.ptr,
        &password_len,
        &password_ptr,
        null, // item reference not needed for retrieval
    );

    if (status == c.errSecItemNotFound) {
        writeSecretError(result, "credential not found in macOS Keychain");
        return false;
    }

    if (status != c.errSecSuccess) {
        var gpa = std.heap.GeneralPurposeAllocator(.{}){};
        defer _ = gpa.deinit();
        const msg = std.fmt.allocPrint(
            gpa.allocator(),
            "Keychain lookup failed: OSStatus {d}",
            .{status},
        ) catch {
            writeSecretError(result, "Keychain lookup failed");
            return false;
        };
        defer gpa.allocator().free(msg);
        writeSecretError(result, msg);
        return false;
    }

    // CRITICAL: free with SecKeychainItemFreeContent, not free() or allocator.free().
    // The Security framework allocates this buffer internally using its own heap.
    defer _ = c.SecKeychainItemFreeContent(null, password_ptr);

    if (password_ptr == null or password_len == 0) {
        writeSecretError(result, "Keychain returned empty credential");
        return false;
    }

    const secret_bytes: [*]const u8 = @ptrCast(password_ptr.?);
    const copy_len = @min(password_len, result.secret.len - 1);
    @memcpy(result.secret[0..copy_len], secret_bytes[0..copy_len]);
    result.secret_len = copy_len;

    return true;
}

/// Deletes a credential from the macOS Keychain.
/// Returns true both when deleted AND when the item was not found —
/// idempotent deletion is the correct contract for a credential store.
pub export fn gm_platform_delete_secret(
    label:    [*:0]const u8,
    username: [*:0]const u8,
    result:   *FsResult,
) bool {
    if (comptime builtin.os.tag != .macos) {
        writeError(result, "macOS Keychain unavailable on this platform");
        return false;
    }

    const label_slice    = std.mem.span(label);
    const username_slice = std.mem.span(username);

    var item_ref: c.SecKeychainItemRef = null;

    const find_status = c.SecKeychainFindGenericPassword(
        null,
        @intCast(label_slice.len),    label_slice.ptr,
        @intCast(username_slice.len), username_slice.ptr,
        null, null, // password data not needed for deletion
        &item_ref,
    );

    if (find_status == c.errSecItemNotFound) {
        return true; // Not found — deletion succeeds idempotently.
    }

    if (find_status != c.errSecSuccess) {
        writeError(result, "Keychain lookup before delete failed");
        return false;
    }

    defer if (item_ref != null) c.CFRelease(item_ref);

    const delete_status = c.SecKeychainItemDelete(item_ref);
    if (delete_status != c.errSecSuccess) {
        writeError(result, "Keychain item deletion failed");
        return false;
    }

    return true;
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn writeError(result: *FsResult, msg: []const u8) void {
    const len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..len], msg[0..len]);
    result.error_message[len] = 0;
    result.error_len = len;
}

fn writeSecretError(result: *SecretResult, msg: []const u8) void {
    const len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..len], msg[0..len]);
    result.error_message[len] = 0;
    result.error_len = len;
}