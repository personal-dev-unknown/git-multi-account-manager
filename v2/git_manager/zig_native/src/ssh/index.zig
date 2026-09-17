// zig_native/src/ssh/index.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// SSH module root. Imports all SSH sub-modules and re-exports their C ABI
// functions. The root.zig file imports this module as `pub const ssh = @import("ssh/index.zig")`
// and then re-exports from here.
//
// This layered re-export pattern means that adding a new SSH function only
// requires: (1) implementing it in its own file, (2) importing and re-exporting
// in this index, (3) re-exporting in root.zig. The C header is regenerated
// automatically. The Rust ffi.rs file then needs a matching extern "C" declaration.

pub const keygen        = @import("keygen.zig");
pub const agent         = @import("agent.zig");
pub const config_writer = @import("config_writer.zig");
pub const connection    = @import("connection.zig");

// C ABI re-exports — all SSH functions in one flat namespace
pub const gm_ssh_generate_key        = keygen.gm_ssh_generate_key;
pub const gm_ssh_agent_running        = agent.gm_ssh_agent_running;
pub const gm_ssh_add_to_agent         = agent.gm_ssh_add_to_agent;
pub const gm_ssh_remove_from_agent    = agent.gm_ssh_remove_from_agent;
pub const gm_ssh_agent_has_key        = agent.gm_ssh_agent_has_key;
pub const gm_ssh_test_connection      = connection.gm_ssh_test_connection;
pub const gm_ssh_write_config_entry   = config_writer.gm_ssh_write_config_entry;
pub const gm_ssh_remove_config_entry  = config_writer.gm_ssh_remove_config_entry;

// Shared result types — defined here so all SSH files can import them
// without circular dependencies.
pub const KeygenResult = keygen.KeygenResult;
pub const AgentResult  = agent.AgentResult;
pub const ConnResult   = connection.ConnResult;
pub const FsResult     = config_writer.FsResult;