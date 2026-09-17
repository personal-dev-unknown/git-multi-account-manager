// zig_native/src/ssh/connection.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Tests SSH connectivity to a Git hosting platform using a specific key.
// This is the "verify your key works" step that users run after adding a
// new SSH public key to their GitHub/GitLab/Bitbucket account.
//
// One exported function:
//   gm_ssh_test_connection() — runs `ssh -T host` and parses the response
//
// ── What "testing" a connection means ─────────────────────────────────────────
// GitHub/GitLab/Bitbucket implement a test endpoint where SSH clients can
// verify authentication without performing any git operation:
//
//   $ ssh -T git@github.com
//   Hi username! You've successfully authenticated, but GitHub does not
//   provide shell access.   (exit code 1 — intentional, no shell means no 0)
//
//   $ ssh -T git@gitlab.com
//   Welcome to GitLab, @username!   (exit code 0)
//
// The authentication is proven successful by the server responding with
// the username. Failure looks like:
//   Permission denied (publickey).   (exit code 255)
//
// We detect success by searching the response for known success phrases
// rather than by exit code, because GitHub deliberately returns exit code 1
// even on successful authentication (since it does not provide a shell).
//
// ── Timeout enforcement ────────────────────────────────────────────────────────
// Network operations must have timeouts. An SSH connection to a wrong hostname
// or an unreachable server could block indefinitely. We enforce the caller-provided
// timeout_ms using the ssh ConnectTimeout option, which tells the SSH client
// to give up on the TCP connection after that many seconds.
//
// ── Key isolation ─────────────────────────────────────────────────────────────
// The -o IdentitiesOnly=yes flag prevents SSH from trying other keys loaded
// in the agent before trying our specified key. Without this, the test might
// succeed using a different key that happens to be in the agent, giving a false
// positive for the key we are trying to verify.

const std = @import("std");

/// Result buffer for connection test operations.
/// Matches #[repr(C)] struct FfiConnResult in gm_adapters/src/ssh/ffi.rs.
pub const ConnResult = extern struct {
    /// true if authentication succeeded (username was extracted from response)
    success: bool = false,

    /// The authenticated username as reported by the server.
    /// e.g. "shakamoses" from "Hi shakamoses! You've successfully authenticated..."
    username: [256]u8 = std.mem.zeroes([256]u8),
    username_len: usize = 0,

    /// The full response text from the server (stdout + stderr combined).
    /// Useful for displaying to the user during account setup.
    response: [1024]u8 = std.mem.zeroes([1024]u8),
    response_len: usize = 0,

    /// Diagnostic message when success == false.
    error_message: [512]u8 = std.mem.zeroes([512]u8),
    error_len: usize = 0,
};

/// Tests SSH authentication to a Git hosting platform using a specific key.
///
/// Parameters:
///   host       — The SSH host alias from ~/.ssh/config (e.g. "github.com-work")
///                OR the raw hostname (e.g. "github.com"). When using a host alias,
///                SSH automatically uses the configured IdentityFile.
///   key_path   — Absolute path to the private key file. Passed as -i to ssh,
///                overriding whatever ~/.ssh/config might specify for this host.
///   timeout_ms — Connection timeout in milliseconds. The ssh -o ConnectTimeout
///                option accepts seconds, so we divide by 1000 (minimum 1 second).
///   result     — Caller-allocated output buffer.
///
/// Returns: true if the server confirms authentication (not necessarily exit 0).
/// Returns: false if connection failed, authentication was rejected, or timeout.
pub export fn gm_ssh_test_connection(
    host: [*:0]const u8,
    key_path: [*:0]const u8,
    timeout_ms: u32,
    result: *ConnResult,
) bool {
    return testConnectionImpl(
        std.mem.span(host),
        std.mem.span(key_path),
        timeout_ms,
        result,
    ) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

fn testConnectionImpl(
    host: []const u8,
    key_path: []const u8,
    timeout_ms: u32,
    result: *ConnResult,
) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Convert timeout from milliseconds to seconds, minimum 1 second.
    // The ssh ConnectTimeout option does not support sub-second precision.
    const timeout_secs = @max(1, timeout_ms / 1000);
    const timeout_str = try std.fmt.allocPrint(allocator, "{d}", .{timeout_secs});
    defer allocator.free(timeout_str);

    // Build the ssh command that performs authentication without executing any
    // remote command. -T disables pseudo-terminal allocation (appropriate for
    // non-interactive use). -o BatchMode=yes prevents interactive prompts.
    const connect_timeout_opt = try std.fmt.allocPrint(allocator, "ConnectTimeout={s}", .{timeout_str});
    defer allocator.free(connect_timeout_opt);

    const run_result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = &[_][]const u8{
            "ssh",
            "-T", // No PTY allocation
            "-i", key_path, // Use this specific key
            "-o", "BatchMode=yes", // No interactive prompts
            "-o", "IdentitiesOnly=yes", // Only use the specified key
            "-o", "StrictHostKeyChecking=accept-new", // Accept new host keys
            "-o", connect_timeout_opt,
            host,
        },
        .max_output_bytes = 4096,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    // Combine stdout and stderr for analysis.
    // Different platforms put the success message in different streams:
    // GitHub: stderr with exit code 1
    // GitLab: stdout with exit code 0
    // Bitbucket: stdout with exit code 1
    const combined = try std.mem.concat(allocator, u8, &[_][]const u8{ run_result.stdout, run_result.stderr });
    defer allocator.free(combined);

    // Store the combined response for display to the user
    const resp_len = @min(combined.len, result.response.len - 1);
    @memcpy(result.response[0..resp_len], combined[0..resp_len]);
    result.response[resp_len] = 0;
    result.response_len = resp_len;

    // Success detection: look for known success response patterns.
    // These are the exact strings each platform sends on successful authentication.
    const success = extractUsernameFromResponse(combined, result);

    if (!success) {
        // Provide a helpful error message based on the response content
        const error_msg = if (std.mem.indexOf(u8, combined, "Permission denied") != null)
            "Authentication failed: key not recognized by the server. Ensure the public key is added to your account."
        else if (std.mem.indexOf(u8, combined, "Connection timed out") != null or
            std.mem.indexOf(u8, combined, "Connection refused") != null)
            "Connection failed: cannot reach the SSH server. Check network connectivity."
        else if (std.mem.indexOf(u8, combined, "No such file") != null)
            "Key file not found: the private key path does not exist."
        else if (combined.len == 0)
            "No response from server: connection may have timed out."
        else
            "Authentication unsuccessful.";
        writeError(result, error_msg);
    }

    return success;
}

/// Extracts the authenticated username from a server response.
/// Sets result.success, result.username, result.username_len on success.
/// Returns true if a success pattern was found.
fn extractUsernameFromResponse(response: []const u8, result: *ConnResult) bool {
    // GitHub pattern: "Hi username! You've successfully authenticated"
    if (std.mem.indexOf(u8, response, "Hi ")) |hi_pos| {
        if (std.mem.indexOf(u8, response[hi_pos..], "! ")) |excl_pos| {
            const name_start = hi_pos + 3; // skip "Hi "
            const name_end = hi_pos + excl_pos;
            if (name_end > name_start) {
                const username = response[name_start..name_end];
                const uname_len = @min(username.len, result.username.len - 1);
                @memcpy(result.username[0..uname_len], username[0..uname_len]);
                result.username[uname_len] = 0;
                result.username_len = uname_len;
                result.success = true;
                return true;
            }
        }
    }

    // GitLab pattern: "Welcome to GitLab, @username!"
    if (std.mem.indexOf(u8, response, "Welcome to GitLab, @")) |welcome_pos| {
        const name_start = welcome_pos + "Welcome to GitLab, @".len;
        if (std.mem.indexOfScalar(u8, response[name_start..], '!')) |excl_rel| {
            const username = response[name_start .. name_start + excl_rel];
            const uname_len = @min(username.len, result.username.len - 1);
            @memcpy(result.username[0..uname_len], username[0..uname_len]);
            result.username[uname_len] = 0;
            result.username_len = uname_len;
            result.success = true;
            return true;
        }
    }

    // Bitbucket pattern: "logged in as username."
    if (std.mem.indexOf(u8, response, "logged in as ")) |login_pos| {
        const name_start = login_pos + "logged in as ".len;
        const name_slice = response[name_start..];
        const end = std.mem.indexOfScalar(u8, name_slice, '.') orelse name_slice.len;
        const username = name_slice[0..end];
        const uname_len = @min(username.len, result.username.len - 1);
        @memcpy(result.username[0..uname_len], username[0..uname_len]);
        result.username[uname_len] = 0;
        result.username_len = uname_len;
        result.success = true;
        return true;
    }

    // Azure DevOps pattern: "remote: Shell access is not supported."
    // Azure confirms authentication differently — any non-permission-denied response
    // from SSH to dev.azure.com indicates successful key authentication.
    if (std.mem.indexOf(u8, response, "Shell access is not supported") != null) {
        const username = "azure_authenticated";
        const uname_len = @min(username.len, result.username.len - 1);
        @memcpy(result.username[0..uname_len], username[0..uname_len]);
        result.username[uname_len] = 0;
        result.username_len = uname_len;
        result.success = true;
        return true;
    }

    return false;
}

fn writeError(result: *ConnResult, msg: []const u8) void {
    const copy_len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..copy_len], msg[0..copy_len]);
    result.error_message[copy_len] = 0;
    result.error_len = copy_len;
}

test "extractUsernameFromResponse handles GitHub pattern" {
    var result = ConnResult{};
    const response = "Hi shakamoses! You've successfully authenticated, but GitHub does not provide shell access.";
    const success = extractUsernameFromResponse(response, &result);
    try std.testing.expect(success);
    try std.testing.expectEqualStrings("shakamoses", result.username[0..result.username_len]);
}

test "extractUsernameFromResponse handles GitLab pattern" {
    var result = ConnResult{};
    const response = "Welcome to GitLab, @devmoses!\r\n";
    const success = extractUsernameFromResponse(response, &result);
    try std.testing.expect(success);
    try std.testing.expectEqualStrings("devmoses", result.username[0..result.username_len]);
}

test "extractUsernameFromResponse returns false for permission denied" {
    var result = ConnResult{};
    const response = "git@github.com: Permission denied (publickey).";
    const success = extractUsernameFromResponse(response, &result);
    try std.testing.expect(!success);
    try std.testing.expectEqual(@as(usize, 0), result.username_len);
}
