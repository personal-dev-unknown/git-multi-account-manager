// zig_native/src/git_exec/index.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Git execution module root. Coordinates the three sub-modules that together
// provide a complete, SSH-isolated Git subprocess execution layer:
//
//   env_builder.zig  — constructs the GIT_SSH_COMMAND environment variable
//                      that forces git to use a specific SSH key + host config
//   output_parser.zig — parses raw git subprocess output into structured types
//   executor.zig      — the exported C ABI functions that orchestrate everything
//
// The three-way separation exists for testability: env_builder and output_parser
// are pure functions that can be unit-tested without spawning subprocesses.
// executor.zig is the only module that actually forks and execs git processes,
// which means integration tests can focus exclusively on executor.zig.

pub const env_builder = @import("env_builder.zig");
pub const output_parser = @import("output_parser.zig");
pub const executor = @import("executor.zig");

// C ABI re-exports
pub const gm_git_clone = executor.gm_git_clone;
pub const gm_git_pull = executor.gm_git_pull;
pub const gm_git_push = executor.gm_git_push;
pub const gm_git_commit = executor.gm_git_commit;
pub const gm_git_status = executor.gm_git_status;
pub const gm_git_stage_all = executor.gm_git_stage_all;
pub const gm_git_fetch = executor.gm_git_fetch;

// Shared result types
pub const GitResult = executor.GitResult;
pub const GitStatus = executor.GitStatus;