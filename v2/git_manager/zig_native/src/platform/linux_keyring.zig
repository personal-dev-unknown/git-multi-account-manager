// zig_native/src/platform/linux_keyring.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Provides credential storage using the GNOME libsecret D-Bus API on Linux.
// libsecret is the modern replacement for gnome-keyring-daemon's direct API —
// it communicates with the keyring daemon over D-Bus and works across multiple
// keyring implementations (GNOME Keyring, KWallet via a compatibility layer,
// KeePassXC with its browser integration, and Seahorse).
//
// ── Why use the system keychain at all? ───────────────────────────────────────
// The fallback_keyring.zig stores secrets in an encrypted file in the user's
// home directory. While the encryption is strong (AES-256-GCM), the encrypted
// file is accessible to any process running as the user. The system keychain
// (libsecret on Linux) provides additional protections:
//   - Secrets are locked when the session is idle or the screen is locked
//   - Access to the keychain requires the session authentication (unlock prompt)
//   - The keychain daemon can enforce per-application access policies
//   - Secrets survive app reinstallation but can be revoked centrally
//
// ── libsecret API strategy ────────────────────────────────────────────────────
// libsecret's primary API uses varargs C functions (secret_password_store_sync,
// etc.) which are difficult to call from Zig because Zig's @cImport does not
// support calling C varargs functions with arbitrary argument types. We use the
// non-varargs "v" variants instead:
//   secret_password_storev_sync()    — takes GHashTable* instead of varargs
//   secret_password_lookupv_sync()   — takes GHashTable* instead of varargs
//   secret_password_clearv_sync()    — takes GHashTable* instead of varargs
//
// These "v" functions take a GHashTable of attributes, which we build manually
// using GLib's g_hash_table_new / g_hash_table_insert functions.
//
// ── Fallback behavior ─────────────────────────────────────────────────────────
// If the GNOME keyring daemon is not running (e.g., on a headless server or in
// a minimal desktop environment), libsecret operations fail with a D-Bus error.
// We detect this and delegate to the fallback_keyring.zig implementation so
// the application remains functional even without a desktop keyring daemon.
//
// ── Build dependency ──────────────────────────────────────────────────────────
// This file is only compiled on Linux (selected by platform/index.zig).
// build.zig links against libsecret-1 via: lib.linkSystemLibrary("secret-1");
// Install the dev headers: sudo apt install libsecret-1-dev

const std = @import("std");
const builtin = @import("builtin");
const fallback = @import("fallback_keyring.zig");

const FsResult = @import("../filesystem/atomic_write.zig").FsResult;
pub const SecretResult = @import("types.zig").SecretResult;

// Import libsecret and GLib C headers.
// Zig's @cImport translates C types to Zig types and makes C functions callable.
const c = @cImport({
    @cInclude("libsecret/secret.h");
});

/// The SecretSchema describes the structure of secrets we store.
/// It is analogous to a database table schema: it names the secret collection
/// and declares which attributes (columns) identify a secret within it.
///
/// We use three attributes:
///   "application" — always "git-manager" — distinguishes our secrets
///   "label"       — the credential label value, e.g. "account-pat"
///   "username"    — the account UUID, e.g. "550e8400-e29b-41d4-a716-446655440000"
///
/// The schema is declared as a global constant because libsecret expects a
/// stable pointer to it across all calls. A local variable would be invalid
/// after the function returns.
///
/// ── Attribute array size ────────────────────────────────────────────────────
/// Newer versions of libsecret changed the SecretSchemaAttribute fixed array
/// from [8] to [32] elements. We pad all remaining slots with null entries to
/// satisfy the struct layout regardless of which libsecret version is installed.
/// Three named attributes + 29 null pads = 32 total, covering both sizes.
var schema: c.SecretSchema = .{
    .name = "dev.gitmanager.Credentials",
    .flags = c.SECRET_SCHEMA_NONE,
    .attributes = [_]c.SecretSchemaAttribute{
        .{ .name = "application", .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = "label", .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = "username", .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        // Remaining slots padded with null to satisfy the [32] fixed-size array
        // in the newer libsecret SecretSchema struct (updated from [8] to [32]).
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
        .{ .name = null, .type = c.SECRET_SCHEMA_ATTRIBUTE_STRING },
    },
};

/// Builds a GHashTable containing our standard credential attributes.
/// The caller is responsible for calling c.g_hash_table_destroy(attrs) when done.
///
/// Uses compile-time string literals for static keys/values and passes through
/// the caller's null-terminated pointers directly — zero allocations, zero leaks.
fn buildAttributes(
    label: [*:0]const u8,
    username: [*:0]const u8,
) !*c.GHashTable {
    const attrs = c.g_hash_table_new(c.g_str_hash, c.g_str_equal) orelse {
        return error.OutOfMemory;
    };

    // Keys MUST be the declared schema attribute names ("application", "label", "username")
    // — libsecret validates the hash table against the schema by matching key names.
    // Values are the runtime parameters. No allocator needed, zero memory leaks.
    // @constCast is required because g_hash_table_insert takes non-const gpointer,
    // but GLib does not modify keys/values in practice.
    _ = c.g_hash_table_insert(attrs, @ptrCast(@constCast(@as([*:0]const u8, "application"))), @ptrCast(@constCast(@as([*:0]const u8, "git-manager"))));
    _ = c.g_hash_table_insert(attrs, @ptrCast(@constCast(@as([*:0]const u8, "label"))),     @ptrCast(@constCast(label)));
    _ = c.g_hash_table_insert(attrs, @ptrCast(@constCast(@as([*:0]const u8, "username"))),  @ptrCast(@constCast(username)));

    return attrs;
}

/// Stores a secret in the GNOME keychain.
/// Falls back to fallback_keyring.zig if the keyring daemon is unavailable.
pub export fn gm_platform_store_secret(
    label: [*:0]const u8,
    username: [*:0]const u8,
    secret: [*]const u8,
    secret_len: usize,
    result: *FsResult,
) bool {
    return storeSecretImpl(
        label,
        username,
        secret[0..secret_len],
        result,
    ) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

fn storeSecretImpl(
    label: [*:0]const u8,
    username: [*:0]const u8,
    secret: []const u8,
    result: *FsResult,
) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const label_s = std.mem.span(label);
    const user_s  = std.mem.span(username);

    // Build the attribute hash table — zero-alloc, direct pointer insertion.
    const attrs = try buildAttributes(label, username);
    defer c.g_hash_table_destroy(attrs);

    // Build a human-readable description for the keychain entry.
    const description_str = try std.fmt.allocPrint(
        allocator,
        "Git Manager credential: {s} ({s})",
        .{ label_s, user_s },
    );
    defer allocator.free(description_str);
    const description = try allocator.dupeZ(u8, description_str);
    defer allocator.free(description);

    // null-terminate the secret for libsecret (it stores it as a string).
    // Our secrets (PATs, OAuth tokens) are ASCII strings, so this is safe.
    const secret_z = try allocator.dupeZ(u8, secret);
    defer {
        @memset(secret_z, 0);
        allocator.free(secret_z);
    }

    var err: ?*c.GError = null;
    const stored = c.secret_password_storev_sync(
        &schema,
        attrs,
        null,
        description.ptr,
        secret_z.ptr,
        null,
        &err,
    );

    if (err != null) {
        defer c.g_error_free(err);

        if (err.?.domain == c.g_dbus_error_quark() or err.?.domain == c.secret_error_get_quark()) {
            return fallback.gm_platform_store_secret(
                label,
                username,
                secret.ptr,
                secret.len,
                result,
            );
        }
        const msg = std.mem.span(err.?.message);
        writeError(result, msg);
        return false;
    }

    if (stored == 0 or stored == c.FALSE) {
        writeError(result, "libsecret stored=false with no error — unknown failure");
        return false;
    }

    return true;
}

/// Retrieves a secret from the GNOME keychain.
pub export fn gm_platform_retrieve_secret(
    label: [*:0]const u8,
    username: [*:0]const u8,
    result: *SecretResult,
) bool {
    return retrieveSecretImpl(
        label,
        username,
        result,
    ) catch |err| {
        writeSecretError(result, @errorName(err));
        return false;
    };
}

fn retrieveSecretImpl(label: [*:0]const u8, username: [*:0]const u8, result: *SecretResult) !bool {
    const attrs = try buildAttributes(label, username);
    defer c.g_hash_table_destroy(attrs);

    var err: ?*c.GError = null;
    const secret_cstr = c.secret_password_lookupv_sync(
        &schema,
        attrs,
        null,
        &err,
    );

    if (err != null) {
        defer c.g_error_free(err);
        if (err.?.domain == c.g_dbus_error_quark() or err.?.domain == c.secret_error_get_quark()) {
            return fallback.gm_platform_retrieve_secret(
                label,
                username,
                result,
            );
        }
        const msg = std.mem.span(err.?.message);
        writeSecretError(result, msg);
        return false;
    }

    if (secret_cstr == null) {
        writeSecretError(result, "credential not found in keychain");
        return false;
    }
    defer c.secret_password_free(secret_cstr);

    const secret_slice = std.mem.span(secret_cstr.?);
    const copy_len = @min(secret_slice.len, result.secret.len - 1);
    @memcpy(result.secret[0..copy_len], secret_slice[0..copy_len]);
    result.secret_len = copy_len;

    return true;
}

/// Deletes a secret from the GNOME keychain.
pub export fn gm_platform_delete_secret(
    label: [*:0]const u8,
    username: [*:0]const u8,
    result: *FsResult,
) bool {
    return deleteSecretImpl(
        label,
        username,
        result,
    ) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

fn deleteSecretImpl(label: [*:0]const u8, username: [*:0]const u8, result: *FsResult) !bool {
    const attrs = try buildAttributes(label, username);
    defer c.g_hash_table_destroy(attrs);

    var err: ?*c.GError = null;
    _ = c.secret_password_clearv_sync(
        &schema,
        attrs,
        null,
        &err,
    );

    if (err != null) {
        defer c.g_error_free(err);
        if (err.?.domain == c.g_dbus_error_quark() or err.?.domain == c.secret_error_get_quark()) {
            return fallback.gm_platform_delete_secret(
                label,
                username,
                result,
            );
        }
        const msg = std.mem.span(err.?.message);
        writeError(result, msg);
        return false;
    }

    return true;
}

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