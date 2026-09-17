// zig_native/src/ssh/keygen.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Generates SSH key pairs by invoking the system's ssh-keygen binary.
// Exposes one C ABI function: gm_ssh_generate_key().
//
// ── Why ssh-keygen instead of a pure Zig crypto implementation? ───────────────
// ssh-keygen is the canonical, battle-tested, widely-audited tool for SSH key
// generation. A pure Zig Ed25519 implementation would need independent
// cryptographic audit before being trusted in a security-sensitive tool.
// ssh-keygen produces keys in exactly the format the ssh-agent and OpenSSH
// expect, handling edge cases (passphrase encoding, key comment format, public
// key file naming) that a custom implementation would need to replicate exactly.
// The subprocess approach trades a tiny performance cost (fork+exec) for
// significant security confidence.
//
// ── Memory model ──────────────────────────────────────────────────────────────
// All result data is written into a caller-provided fixed-size buffer (KeygenResult).
// No heap memory is returned across the FFI boundary — the caller owns the struct
// on the stack or heap, passes a pointer to it, and reads the results from it after
// the call returns. This prevents the class of bugs where the caller must free
// memory allocated by the callee across a language boundary.
//
// ── Buffer sizing rationale ────────────────────────────────────────────────────
// public_key  [4096]u8 — Ed25519 public keys are ~100 chars; RSA 4096-bit are ~740.
//                        4096 bytes gives plenty of headroom for future key types.
// fingerprint [256]u8  — SHA256 fingerprints are "SHA256:" + 43 base64 chars = 50.
//                        256 bytes is generous but the struct is stack-allocated so
//                        the extra bytes cost nothing beyond stack frame size.
// error_message [512]u8 — enough for any ssh-keygen error message including file paths.
//
// ── Concurrency ───────────────────────────────────────────────────────────────
// This function is safe to call from multiple threads simultaneously AS LONG AS
// they write to different out_path values. If two calls write to the same path,
// the last writer wins and the first call's key file is overwritten. The Rust
// domain service enforces uniqueness of key paths before calling this function.

const std = @import("std");

/// Result buffer written by gm_ssh_generate_key.
///
/// CRITICAL ABI CONTRACT: This struct's memory layout MUST exactly match
/// the #[repr(C)] struct FfiKeygenResult in gm_adapters/src/ssh/ffi.rs.
/// Field order, types, and sizes must be identical. Any change here requires
/// a corresponding change in the Rust struct or memory corruption will occur.
///
/// The `extern struct` keyword guarantees C-compatible memory layout:
/// no padding insertion between fields, no reordering for alignment,
/// identical to a C struct with the same field declarations.
pub const KeygenResult = extern struct {
    /// The complete public key string: "ssh-ed25519 AAAA...base64... email@example.com\n"
    /// Null-terminated. public_key_len contains the length excluding the null.
    public_key: [4096]u8 = std.mem.zeroes([4096]u8),
    public_key_len: usize = 0,

    /// The key fingerprint: "SHA256:base64encodedfingerprint"
    /// Null-terminated. fingerprint_len excludes the null.
    fingerprint: [256]u8 = std.mem.zeroes([256]u8),
    fingerprint_len: usize = 0,

    /// Human-readable error description when the function returns false.
    /// Empty (error_len == 0) when the function returns true.
    error_message: [512]u8 = std.mem.zeroes([512]u8),
    error_len: usize = 0,
};

/// Generates an SSH key pair at the specified path.
///
/// Parameters:
///   key_type   — Null-terminated C string: "ed25519", "rsa", or "ecdsa"
///   email      — Null-terminated C string: embedded in the key comment field
///   out_path   — Null-terminated C string: absolute path for the PRIVATE key.
///                The public key is written to out_path ++ ".pub" by ssh-keygen.
///   passphrase — Null-terminated C string: key passphrase. Pass "" (empty) for
///                no passphrase (recommended for automated key management).
///   result     — Caller-allocated output buffer. Must be valid and non-null.
///                Written on both success and failure paths.
///
/// Returns: true on success, false on any failure.
/// On false, result.error_message contains the diagnostic.
/// On true, result.public_key and result.fingerprint contain the key data.
///
/// Safety contract for Rust callers:
///   - All pointer parameters must be valid null-terminated C strings
///   - result must point to a valid KeygenResult (zeroed is correct)
///   - The caller must not free any memory after this call (nothing is heap-allocated
///     on the Zig side that the caller is responsible for)
///   - This function must be called from a blocking thread (not from an async executor
///     directly) because it blocks waiting for ssh-keygen to complete
pub export fn gm_ssh_generate_key(
    key_type: [*:0]const u8,
    email: [*:0]const u8,
    out_path: [*:0]const u8,
    passphrase: [*:0]const u8,
    result: *KeygenResult,
) bool {
    return generateKeyImpl(
        std.mem.span(key_type),
        std.mem.span(email),
        std.mem.span(out_path),
        std.mem.span(passphrase),
        result,
    ) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

/// Internal implementation using Zig error unions for clean error propagation.
/// The export wrapper above catches any error and writes it to the result buffer.
fn generateKeyImpl(
    key_type: []const u8,
    email: []const u8,
    out_path: []const u8,
    passphrase: []const u8,
    result: *KeygenResult,
) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer {
        const check = gpa.deinit();
        // In debug builds, leak detection fires an assertion.
        // In release builds, leaks are silently ignored (they won't happen
        // in practice because defer frees everything before returning).
        _ = check;
    }
    const allocator = gpa.allocator();

    // Determine RSA key size from the key_type string.
    // "rsa" defaults to 4096 bits; "rsa-2048" requests 2048 bits.
    // Ed25519 and ECDSA ignore the -b flag.
    const key_bits: ?[]const u8 = if (std.mem.eql(u8, key_type, "rsa"))
        "4096"
    else if (std.mem.eql(u8, key_type, "rsa-2048"))
        "2048"
    else
        null;

    // Build the ssh-keygen argument list.
    // We use an ArrayList so we can conditionally append the -b flag.
    var args = std.ArrayList([]const u8).empty;
    defer args.deinit(allocator);

    try args.appendSlice(allocator, &[_][]const u8{
        "ssh-keygen",
        "-t", key_type, // key type: ed25519, rsa, ecdsa
        "-C", email, // key comment (shown in authorized_keys)
        "-f", out_path, // output file path (private key)
        "-N", passphrase, // passphrase (-N "" = no passphrase)
    });

    if (key_bits) |bits| {
        // -b sets the bit size for RSA/ECDSA keys.
        // For Ed25519, this flag is silently ignored by ssh-keygen but we
        // omit it to avoid any future ssh-keygen warning about invalid args.
        try args.appendSlice(allocator, &[_][]const u8{ "-b", bits });
    }

    // Remove existing key files so ssh-keygen does not enter an interactive
    // "Overwrite (y/n)?" prompt. When run as a child process with closed stdin
    // the prompt is unanswered, ssh-keygen exits 1, and stderr is empty —
    // resulting in the opaque "ssh-keygen failed with no error output" message.
    // The domain layer has already deactivated the old DB record before calling
    // here, so deleting the stale files is correct and safe.
    std.fs.deleteFileAbsolute(out_path) catch |err| switch (err) {
        error.FileNotFound => {}, // first-time generation — nothing to remove
        else => {}, // permission error etc. — let ssh-keygen surface it
    };
    const pub_path_pre = try std.fmt.allocPrint(allocator, "{s}.pub", .{out_path});
    defer allocator.free(pub_path_pre);
    std.fs.deleteFileAbsolute(pub_path_pre) catch {};

    // Run ssh-keygen and capture stdout + stderr.
    const run_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = args.items,
        .max_output_bytes = 4096,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    // Check exit code. ssh-keygen exits 0 on success, non-zero on failure.
    switch (run_result.term) {
        .Exited => |code| {
            if (code != 0) {
                const msg = if (run_result.stderr.len > 0) run_result.stderr else "ssh-keygen failed with no error output";
                writeError(result, msg);
                return false;
            }
        },
        .Signal => |sig| {
            const msg = try std.fmt.allocPrint(allocator, "ssh-keygen killed by signal {d}", .{sig});
            defer allocator.free(msg);
            writeError(result, msg);
            return false;
        },
        else => {
            writeError(result, "ssh-keygen terminated abnormally");
            return false;
        },
    }

    // Read the generated public key file (ssh-keygen appends ".pub")
    const pub_path = try std.fmt.allocPrint(allocator, "{s}.pub", .{out_path});
    defer allocator.free(pub_path);

    const pub_key_content = std.fs.cwd().readFileAlloc(pub_path, allocator, std.Io.Limit.limited(8192)) catch |err| {
        const msg = try std.fmt.allocPrint(allocator, "Failed to read public key file {s}: {s}", .{ pub_path, @errorName(err) });
        defer allocator.free(msg);
        writeError(result, msg);
        return false;
    };
    defer allocator.free(pub_key_content);

    // Trim trailing whitespace/newlines — ssh-keygen always adds a newline
    const trimmed_key = std.mem.trimRight(u8, pub_key_content, " \t\r\n");
    const key_copy_len = @min(trimmed_key.len, result.public_key.len - 1);
    @memcpy(result.public_key[0..key_copy_len], trimmed_key[0..key_copy_len]);
    result.public_key[key_copy_len] = 0; // null terminate
    result.public_key_len = key_copy_len;

    // Extract the fingerprint by running: ssh-keygen -l -E sha256 -f <path>.pub
    // The fingerprint format is: "256 SHA256:base64== email (KEY_TYPE)"
    // We extract just the "SHA256:base64==" portion.
    const fp_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = &[_][]const u8{ "ssh-keygen", "-l", "-E", "sha256", "-f", pub_path },
        .max_output_bytes = 1024,
    });
    defer allocator.free(fp_result.stdout);
    defer allocator.free(fp_result.stderr);

    // Fingerprint extraction is non-fatal — if it fails, the public key was
    // still generated successfully. We log the failure in the fingerprint field
    // with a placeholder so the Rust layer can distinguish "generated but no
    // fingerprint" from "generation failed".
    if (fp_result.term == .Exited and fp_result.term.Exited == 0) {
        if (std.mem.indexOf(u8, fp_result.stdout, "SHA256:")) |sha_start| {
            const fp_slice = fp_result.stdout[sha_start..];
            const fp_end = std.mem.indexOfScalar(u8, fp_slice, ' ') orelse fp_slice.len;
            const fp_str = fp_slice[0..fp_end];
            const fp_len = @min(fp_str.len, result.fingerprint.len - 1);
            @memcpy(result.fingerprint[0..fp_len], fp_str[0..fp_len]);
            result.fingerprint[fp_len] = 0;
            result.fingerprint_len = fp_len;
        } else {
            // ssh-keygen -l output format was unexpected — write what we got
            const msg = "fingerprint_parse_failed";
            const msg_len = @min(msg.len, result.fingerprint.len - 1);
            @memcpy(result.fingerprint[0..msg_len], msg[0..msg_len]);
            result.fingerprint_len = msg_len;
        }
    } else {
        const msg = "fingerprint_unavailable";
        const msg_len = @min(msg.len, result.fingerprint.len - 1);
        @memcpy(result.fingerprint[0..msg_len], msg[0..msg_len]);
        result.fingerprint_len = msg_len;
    }

    return true;
}

/// Writes an error message into the result buffer.
/// Truncates silently if the message exceeds the buffer size.
fn writeError(result: *KeygenResult, msg: []const u8) void {
    const copy_len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..copy_len], msg[0..copy_len]);
    result.error_message[copy_len] = 0; // null terminate
    result.error_len = copy_len;
}

// ── Unit tests ────────────────────────────────────────────────────────────────

test "KeygenResult has expected field sizes" {
    // Verify the struct layout matches our documentation and the Rust mirror.
    // If these sizes change, the Rust ffi.rs struct MUST be updated to match.
    try std.testing.expectEqual(@as(usize, 4096), @sizeOf([4096]u8));
    try std.testing.expectEqual(@as(usize, 256), @sizeOf([256]u8));
    try std.testing.expectEqual(@as(usize, 512), @sizeOf([512]u8));
    // The struct is extern so its size is deterministic:
    // 4096 + 8 + 256 + 8 + 512 + 8 = 4888 bytes on 64-bit systems
    const expected_size = 4096 + @sizeOf(usize) + 256 + @sizeOf(usize) + 512 + @sizeOf(usize);
    try std.testing.expectEqual(expected_size, @sizeOf(KeygenResult));
}

test "writeError truncates correctly at buffer boundary" {
    var result = KeygenResult{};
    // Write exactly buffer_size - 1 characters (leaves room for null terminator)
    const long_msg = "x" ** 511;
    writeError(&result, long_msg);
    try std.testing.expectEqual(@as(usize, 511), result.error_len);
    try std.testing.expectEqual(@as(u8, 0), result.error_message[511]);

    // Write a message longer than the buffer — should truncate to 511 chars
    var result2 = KeygenResult{};
    const too_long = "y" ** 600;
    writeError(&result2, too_long);
    try std.testing.expectEqual(@as(usize, 511), result2.error_len);
    try std.testing.expectEqual(@as(u8, 0), result2.error_message[511]);
}
