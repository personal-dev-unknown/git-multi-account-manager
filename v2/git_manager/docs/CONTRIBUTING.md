# Contributing to Git Manager

Thank you for contributing to Git Manager. Before writing code, read the architecture guide in `ARCHITECTURE.md` so your changes land in the right layer.

## Getting started

You need Rust stable (≥ 1.77), Zig 0.12.x, and MySQL or SQLite. The `rust-toolchain.toml` at the workspace root pins the exact Rust toolchain. Run `rustup show` to confirm the toolchain is active.

Build the Zig layer before any Rust code: `cd zig_native && zig build`. The Zig build produces `zig-out/lib/libgm_native.a`, which `gm_adapters/build.rs` links against. If you see "could not find native static library `gm_native`", this step was skipped.

After the Zig build, run the full workspace check: `cargo check --workspace`. This does not compile binaries but validates every crate's types.

## Code organisation rules

The dependency direction is non-negotiable. A new file in `gm_domain` must never import from `gm_adapters`, `gm_kernel`, or any interface crate. If you need to call a database from the domain, you are in the wrong layer — declare a port trait in the domain and implement it in an adapter.

Every new domain entity must enforce its invariants in `new()`. Construction must fail before an invalid entity reaches memory. If you find yourself validating the same rules in a service layer that a correctly-built entity should already guarantee, the entity is missing a constraint.

New provider plugins (for a new Git hosting platform) should follow the structure of `gm_plugin_github`. Implement `RepositoryProvider`, `AuthProvider`, and register both in `on_load()`. The plugin must not import any other plugin crate.

## Testing expectations

Unit tests live next to the source file in a `#[cfg(test)]` module. They use no database, no network, and no Zig layer — pure Rust, pure domain logic.

Integration tests in `tests/integration/` use a real SQLite in-memory database. Run them with `GIT_MANAGER_SQLITE_PATH=:memory: cargo test --test integration`.

End-to-end tests in `tests/e2e/` require a compiled binary. Run the full build first with the script in the root: `./build_all.sh`, then `cargo test --test e2e`.

## Pull request checklist

Before opening a pull request, verify that `cargo check --workspace` passes, `cargo clippy --workspace` produces no warnings, `cargo fmt --all -- --check` passes, and `cargo test --workspace` passes. The CI pipeline runs all four checks automatically, but catching them locally first saves time.