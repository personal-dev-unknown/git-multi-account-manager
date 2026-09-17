// zig_native/src/root.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// This is the single entry point for the entire Zig native library. When Zig
// compiles libgm_native.a, it starts here and transitively includes everything
// imported by this file. The re-exports at the bottom create the flat C-ABI
// namespace that Rust's extern "C" blocks declare.
//
// ── Design decision: flat C namespace ─────────────────────────────────────────
// All exported functions begin with `gm_` and use snake_case with a domain
// prefix (gm_ssh_, gm_git_, gm_fs_, gm_platform_). This produces a C API
// that is:
//   - Unambiguous: no name collisions with libc or other linked libraries
//   - Greppable: `grep gm_ssh` finds all SSH-related calls across both languages
//   - Stable: adding a new function does not require changes to existing callers
//
// ── Threading model ───────────────────────────────────────────────────────────
// Every exported function is designed to be called from a single Rust async
// task that has been dispatched to a blocking thread pool via
// `tokio::task::spawn_blocking`. The Zig functions themselves are synchronous
// and blocking — they run ssh-keygen, git, or libsecret calls to completion
// before returning. This is intentional: the OS operations we perform (key
// generation, git clone) are inherently sequential and must not be interrupted.
//
// The functions are NOT thread-safe with respect to shared mutable state because
// they have no shared mutable state. Each call takes caller-allocated output
// buffers and writes results into them. Multiple concurrent calls to different
// functions are safe. Multiple concurrent calls to the SAME function with the
// SAME output buffer are not safe (the caller must not do this).

const std = @import("std");

// ── Module imports ─────────────────────────────────────────────────────────────
// Each module is a separate file. We import them here to include their
// source in the compilation unit and to expose their types.
pub const ssh = @import("ssh/index.zig");
pub const git_exec = @import("git_exec/index.zig");
pub const filesystem = @import("filesystem/index.zig");
pub const platform = @import("platform/index.zig");

// ── Force evaluation of all sub-modules ───────────────────────────────────────
// Zig 0.16.0-dev lazily skips imports that appear unused when building a
// static library.  The `pub const` aliases below are type references, not
// value references, so the compiler still drops sub-module `export fn` symbols
// from the ZCU object.  Referencing the modules inside a comptime block forces
// full analysis, which registers every `pub export fn` in those files into the
// archive's symbol table.
comptime {
    // Taking a function pointer forces the compiler to code-generate each body
    // and emit the symbol into the ZCU object. _ = module only triggers semantic
    // analysis (type-checking) without compiling function bodies, so export fn
    // symbols in lazily-evaluated submodules are silently dropped from the archive.
    _ = &ssh.gm_ssh_generate_key;
    _ = &ssh.gm_ssh_agent_running;
    _ = &ssh.gm_ssh_add_to_agent;
    _ = &ssh.gm_ssh_remove_from_agent;
    _ = &ssh.gm_ssh_agent_has_key;
    _ = &ssh.gm_ssh_test_connection;
    _ = &ssh.gm_ssh_write_config_entry;
    _ = &ssh.gm_ssh_remove_config_entry;
    _ = &git_exec.gm_git_clone;
    _ = &git_exec.gm_git_pull;
    _ = &git_exec.gm_git_push;
    _ = &git_exec.gm_git_commit;
    _ = &git_exec.gm_git_status;
    _ = &git_exec.gm_git_stage_all;
    _ = &git_exec.gm_git_fetch;
    _ = &filesystem.gm_fs_atomic_write;
    _ = &filesystem.gm_fs_set_permissions;
    _ = &filesystem.gm_fs_expand_path;
    _ = &filesystem.gm_fs_ensure_dir;
    _ = &filesystem.gm_fs_available_disk_space;
    _ = &filesystem.gm_fs_ensure_disk_space;
    _ = &platform.gm_platform_store_secret;
    _ = &platform.gm_platform_retrieve_secret;
    _ = &platform.gm_platform_delete_secret;
}

// ── C ABI re-exports ──────────────────────────────────────────────────────────
// These create the symbols that the C linker (and by extension Rust's linker)
// looks for when linking against libgm_native.a. Without these re-exports,
// the functions would only be visible within their own module.
//
// The `export` keyword in Zig declares a function with C calling convention
// and makes it visible in the compiled object file. The function name in the
// compiled binary matches exactly the identifier here — no name mangling.

// SSH
pub const gm_ssh_generate_key = ssh.gm_ssh_generate_key;
pub const gm_ssh_agent_running = ssh.gm_ssh_agent_running;
pub const gm_ssh_add_to_agent = ssh.gm_ssh_add_to_agent;
pub const gm_ssh_remove_from_agent = ssh.gm_ssh_remove_from_agent;
pub const gm_ssh_agent_has_key = ssh.gm_ssh_agent_has_key;
pub const gm_ssh_test_connection = ssh.gm_ssh_test_connection;
pub const gm_ssh_write_config_entry = ssh.gm_ssh_write_config_entry;
pub const gm_ssh_remove_config_entry = ssh.gm_ssh_remove_config_entry;

// Git execution
pub const gm_git_clone = git_exec.gm_git_clone;
pub const gm_git_pull = git_exec.gm_git_pull;
pub const gm_git_push = git_exec.gm_git_push;
pub const gm_git_commit = git_exec.gm_git_commit;
pub const gm_git_status = git_exec.gm_git_status;
pub const gm_git_stage_all = git_exec.gm_git_stage_all;
pub const gm_git_fetch = git_exec.gm_git_fetch;

// Filesystem
pub const gm_fs_atomic_write = filesystem.gm_fs_atomic_write;
pub const gm_fs_set_permissions = filesystem.gm_fs_set_permissions;
pub const gm_fs_expand_path = filesystem.gm_fs_expand_path;
pub const gm_fs_ensure_dir = filesystem.gm_fs_ensure_dir;
pub const gm_fs_available_disk_space = filesystem.gm_fs_available_disk_space;
pub const gm_fs_ensure_disk_space = filesystem.gm_fs_ensure_disk_space;

// Platform credential storage
pub const gm_platform_store_secret = platform.gm_platform_store_secret;
pub const gm_platform_retrieve_secret = platform.gm_platform_retrieve_secret;
pub const gm_platform_delete_secret = platform.gm_platform_delete_secret;

// ── Smoke test ────────────────────────────────────────────────────────────────
// This test verifies that the module structure is wired correctly — if any
// import fails to resolve, this test will fail with a compile error, not a
// confusing linker error from the Rust side.
test "root module smoke test" {
    // Verify function pointers are non-null (they always are for exported fns,
    // but this forces the compiler to actually resolve the symbols)
    try std.testing.expect(@TypeOf(gm_ssh_generate_key) == @TypeOf(ssh.gm_ssh_generate_key));
    try std.testing.expect(@TypeOf(gm_git_clone) == @TypeOf(git_exec.gm_git_clone));
    try std.testing.expect(@TypeOf(gm_fs_atomic_write) == @TypeOf(filesystem.gm_fs_atomic_write));
    try std.testing.expect(@TypeOf(gm_platform_store_secret) == @TypeOf(platform.gm_platform_store_secret));
}
