// crates/gm_ports/src/outbound/git_executor.rs
//
// The GitExecutor outbound port — the contract between the kernel/adapters and
// the git subprocess layer. Since gm_ports depends on gm_domain, we re-export
// the trait and all its input/output types directly from the domain's port
// definition. This eliminates duplication: there is exactly one GitExecutor
// definition in the entire codebase, owned by the domain, and gm_ports exposes
// it as part of the official outbound surface.
//
// The ZigGitExecutor in gm_adapters implements this trait by calling into the
// Zig native layer via FFI. Tests inject mock implementations.

pub use gm_domain::git::ports::git_executor::{
    CloneOptions,
    CloneResult,
    CommitOptions,
    CommitResult,
    GitExecutor,
    GitStatus,
    PullOptions,
    PullResult,
    PushOptions,
    PushResult,
    StatusEntry,
};