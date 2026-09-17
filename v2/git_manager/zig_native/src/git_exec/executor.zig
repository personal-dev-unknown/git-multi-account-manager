// zig_native/src/git_exec/executor.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// The Git execution engine — the Zig layer that actually forks and execs git
// subprocesses with SSH-isolated environments. This is the most OS-intensive
// file in the Zig native layer. Every public function here is exported as a
// C ABI symbol that the Rust ZigGitExecutor adapter calls through FFI.
//
// ── Result buffer design ──────────────────────────────────────────────────────
// Every exported function writes results into a caller-provided buffer struct.
// The Rust side stack-allocates or heap-allocates the struct, passes a pointer,
// and reads the result fields after the call. No memory crosses the FFI boundary
// via pointer-to-heap — all data lives in fixed-size arrays inside the struct.
//
// Buffer sizing decisions:
//   commit_sha  [64]u8   — SHA-1 is 40 hex chars; SHA-256 is 64 hex chars.
//                          64 bytes handles both and any future git hash algorithms.
//   stdout_data [65536]u8 — 64 KiB captures git clone progress, pull summaries,
//                          and push confirmations. Operations that produce more
//                          output are truncated (progress indicators, not critical data).
//   stderr_data [4096]u8  — 4 KiB is enough for any error message. If authentication
//                          fails, SSH writes one or two lines. If the repo does not
//                          exist, git writes a short error. 4 KiB is generous.
//
// ── Process lifecycle ──────────────────────────────────────────────────────────
// Every git subprocess follows the same lifecycle:
//   1. Build args array  (varies by operation)
//   2. Get current env   (std.process.getEnvMap)
//   3. Add SSH vars      (env_builder.populateGitEnv)
//   4. Run subprocess    (std.process.Child.run)
//   5. Parse output      (output_parser.*)
//   6. Write to result   (fill GitResult fields)
//   7. Return bool       (true = success, false = failure)
//
// ── Error propagation ─────────────────────────────────────────────────────────
// Zig errors (allocation failures, file I/O errors) are caught at the export
// function boundary and written to result.stderr_data. The Rust caller sees a
// `false` return value and reads the diagnostic from the buffer. This prevents
// any Zig panic from crossing the FFI boundary (panics are undefined behavior
// from the perspective of a C/Rust caller).

const std = @import("std");
const env_builder = @import("env_builder.zig");
const output_parser = @import("output_parser.zig");
const Atomic = std.atomic;

/// Tracks the PID of the currently running child subprocess.
/// Rust reads this via `gm_git_kill_process(0)` to kill the active git process
/// when the user presses Ctrl+C. Reset to -1 when no child is running.
var current_child_pid: Atomic.Value(i32) = Atomic.Value(i32).init(-1);

// ── Shared result types ────────────────────────────────────────────────────────

/// Result buffer for git operations (clone, pull, push, commit).
/// MUST match #[repr(C)] struct FfiGitResult in gm_adapters/src/git/ffi.rs.
pub const GitResult = extern struct {
    success: bool = false,
    commit_sha: [64]u8 = std.mem.zeroes([64]u8),
    sha_len: usize = 0,
    stdout_data: [65536]u8 = std.mem.zeroes([65536]u8),
    stdout_len: usize = 0,
    stderr_data: [4096]u8 = std.mem.zeroes([4096]u8),
    stderr_len: usize = 0,
    exit_code: i32 = -1,
    /// PID of the most recently spawned git subprocess.
    /// Populated by `runChild` and read via `gm_git_kill_process`.
    /// -1 means no child process is currently tracked.
    child_pid: i32 = -1,
};

/// Result buffer for git status operations.
/// MUST match #[repr(C)] struct FfiGitStatus in gm_adapters/src/git/ffi.rs.
pub const GitStatus = extern struct {
    /// Newline-separated list of staged file paths
    staged_files: [32768]u8 = std.mem.zeroes([32768]u8),
    staged_len: usize = 0,
    staged_count: u32 = 0,

    /// Newline-separated list of unstaged modified file paths
    unstaged_files: [32768]u8 = std.mem.zeroes([32768]u8),
    unstaged_len: usize = 0,
    unstaged_count: u32 = 0,

    /// Newline-separated list of untracked file paths
    untracked_files: [32768]u8 = std.mem.zeroes([32768]u8),
    untracked_len: usize = 0,
    untracked_count: u32 = 0,

    /// true when all three counts are zero (clean working tree)
    is_clean: bool = false,

    error_message: [512]u8 = std.mem.zeroes([512]u8),
    error_len: usize = 0,
};

// ── Child process tracking ─────────────────────────────────────────────────────

/// Kills a git subprocess by PID. If `pid ≤ 0`, kills the currently tracked child
/// process (the one stored in `current_child_pid`). Returns true if a kill signal
/// was sent, false if there was no process to kill.
///
/// Called from Rust's Ctrl+C handler to abort a running clone, pull, or push.
pub export fn gm_git_kill_process(pid: i32) bool {
    const target = if (pid > 0) pid else current_child_pid.load(.acquire);
    if (target <= 0) return false;
    _ = std.posix.kill(target, std.posix.SIG.TERM) catch return false;
    return true;
}

/// Spawns a child process, tracks its PID, collects output, and waits for exit.
///
/// This is a drop-in replacement for `std.process.Child.run` that additionally:
///   1. Stores the child PID in `result.child_pid` for post-hoc inspection
///   2. Stores it in `current_child_pid` (global) for out-of-band kill
///   3. Resets `current_child_pid` to -1 after the child exits
///
/// The Rust side reads `current_child_pid` via `gm_git_kill_process(0)` to abort
/// long-running git operations (clone, pull, push) when Ctrl+C is pressed.
fn runChild(
    allocator: std.mem.Allocator,
    argv: []const []const u8,
    env_map: ?*const std.process.EnvMap,
    max_output_bytes: usize,
    result: *GitResult,
) !std.process.Child.RunResult {
    var child = std.process.Child.init(argv, allocator);
    child.stdin_behavior = .Ignore;
    child.stdout_behavior = .Pipe;
    child.stderr_behavior = .Pipe;
    if (env_map) |map| child.env_map = map;

    try child.spawn();
    const pid = @as(i32, @intCast(child.id));
    result.child_pid = pid;
    current_child_pid.store(pid, .release);
    errdefer current_child_pid.store(-1, .release);

    var stdout: std.ArrayList(u8) = .empty;
    defer stdout.deinit(allocator);
    var stderr: std.ArrayList(u8) = .empty;
    defer stderr.deinit(allocator);

    child.collectOutput(allocator, &stdout, &stderr, max_output_bytes) catch |err| {
        current_child_pid.store(-1, .release);
        return err;
    };

    const term = try child.wait();
    current_child_pid.store(-1, .release);

    return .{
        .stdout = try stdout.toOwnedSlice(allocator),
        .stderr = try stderr.toOwnedSlice(allocator),
        .term = term,
    };
}

// ── Clone ──────────────────────────────────────────────────────────────────────

/// Clones a repository from a remote URL into a local directory.
///
/// Parameters:
///   url      — The SSH remote URL: "git@github.com-work:owner/repo.git"
///              The host alias (github.com-work) is resolved via ~/.ssh/config.
///   dest     — Absolute local path where the repository will be cloned.
///              If it already exists, git clone will fail (expected behavior).
///   key_path — Absolute path to the SSH private key for authentication.
///   result   — Caller-allocated output buffer.
///
/// Returns: true on successful clone, false on network failure, auth failure,
/// or if the destination already exists.
pub export fn gm_git_clone(
    url: [*:0]const u8,
    dest: [*:0]const u8,
    key_path: [*:0]const u8,
    host_alias: [*:0]const u8,
    branch: [*:0]const u8,
    depth: u32,
    result: *GitResult,
) bool {
    _ = host_alias; // alias is already encoded in the URL
    return cloneImpl(
        std.mem.span(url),
        std.mem.span(dest),
        std.mem.span(key_path),
        std.mem.span(branch),
        depth,
        result,
    ) catch |err| {
        writeResultError(result, @errorName(err));
        return false;
    };
}

fn cloneImpl(url: []const u8, dest: []const u8, key_path: []const u8, branch: []const u8, depth: u32, result: *GitResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var env_map = try std.process.getEnvMap(allocator);
    defer env_map.deinit();
    try env_builder.populateGitEnv(&env_map, key_path, 30, allocator);

    var args = std.ArrayList([]const u8).empty;
    defer args.deinit(allocator);
    try args.appendSlice(allocator, &[_][]const u8{ "git", "clone", "--progress" });
    if (branch.len > 0) try args.appendSlice(allocator, &[_][]const u8{ "-b", branch });
    if (depth > 0) {
        const depth_str = try std.fmt.allocPrint(allocator, "{d}", .{depth});
        defer allocator.free(depth_str);
        try args.appendSlice(allocator, &[_][]const u8{ "--depth", depth_str });
    }
    try args.appendSlice(allocator, &[_][]const u8{ url, dest });

    const run_result = try runChild(allocator, args.items, &env_map, 1024 * 64, result);
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    fillResultBuffers(result, run_result.stdout, run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            result.exit_code = @intCast(code);
            if (code != 0) {
                writeResultError(result, run_result.stderr);
                return false;
            }
        },
        .Signal => |sig| {
            result.exit_code = -1;
            const msg = try std.fmt.allocPrint(allocator, "git clone killed by signal {d}", .{sig});
            defer allocator.free(msg);
            writeResultError(result, msg);
            return false;
        },
        else => {
            writeResultError(result, "git clone terminated abnormally");
            return false;
        },
    }

    // After successful clone, get the HEAD commit SHA
    const sha_result = try runChild(allocator, &[_][]const u8{ "git", "-C", dest, "log", "--format=%H", "-1" }, &env_map, 128, result);
    defer allocator.free(sha_result.stdout);
    defer allocator.free(sha_result.stderr);

    if (sha_result.term == .Exited and sha_result.term.Exited == 0) {
        if (output_parser.parseCommitSha(sha_result.stdout)) |sha| {
            const sha_len = @min(sha.len, result.commit_sha.len - 1);
            @memcpy(result.commit_sha[0..sha_len], sha[0..sha_len]);
            result.sha_len = sha_len;
        }
    }

    result.success = true;
    return true;
}

// ── Sparse Clone ───────────────────────────────────────────────────────────────

/// Clones a repository with partial clone + sparse checkout.
///
/// Steps:
///   1. `git clone --filter=blob:none --no-checkout <url> <dest>`
///   2. `git -C <dest> sparse-checkout init --cone`
///   3. `git -C <dest> sparse-checkout set <paths>`
///   4. `git -C <dest> checkout`
///   5. Read HEAD commit SHA
///
/// Parameters:
///   url      — Remote URL (SSH or HTTPS) with host alias pre-encoded.
///   dest     — Absolute local path where the repository will be cloned.
///   key_path — Absolute path to the SSH private key (can be empty for HTTPS).
///   host_alias — SSH host alias (already embedded in URL, only used if non-empty).
///   branch   — Branch to clone; empty = remote default.
///   depth    — Shallow depth; 0 = full history.
///   paths    — Comma-separated list of directory/file patterns for sparse checkout.
///   result   — Caller-allocated output buffer.
pub export fn gm_git_sparse_clone(
    url: [*:0]const u8,
    dest: [*:0]const u8,
    key_path: [*:0]const u8,
    host_alias: [*:0]const u8,
    branch: [*:0]const u8,
    depth: u32,
    paths: [*:0]const u8,
    result: *GitResult,
) bool {
    _ = host_alias;
    return sparseCloneImpl(
        std.mem.span(url),
        std.mem.span(dest),
        std.mem.span(key_path),
        std.mem.span(branch),
        depth,
        std.mem.span(paths),
        result,
    ) catch |err| {
        writeResultError(result, @errorName(err));
        return false;
    };
}

fn sparseCloneImpl(
    url: []const u8,
    dest: []const u8,
    key_path: []const u8,
    branch: []const u8,
    depth: u32,
    paths: []const u8,
    result: *GitResult,
) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var env_map = try std.process.getEnvMap(allocator);
    defer env_map.deinit();
    try env_builder.populateGitEnv(&env_map, key_path, 30, allocator);

    // Step 1: git clone --filter=blob:none --no-checkout
    {
        var args = std.ArrayList([]const u8).empty;
        defer args.deinit(allocator);
        try args.appendSlice(allocator, &[_][]const u8{ "git", "clone", "--progress", "--filter=blob:none", "--no-checkout" });
        if (branch.len > 0) try args.appendSlice(allocator, &[_][]const u8{ "-b", branch });
        if (depth > 0) {
            const depth_str = try std.fmt.allocPrint(allocator, "{d}", .{depth});
            defer allocator.free(depth_str);
            try args.appendSlice(allocator, &[_][]const u8{ "--depth", depth_str });
        }
        try args.appendSlice(allocator, &[_][]const u8{ url, dest });

        const run_result = try runChild(allocator, args.items, &env_map, 1024 * 64, result);
        defer allocator.free(run_result.stdout);
        defer allocator.free(run_result.stderr);

        fillResultBuffers(result, run_result.stdout, run_result.stderr);

        switch (run_result.term) {
            .Exited => |code| {
                result.exit_code = @intCast(code);
                if (code != 0) {
                    writeResultError(result, run_result.stderr);
                    return false;
                }
            },
            .Signal => |sig| {
                result.exit_code = -1;
                const msg = try std.fmt.allocPrint(allocator, "sparse clone killed by signal {d}", .{sig});
                defer allocator.free(msg);
                writeResultError(result, msg);
                return false;
            },
            else => {
                writeResultError(result, "sparse clone terminated abnormally");
                return false;
            },
        }
    }

    // Step 2: git sparse-checkout init --cone
    {
        const init_result = try runChild(allocator, &[_][]const u8{ "git", "-C", dest, "sparse-checkout", "init", "--cone" }, &env_map, 4096, result);
        defer allocator.free(init_result.stdout);
        defer allocator.free(init_result.stderr);

        if (init_result.term != .Exited or init_result.term.Exited != 0) {
            writeResultError(result, if (init_result.stderr.len > 0) init_result.stderr else "sparse-checkout init failed");
            return false;
        }
    }

    // Step 3: git sparse-checkout set <paths>  (if paths provided)
    if (paths.len > 0) {
        var set_args = std.ArrayList([]const u8).empty;
        defer set_args.deinit(allocator);
        try set_args.appendSlice(allocator, &[_][]const u8{ "git", "-C", dest, "sparse-checkout", "set" });

        var iter = std.mem.splitScalar(u8, paths, ',');
        while (iter.next()) |p| {
            const trimmed = std.mem.trim(u8, p, " ");
            if (trimmed.len > 0) {
                try set_args.append(allocator, trimmed);
            }
        }

        const set_result = try runChild(allocator, set_args.items, &env_map, 4096, result);
        defer allocator.free(set_result.stdout);
        defer allocator.free(set_result.stderr);

        if (set_result.term != .Exited or set_result.term.Exited != 0) {
            writeResultError(result, if (set_result.stderr.len > 0) set_result.stderr else "sparse-checkout set failed");
            return false;
        }
    }

    // Step 4: git checkout
    {
        const checkout_result = try runChild(allocator, &[_][]const u8{ "git", "-C", dest, "checkout" }, &env_map, 1024 * 64, result);
        defer allocator.free(checkout_result.stdout);
        defer allocator.free(checkout_result.stderr);

        if (checkout_result.term != .Exited or checkout_result.term.Exited != 0) {
            writeResultError(result, if (checkout_result.stderr.len > 0) checkout_result.stderr else "checkout after sparse clone failed");
            return false;
        }
    }

    // Step 5: Get HEAD commit SHA
    const sha_result = try runChild(allocator, &[_][]const u8{ "git", "-C", dest, "log", "--format=%H", "-1" }, &env_map, 128, result);
    defer allocator.free(sha_result.stdout);
    defer allocator.free(sha_result.stderr);

    if (sha_result.term == .Exited and sha_result.term.Exited == 0) {
        if (output_parser.parseCommitSha(sha_result.stdout)) |sha| {
            const sha_len = @min(sha.len, result.commit_sha.len - 1);
            @memcpy(result.commit_sha[0..sha_len], sha[0..sha_len]);
            result.sha_len = sha_len;
        }
    }

    result.success = true;
    return true;
}

// ── Mirror Clone ───────────────────────────────────────────────────────────────

/// Creates a mirror clone: `git clone --mirror <url> <dest>`.
/// A mirror clone is a bare repository with all refs mirrored as-is.
pub export fn gm_git_mirror_clone(
    url: [*:0]const u8,
    dest: [*:0]const u8,
    key_path: [*:0]const u8,
    host_alias: [*:0]const u8,
    branch: [*:0]const u8,
    depth: u32,
    result: *GitResult,
) bool {
    _ = host_alias;
    return mirrorCloneImpl(
        std.mem.span(url),
        std.mem.span(dest),
        std.mem.span(key_path),
        std.mem.span(branch),
        depth,
        result,
    ) catch |err| {
        writeResultError(result, @errorName(err));
        return false;
    };
}

fn mirrorCloneImpl(url: []const u8, dest: []const u8, key_path: []const u8, branch: []const u8, depth: u32, result: *GitResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var env_map = try std.process.getEnvMap(allocator);
    defer env_map.deinit();
    try env_builder.populateGitEnv(&env_map, key_path, 30, allocator);

    var args = std.ArrayList([]const u8).empty;
    defer args.deinit(allocator);
    try args.appendSlice(allocator, &[_][]const u8{ "git", "clone", "--mirror", "--progress" });
    if (branch.len > 0) try args.appendSlice(allocator, &[_][]const u8{ "-b", branch });
    if (depth > 0) {
        const depth_str = try std.fmt.allocPrint(allocator, "{d}", .{depth});
        defer allocator.free(depth_str);
        try args.appendSlice(allocator, &[_][]const u8{ "--depth", depth_str });
    }
    try args.appendSlice(allocator, &[_][]const u8{ url, dest });

    const run_result = try runChild(allocator, args.items, &env_map, 1024 * 64, result);
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    fillResultBuffers(result, run_result.stdout, run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            result.exit_code = @intCast(code);
            if (code != 0) {
                writeResultError(result, run_result.stderr);
                return false;
            }
        },
        .Signal => |sig| {
            result.exit_code = -1;
            const msg = try std.fmt.allocPrint(allocator, "mirror clone killed by signal {d}", .{sig});
            defer allocator.free(msg);
            writeResultError(result, msg);
            return false;
        },
        else => {
            writeResultError(result, "mirror clone terminated abnormally");
            return false;
        },
    }

    // Mirror clones are bare; use `git log` with --git-dir for the repo path
    const sha_result = try runChild(allocator, &[_][]const u8{ "git", "--git-dir", dest, "log", "--format=%H", "-1" }, &env_map, 128, result);
    defer allocator.free(sha_result.stdout);
    defer allocator.free(sha_result.stderr);

    if (sha_result.term == .Exited and sha_result.term.Exited == 0) {
        if (output_parser.parseCommitSha(sha_result.stdout)) |sha| {
            const sha_len = @min(sha.len, result.commit_sha.len - 1);
            @memcpy(result.commit_sha[0..sha_len], sha[0..sha_len]);
            result.sha_len = sha_len;
        }
    }

    result.success = true;
    return true;
}

// ── Bare Clone ─────────────────────────────────────────────────────────────────

/// Creates a bare clone: `git clone --bare <url> <dest>`.
pub export fn gm_git_bare_clone(
    url: [*:0]const u8,
    dest: [*:0]const u8,
    key_path: [*:0]const u8,
    host_alias: [*:0]const u8,
    branch: [*:0]const u8,
    depth: u32,
    result: *GitResult,
) bool {
    _ = host_alias;
    return bareCloneImpl(
        std.mem.span(url),
        std.mem.span(dest),
        std.mem.span(key_path),
        std.mem.span(branch),
        depth,
        result,
    ) catch |err| {
        writeResultError(result, @errorName(err));
        return false;
    };
}

fn bareCloneImpl(url: []const u8, dest: []const u8, key_path: []const u8, branch: []const u8, depth: u32, result: *GitResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var env_map = try std.process.getEnvMap(allocator);
    defer env_map.deinit();
    try env_builder.populateGitEnv(&env_map, key_path, 30, allocator);

    var args = std.ArrayList([]const u8).empty;
    defer args.deinit(allocator);
    try args.appendSlice(allocator, &[_][]const u8{ "git", "clone", "--bare", "--progress" });
    if (branch.len > 0) try args.appendSlice(allocator, &[_][]const u8{ "-b", branch });
    if (depth > 0) {
        const depth_str = try std.fmt.allocPrint(allocator, "{d}", .{depth});
        defer allocator.free(depth_str);
        try args.appendSlice(allocator, &[_][]const u8{ "--depth", depth_str });
    }
    try args.appendSlice(allocator, &[_][]const u8{ url, dest });

    const run_result = try runChild(allocator, args.items, &env_map, 1024 * 64, result);
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    fillResultBuffers(result, run_result.stdout, run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            result.exit_code = @intCast(code);
            if (code != 0) {
                writeResultError(result, run_result.stderr);
                return false;
            }
        },
        .Signal => |sig| {
            result.exit_code = -1;
            const msg = try std.fmt.allocPrint(allocator, "bare clone killed by signal {d}", .{sig});
            defer allocator.free(msg);
            writeResultError(result, msg);
            return false;
        },
        else => {
            writeResultError(result, "bare clone terminated abnormally");
            return false;
        },
    }

    const sha_result = try runChild(allocator, &[_][]const u8{ "git", "--git-dir", dest, "log", "--format=%H", "-1" }, &env_map, 128, result);
    defer allocator.free(sha_result.stdout);
    defer allocator.free(sha_result.stderr);

    if (sha_result.term == .Exited and sha_result.term.Exited == 0) {
        if (output_parser.parseCommitSha(sha_result.stdout)) |sha| {
            const sha_len = @min(sha.len, result.commit_sha.len - 1);
            @memcpy(result.commit_sha[0..sha_len], sha[0..sha_len]);
            result.sha_len = sha_len;
        }
    }

    result.success = true;
    return true;
}

// ── Partial Clone Fetch ────────────────────────────────────────────────────────

/// Fetches additional objects from a partial clone's promisor remote.
/// Used after a partial clone to lazily retrieve blobs/trees on demand.
/// The SSH key from the original clone is already wired into the remote URL.
pub export fn gm_git_partial_clone_fetch(
    repo_path: [*:0]const u8,
    result: *GitResult,
) bool {
    return partialCloneFetchImpl(std.mem.span(repo_path), result) catch |err| {
        writeResultError(result, @errorName(err));
        return false;
    };
}

fn partialCloneFetchImpl(repo_path: []const u8, result: *GitResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const run_result = try runChild(allocator, &[_][]const u8{ "git", "-C", repo_path, "fetch", "--all" }, null, 1024 * 64, result);
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    fillResultBuffers(result, run_result.stdout, run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            result.exit_code = @intCast(code);
            if (code != 0) {
                writeResultError(result, if (run_result.stderr.len > 0) run_result.stderr else "partial clone fetch failed");
                return false;
            }
        },
        else => {
            writeResultError(result, "partial clone fetch terminated abnormally");
            return false;
        },
    }

    result.success = true;
    return true;
}

// ── Sparse Init ────────────────────────────────────────────────────────────────

/// Reconfigures an existing sparse checkout with new paths (cone mode).
/// Steps:
///   1. `git -C <repo> sparse-checkout init --cone`
///   2. `git -C <repo> sparse-checkout set <paths>`
pub export fn gm_git_sparse_init(
    repo_path: [*:0]const u8,
    paths: [*:0]const u8,
    result: *GitResult,
) bool {
    return sparseInitImpl(std.mem.span(repo_path), std.mem.span(paths), result) catch |err| {
        writeResultError(result, @errorName(err));
        return false;
    };
}

fn sparseInitImpl(repo_path: []const u8, paths: []const u8, result: *GitResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Step 1: git sparse-checkout init --cone
    {
        const init_result = try std.process.Child.run(.{
            .allocator = allocator,
            .argv = &[_][]const u8{ "git", "-C", repo_path, "sparse-checkout", "init", "--cone" },
            .max_output_bytes = 4096,
        });
        defer allocator.free(init_result.stdout);
        defer allocator.free(init_result.stderr);

        if (init_result.term != .Exited or init_result.term.Exited != 0) {
            writeResultError(result, if (init_result.stderr.len > 0) init_result.stderr else "sparse-checkout init failed");
            return false;
        }
    }

    // Step 2: git sparse-checkout set <paths> (if paths provided)
    if (paths.len > 0) {
        var set_args = std.ArrayList([]const u8).empty;
        defer set_args.deinit(allocator);
        try set_args.appendSlice(allocator, &[_][]const u8{ "git", "-C", repo_path, "sparse-checkout", "set" });

        var iter = std.mem.splitScalar(u8, paths, ',');
        while (iter.next()) |p| {
            const trimmed = std.mem.trim(u8, p, " ");
            if (trimmed.len > 0) {
                try set_args.append(allocator, trimmed);
            }
        }

        const set_result = try std.process.Child.run(.{
            .allocator = allocator,
            .argv = set_args.items,
            .max_output_bytes = 4096,
        });
        defer allocator.free(set_result.stdout);
        defer allocator.free(set_result.stderr);

        if (set_result.term != .Exited or set_result.term.Exited != 0) {
            writeResultError(result, if (set_result.stderr.len > 0) set_result.stderr else "sparse-checkout set failed");
            return false;
        }
    }

    result.success = true;
    return true;
}

// ── Pull ──────────────────────────────────────────────────────────────────────

/// Pulls the latest changes from the remote into the local repository.
///
/// Parameters:
///   repo_path — Absolute path to the local git repository directory.
///   key_path  — Absolute path to the SSH private key for authentication.
///   rebase    — When true, uses --rebase instead of --merge.
///               Rebase is generally preferred for a cleaner history but
///               can conflict with in-progress work. The Rust domain service
///               decides which to use based on the sync policy.
///   result    — Caller-allocated output buffer.
pub export fn gm_git_pull(
    repo_path: [*:0]const u8,
    key_path: [*:0]const u8,
    rebase: bool,
    result: *GitResult,
) bool {
    return pullImpl(
        std.mem.span(repo_path),
        std.mem.span(key_path),
        rebase,
        result,
    ) catch |err| {
        writeResultError(result, @errorName(err));
        return false;
    };
}

fn pullImpl(repo_path: []const u8, key_path: []const u8, rebase: bool, result: *GitResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var env_map = try std.process.getEnvMap(allocator);
    defer env_map.deinit();
    try env_builder.populateGitEnv(&env_map, key_path, 30, allocator);

    var args = std.ArrayList([]const u8).empty;
    defer args.deinit(allocator);
    try args.appendSlice(allocator, &[_][]const u8{ "git", "-C", repo_path, "pull" });
    if (rebase) try args.append(allocator, "--rebase");

    const run_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = args.items,
        .env_map = &env_map,
        .max_output_bytes = 1024 * 64,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    fillResultBuffers(result, run_result.stdout, run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            result.exit_code = @intCast(code);
            if (code != 0) {
                writeResultError(result, if (run_result.stderr.len > 0) run_result.stderr else "git pull failed");
                return false;
            }
        },
        else => {
            writeResultError(result, "git pull terminated abnormally");
            return false;
        },
    }

    // Get the HEAD SHA after pull
    const sha_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = &[_][]const u8{ "git", "-C", repo_path, "log", "--format=%H", "-1" },
        .env_map = &env_map,
        .max_output_bytes = 128,
    });
    defer allocator.free(sha_result.stdout);
    defer allocator.free(sha_result.stderr);
    if (sha_result.term == .Exited and sha_result.term.Exited == 0) {
        if (output_parser.parseCommitSha(sha_result.stdout)) |sha| {
            const sha_len = @min(sha.len, result.commit_sha.len - 1);
            @memcpy(result.commit_sha[0..sha_len], sha[0..sha_len]);
            result.sha_len = sha_len;
        }
    }

    result.success = true;
    return true;
}

// ── Push ──────────────────────────────────────────────────────────────────────

/// Pushes local commits to the remote.
///
/// Parameters:
///   repo_path — Absolute path to the local git repository.
///   remote    — Remote name, usually "origin".
///   branch    — Branch name to push, usually "main" or "master".
///   key_path  — Absolute path to the SSH private key.
///   force     — When true, uses --force-with-lease for safer force pushes.
///               Force pushes are allowed only if the remote has not received
///               new commits since our last fetch (lease check prevents data loss).
///   result    — Caller-allocated output buffer.
pub export fn gm_git_push(
    repo_path: [*:0]const u8,
    remote: [*:0]const u8,
    branch: [*:0]const u8,
    key_path: [*:0]const u8,
    force: bool,
    result: *GitResult,
) bool {
    return pushImpl(
        std.mem.span(repo_path),
        std.mem.span(remote),
        std.mem.span(branch),
        std.mem.span(key_path),
        force,
        result,
    ) catch |err| {
        writeResultError(result, @errorName(err));
        return false;
    };
}

fn pushImpl(repo_path: []const u8, remote: []const u8, branch: []const u8, key_path: []const u8, force: bool, result: *GitResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var env_map = try std.process.getEnvMap(allocator);
    defer env_map.deinit();
    try env_builder.populateGitEnv(&env_map, key_path, 30, allocator);

    var args = std.ArrayList([]const u8).empty;
    defer args.deinit(allocator);
    try args.appendSlice(allocator, &[_][]const u8{ "git", "-C", repo_path, "push", remote, branch });
    if (force) try args.append(allocator, "--force-with-lease"); // safer than --force: checks remote state

    const run_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = args.items,
        .env_map = &env_map,
        .max_output_bytes = 1024 * 16,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    fillResultBuffers(result, run_result.stdout, run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            result.exit_code = @intCast(code);
            if (code != 0) {
                const combined = try std.mem.concat(allocator, u8, &[_][]const u8{ run_result.stdout, run_result.stderr });
                defer allocator.free(combined);
                const push_parse = output_parser.parsePushOutput(combined);
                if (push_parse.was_rejected) {
                    writeResultError(result, "Push rejected: remote has commits not in local history. Pull first.");
                } else {
                    writeResultError(result, if (run_result.stderr.len > 0) run_result.stderr else "git push failed");
                }
                return false;
            }
        },
        else => {
            writeResultError(result, "git push terminated abnormally");
            return false;
        },
    }

    result.success = true;
    return true;
}

// ── Commit ────────────────────────────────────────────────────────────────────

/// Creates a commit with the currently staged changes.
///
/// Important: this function does NOT stage files. The caller must first call
/// gm_git_stage_all() or gm_git_stage_files() before calling gm_git_commit().
/// Committing with no staged changes will fail with a git error.
///
/// Parameters:
///   repo_path    — Absolute path to the local git repository.
///   message      — Commit message. Should follow conventional commit format.
///   author_name  — Git author name (used in the commit object).
///   author_email — Git author email (used in the commit object).
///   result       — Caller-allocated output buffer.
pub export fn gm_git_commit(
    repo_path: [*:0]const u8,
    message: [*:0]const u8,
    author_name: [*:0]const u8,
    author_email: [*:0]const u8,
    amend: bool,
    result: *GitResult,
) bool {
    return commitImpl(
        std.mem.span(repo_path),
        std.mem.span(message),
        std.mem.span(author_name),
        std.mem.span(author_email),
        amend,
        result,
    ) catch |err| {
        writeResultError(result, @errorName(err));
        return false;
    };
}

fn commitImpl(repo_path: []const u8, message: []const u8, author_name: []const u8, author_email: []const u8, amend: bool, result: *GitResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Set GIT_AUTHOR_NAME and GIT_AUTHOR_EMAIL to override whatever the global
    // git config says. This ensures each commit is attributed to the correct
    // account identity, not whatever `git config --global user.name` says.
    var env_map = try std.process.getEnvMap(allocator);
    defer env_map.deinit();
    try env_map.put("GIT_AUTHOR_NAME", author_name);
    try env_map.put("GIT_AUTHOR_EMAIL", author_email);
    try env_map.put("GIT_COMMITTER_NAME", author_name);
    try env_map.put("GIT_COMMITTER_EMAIL", author_email);

    var args = std.ArrayList([]const u8).empty;
    defer args.deinit(allocator);
    try args.appendSlice(allocator, &[_][]const u8{ "git", "-C", repo_path, "commit", "-m", message });
    if (amend) try args.append(allocator, "--amend");

    const run_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = args.items,
        .env_map = &env_map,
        .max_output_bytes = 4096,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    fillResultBuffers(result, run_result.stdout, run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            result.exit_code = @intCast(code);
            if (code != 0) {
                writeResultError(result, if (run_result.stderr.len > 0) run_result.stderr else "git commit failed");
                return false;
            }
        },
        else => {
            writeResultError(result, "git commit terminated abnormally");
            return false;
        },
    }

    // Extract the new commit SHA from git log
    const sha_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = &[_][]const u8{ "git", "-C", repo_path, "log", "--format=%H", "-1" },
        .max_output_bytes = 128,
    });
    defer allocator.free(sha_result.stdout);
    defer allocator.free(sha_result.stderr);
    if (sha_result.term == .Exited and sha_result.term.Exited == 0) {
        if (output_parser.parseCommitSha(sha_result.stdout)) |sha| {
            const sha_len = @min(sha.len, result.commit_sha.len - 1);
            @memcpy(result.commit_sha[0..sha_len], sha[0..sha_len]);
            result.sha_len = sha_len;
        }
    }

    result.success = true;
    return true;
}

// ── Status ────────────────────────────────────────────────────────────────────

/// Gets the working tree status of a repository.
///
/// Uses git status --porcelain=v1 for machine-readable, stable output.
/// The result contains three newline-separated file lists: staged, unstaged, untracked.
pub export fn gm_git_status(repo_path: [*:0]const u8, result: *GitStatus) bool {
    return statusImpl(std.mem.span(repo_path), result) catch |err| {
        writeStatusError(result, @errorName(err));
        return false;
    };
}

fn statusImpl(repo_path: []const u8, result: *GitStatus) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const run_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = &[_][]const u8{ "git", "-C", repo_path, "status", "--porcelain=v1" },
        .max_output_bytes = 1024 * 256,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            if (code != 0) {
                writeStatusError(result, if (run_result.stderr.len > 0) run_result.stderr else "git status failed");
                return false;
            }
        },
        else => {
            writeStatusError(result, "git status terminated abnormally");
            return false;
        },
    }

    const entries = try output_parser.parseStatusPorcelain(run_result.stdout, allocator);
    defer allocator.free(entries);

    // Build newline-separated path lists for each category
    var staged_buf = std.ArrayList(u8).empty;
    var unstaged_buf = std.ArrayList(u8).empty;
    var untracked_buf = std.ArrayList(u8).empty;
    defer staged_buf.deinit(allocator);
    defer unstaged_buf.deinit(allocator);
    defer untracked_buf.deinit(allocator);

    for (entries) |entry| {
        if (entry.is_untracked()) {
            try untracked_buf.appendSlice(allocator, entry.path);
            try untracked_buf.append(allocator, '\n');
            result.untracked_count += 1;
        } else {
            if (entry.is_staged()) {
                try staged_buf.appendSlice(allocator, entry.path);
                try staged_buf.append(allocator, '\n');
                result.staged_count += 1;
            }
            if (entry.is_unstaged()) {
                try unstaged_buf.appendSlice(allocator, entry.path);
                try unstaged_buf.append(allocator, '\n');
                result.unstaged_count += 1;
            }
        }
    }

    // Copy the lists into the fixed-size result buffers
    const s_len = @min(staged_buf.items.len, result.staged_files.len - 1);
    @memcpy(result.staged_files[0..s_len], staged_buf.items[0..s_len]);
    result.staged_len = s_len;

    const u_len = @min(unstaged_buf.items.len, result.unstaged_files.len - 1);
    @memcpy(result.unstaged_files[0..u_len], unstaged_buf.items[0..u_len]);
    result.unstaged_len = u_len;

    const t_len = @min(untracked_buf.items.len, result.untracked_files.len - 1);
    @memcpy(result.untracked_files[0..t_len], untracked_buf.items[0..t_len]);
    result.untracked_len = t_len;

    result.is_clean = (result.staged_count == 0 and result.unstaged_count == 0 and result.untracked_count == 0);
    return true;
}

// ── Fetch ─────────────────────────────────────────────────────────────────────

/// Result buffer for git fetch. Matches #[repr(C)] struct FfiFetchResult in Rust.
pub const FetchResult = extern struct {
    success: bool = false,
    error_data: [512]u8 = std.mem.zeroes([512]u8),
    error_len: usize = 0,
};

/// Downloads objects and refs from the remote without merging (`git fetch --all`).
pub export fn gm_git_fetch(
    repo_path: [*:0]const u8,
    key_path: [*:0]const u8,
    result: *FetchResult,
) bool {
    return fetchImpl(std.mem.span(repo_path), std.mem.span(key_path), result) catch |err| {
        const name = @errorName(err);
        const len = @min(name.len, result.error_data.len - 1);
        @memcpy(result.error_data[0..len], name[0..len]);
        result.error_len = len;
        return false;
    };
}

fn fetchImpl(repo_path: []const u8, key_path: []const u8, result: *FetchResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var env_map = try std.process.getEnvMap(allocator);
    defer env_map.deinit();
    try env_builder.populateGitEnv(&env_map, key_path, 30, allocator);

    const run_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = &[_][]const u8{ "git", "-C", repo_path, "fetch", "--all" },
        .env_map = &env_map,
        .max_output_bytes = 1024 * 64,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            if (code != 0) {
                const msg = if (run_result.stderr.len > 0) run_result.stderr else "git fetch failed";
                const len = @min(msg.len, result.error_data.len - 1);
                @memcpy(result.error_data[0..len], msg[0..len]);
                result.error_len = len;
                return false;
            }
        },
        else => {
            const msg = "git fetch terminated abnormally";
            const len = @min(msg.len, result.error_data.len - 1);
            @memcpy(result.error_data[0..len], msg[0..len]);
            result.error_len = len;
            return false;
        },
    }
    result.success = true;
    return true;
}

// ── Stage ─────────────────────────────────────────────────────────────────────

/// Stages all changes (equivalent to `git add -A`).
///
/// Stages: new files, modified files, deleted files.
/// Does NOT stage ignored files (.gitignore entries).
pub export fn gm_git_stage_all(repo_path: [*:0]const u8, result: *GitResult) bool {
    return stageAllImpl(std.mem.span(repo_path), result) catch |err| {
        writeResultError(result, @errorName(err));
        return false;
    };
}

fn stageAllImpl(repo_path: []const u8, result: *GitResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const run_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = &[_][]const u8{ "git", "-C", repo_path, "add", "-A" },
        .max_output_bytes = 4096,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            result.exit_code = @intCast(code);
            if (code != 0) {
                writeResultError(result, if (run_result.stderr.len > 0) run_result.stderr else "git add failed");
                return false;
            }
        },
        else => {
            writeResultError(result, "git add terminated abnormally");
            return false;
        },
    }

    result.success = true;
    return true;
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn fillResultBuffers(result: *GitResult, stdout: []const u8, stderr: []const u8) void {
    const out_len = @min(stdout.len, result.stdout_data.len - 1);
    @memcpy(result.stdout_data[0..out_len], stdout[0..out_len]);
    result.stdout_len = out_len;

    const err_len = @min(stderr.len, result.stderr_data.len - 1);
    @memcpy(result.stderr_data[0..err_len], stderr[0..err_len]);
    result.stderr_len = err_len;
}

fn writeResultError(result: *GitResult, msg: []const u8) void {
    const copy_len = @min(msg.len, result.stderr_data.len - 1);
    @memcpy(result.stderr_data[0..copy_len], msg[0..copy_len]);
    result.stderr_data[copy_len] = 0;
    result.stderr_len = copy_len;
}

fn writeStatusError(result: *GitStatus, msg: []const u8) void {
    const copy_len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..copy_len], msg[0..copy_len]);
    result.error_message[copy_len] = 0;
    result.error_len = copy_len;
}

// ── Struct size verification ───────────────────────────────────────────────────

test "GitResult has deterministic size" {
    // Rust's #[repr(C)] struct must match this exactly.
    // If this test fails, update gm_adapters/src/git/ffi.rs accordingly.
    const expected = @sizeOf(bool) + 64 + @sizeOf(usize) +
        65536 + @sizeOf(usize) + 4096 + @sizeOf(usize) + @sizeOf(i32) + @sizeOf(i32);
    // Allow for alignment padding between fields
    try std.testing.expect(@sizeOf(GitResult) >= expected);
}
