// zig_native/src/filesystem/disk_space.zig
//
// Disk space checking using the POSIX statvfs system call.
// Returns available bytes for the filesystem containing the given path.

const std = @import("std");

const c = @cImport({
    @cInclude("sys/statvfs.h");
});

/// Returns the number of available bytes on the filesystem containing `path`.
/// Returns 0 on error (path doesn't exist, permission denied, etc.).
pub export fn gm_fs_available_disk_space(path: [*:0]const u8) u64 {
    var stat: c.struct_statvfs = undefined;
    const rc = c.statvfs(path, &stat);
    if (rc != 0) return 0;

    // f_frsize = fundamental filesystem block size (preferred for space calc)
    // f_bavail = number of free blocks available to unprivileged user
    return @as(u64, @intCast(stat.f_frsize)) * @as(u64, @intCast(stat.f_bavail));
}

/// Checks whether at least `required_bytes` are available on the filesystem
/// containing `path`. Returns 0 if sufficient, -1 if insufficient, -2 on error.
pub export fn gm_fs_ensure_disk_space(path: [*:0]const u8, required_bytes: u64) i32 {
    const avail = gm_fs_available_disk_space(path);
    if (avail == 0) return -2; // error (path not found, etc.)
    if (avail < required_bytes) return -1; // insufficient
    return 0; // ok
}

test "disk_space works for root" {
    const avail = gm_fs_available_disk_space("/");
    try std.testing.expect(avail > 0);
}

test "ensure_disk_space passes for root with small requirement" {
    const rc = gm_fs_ensure_disk_space("/", 1024);
    try std.testing.expectEqual(@as(i32, 0), rc);
}

test "ensure_disk_space fails for nonexistent path" {
    const rc = gm_fs_ensure_disk_space("/nonexistent_path_xyz", 1024);
    try std.testing.expectEqual(@as(i32, -2), rc);
}
