// zig_native/src/git_exec/output_parser.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Parses raw text output from git subprocesses into structured data that
// the Rust domain layer can work with directly. All functions here are pure
// (no side effects, no allocations beyond what the caller provides) which
// makes them trivially unit-testable without spawning any processes.
//
// ── Why parse git output instead of using libgit2? ───────────────────────────
// We delegate to the git binary (rather than libgit2 bindings) for clone, push,
// and pull because those operations involve network I/O and SSH authentication
// through our custom GIT_SSH_COMMAND. libgit2 has its own SSH transport that
// bypasses our key isolation mechanism. Parsing the text output of the git
// binary is a deliberate trade-off: we lose some type safety but gain correct
// behavior with our SSH environment setup.
//
// The porcelain format (`--porcelain`) is machine-readable and stable across
// git versions. We always use porcelain format for operations where we need
// to parse the output. We never parse git's human-readable output because it
// changes between versions and is localized in different languages.
//
// ── Output format reference ───────────────────────────────────────────────────
// git status --porcelain=v1 format (two-character XY code per file):
//   ' M' = modified in work tree, not staged
//   'M ' = modified and staged
//   'MM' = modified staged AND modified unstaged
//   'A ' = new file staged
//   '??' = untracked
//   'D ' = deleted staged
//   ' D' = deleted unstaged
//   'R ' = renamed staged
//
// git push output (human readable but consistent for our purposes):
//   branch -> branch (progress)
//   [new branch]     (first push of a branch)
//   [rejected]       (push was rejected)
//
// git clone progress (written to stderr):
//   Cloning into 'repo'...
//   remote: Enumerating objects: N, done.
//   Receiving objects: N% (M/K)...
//   Resolving deltas: N%...

const std = @import("std");

/// A parsed file from git status --porcelain output.
pub const StatusEntry = struct {
    /// The two-character XY status code
    xy: [2]u8,
    /// The file path (may include -> for renames)
    path: []const u8,

    pub fn is_staged(self: StatusEntry) bool {
        return self.xy[0] != ' ' and self.xy[0] != '?';
    }

    pub fn is_unstaged(self: StatusEntry) bool {
        return self.xy[1] != ' ' and self.xy[1] != '?';
    }

    pub fn is_untracked(self: StatusEntry) bool {
        return self.xy[0] == '?' and self.xy[1] == '?';
    }
};

/// Parses `git status --porcelain=v1` output into a list of StatusEntry values.
///
/// Parameters:
///   output    — Raw output from `git status --porcelain=v1`
///   allocator — Used to allocate the returned slice; caller must free it.
///
/// Returns: A slice of StatusEntry, or an allocation error.
///
/// Notes on the format: Each line is "XY path" where XY is a two-character
/// code and path follows a space. For renames, path looks like "oldpath -> newpath".
/// We do not split renames here — the caller receives the full path string.
pub fn parseStatusPorcelain(output: []const u8, allocator: std.mem.Allocator) ![]StatusEntry {
    var entries = std.ArrayList(StatusEntry).empty;
    errdefer entries.deinit(allocator);

    var lines = std.mem.splitScalar(u8, output, '\n');
    while (lines.next()) |line| {
        if (line.len < 4) continue; // need at least "XY p" (2 code + space + 1 char path)

        const xy = [2]u8{ line[0], line[1] };
        // line[2] is always a space separator
        const path = line[3..];

        if (path.len == 0) continue;

        try entries.append(allocator, .{ .xy = xy, .path = path });
    }

    return entries.toOwnedSlice(allocator);
}

/// Extracts the commit SHA from `git log --format=%H -1` output.
/// Returns the first 40-character hex string found, or null if not found.
pub fn parseCommitSha(output: []const u8) ?[]const u8 {
    const trimmed = std.mem.trim(u8, output, " \t\r\n");
    if (trimmed.len < 40) return null;

    // Validate that the first 40 characters are all hex digits
    for (trimmed[0..40]) |c| {
        if (!std.ascii.isHex(c)) return null;
    }

    return trimmed[0..40];
}

/// Result of parsing push output.
pub const PushParseResult = struct {
    commits_pushed: u32 = 0,
    was_rejected: bool = false,
    remote_branch: []const u8 = "",
    is_new_branch: bool = false,
};

/// Extracts push statistics from git push output (typically stderr from git push).
///
/// Git push output is technically human-readable but consistent enough to parse.
/// We look for the summary line that appears at the end of push output.
pub fn parsePushOutput(output: []const u8) PushParseResult {
    var result = PushParseResult{};

    if (std.mem.indexOf(u8, output, "[rejected]") != null) {
        result.was_rejected = true;
        return result;
    }

    if (std.mem.indexOf(u8, output, "[new branch]") != null) {
        result.is_new_branch = true;
    }

    // Look for the SHA range in the push summary: "abc123..def456  main -> main"
    // The ".." indicates commits were pushed. Count the range size from `git log`.
    // Since we cannot count from push output alone, we default to 1 if we see
    // a successful push reference update. The exact count is tracked by the domain
    // layer which queries git log after the push.
    if (std.mem.indexOf(u8, output, "->") != null) {
        // A successful reference update occurred
        if (!result.was_rejected) {
            result.commits_pushed = 1; // minimum — actual count tracked elsewhere
        }
    }

    return result;
}

/// Extracts the "Cloning into 'name'" repo name from git clone stderr output.
/// Returns null if the pattern is not found.
pub fn parseCloneRepoName(stderr_output: []const u8) ?[]const u8 {
    const prefix = "Cloning into '";
    const start = std.mem.indexOf(u8, stderr_output, prefix) orelse return null;
    const after_prefix = stderr_output[start + prefix.len ..];
    const end = std.mem.indexOfScalar(u8, after_prefix, '\'') orelse return null;
    return after_prefix[0..end];
}

/// Counts staged, unstaged, and untracked files from a parsed status.
pub const FileCounts = struct {
    staged: u32,
    unstaged: u32,
    untracked: u32,
};

pub fn countFiles(entries: []const StatusEntry) FileCounts {
    var counts = FileCounts{ .staged = 0, .unstaged = 0, .untracked = 0 };
    for (entries) |entry| {
        if (entry.is_untracked()) {
            counts.untracked += 1;
        } else {
            if (entry.is_staged()) counts.staged += 1;
            if (entry.is_unstaged()) counts.unstaged += 1;
        }
    }
    return counts;
}

// ── Unit tests ────────────────────────────────────────────────────────────────

test "parseStatusPorcelain parses modified files" {
    const allocator = std.testing.allocator;
    const input =
        " M src/main.rs\n" ++
        "M  src/lib.rs\n" ++
        "?? Cargo.lock\n";

    const entries = try parseStatusPorcelain(input, allocator);
    defer allocator.free(entries);

    try std.testing.expectEqual(@as(usize, 3), entries.len);
    // " M" — unstaged modification
    try std.testing.expect(!entries[0].is_staged());
    try std.testing.expect(entries[0].is_unstaged());
    // "M " — staged modification
    try std.testing.expect(entries[1].is_staged());
    try std.testing.expect(!entries[1].is_unstaged());
    // "??" — untracked
    try std.testing.expect(entries[2].is_untracked());
}

test "parseStatusPorcelain returns empty slice for clean repo" {
    const allocator = std.testing.allocator;
    const entries = try parseStatusPorcelain("", allocator);
    defer allocator.free(entries);
    try std.testing.expectEqual(@as(usize, 0), entries.len);
}

test "parseCommitSha extracts 40-char hex" {
    const sha = parseCommitSha("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2\n");
    try std.testing.expect(sha != null);
    try std.testing.expectEqualStrings("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2", sha.?);
}

test "parseCommitSha returns null for non-SHA input" {
    try std.testing.expect(parseCommitSha("not a sha") == null);
    try std.testing.expect(parseCommitSha("") == null);
    try std.testing.expect(parseCommitSha("g1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2") == null); // 'g' not hex
}

test "parsePushOutput detects rejection" {
    const output = "To github.com:user/repo.git\n ! [rejected]  main -> main (non-fast-forward)\n";
    const result = parsePushOutput(output);
    try std.testing.expect(result.was_rejected);
    try std.testing.expectEqual(@as(u32, 0), result.commits_pushed);
}

test "parseCloneRepoName extracts name" {
    const stderr = "Cloning into 'my-repo'...\n";
    const name = parseCloneRepoName(stderr);
    try std.testing.expect(name != null);
    try std.testing.expectEqualStrings("my-repo", name.?);
}
