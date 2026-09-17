// zig_native/src/ssh/config_writer.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Manages Host blocks in the user's ~/.ssh/config file. The SSH client reads
// this file to know which private key to use for which hostname alias, which
// is the mechanism that allows one machine to authenticate as different
// GitHub accounts simultaneously (github.com-work vs github.com-personal).
//
// Two exported functions:
//   gm_ssh_write_config_entry() — appends a Host block if it does not exist
//   gm_ssh_remove_config_entry() — removes a Host block by alias
//
// ── The ~/.ssh/config Host block format ───────────────────────────────────────
// Each account gets one Host block like this:
//
//   Host github.com-work
//     HostName github.com
//     User git
//     IdentityFile /home/user/.ssh/id_ed25519_work
//     IdentitiesOnly yes
//     AddKeysToAgent yes
//     StrictHostKeyChecking accept-new
//     Port 22
//
// The "Host" alias is what the user puts in git remote URLs:
//   git@github.com-work:owner/repo.git   (instead of git@github.com:...)
// SSH resolves "github.com-work" via this config to use the right key.
//
// ── Why atomic writes are critical here ───────────────────────────────────────
// The ~/.ssh/config file is read by the SSH client on every connection attempt.
// If we write directly to the file and the process crashes midway, the file
// could be left in a partially-written state, breaking ALL SSH connections for
// the user until they manually fix the file. Atomic write (write-to-temp then
// rename) prevents this: the rename operation is atomic at the OS level, so
// the file is always either the old complete version or the new complete version.
//
// ── Duplicate detection ───────────────────────────────────────────────────────
// Before appending, we scan the existing config for a Host line that matches
// the alias. If found, we skip the write and return success. This makes the
// function idempotent — calling it multiple times with the same alias is safe.

const std = @import("std");

/// Result buffer for filesystem operations.
/// Matches #[repr(C)] struct FfiFsResult in gm_adapters/src/filesystem/ffi.rs.
pub const FsResult = extern struct {
    error_message: [512]u8 = std.mem.zeroes([512]u8),
    error_len: usize = 0,
};

/// Appends a Host block to ~/.ssh/config for the given host alias.
///
/// Parameters:
///   host_alias     — The SSH host alias: "github.com-work"
///   hostname       — The actual hostname: "github.com"
///   identity_file  — Absolute path to the private key: "/home/user/.ssh/id_ed25519_work"
///   port           — SSH port (usually 22; use 443 for GitHub over HTTPS port)
///   result         — Caller-allocated output buffer.
///
/// Idempotent: if a Host block with the same alias already exists, this function
/// returns true without modifying the file.
///
/// Thread safety: NOT safe for concurrent calls with the same or overlapping
/// host aliases because the read-check-write sequence is not atomic at the
/// file level. The Rust domain service serializes calls to this function.
pub export fn gm_ssh_write_config_entry(
    host_alias: [*:0]const u8,
    hostname: [*:0]const u8,
    identity_file: [*:0]const u8,
    port: u16,
    result: *FsResult,
) bool {
    return writeConfigEntryImpl(
        std.mem.span(host_alias),
        std.mem.span(hostname),
        std.mem.span(identity_file),
        port,
        result,
    ) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

fn writeConfigEntryImpl(
    host_alias: []const u8,
    hostname: []const u8,
    identity_file: []const u8,
    port: u16,
    result: *FsResult,
) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Resolve the SSH config file path.
    // We do not hardcode ~/.ssh/config because ~ is a shell expansion, not
    // an OS concept. We resolve it via the HOME environment variable.
    const home = std.posix.getenv("HOME") orelse {
        writeError(result, "HOME environment variable is not set");
        return false;
    };
    const config_path = try std.fmt.allocPrint(allocator, "{s}/.ssh/config", .{home});
    defer allocator.free(config_path);

    // Ensure ~/.ssh/ exists with the correct permissions (0700 required by OpenSSH)
    const ssh_dir = try std.fmt.allocPrint(allocator, "{s}/.ssh", .{home});
    defer allocator.free(ssh_dir);
    std.fs.makeDirAbsolute(ssh_dir) catch |err| switch (err) {
        error.PathAlreadyExists => {}, // fine — directory already exists
        else => return err,
    };
    // Set directory permissions to 0700 regardless of current state
    try std.posix.fchmodat(std.posix.AT.FDCWD, ssh_dir, 0o700, 0);

    // Read existing config (empty string if the file does not yet exist)
    const existing_content = std.fs.cwd().readFileAlloc(config_path, allocator, std.Io.Limit.limited(1024 * 1024)) catch |err| switch (err) {
        error.FileNotFound => try allocator.dupe(u8, ""),
        else => return err,
    };
    defer allocator.free(existing_content);

    // Check for duplicate: scan line by line for "Host <host_alias>"
    // We match exactly on the host alias with word boundaries to avoid
    // "github.com-work" matching "github.com-work-old".
    var lines = std.mem.splitScalar(u8, existing_content, '\n');
    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t\r");
        // A Host line looks like: "Host alias1 alias2 ..." (space-separated)
        if (std.mem.startsWith(u8, trimmed, "Host ") or std.mem.eql(u8, trimmed, "Host")) {
            const after_host = std.mem.trimLeft(u8, trimmed["Host".len..], " \t");
            // Check each alias on this Host line
            var aliases = std.mem.splitScalar(u8, after_host, ' ');
            while (aliases.next()) |alias| {
                if (std.mem.eql(u8, alias, host_alias)) {
                    // This alias already exists — idempotent success
                    return true;
                }
            }
        }
    }

    // Build the new Host block to append.
    // We always add a blank line before the block for readability.
    // The IdentitiesOnly yes option is critical: it prevents SSH from trying
    // other keys in the agent before using our specified key, which would
    // cause authentication failures when multiple GitHub keys are loaded.
    const new_block = try std.fmt.allocPrint(allocator,
        \\
        \\# Git Manager: {s}
        \\Host {s}
        \\  HostName {s}
        \\  User git
        \\  IdentityFile {s}
        \\  IdentitiesOnly yes
        \\  AddKeysToAgent yes
        \\  StrictHostKeyChecking accept-new
        \\  Port {d}
        \\
    , .{ host_alias, host_alias, hostname, identity_file, port });
    defer allocator.free(new_block);

    // Combine existing content with the new block
    const new_content = try std.mem.concat(allocator, u8, &[_][]const u8{ existing_content, new_block });
    defer allocator.free(new_content);

    // Atomic write: write to a temp file then rename.
    // rename() is atomic on POSIX systems — the file transitions instantly
    // from old to new with no intermediate state visible to other processes.
    const temp_path = try std.fmt.allocPrint(allocator, "{s}.gm_tmp_{d}", .{ config_path, std.time.milliTimestamp() });
    defer allocator.free(temp_path);

    {
        var temp_file = try std.fs.createFileAbsolute(temp_path, .{ .mode = 0o600 });
        defer temp_file.close();
        try temp_file.writeAll(new_content);
        // Flush to ensure all bytes are in the file before rename
        try temp_file.sync();
    }

    // Atomic rename: replaces config_path with temp_path in one kernel operation
    try std.fs.renameAbsolute(temp_path, config_path);

    return true;
}

/// Removes a Host block from ~/.ssh/config by its alias.
///
/// The removal is exact-match on the alias name. Only the block belonging to
/// this specific alias is removed; other Host blocks are preserved intact.
/// The write is atomic (temp file + rename pattern).
///
/// Returns: true if removed or if the alias was not found.
/// Returns: false only if reading/writing the config file fails.
pub export fn gm_ssh_remove_config_entry(
    host_alias: [*:0]const u8,
    result: *FsResult,
) bool {
    return removeConfigEntryImpl(std.mem.span(host_alias), result) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

fn removeConfigEntryImpl(host_alias: []const u8, result: *FsResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const home = std.posix.getenv("HOME") orelse {
        writeError(result, "HOME environment variable is not set");
        return false;
    };
    const config_path = try std.fmt.allocPrint(allocator, "{s}/.ssh/config", .{home});
    defer allocator.free(config_path);

    const existing = std.fs.cwd().readFileAlloc(config_path, allocator, std.Io.Limit.limited(1024 * 1024)) catch |err| switch (err) {
        error.FileNotFound => return true, // nothing to remove
        else => return err,
    };
    defer allocator.free(existing);

    // Parse the file into blocks separated by blank lines before "Host" keywords.
    // We rebuild the file with the target block omitted.
    var output = std.ArrayList(u8).empty;
    defer output.deinit(allocator);

    var in_target_block = false;
    var lines = std.mem.splitScalar(u8, existing, '\n');

    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t\r");

        if (std.mem.startsWith(u8, trimmed, "Host ") or std.mem.eql(u8, trimmed, "Host")) {
            const after_host = std.mem.trimLeft(u8, trimmed["Host".len..], " \t");
            // Check if this Host line contains our target alias
            var aliases = std.mem.splitScalar(u8, after_host, ' ');
            var found = false;
            while (aliases.next()) |alias| {
                if (std.mem.eql(u8, alias, host_alias)) {
                    found = true;
                    break;
                }
            }
            in_target_block = found;
        } else if (!std.mem.startsWith(u8, trimmed, " ") and !std.mem.startsWith(u8, trimmed, "\t") and trimmed.len > 0 and !std.mem.startsWith(u8, trimmed, "#")) {
            // Non-indented, non-comment, non-empty line that is not a Host line
            // signals the start of a new top-level block. Exit target block mode.
            in_target_block = false;
        }

        if (!in_target_block) {
            try output.appendSlice(allocator, line);
            try output.append(allocator, '\n');
        }
    }

    // Atomic write of the filtered content
    const temp_path = try std.fmt.allocPrint(allocator, "{s}.gm_tmp_{d}", .{ config_path, std.time.milliTimestamp() });
    defer allocator.free(temp_path);

    {
        var temp_file = try std.fs.createFileAbsolute(temp_path, .{ .mode = 0o600 });
        defer temp_file.close();
        try temp_file.writeAll(output.items);
        try temp_file.sync();
    }

    try std.fs.renameAbsolute(temp_path, config_path);
    return true;
}

fn writeError(result: *FsResult, msg: []const u8) void {
    const copy_len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..copy_len], msg[0..copy_len]);
    result.error_message[copy_len] = 0;
    result.error_len = copy_len;
}

test "FsResult layout" {
    const expected = 512 + @sizeOf(usize);
    try std.testing.expectEqual(expected, @sizeOf(FsResult));
}
