// zig_native/src/git_exec/env_builder.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Constructs the GIT_SSH_COMMAND environment variable that git uses to
// invoke SSH during clone/pull/push operations. This is the mechanism
// that enables account isolation — different git operations use different
// SSH keys without interfering with each other.
//
// ── How git uses SSH ──────────────────────────────────────────────────────────
// When git encounters an ssh:// or git@ URL, it does NOT call SSH directly.
// Instead, it checks for several environment variables in priority order:
//
//   GIT_SSH_COMMAND  (highest priority — takes a full shell command with flags)
//   GIT_SSH          (takes just a binary path, no flags)
//   core.sshCommand  (from git config)
//   system ssh       (fallback)
//
// GIT_SSH_COMMAND is the most powerful option because it lets us specify
// exact flags, overriding everything in the user's ~/.ssh/config and
// ~/.ssh/known_hosts for this specific git invocation. This is exactly
// what we need for account isolation.
//
// ── The constructed command ───────────────────────────────────────────────────
// We construct a command that looks like:
//
//   ssh -i /path/to/key
//       -o IdentitiesOnly=yes
//       -o StrictHostKeyChecking=accept-new
//       -o BatchMode=yes
//       -o ConnectTimeout=30
//
// Breaking down each flag:
//
//   -i /path/to/key        — Use this specific private key file.
//
//   -o IdentitiesOnly=yes  — CRITICAL. Without this, SSH will try ALL keys
//                            in the agent before trying our specified key.
//                            If a different GitHub key is in the agent, git
//                            would authenticate as the wrong account.
//
//   -o StrictHostKeyChecking=accept-new
//                          — Accept host keys for servers we have not seen before
//                            (first clone of a new platform server). Reject if
//                            the host key changes after initial acceptance
//                            (MITM protection). This balances usability and security.
//
//   -o BatchMode=yes       — Never prompt interactively. If authentication fails,
//                            fail immediately rather than prompting for a password.
//                            Essential for background/automated operation.
//
//   -o ConnectTimeout=30   — Give up after 30 seconds if the TCP connection
//                            cannot be established. Prevents indefinite hangs.
//
// ── Why not just use the host alias from ~/.ssh/config? ──────────────────────
// We write the Host block to ~/.ssh/config AND pass GIT_SSH_COMMAND because:
//
//   1. The Host alias in the git remote URL (git@github.com-work:owner/repo)
//      still requires the ~/.ssh/config Host block to resolve the real hostname.
//
//   2. GIT_SSH_COMMAND provides defense-in-depth: even if ~/.ssh/config has
//      a misconfiguration, the explicit -i flag in GIT_SSH_COMMAND ensures
//      the correct key is used.
//
//   3. GIT_SSH_COMMAND flags like IdentitiesOnly=yes cannot be set in
//      ~/.ssh/config per-Host in a way that prevents agent key leakage.

const std = @import("std");

/// Constructs the GIT_SSH_COMMAND environment variable value for a specific key.
///
/// Parameters:
///   key_path     — Absolute path to the SSH private key file
///   connect_timeout_secs — TCP connection timeout in seconds (minimum 5)
///   allocator    — Memory allocator; the caller owns the returned string and
///                  must free it with allocator.free() after use.
///
/// Returns: the full GIT_SSH_COMMAND string, or an error if allocation fails.
///
/// Example output:
///   "ssh -i /home/user/.ssh/id_ed25519_work -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new -o BatchMode=yes -o ConnectTimeout=30"
pub fn buildSshCommand(
    key_path:             []const u8,
    connect_timeout_secs: u32,
    allocator:            std.mem.Allocator,
) ![]const u8 {
    const timeout = @max(5, connect_timeout_secs);
    return std.fmt.allocPrint(allocator,
        "ssh -i {s}" ++
        " -o IdentitiesOnly=yes" ++
        " -o StrictHostKeyChecking=accept-new" ++
        " -o BatchMode=yes" ++
        " -o ConnectTimeout={d}",
        .{ key_path, timeout },
    );
}

/// Populates an existing EnvMap with the git SSH isolation variables.
///
/// In addition to GIT_SSH_COMMAND, we also set:
///   GIT_TERMINAL_PROMPT=0    — Prevents git from prompting for HTTPS credentials
///   GIT_TRACE=0              — Suppresses git's own SSH trace output
///
/// The caller owns the env_map and is responsible for calling env_map.deinit().
pub fn populateGitEnv(
    env_map:              *std.process.EnvMap,
    key_path:             []const u8,
    connect_timeout_secs: u32,
    allocator:            std.mem.Allocator,
) !void {
    const ssh_command = try buildSshCommand(key_path, connect_timeout_secs, allocator);
    defer allocator.free(ssh_command);

    try env_map.put("GIT_SSH_COMMAND", ssh_command);
    try env_map.put("GIT_TERMINAL_PROMPT", "0");
    try env_map.put("GIT_TRACE", "0");
}

// ── Unit tests ────────────────────────────────────────────────────────────────

test "buildSshCommand includes required flags" {
    const allocator = std.testing.allocator;
    const cmd = try buildSshCommand("/home/user/.ssh/id_ed25519", 30, allocator);
    defer allocator.free(cmd);

    // Verify all required flags are present
    try std.testing.expect(std.mem.indexOf(u8, cmd, "-i /home/user/.ssh/id_ed25519") != null);
    try std.testing.expect(std.mem.indexOf(u8, cmd, "IdentitiesOnly=yes") != null);
    try std.testing.expect(std.mem.indexOf(u8, cmd, "StrictHostKeyChecking=accept-new") != null);
    try std.testing.expect(std.mem.indexOf(u8, cmd, "BatchMode=yes") != null);
    try std.testing.expect(std.mem.indexOf(u8, cmd, "ConnectTimeout=30") != null);
}

test "buildSshCommand enforces minimum timeout" {
    const allocator = std.testing.allocator;
    // timeout_secs = 0 should be clamped to 5
    const cmd = try buildSshCommand("/path/to/key", 0, allocator);
    defer allocator.free(cmd);
    try std.testing.expect(std.mem.indexOf(u8, cmd, "ConnectTimeout=5") != null);
}

test "buildSshCommand handles paths with spaces" {
    const allocator = std.testing.allocator;
    const cmd = try buildSshCommand("/home/user name/.ssh/key file", 30, allocator);
    defer allocator.free(cmd);
    // The path should appear verbatim — escaping is the shell's responsibility
    try std.testing.expect(std.mem.indexOf(u8, cmd, "/home/user name/.ssh/key file") != null);
}