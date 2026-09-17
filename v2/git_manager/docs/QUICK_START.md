# Git Manager — Quick Start

## Prerequisites

Install Rust stable via rustup (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`), Zig 0.12.0 from [ziglang.org/download](https://ziglang.org/download/), and MySQL or use SQLite for development. On Debian/Ubuntu also run `sudo apt install libsecret-1-dev pkg-config`.

## Build

```bash
# 1 — Build the Zig native layer first (always required before cargo build)
cd git_manager/zig_native
zig build -Doptimize=ReleaseSafe
cd ..

# 2 — Set up the database (skip and set GIT_MANAGER_SQLITE_PATH for SQLite)
export DATABASE_URL="mysql://git_manager:dev_password@localhost/git_manager_dev"
cargo sqlx migrate run --source crates/gm_adapters/src/persistence/migrations

# 3 — Build the workspace
cargo build --workspace --release

# 4 — Run the CLI
./target/release/git-manager --help
```

## Your first account

```bash
# Add a GitHub account
git-manager account add \
  --alias work \
  --platform github \
  --username your-username \
  --email you@example.com

# Generate an SSH key for it
git-manager ssh generate --account work

# Copy the public key output and add it to https://github.com/settings/ssh/new

# Test the connection
git-manager ssh test --account work

# Clone a repository
git-manager clone git@github.com:owner/repo.git --account work
```

## Web interface

```bash
export GIT_MANAGER_DB_URL="mysql://git_manager:dev_password@localhost/git_manager_dev"
./target/release/git-manager-web
# Open http://localhost:5000
```

## Development mode

For development without MySQL, SQLite works out of the box. Set `GIT_MANAGER_SQLITE_PATH=./dev.db` before running any binary and the database file will be created automatically.

## Troubleshooting

If you see `could not find native static library gm_native`, the Zig build was not run first. Run `cd zig_native && zig build` from the workspace root and try again.

If `cargo check` fails with missing types after pulling new changes, run `cargo build -p gm_shared` first to ensure the shared types are up to date.