// zig_native/src/filesystem/atomic_write.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Provides crash-safe file writing using the write-to-temp-then-rename pattern.
// Also handles directory creation with correct permissions.
//
// ── Why atomicity matters for this project ────────────────────────────────────
// Three categories of files this function writes are critical to user safety:
//
//   1. ~/.ssh/config — if this file is corrupt, ALL SSH connections fail until
//      the user manually repairs it. A crash midway through a non-atomic write
//      could leave this file partially written with a syntax error.
//
//   2. ~/.git-manager/config.json — the app's own configuration. Corruption here
//      means the app cannot start until the user deletes and recreates the file.
//
//   3. ~/.git-manager/secrets.enc — the encrypted credential store (fallback mode).
//      A partially-written file here could cause credential loss.
//
// The rename() system call on POSIX (Linux, macOS) is atomic — it either succeeds
// completely or the filesystem is left in its original state. There is no
// intermediate state. This guarantee comes from the POSIX specification and is
// implemented correctly by all major filesystems (ext4, APFS, ZFS, tmpfs).
//
// ── Implementation details ────────────────────────────────────────────────────
// 1. Write the new content to a temporary file in the same directory as the target.
//    Same directory is important: rename() across different filesystems/partitions
//    is NOT atomic (it becomes a copy + delete). Using the same directory guarantees
//    same filesystem.
//
// 2. Call fsync() on the temp file to flush it from the page cache to disk.
//    Without fsync(), the rename could succeed but the data might not be on disk
//    yet. If power is lost after rename but before fsync, the file exists with
//    the new name but potentially empty or partial content.
//
// 3. Call rename() to atomically replace the target. At this point, the target
//    file is guaranteed to contain the new content or the old content — never
//    a mix of both.

const std = @import("std");

/// Standard result buffer for filesystem operations.
pub const FsResult = extern struct {
    error_message: [512]u8 = std.mem.zeroes([512]u8),
    error_len:     usize   = 0,
};

/// Atomically writes data to a file using temp-file + rename.
///
/// Parameters:
///   path   — Absolute path to the target file. Must not be NULL.
///   data   — Pointer to the bytes to write. Must point to at least `len` bytes.
///   len    — Number of bytes to write.
///   result — Caller-allocated output buffer for error details.
///
/// Thread safety: calling this function concurrently with different paths is safe.
/// Calling it concurrently with the SAME path may result in one call's data being
/// lost (the last rename wins). The Rust domain service serializes file writes.
///
/// The created file has permissions 0o600 (owner read/write only).
/// Use gm_fs_set_permissions() after this call if different permissions are needed.
pub export fn gm_fs_atomic_write(
    path:   [*:0]const u8,
    data:   [*]const u8,
    len:    usize,
    result: *FsResult,
) bool {
    return atomicWriteImpl(
        std.mem.span(path),
        data[0..len],
        result,
    ) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

fn atomicWriteImpl(target_path: []const u8, data: []const u8, result: *FsResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Build the temp path: same directory as target, with a unique suffix.
    // The timestamp-based suffix prevents collisions between concurrent calls.
    const temp_path = try std.fmt.allocPrint(
        allocator,
        "{s}.gm_write_{d}_{d}",
        .{ target_path, std.time.milliTimestamp(), std.Thread.getCurrentId() },
    );
    defer allocator.free(temp_path);

    // Write to the temp file. Any failure here leaves the target untouched.
    {
        const temp_file = std.fs.createFileAbsolute(temp_path, .{
            .mode      = 0o600,   // owner read/write; others cannot access
            .exclusive = true,    // fail if temp path already exists (paranoia)
        }) catch |err| {
            const msg = try std.fmt.allocPrint(allocator, "Cannot create temp file {s}: {s}", .{ temp_path, @errorName(err) });
            defer allocator.free(msg);
            writeError(result, msg);
            return false;
        };
        defer temp_file.close();

        temp_file.writeAll(data) catch |err| {
            // Clean up the partially-written temp file before returning
            std.fs.deleteFileAbsolute(temp_path) catch {};
            const msg = try std.fmt.allocPrint(allocator, "Write to temp file failed: {s}", .{@errorName(err)});
            defer allocator.free(msg);
            writeError(result, msg);
            return false;
        };

        // fsync flushes page cache to physical storage.
        // This is the call that makes the atomic write truly durable.
        // Omitting it means a power failure after rename could corrupt the file.
        temp_file.sync() catch |err| {
            std.fs.deleteFileAbsolute(temp_path) catch {};
            const msg = try std.fmt.allocPrint(allocator, "fsync failed: {s}", .{@errorName(err)});
            defer allocator.free(msg);
            writeError(result, msg);
            return false;
        };
    } // temp_file.close() is called here by defer

    // Atomic rename. If this fails, the original target is intact.
    std.fs.renameAbsolute(temp_path, target_path) catch |err| {
        std.fs.deleteFileAbsolute(temp_path) catch {};
        const msg = try std.fmt.allocPrint(allocator, "Rename failed: {s}", .{@errorName(err)});
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

test "FsResult layout" {
    try std.testing.expectEqual(512 + @sizeOf(usize), @sizeOf(FsResult));
}