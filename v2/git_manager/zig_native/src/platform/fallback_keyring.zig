// zig_native/src/platform/fallback_keyring.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Provides credential storage using an AES-256-GCM encrypted file when no
// system keychain is available. This backend is used:
//   - On Linux without a running GNOME keyring daemon (headless servers, CI)
//   - On Windows (future support)
//   - On any other platform not explicitly handled
//   - As the fallback when the system keychain is inaccessible
//
// ── Security model ────────────────────────────────────────────────────────────
// The encrypted file lives at: ~/.git-manager/secrets.enc
// Encryption: AES-256-GCM (authenticated encryption — detects tampering)
// Key derivation: HKDF-SHA256 using machine-specific material:
//   Input key material (IKM) = /etc/machine-id + $HOME
//   This ties the key to a specific machine+user combination. Copying the
//   secrets.enc file to another machine or another user's home directory
//   will produce decryption failures.
//
// ── File format ───────────────────────────────────────────────────────────────
// The file is a flat binary store. Each entry is:
//   [4 bytes: entry_len as little-endian u32]
//   [entry_len bytes: JSON-encoded EncryptedEntry]
//
// The JSON is base64-encoded to make the file ASCII-safe. Each entry contains:
//   { "label": "...", "username": "...", "nonce": "base64...", "tag": "base64...", "ciphertext": "base64..." }
//
// We use JSON + base64 for the entry format (rather than raw binary) because it:
//   - Is human-inspectable for debugging (after decryption)
//   - Supports adding new fields in future versions without migration
//   - Avoids alignment issues from mixed-size binary fields
//
// ── Thread safety ─────────────────────────────────────────────────────────────
// NOT thread-safe. The Rust domain service must serialize all credential
// operations when running on the fallback backend (same as the Linux/macOS
// backends). The file is read into memory, modified, and written atomically,
// but concurrent modifications would produce data loss via race conditions.
//
// ── Nonce uniqueness ──────────────────────────────────────────────────────────
// AES-GCM requires that the nonce (IV) is never reused with the same key.
// We generate a fresh random 96-bit (12-byte) nonce for each encryption
// using Zig's std.crypto.random, which reads from the OS CSPRNG (/dev/urandom
// on Linux). The probability of nonce collision across 2^32 entries is
// approximately 2^(-64) — negligible in practice.

const std = @import("std");
const crypto = std.crypto;

const Aes256Gcm = crypto.aead.aes_gcm.Aes256Gcm;
const HkdfSha256 = crypto.kdf.hkdf.HkdfSha256;
const Base64 = std.base64.standard;

const FsResult = @import("../filesystem/atomic_write.zig").FsResult;
pub const SecretResult = @import("types.zig").SecretResult;

/// The store file path, relative to $HOME
const STORE_RELATIVE_PATH = ".git-manager/secrets.enc";

/// Derives the 32-byte AES-256 encryption key from machine-specific material.
///
/// The derivation is deterministic — running this function on the same machine
/// as the same user always produces the same key. This means the secrets.enc
/// file is portable only as long as:
///   - /etc/machine-id does not change (it changes on re-installation)
///   - The HOME directory path does not change (it changes if the user is renamed)
///
/// On macOS and other systems without /etc/machine-id, we fall back to using
/// only the HOME directory path as the IKM.
fn deriveKey(key_out: *[32]u8, allocator: std.mem.Allocator) !void {
    var ikm_buf = std.ArrayList(u8).empty;
    defer ikm_buf.deinit(allocator);

    // Read /etc/machine-id if available (Linux). On failure, skip silently.
    // Use a fixed stack buffer — readFileAlloc's API changed in Zig 0.16.0-dev.
    var mid_buf: [256]u8 = undefined;
    const machine_id: ?[]const u8 = blk: {
        const mid_file = std.fs.cwd().openFile("/etc/machine-id", .{}) catch break :blk null;
        defer mid_file.close();
        const n = mid_file.read(&mid_buf) catch break :blk null;
        break :blk mid_buf[0..n];
    };

    if (machine_id) |m| {
        // Trim whitespace/newlines from machine-id
        try ikm_buf.appendSlice(allocator, std.mem.trim(u8, m, " \t\r\n"));
    }

    // Append the HOME directory path as additional machine-specific material
    const home = std.posix.getenv("HOME") orelse "";
    try ikm_buf.appendSlice(allocator, home);

    if (ikm_buf.items.len == 0) {
        // No machine-specific material available — this should not happen in practice
        // but we handle it gracefully by using a static fallback string.
        // This is less secure but better than failing completely.
        try ikm_buf.appendSlice(allocator, "git-manager-fallback-ikm");
    }

    // HKDF-SHA256: extract → expand
    // "git-manager-v1-credentials" is the info string that scopes this key derivation
    // to our specific use case. If we ever need to rotate keys, we change this string.
    const salt: ?[]const u8 = null; // null = use HKDF's default all-zeros salt
    const prk = HkdfSha256.extract(salt orelse "", ikm_buf.items);
    HkdfSha256.expand(key_out, "git-manager-v1-credentials", prk);
}

/// Reads and deserializes all entries from the secrets file.
/// Returns a slice of owned JSON strings; caller frees each and the slice.
fn readEntries(path: []const u8, allocator: std.mem.Allocator) ![][]const u8 {
    // 1. Attempt to open the file. If it doesn't exist, return an empty slice.
    const file = std.fs.cwd().openFile(path, .{}) catch |err| switch (err) {
        error.FileNotFound => return &[_][]const u8{},
        else => return err,
    };
    defer file.close();

    // 2. Read the file contents up to 1MB. Use `try` since any read errors
    // at this point should just be propagated up.
    const stat = try file.stat();
    if (stat.size > 1024 * 1024) return error.FileTooBig;
    const data = try allocator.alloc(u8, @intCast(stat.size));
    defer allocator.free(data);
    _ = try file.readAll(data);

    var entries = std.ArrayList([]const u8).empty;
    errdefer {
        for (entries.items) |e| allocator.free(e);
        entries.deinit(allocator);
    }

    var pos: usize = 0;
    while (pos + 4 <= data.len) {
        const entry_len = std.mem.readInt(u32, data[pos..][0..4], .little);
        pos += 4;
        if (pos + entry_len > data.len) break;
        const entry_json = try allocator.dupe(u8, data[pos .. pos + entry_len]);
        try entries.append(allocator, entry_json);
        pos += entry_len;
    }

    return entries.toOwnedSlice(allocator);
}

/// Serializes entries back to the binary store format and writes atomically.
fn writeEntries(path: []const u8, entries: []const []const u8, allocator: std.mem.Allocator) !void {
    var buf = std.ArrayList(u8).empty;
    defer buf.deinit(allocator);

    for (entries) |entry| {
        var len_bytes: [4]u8 = undefined;
        std.mem.writeInt(u32, &len_bytes, @intCast(entry.len), .little);
        try buf.appendSlice(allocator, &len_bytes);
        try buf.appendSlice(allocator, entry);
    }

    // Atomic write via temp file + rename
    const temp_path = try std.fmt.allocPrint(allocator, "{s}.tmp_{d}", .{ path, std.time.milliTimestamp() });
    defer allocator.free(temp_path);

    {
        const f = try std.fs.createFileAbsolute(temp_path, .{ .mode = 0o600 });
        defer f.close();
        try f.writeAll(buf.items);
        try f.sync();
    }
    try std.fs.renameAbsolute(temp_path, path);
}

/// Stores a secret in the encrypted file store.
pub fn gm_platform_store_secret(
    label: [*:0]const u8,
    username: [*:0]const u8,
    secret: [*]const u8,
    secret_len: usize,
    result: *FsResult,
) bool {
    return storeImpl(
        std.mem.span(label),
        std.mem.span(username),
        secret[0..secret_len],
        result,
    ) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

fn storeImpl(label: []const u8, username: []const u8, secret: []const u8, result: *FsResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const home = std.posix.getenv("HOME") orelse {
        writeError(result, "HOME not set");
        return false;
    };
    const store_path = try std.fmt.allocPrint(allocator, "{s}/{s}", .{ home, STORE_RELATIVE_PATH });
    defer allocator.free(store_path);

    // Ensure the .git-manager directory exists
    const dir_path = try std.fmt.allocPrint(allocator, "{s}/.git-manager", .{home});
    defer allocator.free(dir_path);
    std.fs.makeDirAbsolute(dir_path) catch |e| switch (e) {
        error.PathAlreadyExists => {},
        else => return e,
    };
    var dir = try std.fs.cwd().openDir(dir_path, .{});
    defer dir.close();
    try std.posix.fchmod(dir.fd, 0o700);

    // Derive the encryption key
    var key: [32]u8 = undefined;
    try deriveKey(&key, allocator);
    defer @memset(&key, 0); // zero the key after use

    // Generate a random 12-byte nonce for this encryption
    var nonce: [Aes256Gcm.nonce_length]u8 = undefined;
    crypto.random.bytes(&nonce);

    // Encrypt the secret
    const ciphertext = try allocator.alloc(u8, secret.len);
    defer {
        @memset(ciphertext, 0);
        allocator.free(ciphertext);
    }
    var tag: [Aes256Gcm.tag_length]u8 = undefined;
    Aes256Gcm.encrypt(ciphertext, &tag, secret, "", nonce, key);

    // Base64-encode the binary fields for JSON embedding
    const nonce_b64_len = Base64.Encoder.calcSize(nonce.len);
    const tag_b64_len = Base64.Encoder.calcSize(tag.len);
    const ct_b64_len = Base64.Encoder.calcSize(ciphertext.len);

    const nonce_b64 = try allocator.alloc(u8, nonce_b64_len);
    defer allocator.free(nonce_b64);
    const tag_b64 = try allocator.alloc(u8, tag_b64_len);
    defer allocator.free(tag_b64);
    const ct_b64 = try allocator.alloc(u8, ct_b64_len);
    defer {
        @memset(ct_b64, 0);
        allocator.free(ct_b64);
    }

    _ = Base64.Encoder.encode(nonce_b64, &nonce);
    _ = Base64.Encoder.encode(tag_b64, &tag);
    _ = Base64.Encoder.encode(ct_b64, ciphertext);

    // Build the JSON entry
    const entry_json = try std.fmt.allocPrint(
        allocator,
        "{{\"label\":\"{s}\",\"username\":\"{s}\",\"nonce\":\"{s}\",\"tag\":\"{s}\",\"ct\":\"{s}\"}}",
        .{ label, username, nonce_b64, tag_b64, ct_b64 },
    );
    defer allocator.free(entry_json);

    // Read existing entries, removing any with the same (label, username)
    const existing = try readEntries(store_path, allocator);
    defer {
        for (existing) |e| allocator.free(e);
        allocator.free(existing);
    }

    // Build a new entry list: all existing except matching, plus the new one
    const search_label = try std.fmt.allocPrint(allocator, "\"label\":\"{s}\"", .{label});
    defer allocator.free(search_label);
    const search_username = try std.fmt.allocPrint(allocator, "\"username\":\"{s}\"", .{username});
    defer allocator.free(search_username);

    var new_entries = std.ArrayList([]const u8).empty;
    defer new_entries.deinit(allocator);

    for (existing) |e| {
        const is_match = std.mem.indexOf(u8, e, search_label) != null and
            std.mem.indexOf(u8, e, search_username) != null;
        if (!is_match) try new_entries.append(allocator, e);
    }
    try new_entries.append(allocator, entry_json);

    try writeEntries(store_path, new_entries.items, allocator);
    return true;
}

/// Retrieves a secret from the encrypted file store.
pub fn gm_platform_retrieve_secret(
    label: [*:0]const u8,
    username: [*:0]const u8,
    result: *SecretResult,
) bool {
    return retrieveImpl(std.mem.span(label), std.mem.span(username), result) catch |err| {
        writeSecretError(result, @errorName(err));
        return false;
    };
}

fn retrieveImpl(label: []const u8, username: []const u8, result: *SecretResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const home = std.posix.getenv("HOME") orelse {
        writeSecretError(result, "HOME not set");
        return false;
    };
    const store_path = try std.fmt.allocPrint(allocator, "{s}/{s}", .{ home, STORE_RELATIVE_PATH });
    defer allocator.free(store_path);

    const entries = try readEntries(store_path, allocator);
    defer {
        for (entries) |e| allocator.free(e);
        allocator.free(entries);
    }

    const search_label = try std.fmt.allocPrint(allocator, "\"label\":\"{s}\"", .{label});
    defer allocator.free(search_label);
    const search_username = try std.fmt.allocPrint(allocator, "\"username\":\"{s}\"", .{username});
    defer allocator.free(search_username);

    // Find the matching entry
    const found_entry: ?[]const u8 = blk: {
        for (entries) |e| {
            if (std.mem.indexOf(u8, e, search_label) != null and
                std.mem.indexOf(u8, e, search_username) != null)
            {
                break :blk e;
            }
        }
        break :blk null;
    };

    if (found_entry == null) {
        writeSecretError(result, "credential not found in fallback store");
        return false;
    }

    const entry = found_entry.?;

    // Extract the base64-encoded fields from the JSON using simple string parsing.
    // We avoid a full JSON parser to keep the dependency footprint minimal.
    const nonce_b64 = extractJsonField(entry, "nonce", allocator) orelse {
        writeSecretError(result, "corrupt entry: missing nonce");
        return false;
    };
    defer allocator.free(nonce_b64);

    const tag_b64 = extractJsonField(entry, "tag", allocator) orelse {
        writeSecretError(result, "corrupt entry: missing tag");
        return false;
    };
    defer allocator.free(tag_b64);

    const ct_b64 = extractJsonField(entry, "ct", allocator) orelse {
        writeSecretError(result, "corrupt entry: missing ciphertext");
        return false;
    };
    defer {
        @memset(ct_b64, 0);
        allocator.free(ct_b64);
    }

    // Decode from base64
    const nonce_bytes = try allocator.alloc(u8, try Base64.Decoder.calcSizeForSlice(nonce_b64));
    defer allocator.free(nonce_bytes);
    try Base64.Decoder.decode(nonce_bytes, nonce_b64);

    const tag_bytes = try allocator.alloc(u8, try Base64.Decoder.calcSizeForSlice(tag_b64));
    defer allocator.free(tag_bytes);
    try Base64.Decoder.decode(tag_bytes, tag_b64);

    const ct_bytes = try allocator.alloc(u8, try Base64.Decoder.calcSizeForSlice(ct_b64));
    defer {
        @memset(ct_bytes, 0);
        allocator.free(ct_bytes);
    }
    try Base64.Decoder.decode(ct_bytes, ct_b64);

    if (nonce_bytes.len != Aes256Gcm.nonce_length or tag_bytes.len != Aes256Gcm.tag_length) {
        writeSecretError(result, "corrupt entry: invalid nonce or tag length");
        return false;
    }

    // Derive the key and decrypt
    var key: [32]u8 = undefined;
    try deriveKey(&key, allocator);
    defer @memset(&key, 0);

    const plaintext = try allocator.alloc(u8, ct_bytes.len);
    defer {
        @memset(plaintext, 0);
        allocator.free(plaintext);
    }

    var nonce: [Aes256Gcm.nonce_length]u8 = undefined;
    @memcpy(&nonce, nonce_bytes[0..Aes256Gcm.nonce_length]);
    var tag: [Aes256Gcm.tag_length]u8 = undefined;
    @memcpy(&tag, tag_bytes[0..Aes256Gcm.tag_length]);

    Aes256Gcm.decrypt(plaintext, ct_bytes, tag, "", nonce, key) catch {
        writeSecretError(result, "decryption failed: tag mismatch (key changed or data corrupted)");
        return false;
    };

    const copy_len = @min(plaintext.len, result.secret.len - 1);
    @memcpy(result.secret[0..copy_len], plaintext[0..copy_len]);
    result.secret_len = copy_len;

    return true;
}

/// Deletes a secret from the encrypted file store.
pub fn gm_platform_delete_secret(
    label: [*:0]const u8,
    username: [*:0]const u8,
    result: *FsResult,
) bool {
    return deleteImpl(std.mem.span(label), std.mem.span(username), result) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

fn deleteImpl(label: []const u8, username: []const u8, result: *FsResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const home = std.posix.getenv("HOME") orelse return true; // no HOME = nothing to delete
    const store_path = try std.fmt.allocPrint(allocator, "{s}/{s}", .{ home, STORE_RELATIVE_PATH });
    defer allocator.free(store_path);

    const entries = try readEntries(store_path, allocator);
    defer {
        for (entries) |e| allocator.free(e);
        allocator.free(entries);
    }

    const search_label = try std.fmt.allocPrint(allocator, "\"label\":\"{s}\"", .{label});
    defer allocator.free(search_label);
    const search_username = try std.fmt.allocPrint(allocator, "\"username\":\"{s}\"", .{username});
    defer allocator.free(search_username);

    var filtered = std.ArrayList([]const u8).empty;
    defer filtered.deinit(allocator);

    for (entries) |e| {
        const is_match = std.mem.indexOf(u8, e, search_label) != null and
            std.mem.indexOf(u8, e, search_username) != null;
        if (!is_match) try filtered.append(allocator, e);
    }

    try writeEntries(store_path, filtered.items, allocator);
    _ = result; // no error to report on success
    return true;
}

/// Extracts a quoted string value from a simple JSON object by key.
/// Returns null if the key is not found. Caller frees the returned slice.
fn extractJsonField(json: []const u8, key: []const u8, allocator: std.mem.Allocator) ?[]u8 {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const tmp = gpa.allocator();
    const search = std.fmt.allocPrint(tmp, "\"{s}\":\"", .{key}) catch return null;
    defer tmp.free(search);

    const start = (std.mem.indexOf(u8, json, search) orelse return null) + search.len;
    const end = std.mem.indexOfScalarPos(u8, json, start, '"') orelse return null;
    return allocator.dupe(u8, json[start..end]) catch null;
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

test "extractJsonField extracts simple string values" {
    const json = "{\"label\":\"work-pat\",\"username\":\"shaka\",\"nonce\":\"abc123\"}";
    const allocator = std.testing.allocator;

    const label = extractJsonField(json, "label", allocator);
    defer if (label) |l| allocator.free(l);
    try std.testing.expect(label != null);
    try std.testing.expectEqualStrings("work-pat", label.?);

    const username = extractJsonField(json, "username", allocator);
    defer if (username) |u| allocator.free(u);
    try std.testing.expectEqualStrings("shaka", username.?);

    const missing = extractJsonField(json, "nonexistent", allocator);
    try std.testing.expect(missing == null);
}