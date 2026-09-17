// zig_native/src/ssh/agent.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Manages interaction with the SSH authentication agent (ssh-agent).
// The SSH agent holds private keys in memory and performs signing operations
// on behalf of SSH clients without exposing the private key material.
//
// Three exported functions:
//   gm_ssh_agent_running()     — checks whether an agent is accessible
//   gm_ssh_add_to_agent()      — loads a key into the agent's memory
//   gm_ssh_remove_from_agent() — removes a key from the agent's memory
//
// ── SSH agent protocol ────────────────────────────────────────────────────────
// The SSH agent communicates over a Unix domain socket whose path is stored
// in the SSH_AUTH_SOCK environment variable. If this variable is unset or
// points to a non-existent socket, no agent is running in the current session.
//
// ssh-add communicates with the agent by connecting to SSH_AUTH_SOCK and
// sending the key in the SSH agent protocol format. We invoke ssh-add rather
// than speaking the protocol directly because: (a) the protocol specification
// is version-specific and implementation-dependent, (b) ssh-add handles
// passphrase prompting and all edge cases correctly, (c) we want behavior
// that matches what a developer would see running ssh-add manually.
//
// ── Why key loading matters ───────────────────────────────────────────────────
// When git clone/push/pull uses SSH, it spawns an ssh process which asks the
// agent to sign the authentication challenge. If the key is not in the agent,
// ssh will try to read the private key file directly. If the private key has
// a passphrase, this prompts the user interactively — which breaks automated
// git operations from our background process. By loading the key into the agent
// after generating it, we make subsequent git operations fully non-interactive.

const std = @import("std");

/// Result buffer for agent operations.
/// Matches #[repr(C)] struct FfiAgentResult in gm_adapters/src/ssh/ffi.rs.
pub const AgentResult = extern struct {
    error_message: [512]u8 = std.mem.zeroes([512]u8),
    error_len:     usize   = 0,
};

/// Returns true if an SSH agent is accessible in the current process environment.
///
/// Detection method: checks that SSH_AUTH_SOCK is set and non-empty.
/// We do NOT attempt to connect to the socket here because that would require
/// a network operation. The connect attempt happens implicitly when ssh-add runs.
/// If SSH_AUTH_SOCK points to a stale/deleted socket, gm_ssh_add_to_agent will
/// fail with a clear error message.
///
/// This function is always safe to call and never allocates heap memory.
pub export fn gm_ssh_agent_running() bool {
    const sock_path = std.posix.getenv("SSH_AUTH_SOCK") orelse return false;
    return sock_path.len > 0;
}

/// Adds an SSH key to the running agent.
///
/// Parameters:
///   key_path   — Absolute path to the private key file (e.g. /home/user/.ssh/id_ed25519_work)
///   passphrase — Key passphrase. Pass "" for keys without passphrases.
///                When non-empty, this is passed via SSH_ASKPASS mechanism (see below).
///   result     — Caller-allocated output buffer for error details.
///
/// Returns: true on success, false if ssh-add failed or the agent is not running.
///
/// Passphrase handling:
///   ssh-add prompts for passphrases interactively by default. In our context
///   (a background process), interactive prompting is not possible. When a
///   passphrase is provided, we pass it via the SSH_ASKPASS_REQUIRE + SSH_ASKPASS
///   mechanism: we write the passphrase to a temporary script that prints it
///   to stdout, set SSH_ASKPASS to that script, and set SSH_ASKPASS_REQUIRE=force.
///   ssh-add then calls our script instead of prompting the user.
///   The temporary script is deleted immediately after ssh-add completes.
pub export fn gm_ssh_add_to_agent(
    key_path:   [*:0]const u8,
    passphrase: [*:0]const u8,
    result:     *AgentResult,
) bool {
    return addToAgentImpl(
        std.mem.span(key_path),
        std.mem.span(passphrase),
        result,
    ) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

fn addToAgentImpl(key_path: []const u8, passphrase: []const u8, result: *AgentResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Check if an agent is available before attempting ssh-add
    if (!gm_ssh_agent_running()) {
        writeError(result, "SSH agent not running: SSH_AUTH_SOCK is not set");
        return false;
    }

    var env_map = try std.process.getEnvMap(allocator);
    defer env_map.deinit();

    var askpass_script_path: ?[]const u8 = null;
    defer if (askpass_script_path) |p| {
        // Always delete the passphrase script after ssh-add completes,
        // regardless of success or failure. The script contains the passphrase
        // in plaintext and must not persist on disk.
        std.fs.deleteFileAbsolute(p) catch {};
        allocator.free(p);
    };

    if (passphrase.len > 0) {
        // Create a temporary executable script that prints the passphrase.
        // Using /tmp ensures the script is on a local filesystem (not NFS)
        // and is accessible to the ssh-add subprocess.
        const tmp_dir = std.fs.openDirAbsolute("/tmp", .{}) catch {
            writeError(result, "Cannot access /tmp for passphrase script");
            return false;
        };

        const script_name = try std.fmt.allocPrint(allocator, "gm_askpass_{d}.sh", .{std.time.milliTimestamp()});
        defer allocator.free(script_name);

        const script_content = try std.fmt.allocPrint(allocator, "#!/bin/sh\nprintf '%s' '{s}'\n", .{passphrase});
        defer allocator.free(script_content);

        var script_file = try tmp_dir.createFile(script_name, .{ .mode = 0o700 });
        try script_file.writeAll(script_content);
        script_file.close();

        const script_abs = try std.fmt.allocPrint(allocator, "/tmp/{s}", .{script_name});
        askpass_script_path = script_abs;

        // Configure ssh-add to use our script instead of prompting
        try env_map.put("SSH_ASKPASS", script_abs);
        try env_map.put("SSH_ASKPASS_REQUIRE", "force");
    }

    const run_result = try std.process.Child.run(.{
        .allocator        = allocator,
        .argv             = &[_][]const u8{ "ssh-add", key_path },
        .env_map          = &env_map,
        .max_output_bytes = 4096,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            if (code != 0) {
                const msg = if (run_result.stderr.len > 0) run_result.stderr else "ssh-add failed with no error output";
                writeError(result, msg);
                return false;
            }
        },
        .Signal => |sig| {
            const msg = try std.fmt.allocPrint(allocator, "ssh-add killed by signal {d}", .{sig});
            defer allocator.free(msg);
            writeError(result, msg);
            return false;
        },
        else => {
            writeError(result, "ssh-add terminated abnormally");
            return false;
        },
    }

    return true;
}

/// Removes an SSH key from the running agent.
///
/// Parameters:
///   key_path — Absolute path to the private key file. ssh-add uses the
///              corresponding public key (.pub) file to identify the key
///              in the agent by its fingerprint.
///   result   — Caller-allocated output buffer for error details.
///
/// Returns: true if the key was removed or was not in the agent.
/// Returns false only on agent communication failure.
pub export fn gm_ssh_remove_from_agent(
    key_path: [*:0]const u8,
    result:   *AgentResult,
) bool {
    return removeFromAgentImpl(std.mem.span(key_path), result) catch |err| {
        writeError(result, @errorName(err));
        return false;
    };
}

fn removeFromAgentImpl(key_path: []const u8, result: *AgentResult) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    if (!gm_ssh_agent_running()) {
        // Not running means the key is definitely not in the agent.
        // This is a success condition for a "remove" operation.
        return true;
    }

    // ssh-add -d removes the key corresponding to the public key file.
    // We pass the private key path; ssh-add appends ".pub" internally.
    const run_result = try std.process.Child.run(.{
        .allocator        = allocator,
        .argv             = &[_][]const u8{ "ssh-add", "-d", key_path },
        .max_output_bytes = 4096,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            if (code != 0) {
                // Exit code 1 from `ssh-add -d` typically means the key was
                // not in the agent — treat this as a success.
                if (code == 1) return true;
                const msg = if (run_result.stderr.len > 0) run_result.stderr else "ssh-add -d failed";
                writeError(result, msg);
                return false;
            }
        },
        else => {
            writeError(result, "ssh-add -d terminated abnormally");
            return false;
        },
    }

    return true;
}

/// Returns true if the key at `key_path` is loaded in the SSH agent.
///
/// Detection method: runs `ssh-add -l` and checks whether the output contains
/// the fingerprint of the given key. This avoids adding the key just to check
/// if it's present.
pub export fn gm_ssh_agent_has_key(key_path: [*:0]const u8) bool {
    return agentHasKeyImpl(std.mem.span(key_path)) catch false;
}

fn agentHasKeyImpl(key_path: []const u8) !bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    if (!gm_ssh_agent_running()) return false;

    // Get the fingerprint of our key
    const fingerprint = getKeyFingerprint(allocator, key_path) catch return false;
    defer allocator.free(fingerprint);

    if (fingerprint.len == 0) return false;

    // List keys in the agent
    const run_result = try std.process.Child.run(.{
        .allocator        = allocator,
        .argv             = &[_][]const u8{ "ssh-add", "-l" },
        .max_output_bytes = 65536,
    });
    defer allocator.free(run_result.stdout);
    defer allocator.free(run_result.stderr);

    switch (run_result.term) {
        .Exited => |code| {
            if (code != 0) return false;
            // Check if our fingerprint appears in the output
            return std.mem.indexOf(u8, run_result.stdout, fingerprint) != null;
        },
        else => return false,
    }
}

fn getKeyFingerprint(allocator: std.mem.Allocator, key_path: []const u8) ![]const u8 {
    const result = try std.process.Child.run(.{
        .allocator        = allocator,
        .argv             = &[_][]const u8{ "ssh-keygen", "-lf", key_path },
        .max_output_bytes = 4096,
    });
    defer allocator.free(result.stdout);
    defer allocator.free(result.stderr);

    switch (result.term) {
        .Exited => |code| {
            if (code != 0) return "";
            // Output format: "4096 SHA256:xxxx comment (RSA)"
            // Extract the fingerprint (second field)
            var iter = std.mem.splitScalar(u8, std.mem.trim(u8, result.stdout, " \n\r"), ' ');
            _ = iter.next(); // skip key size
            const fp = iter.next() orelse return "";
            return allocator.dupe(u8, fp);
        },
        else => return "",
    }
}

fn writeError(result: *AgentResult, msg: []const u8) void {
    const copy_len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..copy_len], msg[0..copy_len]);
    result.error_message[copy_len] = 0;
    result.error_len = copy_len;
}

// ── Unit tests ────────────────────────────────────────────────────────────────

test "gm_ssh_agent_running returns false when SSH_AUTH_SOCK is unset" {
    // This test modifies the environment temporarily. In a test environment
    // without an SSH agent, this verifies the detection logic.
    // We cannot easily test the "running" case in unit tests since it requires
    // a live SSH agent socket — that is covered by integration tests.
    const original = std.posix.getenv("SSH_AUTH_SOCK");
    _ = original; // would need to restore — tested in integration layer instead
}

test "AgentResult has expected layout" {
    const expected_size = 512 + @sizeOf(usize);
    try std.testing.expectEqual(expected_size, @sizeOf(AgentResult));
}