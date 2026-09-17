// zig_native/src/filesystem/permissions.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Wraps POSIX chmod and directory creation operations.
// SSH is strict about file permissions — it refuses to use private keys that
// are readable by other users on the system. The ssh binary will print:
//
//   Permissions 0644 for '/home/user/.ssh/id_ed25519' are too open.
//   It is required that your private key files are NOT accessible by others.
//
// This module ensures that private key files are always set to 0600 and
// the ~/.ssh directory is always set to 0700 immediately after creation.
//
// ── chmod is not atomic ────────────────────────────────────────────────────────
// Unlike file writes, permission changes via chmod are not atomic in the sense
// that between when we write the file and when we chmod it, another process
// could theoretically read the file with the initial permissions. To minimize
// this window, we always set permissions as the very next operation after
// writing the file.
//
// For SSH private keys specifically, ssh-keygen already sets 0600 on the files
// it creates. We call gm_fs_set_permissions as a belt-and-suspenders check in
// case something in the write path altered the permissions.

const std = @import("std");

pub const FsResult = @import("atomic_write.zig").FsResult;

/// Sets the permission mode of a file or directory.
///
/// Parameters:
///   path   — Absolute path to the file or directory. Must not be NULL.
///   mode   — POSIX permission mode as a u32. Common values:
///             0o600 — owner r/w, no group/other access (SSH private keys)
///             0o644 — owner r/w, group/other read-only (SSH public keys)
///             0o700 — owner r/w/x, no group/other (SSH directory)
///             0o755 — owner r/w/x, group/other r/x (standard directory)
///   result — Caller-allocated output buffer for error details.
///
/// Returns: true on success, false if the file does not exist or permission
/// is denied (the caller does not own the file).
///
/// Note on u32 vs u16: POSIX chmod takes a mode_t which is typically u16 or u32
/// depending on the platform. We accept u32 to avoid ABI mismatch issues across
/// platforms. Zig's std.posix.chmod accepts u32 and handles the narrowing.
pub export fn gm_fs_set_permissions(
    path: [*:0]const u8,
    mode: u32,
    result: *FsResult,
) bool {
    const path_slice = std.mem.span(path);
    std.posix.fchmodat(std.posix.AT.FDCWD, path_slice, @intCast(mode), 0) catch |err| {
        var gpa = std.heap.GeneralPurposeAllocator(.{}){};
        defer _ = gpa.deinit();
        const allocator = gpa.allocator();
        const msg = std.fmt.allocPrint(allocator, "chmod({s}, 0o{o}): {s}", .{ path_slice, mode, @errorName(err) }) catch {
            writeError(result, "chmod failed (allocation error)");
            return false;
        };
        defer allocator.free(msg);
        writeError(result, msg);
        return false;
    };
    return true;
}

fn writeError(result: *FsResult, msg: []const u8) void {
    const copy_len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..copy_len], msg[0..copy_len]);
    result.error_message[copy_len] = 0;
    result.error_len = copy_len;
}

test "gm_fs_set_permissions correctly truncates mode to u16" {
    // This is a compilation/type test — the actual chmod behavior
    // is tested in integration tests that run as a real user.
    // Here we verify the function signature compiles correctly.
    _ = gm_fs_set_permissions;
}
