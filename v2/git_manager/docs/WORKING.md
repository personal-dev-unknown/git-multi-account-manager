# Git Multi-Account Manager

**Complete Reference Manual** — Architecture, Installation, CLI Commands, Web Interface, Desktop App, API, Configuration, Security, and Developer Guide.

---

## Table of Contents

1. [What Is Git Manager?](#1-what-is-git-manager)
2. [How It Works — Architecture Overview](#2-how-it-works--architecture-overview)
3. [Supported Platforms](#3-supported-platforms)
4. [Prerequisites and Installation](#4-prerequisites-and-installation)
5. [Environment Variables](#5-environment-variables)
6. [Database Setup](#6-database-setup)
7. [The SSH Key Isolation Model](#7-the-ssh-key-isolation-model)
8. [CLI Reference — Account Commands](#8-cli-reference--account-commands)
9. [CLI Reference — SSH Key Commands](#9-cli-reference--ssh-key-commands)
10. [CLI Reference — Repository Commands](#10-cli-reference--repository-commands)
11. [CLI Reference — Git Operation Commands](#11-cli-reference--git-operation-commands)
12. [CLI Reference — Configuration Commands](#12-cli-reference--configuration-commands)
13. [CLI Reference — Operation Log](#13-cli-reference--operation-log)
14. [Web Interface](#14-web-interface)
15. [REST API Reference](#15-rest-api-reference)
16. [Desktop Application](#16-desktop-application)
17. [Credential Storage and Security](#17-credential-storage-and-security)
18. [Configuration Reference](#18-configuration-reference)
19. [Platform-Specific Guides](#19-platform-specific-guides)
20. [Workflow Examples](#20-workflow-examples)
21. [Plugin System](#21-plugin-system)
22. [Troubleshooting](#22-troubleshooting)
23. [Developer Guide](#23-developer-guide)

---

## 1. What Is Git Manager?

Git Manager solves the friction of working with multiple Git hosting accounts simultaneously. The classic problem: you have a personal GitHub account and a work GitHub account, or a GitHub account and a GitLab account. Standard `git` only reads a single `~/.ssh/config` entry per hostname, which means either manually editing config files for every new repository or using complex alias tricks that break when you clone fresh.

Git Manager gives every account its own **completely isolated identity**:

- Its own Ed25519 or RSA SSH key pair
- Its own `~/.ssh/config` Host alias that maps to a unique key
- Its own encrypted credential store entry (PAT, OAuth token, or app password)
- Its own repository list in the database

When you clone a repository with Git Manager, it automatically rewrites the SSH URL to use the correct host alias, so `git pull` and `git push` always authenticate with the right account — without any manual configuration.

### What Git Manager is **not**

Git Manager is not a replacement for `git`. It calls the system `git` binary (via a Zig native subprocess layer) for all actual git operations. It is an orchestration layer that handles the identity and credential management so you never have to think about which key to use.

---

## 2. How It Works — Architecture Overview

Git Manager is built as a **microkernel** with a **hexagonal (ports and adapters) core**. The design allows the CLI, web interface, and desktop app to share exactly the same domain logic and plugin system without any code duplication.

```
┌─────────────────────────────────────────────────────────┐
│  Interfaces (CLI · Web · Desktop)                        │
│  git-manager binary / Axum HTTP server / Tauri app       │
├─────────────────────────────────────────────────────────┤
│  Kernel                                                  │
│  Plugin loader · Service registry · Event bus           │
│  Command bus · Workflow engine · CredentialService       │
├─────────────────────────────────────────────────────────┤
│  Ports (Traits)                                          │
│  RepositoryProvider · AuthProvider · CredentialStore     │
│  GitExecutor · SshOperations · EventStore                │
├───────────────────┬─────────────────────────────────────┤
│  Domain           │  Adapters                           │
│  Account          │  MySQL / SQLite repositories        │
│  SSH key          │  Zig SSH provider                   │
│  Repository       │  Zig Git executor                   │
│  Git operations   │  Platform OS keychain               │
├───────────────────┴─────────────────────────────────────┤
│  Shared types (DTOs, errors, value objects)             │
└─────────────────────────────────────────────────────────┘
          ↕ Plugin boundary (static or dynamic .so)
┌─────────────────────────────────────────────────────────┐
│  Plugins                                                 │
│  GitHub · GitLab · Bitbucket · Azure DevOps             │
└─────────────────────────────────────────────────────────┘
```

### Technology stack

| Layer | Technology |
|-------|-----------|
| Business logic, kernel, plugins | Rust |
| SSH operations, git subprocess, filesystem, OS keychain | Zig (compiles to `libgm_native.a`, linked into Rust) |
| CLI interface | Rust (via `gm_interface_cli`) |
| Web interface | Rust + Axum + Tera templates |
| Desktop interface | Rust (Tauri backend) + TypeScript + Svelte |
| Database | MySQL (production) / SQLite (development) |
| Query layer | SQLx with compile-time verified queries |
| Async runtime | Tokio |

### Zig native layer

Git Manager uses Zig for all OS-level operations because Zig can call platform APIs (macOS Security.framework, GNOME libsecret) directly without needing a Rust FFI crate for every platform. The Zig code compiles into a static library (`zig-out/lib/libgm_native.a`) that the Rust build links against. You must run `zig build` before `cargo build`.

---

## 3. Supported Platforms

| Platform | Slug | SSH Host | SSH Port | Auth Methods |
|----------|------|----------|----------|--------------|
| GitHub | `github` | `github.com` | 22 | SSH key, PAT (`ghp_...`) |
| GitLab | `gitlab` | `gitlab.com` | 22 | SSH key, PAT (`glpat-...`) |
| Bitbucket | `bitbucket` | `bitbucket.org` | 22 | SSH key, App Password (`username:app_password`) |
| Azure DevOps | `azure_devops` | `ssh.dev.azure.com` | 22 | SSH key, PAT (`organization:token`) |
| SourceForge | `sourceforge` | `git.code.sf.net` | 22 | SSH key |

### Platform UUIDs (fixed by database seed migration)

These UUIDs are stable across all installations. Use them when scripting against the REST API.

| Platform | UUID |
|----------|------|
| GitHub | `00000000-0001-0000-0000-000000000001` |
| GitLab | `00000000-0002-0000-0000-000000000001` |
| Bitbucket | `00000000-0003-0000-0000-000000000001` |
| Azure DevOps | `00000000-0004-0000-0000-000000000001` |
| SourceForge | `00000000-0005-0000-0000-000000000001` |

---

## 4. Prerequisites and Installation

### System requirements

- **Rust** toolchain (stable, 1.75+): Install via [rustup.rs](https://rustup.rs)
- **Zig** 0.13 or 0.14: Download from [ziglang.org](https://ziglang.org/download/)
- **MySQL** 8.0+ (production) or SQLite 3 (development)
- **libsecret-1** dev headers (Linux only): `sudo apt install libsecret-1-dev`
- **Node.js** 18+ and npm (desktop app frontend only)

### Build sequence

The Zig library **must be built before Cargo**. The Rust `build.rs` in `gm_adapters` looks for `libgm_native.a` at the path Zig outputs.

```bash
# Step 1 — build the Zig native library
cd zig_native
zig build --release=safe
# Output: zig-out/lib/libgm_native.a
#         zig-out/include/gm_native.h
cd ..

# Step 2 — build the Rust workspace
cargo build --release

# Step 3 — (desktop only) build the Svelte frontend
cd crates/gm_interface_desktop/frontend
npm install
npm run build
cd ../../..
```

### Running the binaries

```bash
# CLI
GIT_MANAGER_DB_URL="mysql://user:pass@localhost/git_manager" \
  ./target/release/git-manager --help

# Web server
GIT_MANAGER_DB_URL="mysql://user:pass@localhost/git_manager" \
GIT_MANAGER_WEB_ADDR="127.0.0.1:8080" \
  ./target/release/git-manager-web

# Desktop app (development)
cd apps/desktop
cargo tauri dev
```

---

## 5. Environment Variables

All configuration is read from environment variables at startup. No config file is required.

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `GIT_MANAGER_DB_URL` | Yes (prod) | `mysql://root:root@localhost/git_manager` | MySQL connection URL. Format: `mysql://user:pass@host:port/dbname` |
| `GIT_MANAGER_SQLITE_PATH` | No | — | SQLite database file path. If set, overrides MySQL. Use `:memory:` for tests. |
| `RUST_LOG` | No | `info` | Log filter in [`env_logger` format](https://docs.rs/env_logger). Examples: `debug`, `git_manager=trace`, `warn` |
| `GIT_MANAGER_WEB_ADDR` | No | `127.0.0.1:8080` | Bind address for the web server. Use `0.0.0.0:8080` to listen on all interfaces. |
| `GIT_MANAGER_PLUGIN_DIR` | No | — | Directory to scan for dynamic `.so` / `.dylib` plugins. Absent = dynamic plugins disabled. |

### Example `.env` file

Create `.env` in the project root for local development:

```bash
GIT_MANAGER_SQLITE_PATH=./dev.db
RUST_LOG=git_manager=debug,info
GIT_MANAGER_WEB_ADDR=127.0.0.1:3000
```

> **Note:** The binaries do not automatically read `.env` files. Use a tool like `dotenv` or `source .env` before running, or integrate with your shell's environment.

---

## 6. Database Setup

Git Manager runs its own SQL migrations on startup. No manual `CREATE TABLE` steps are needed.

### MySQL (production)

```sql
-- Create the database and user
CREATE DATABASE git_manager CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'git_manager'@'localhost' IDENTIFIED BY 'your_password';
GRANT ALL PRIVILEGES ON git_manager.* TO 'git_manager'@'localhost';
FLUSH PRIVILEGES;
```

Then set:

```bash
GIT_MANAGER_DB_URL="mysql://git_manager:your_password@localhost/git_manager"
```

### SQLite (development and testing)

```bash
GIT_MANAGER_SQLITE_PATH="./git_manager_dev.db"
```

The file is created automatically if it doesn't exist.

### Migration behaviour

Migrations run automatically in order on every startup. They are additive (no destructive changes). If the database is already at the latest migration, startup proceeds immediately without any delay.

The migration files live in `crates/gm_adapters/src/persistence/migrations/`.

---

## 7. The SSH Key Isolation Model

This is the most important concept in Git Manager. Understanding it makes everything else clear.

### The problem with shared SSH keys

When you have two GitHub accounts — say `personal` (username: `alice`) and `work` (username: `alice-corp`) — and you set up one SSH key in `~/.ssh/config` pointing to `github.com`, every `git` command to GitHub uses the same key. The second account's repositories will always fail to authenticate because GitHub sees the key for `alice`, not `alice-corp`.

The traditional workaround is to add two Host entries in `~/.ssh/config`:

```
Host github.com-personal
  HostName github.com
  IdentityFile ~/.ssh/id_ed25519_personal

Host github.com-work
  HostName github.com
  IdentityFile ~/.ssh/id_ed25519_work
```

And clone repositories using the alias URL: `git@github.com-work:corp/repo.git` instead of `git@github.com:corp/repo.git`. This works but requires manual setup for every new repository.

### How Git Manager handles it

Git Manager automates this entirely:

1. **When you add an account**, Git Manager records the platform, alias, username, and email in the database.

2. **When you generate an SSH key for that account**, Git Manager:
   - Creates a key pair named `id_{key_type}_{platform}_{alias}` in `~/.ssh/` (e.g., `id_ed25519_github_work`)
   - Writes a Host block to `~/.ssh/config` with the alias `github.com-work`
   - Stores the key fingerprint and path in the database

3. **When you clone a repository**, Git Manager uses `GIT_SSH_COMMAND="ssh -i ~/.ssh/id_ed25519_github_work -o IdentitiesOnly=yes"` to isolate which key is used, regardless of which Host block matches.

4. **For all subsequent operations** (pull, push), Git Manager retrieves the active key for the account and passes it to the Zig git executor, which sets `GIT_SSH_COMMAND` accordingly.

### The `~/.ssh/config` entry format

For an account with alias `work` on GitHub:

```
Host github.com-work
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_github_work
  IdentitiesOnly yes
  Port 22
```

### SSH key naming convention

```
id_{key_type}_{platform_slug}_{account_alias}
```

Examples:
- `id_ed25519_github_personal` — Ed25519 key for personal GitHub account
- `id_ed25519_gitlab_freelance` — Ed25519 key for freelance GitLab account
- `id_rsa_bitbucket_client_acme` — RSA key for a Bitbucket client account

### Key types

| Type | Command string | Recommended? | Notes |
|------|---------------|-------------|-------|
| Ed25519 | `ed25519` | ✅ Yes — default | Small key, fast, modern. Supported by OpenSSH 6.5+ (2014). |
| RSA | `rsa` | ⚠️ Legacy systems only | Defaults to 4096 bits. Use only when the platform doesn't support Ed25519. |
| ECDSA | `ecdsa` | ❌ Not recommended | Vulnerable to weak RNG; prefer Ed25519. |

---

## 8. CLI Reference — Account Commands

### `git-manager account list`

Lists all registered accounts.

```bash
git-manager account list
git-manager account list --platform github
git-manager account list --platform gitlab
```

**Flags:**

| Flag | Type | Description |
|------|------|-------------|
| `--platform <slug>` | String | Filter by platform slug: `github`, `gitlab`, `bitbucket`, `azure_devops`, `sourceforge` |

**Output columns:** alias · platform · username · email · auth method · SSH key status · default

**Example output:**

```
ALIAS       PLATFORM   USERNAME     EMAIL                    AUTH  KEY     DEFAULT
──────────────────────────────────────────────────────────────────────────────────
personal    GitHub     alice        alice@personal.com       SSH   Active  ✓
work        GitHub     alice-corp   alice@company.com        SSH   Active
freelance   GitLab     alice-gl     alice@client.com         PAT   Active
```

---

### `git-manager account get`

Displays full details for one account.

```bash
git-manager account get personal
git-manager account get 00000000-0001-0000-0000-000000000099
```

**Arguments:**

| Argument | Description |
|---------|-------------|
| `<alias or UUID>` | Account alias (e.g., `work`) or UUID |

**Output:** all account fields including UUID, platform UUID, SSH host alias, timestamps.

---

### `git-manager account add`

Registers a new account.

```bash
git-manager account add \
  --alias work \
  --platform github \
  --username alice-corp \
  --email alice@company.com \
  --auth-method ssh
```

**Flags:**

| Flag | Required | Type | Description |
|------|----------|------|-------------|
| `--alias <alias>` | ✅ | String | Short name for this account. Lowercase letters, numbers, hyphens. Used in SSH key filenames and `~/.ssh/config`. Max 39 characters. |
| `--platform <slug>` | ✅ | String | `github`, `gitlab`, `bitbucket`, `azure_devops`, `sourceforge` |
| `--username <username>` | ✅ | String | Your username on the hosting platform |
| `--email <email>` | ✅ | String | Email for git commit author fields and SSH key comments |
| `--auth-method <method>` | No | String | `ssh` (default), `https_pat`, `https_password` |
| `--token <token>` | No | String | PAT or app password to store immediately. If omitted, use `account set-token` later. |

**What happens after adding:**
- Account is written to the database
- If `--token` was provided, it is encrypted and stored in the OS keychain
- No SSH key is generated yet — run `ssh generate` next

**Alias rules:**
- Must start with a letter or digit
- May contain lowercase letters, digits, and hyphens
- Maximum 39 characters
- Must be unique per platform (two accounts on the same platform cannot share an alias)

---

### `git-manager account remove`

Permanently deletes an account and all associated data.

```bash
git-manager account remove work
git-manager account remove 00000000-0001-0000-0000-000000000099
```

> ⚠️ **Irreversible.** This deletes:
> - The account database record
> - All SSH key records for this account (the key files in `~/.ssh/` are NOT deleted — remove them manually if desired)
> - The stored credential in the OS keychain
> - The `~/.ssh/config` Host entries for this account's keys

**Arguments:**

| Argument | Description |
|---------|-------------|
| `<alias or UUID>` | Account alias or UUID |

---

### `git-manager account set-default`

Marks one account as the default for its platform. The default account is used when a command doesn't specify `--account`.

```bash
git-manager account set-default work --platform github
```

**Arguments:**

| Argument | Description |
|---------|-------------|
| `<alias or UUID>` | Account to make default |

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--platform <slug>` | ✅ | Platform the account belongs to |

> Only one account per platform can be the default. Setting a new default automatically unsets the previous one.

---

### `git-manager account set-token`

Stores or replaces the API token (PAT or app password) for an account. The token is encrypted by the OS keychain before storage — it is never written to disk in plaintext.

```bash
git-manager account set-token work
# Prompts securely for the token

git-manager account set-token work --token ghp_xxxxxxxxxxxxxxxxxxxx
# Provide token directly (less secure — visible in shell history)
```

**Arguments:**

| Argument | Description |
|---------|-------------|
| `<alias or UUID>` | Account to update |

**Flags:**

| Flag | Description |
|------|-------------|
| `--token <token>` | Token value. Omit to be prompted interactively (recommended). |

**Token formats by platform:**

| Platform | Token format | Where to create |
|----------|-------------|-----------------|
| GitHub | `ghp_...` (classic) or fine-grained PAT | Settings → Developer settings → Personal access tokens |
| GitLab | `glpat-...` | User Settings → Access Tokens. Scopes: `read_user`, `read_api`, `read_repository` |
| Bitbucket | `username:app_password` | Account Settings → App passwords. Permissions: Account:Read, Repositories:Read |
| Azure DevOps | `organization:token` | User Settings → Personal Access Tokens |

---

## 9. CLI Reference — SSH Key Commands

### `git-manager ssh list`

Lists all SSH keys, optionally filtered by account.

```bash
git-manager ssh list
git-manager ssh list --account work
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--account <alias or UUID>` | Show keys for one account only |

**Output columns:** key name · account · type · fingerprint · status · last tested

**Status values:**

| Status | Meaning |
|--------|---------|
| `Active` | This is the current key for the account (only one key per account is active at a time) |
| `Inactive` | An older key for this account, superseded by a newer generation |
| `NotTested` | Key exists but connection has never been verified |
| `Verified` | Last SSH connection test succeeded |
| `Failed` | Last SSH connection test failed |

---

### `git-manager ssh generate`

Generates a new SSH key pair for an account and writes the `~/.ssh/config` Host entry.

```bash
git-manager ssh generate --account work
git-manager ssh generate --account work --key-type rsa
git-manager ssh generate --account work --passphrase
git-manager ssh generate --account work --no-agent
```

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--account <alias or UUID>` | Required | Which account to generate a key for |
| `--key-type <type>` | `ed25519` | Key algorithm: `ed25519`, `rsa` |
| `--passphrase` | No passphrase | Prompt for a passphrase to protect the private key |
| `--add-to-agent / --no-agent` | `--add-to-agent` | Whether to immediately add the key to the running SSH agent |

**What happens:**

1. The previous active key for this account is marked `Inactive` in the database (not deleted)
2. A new key pair is generated via `ssh-keygen` (the Zig layer calls the system binary)
3. The private key is written to `~/.ssh/id_{type}_{platform}_{alias}` with permissions `0600`
4. The public key is written to `~/.ssh/id_{type}_{platform}_{alias}.pub` with permissions `0644`
5. A `Host` block is written to `~/.ssh/config` (atomically via temp-file + rename)
6. The new key is recorded in the database with `is_active = true`
7. If `--add-to-agent`, `ssh-add` is called on the private key

**After generating**, copy the public key and add it to your platform:

```bash
git-manager ssh show-public-key --account work
# Then paste into GitHub/GitLab/etc
```

Or view it in the account detail page of the web interface.

---

### `git-manager ssh test`

Tests the SSH connection for an account's active key. Runs `ssh -T` to the platform and checks whether the server confirms authentication.

```bash
git-manager ssh test --account work
git-manager ssh test --account work --timeout 30000
```

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--account <alias or UUID>` | Required | Account to test |
| `--timeout <ms>` | 10000 | Connection timeout in milliseconds |

**What happens:**

1. Retrieves the active SSH key and host alias for the account
2. Runs: `ssh -T -i <key_path> -o IdentitiesOnly=yes -o BatchMode=yes <host_alias>`
3. Parses the server response for known success patterns:
   - GitHub: `Hi username! You've successfully authenticated...`
   - GitLab: `Welcome to GitLab, @username!`
   - Bitbucket: `logged in as username`
   - Azure DevOps: `remote: Shell access is not supported.` (this is a success — the key was accepted)
4. Updates `last_test_status` and `last_tested_at` in the database

**Exit codes:**

| Code | Meaning |
|------|---------|
| `0` | Connection successful — key is accepted |
| `1` | Connection failed — see error output for details |

**Common failure reasons:**

- Public key not yet added to the platform account
- Key was recently added (propagation delay of a few seconds)
- `ssh-agent` is not running and the key has a passphrase
- Firewall blocking outbound port 22

---

### `git-manager ssh add-to-agent`

Adds an account's active SSH key to the running SSH agent.

```bash
git-manager ssh add-to-agent --account work
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--account <alias or UUID>` | Account whose key to add |
| `--passphrase <pass>` | Passphrase if the key was generated with one |

**Requirement:** The `SSH_AUTH_SOCK` environment variable must be set (the SSH agent must be running).

---

## 10. CLI Reference — Repository Commands

### `git-manager repo list`

Lists all repositories known to Git Manager.

```bash
git-manager repo list
git-manager repo list --account work
git-manager repo list --cloned
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--account <alias or UUID>` | Show repositories for one account |
| `--cloned` | Show only locally cloned repositories |
| `--remote` | Show only remote-only repositories |

**Output columns:** name · account · branch · local path (if cloned) · last commit SHA

---

### `git-manager repo clone`

Clones a remote repository using an account's SSH key.

```bash
git-manager repo clone git@github.com:org/repo.git --account work
git-manager repo clone git@github.com:org/repo.git --account work --dest ~/projects/repo
git-manager repo clone git@github.com:org/repo.git --account work --branch develop
git-manager repo clone git@github.com:org/repo.git --account work --shallow
```

**Arguments:**

| Argument | Description |
|---------|-------------|
| `<url>` | SSH or HTTPS remote URL. For SSH: `git@github.com:owner/repo.git`. For HTTPS: `https://github.com/owner/repo.git` |

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--account <alias or UUID>` | Required | Which account's SSH key to use for authentication |
| `--dest <path>` | `~/git-repos/<repo-name>` | Local directory to clone into. Must not already exist. |
| `--branch <branch>` | Default branch | Clone a specific branch |
| `--shallow` | Off | Perform a depth-1 shallow clone. Faster for large repositories. Full history is not available after a shallow clone. |

**How the clone works internally:**

1. Retrieves the active SSH key path for the account
2. Runs: `GIT_SSH_COMMAND="ssh -i <key_path> -o IdentitiesOnly=yes" git clone <url> <dest>`
3. Records the repository in the database with `is_cloned = true` and the local path
4. Returns the repository details

**URL rewriting:** Git Manager does NOT rewrite the URL. Pass the standard SSH URL from the platform. The `GIT_SSH_COMMAND` environment variable is what ensures the correct key is used, so the URL does not need to contain the host alias.

---

### `git-manager repo status`

Shows the working directory status of a cloned repository.

```bash
git-manager repo status
git-manager repo status ~/projects/my-repo
git-manager repo status --uuid 00000000-aaaa-bbbb-cccc-000000000001
```

**Arguments:**

| Argument | Description |
|---------|-------------|
| `<path>` | Path to the local repository (defaults to current directory) |

**Flags:**

| Flag | Description |
|------|-------------|
| `--uuid <uuid>` | Identify the repository by its Git Manager UUID instead of path |

**Output:** Lists staged, unstaged, and untracked files with their two-character porcelain status codes.

**Status codes:**

| Code | Meaning |
|------|---------|
| `M ` | Modified, staged |
| ` M` | Modified, not staged |
| `A ` | New file, staged |
| `D ` | Deleted, staged |
| ` D` | Deleted, not staged |
| `??` | Untracked |
| `UU` | Merge conflict |

---

## 11. CLI Reference — Git Operation Commands

### `git-manager repo pull`

Fetches and integrates remote changes into the local branch.

```bash
git-manager repo pull ~/projects/my-repo --account work
git-manager repo pull --uuid <repo-uuid> --account work
git-manager repo pull ~/projects/my-repo --account work --rebase
git-manager repo pull ~/projects/my-repo --account work --branch main
```

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--account <alias or UUID>` | Required | Account whose SSH key to use |
| `--rebase` | Off | Use `git pull --rebase` instead of `--merge` |
| `--branch <branch>` | Current branch | Pull a specific branch |

**Output:** Number of commits pulled, new HEAD SHA.

**Conflict handling:** If the pull results in a merge conflict, the operation reports `had_conflicts: true` and exits with code 1. The local repository is left with conflict markers. Resolve them with standard git tools, then commit and push.

---

### `git-manager repo push`

Stages all changes, creates a commit, and pushes to the remote.

```bash
git-manager repo push ~/projects/my-repo \
  --account work \
  --message "feat: add login page"

git-manager repo push ~/projects/my-repo \
  --account work \
  --message "fix: correct null check" \
  --branch feature/login \
  --force
```

**Flags:**

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--account <alias or UUID>` | ✅ | — | Account whose SSH key to use |
| `--message <msg>` | ✅ | — | Commit message |
| `--branch <branch>` | No | Current branch | Branch to push to |
| `--force` | No | Off | Push with `--force-with-lease` (safer than `--force`) |

**Three-step sequence:**

1. `git add -A` — stages all modified, new, and deleted files
2. `git commit -m "<message>"` — creates a commit with the correct author (from the account's username + email)
3. `git push origin <branch>` — pushes to the remote

If any step fails, subsequent steps are skipped.

---

### `git-manager repo sync`

Pulls then pushes in one command. Equivalent to `pull --rebase` followed by `push`.

```bash
git-manager repo sync ~/projects/my-repo \
  --account work \
  --message "chore: sync"
```

**Flags:**

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--account <alias or UUID>` | ✅ | — | Account whose SSH key to use |
| `--message <msg>` | ✅ | — | Commit message for any local changes |
| `--branch <branch>` | No | Default branch | Branch to sync |

**Sequence:**

1. Pull with `--rebase` to integrate remote changes
2. If the pull has conflicts → abort, report conflicts, exit
3. Commit any local changes with the provided message
4. Push to remote

---

## 12. CLI Reference — Configuration Commands

### `git-manager config list`

Shows all configuration keys and their current values.

```bash
git-manager config list
```

**Output:**

```
KEY                         VALUE
────────────────────────────────────────────
ssh.default_key_type        ed25519
ssh.connect_timeout_ms      10000
ssh.auto_add_to_agent       true
git.default_remote          origin
git.max_concurrent_ops      4
ui.show_banner              true
ui.log_level                info
```

---

### `git-manager config get`

Gets the value of a single configuration key.

```bash
git-manager config get ssh.default_key_type
git-manager config get ssh.connect_timeout_ms
```

**Arguments:**

| Argument | Description |
|---------|-------------|
| `<key>` | Configuration key (see table below) |

---

### `git-manager config set`

Sets a configuration value.

```bash
git-manager config set ssh.default_key_type ed25519
git-manager config set ssh.connect_timeout_ms 30000
git-manager config set git.default_remote upstream
```

**Arguments:**

| Argument | Description |
|---------|-------------|
| `<key>` | Configuration key |
| `<value>` | New value |

### All configuration keys

| Key | Default | Description |
|-----|---------|-------------|
| `ssh.default_key_type` | `ed25519` | Default SSH key type when generating new keys. Values: `ed25519`, `rsa` |
| `ssh.connect_timeout_ms` | `10000` | Timeout for SSH connection tests, in milliseconds |
| `ssh.auto_add_to_agent` | `true` | Whether to automatically add newly generated keys to the SSH agent |
| `git.default_remote` | `origin` | Remote name used for push and pull when not specified |
| `git.max_concurrent_ops` | `4` | Maximum number of concurrent git operations (reserved for future batch operations) |
| `ui.show_banner` | `true` | Whether to show the ASCII art banner on CLI startup |
| `ui.log_level` | `info` | Log verbosity for CLI operations: `error`, `warn`, `info`, `debug`, `trace` |

---

## 13. CLI Reference — Operation Log

### `git-manager ops list`

Shows recent git operations (clone, pull, push) with their outcomes.

```bash
git-manager ops list
git-manager ops list --account work
git-manager ops list --limit 50
```

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--account <alias or UUID>` | All accounts | Filter by account |
| `--limit <n>` | `20` | Number of operations to show |

**Output columns:** timestamp · operation · account · repository · result · SHA

---

## 14. Web Interface

The web interface is an Axum-based HTML application served by the `git-manager-web` binary. It provides all account, SSH, and repository management features through a browser UI.

### Starting the web server

```bash
GIT_MANAGER_DB_URL="mysql://user:pass@localhost/git_manager" \
GIT_MANAGER_WEB_ADDR="127.0.0.1:8080" \
  ./target/release/git-manager-web
```

Open `http://127.0.0.1:8080` in a browser.

### Pages

| URL | Description |
|-----|-------------|
| `/` | Dashboard — account count, repository count, recent activity |
| `/accounts` | Account list with status badges |
| `/accounts/new` | Add account form |
| `/accounts/:uuid` | Account detail: SSH key display, repository list, token update |
| `/accounts/:uuid/remove` | POST — removes account |
| `/repositories` | Repository list (cloned and remote-only, filterable by account) |
| `/clone` | Clone wizard — URL input, account picker, options |
| `/git-ops` | Git operations panel — pull/push/sync for cloned repositories |
| `/ssh` | SSH key list filterable by account |
| `/ssh/generate` | SSH key generation form |
| `/ssh/:uuid/test` | POST — triggers SSH connection test, shows result |

### Server-Sent Events

The web server provides a real-time event stream for the dashboard. Subscribe at `/api/v1/sse` to receive events as operations complete. The event format is newline-delimited JSON:

```
event: git_op_completed
data: {"operation":"pull","account":"work","commits_pulled":3,"sha":"abc1234"}
```

---

## 15. REST API Reference

The web server exposes a versioned REST API at `/api/v1`. All endpoints accept and return `application/json`. Pagination uses `?page=1&per_page=50` query parameters.

### Accounts

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/accounts` | List all accounts. Query: `?platform_id=<uuid>` |
| `POST` | `/api/v1/accounts` | Create account. Body: `{alias, platform_id, username, email, auth_method, token?}` |
| `GET` | `/api/v1/accounts/:uuid` | Get account details |
| `DELETE` | `/api/v1/accounts/:uuid` | Remove account |
| `POST` | `/api/v1/accounts/:uuid/default` | Set as default for its platform |

**Create account request body:**

```json
{
  "alias": "work",
  "platform_id": "00000000-0001-0000-0000-000000000001",
  "username": "alice-corp",
  "email": "alice@company.com",
  "auth_method": "ssh",
  "token": "ghp_xxxx"
}
```

### SSH Keys

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/ssh-keys` | List keys. Query: `?account_id=<uuid>` |
| `POST` | `/api/v1/ssh-keys/generate` | Generate key. Body: `{account_id, key_type, add_to_agent, passphrase?}` |
| `POST` | `/api/v1/ssh-keys/:uuid/test` | Test connection. Body: `{timeout_ms?}` |

**Generate key request body:**

```json
{
  "account_id": "00000000-aaaa-bbbb-cccc-000000000001",
  "key_type": "ed25519",
  "add_to_agent": true,
  "passphrase": null
}
```

### Repositories

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/repositories` | List repositories. Query: `?account_id=<uuid>&cloned=true` |
| `POST` | `/api/v1/repositories/clone` | Clone repository |

**Clone request body:**

```json
{
  "url": "git@github.com:org/repo.git",
  "account_id": "00000000-aaaa-bbbb-cccc-000000000001",
  "destination": null,
  "branch": null,
  "shallow": false
}
```

### Git Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/git/pull` | Pull. Body: `{repository_id, account_id, branch?, rebase?}` |
| `POST` | `/api/v1/git/push` | Stage + commit + push. Body: `{repository_id, account_id, message, branch?, force?}` |
| `GET` | `/api/v1/git/status` | Status. Query: `?repository_id=<uuid>` |

**Pull request body:**

```json
{
  "repository_id": "00000000-dddd-eeee-ffff-000000000001",
  "account_id": "00000000-aaaa-bbbb-cccc-000000000001",
  "branch": "main",
  "rebase": true
}
```

**Push request body:**

```json
{
  "repository_id": "00000000-dddd-eeee-ffff-000000000001",
  "account_id": "00000000-aaaa-bbbb-cccc-000000000001",
  "message": "feat: add new feature",
  "branch": "main",
  "force": false
}
```

### Standard response format

All responses follow this envelope:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

On error:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "AccountNotFound",
    "message": "No account with alias 'work' exists"
  }
}
```

---

## 16. Desktop Application

The desktop application is built with Tauri (Rust backend) and Svelte (TypeScript frontend). It provides the full Git Manager feature set in a native window with no browser required.

### Starting the desktop app

```bash
# Development (hot reload, dev tools available)
cd apps/desktop
cargo tauri dev

# Production build
cargo tauri build
# Output binary in target/release/bundle/
```

### Pages

| Page | Description |
|------|-------------|
| Dashboard | Summary stats (accounts, repos, keys) with quick-access cards |
| Accounts | Full account management: list, add, view detail, remove, set token |
| Repositories | Local and remote repository list, detail view with git status |
| SSH Keys | Key list per account, generate new key, test connection, add to agent |
| Clone Wizard | 3-step wizard: URL → account + options → success |
| Sync | Pull/push/sync workflow with real-time operation log |

### Tauri Commands (Frontend → Backend IPC)

The Svelte frontend calls these commands via Tauri's `invoke()` IPC mechanism. All parameter names are in `snake_case` (Tauri 1.x does not auto-convert to camelCase).

**Account commands:**

| Command | Parameters | Returns |
|---------|-----------|---------|
| `list_accounts` | `{platform_id?: string}` | `AccountDto[]` |
| `get_account` | `{uuid: string}` | `AccountDto \| null` |
| `add_account` | `{cmd: AddAccountCommand}` | `AccountDto` |
| `remove_account` | `{uuid: string}` | `void` |
| `set_default_account` | `{account_uuid, platform_id: string}` | `void` |
| `store_account_token` | `{account_uuid, token: string}` | `void` |
| `list_platforms` | — | `Platform[]` |

**SSH key commands:**

| Command | Parameters | Returns |
|---------|-----------|---------|
| `list_ssh_keys` | `{account_uuid: string}` | `SshKeyDto[]` |
| `generate_ssh_key` | `{cmd: GenerateSshKeyCommand}` | `SshKeyDto` |
| `test_ssh_connection` | `{cmd: TestSshConnectionCommand}` | `SshTestResult` |
| `add_key_to_agent` | `{account_uuid: string}` | `void` |
| `get_public_key` | `{account_uuid: string}` | `string \| null` |

**Repository commands:**

| Command | Parameters | Returns |
|---------|-----------|---------|
| `list_repositories` | `{account_uuid?: string}` | `RepositoryDto[]` |
| `clone_repository` | `{cmd: CloneRepositoryCommand}` | `RepositoryDto` |
| `get_repository` | `{uuid: string}` | `RepositoryDto \| null` |
| `get_repository_local_path` | `{uuid: string}` | `string \| null` |

**Git operation commands:**

| Command | Parameters | Returns |
|---------|-----------|---------|
| `git_pull` | `{cmd: PullRepositoryCommand}` | `GitOpResult` |
| `git_push` | `{cmd: PushRepositoryCommand}` | `GitOpResult` |
| `git_status` | `{repository_uuid: string}` | `GitStatusEntry[]` |
| `sync_repository` | `{repository_uuid, account_uuid, commit_message, branch: string}` | `SyncResult` |

### TypeScript type definitions

```typescript
interface AccountDto {
  uuid:          string;
  platform_id:   string;
  platform_name: string | null;
  alias:         string;
  username:      string;
  email:         string;
  auth_method:   'ssh' | 'https_pat' | 'https_password';
  is_default:    boolean;
  is_active:     boolean;
  created_at:    string;
}

interface SshKeyDto {
  uuid:              string;
  account_id:        string;
  name:              string;       // "id_ed25519_github_work"
  key_type:          'Ed25519' | 'Rsa' | 'Ecdsa';
  public_key:        string;       // full public key string
  fingerprint:       string;       // "SHA256:..."
  private_key_path:  string;       // "~/.ssh/id_ed25519_github_work"
  is_active:         boolean;
  last_test_status:  'NotTested' | 'Success' | 'Failed';
  last_tested_at:    string | null;
  created_at:        string;
}

interface RepositoryDto {
  uuid:             string;
  account_id:       string;
  name:             string;
  full_name:        string;
  local_path:       string | null; // null if not cloned
  remote_url:       string;
  clone_url_ssh:    string | null;
  default_branch:   string;
  is_cloned:        boolean;
  last_commit_sha:  string | null;
  created_at:       string;
}

interface GitOpResult {
  commits_transferred: number;
  current_sha:         string | null;
  had_conflicts:       boolean;
}
```

---

## 17. Credential Storage and Security

### Where credentials are stored

Git Manager never writes credentials to plain files. All secrets are stored in the platform OS keychain:

| Platform | Backend | Description |
|----------|---------|-------------|
| Linux (desktop) | GNOME libsecret | Communicates with GNOME Keyring, KWallet (via compat), or KeePassXC over D-Bus |
| Linux (headless/server) | AES-256-GCM file | `~/.git-manager/secrets.enc` — encrypted with a machine-specific key derived from `/etc/machine-id` + `$HOME` using HKDF-SHA256 |
| macOS | Security.framework | macOS system Keychain — same store used by Safari, SSH, and all macOS apps |
| Fallback (any) | AES-256-GCM file | Same as Linux headless |

### Encryption details (file fallback)

When the system keychain is unavailable, credentials are stored in `~/.git-manager/secrets.enc`.

- **Cipher:** AES-256-GCM (authenticated encryption — detects tampering)
- **Key derivation:** HKDF-SHA256 using machine-specific material (`/etc/machine-id` + `$HOME`)
- **Salt:** HKDF default (all-zeros — the IKM provides sufficient entropy)
- **Info string:** `"git-manager-v1-credentials"`
- **Nonce:** 12 random bytes per entry, generated from the OS CSPRNG (`/dev/urandom`)
- **File format:** Length-prefixed binary records containing JSON with base64-encoded nonce, tag, and ciphertext

> ⚠️ The encrypted file is tied to one machine + one user. Copying it to another machine or renaming the user account will cause decryption failures.

### What is stored and under what label

Each credential is stored under a `(label, username)` pair:

- **Label:** `account-pat` (constant for all account credentials)
- **Username:** the account's UUID

Example: the PAT for account `work` (UUID `abc-123`) is stored as `("account-pat", "abc-123")`.

### SSH private keys

SSH private keys are stored as files in `~/.ssh/` with mode `0600` (owner-only read/write). They are never stored in the database — only the path, fingerprint, and public key are recorded.

---

## 18. Configuration Reference

### `~/.ssh/config` entries written by Git Manager

Git Manager adds one `Host` block per SSH key it generates. Example for three accounts:

```
# === Git Manager: personal (github) ===
Host github.com-personal
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_github_personal
  IdentitiesOnly yes
  Port 22

# === Git Manager: work (github) ===
Host github.com-work
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_github_work
  IdentitiesOnly yes
  Port 22

# === Git Manager: freelance (gitlab) ===
Host gitlab.com-freelance
  HostName gitlab.com
  User git
  IdentityFile ~/.ssh/id_ed25519_gitlab_freelance
  IdentitiesOnly yes
  Port 22
```

Git Manager writes these entries atomically (via temp-file + rename) so the config file is never in a partially-written state if the process crashes.

### Database tables (summary)

| Table | Purpose |
|-------|---------|
| `platforms` | Seed data — the 5 supported platforms and their metadata |
| `accounts` | Registered accounts (one row per alias-platform combination) |
| `ssh_keys` | SSH key records (fingerprint, path, status, test results) |
| `ssh_host_configs` | The `~/.ssh/config` Host block metadata for each key |
| `repositories` | Cloned and discovered repositories |
| `git_operations` | Audit log of clone/pull/push operations |
| `domain_events` | Immutable event log (AccountAdded, KeyGenerated, RepositoryCloned, etc.) |

---

## 19. Platform-Specific Guides

### GitHub

**Required scopes for PAT (classic):** `repo`, `user:email`

**PAT creation URL:** `https://github.com/settings/tokens`

**Adding SSH public key:** Settings → SSH and GPG keys → New SSH key

**Verifying the connection:**
```bash
git-manager ssh test --account work
# Expected: "Hi alice-corp! You've successfully authenticated..."
```

---

### GitLab

**Required scopes for PAT:** `read_user`, `read_api`, `read_repository`

**PAT creation URL:** `https://gitlab.com/-/user_settings/personal_access_tokens`

**Adding SSH public key:** User Settings → SSH Keys

**Credential format for Bitbucket-style credentials:** Not applicable — GitLab uses a standard `glpat-...` token.

**Verifying the connection:**
```bash
git-manager ssh test --account freelance
# Expected: "Welcome to GitLab, @alice-gl!"
```

---

### Bitbucket

**Bitbucket does not use PATs for git authentication** — it uses App Passwords.

**Creating an App Password:**
1. Go to: `https://bitbucket.org/account/settings/app-passwords/`
2. Click **Create app password**
3. Required permissions: Account:Read, Repositories:Read, Repositories:Write
4. Copy the generated password

**Credential format:** Enter as `username:app_password` (colon-separated)

```bash
git-manager account set-token client-bitbucket
# Enter: shakamoses:ATBBxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**Adding SSH public key:** Personal settings → SSH keys

**Verifying the connection:**
```bash
git-manager ssh test --account client-bitbucket
# Expected: "logged in as shakamoses"
```

---

### Azure DevOps

**PAT creation:**
1. Go to: `https://dev.azure.com/<your-org>/_usersettings/tokens`
2. Click **New Token**
3. Required scopes: **Code (Read & Write)**, **Project and Team (Read)**
4. Copy the generated token

**Credential format:** Enter as `organization:token` (colon-separated)

```bash
git-manager account set-token corp-azure
# Enter: myorg:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**SSH key note:** Azure DevOps uses `ssh.dev.azure.com` (not `dev.azure.com`) for SSH.

**Repository URL format:** `git@ssh.dev.azure.com:v3/organization/project/repo`

**Verifying the connection:**
```bash
git-manager ssh test --account corp-azure
# Expected: "remote: Shell access is not supported."
# (This is a success — Azure DevOps returns this for authenticated SSH connections)
```

---

## 20. Workflow Examples

### Complete new account setup from scratch

```bash
# 1. Add the account
git-manager account add \
  --alias work \
  --platform github \
  --username alice-corp \
  --email alice@company.com

# 2. Generate an SSH key
git-manager ssh generate --account work
# Output: Key generated at ~/.ssh/id_ed25519_github_work
#         Public key:  ssh-ed25519 AAAA... alice@company.com
#         Fingerprint: SHA256:xxxxx...

# 3. Add the public key to GitHub
git-manager ssh generate --account work
# Copy the displayed public key and paste it at:
# https://github.com/settings/ssh/new

# 4. Test the connection
git-manager ssh test --account work
# Output: ✓ Authenticated as alice-corp

# 5. Store a PAT (for API operations like repository listing)
git-manager account set-token work
# Enter your GitHub PAT when prompted

# 6. Clone a repository
git-manager repo clone git@github.com:myorg/backend.git --account work
# Cloned to ~/git-repos/backend

# 7. Make changes and push
cd ~/git-repos/backend
# ... edit files ...
git-manager repo push . --account work --message "feat: implement search"
# Staged 3 files, committed, pushed 1 commit
```

---

### Switching between accounts for the same platform

```bash
# List all GitHub accounts
git-manager account list --platform github
# personal  (active key, not default)
# work      (active key, DEFAULT)
# client    (active key, not default)

# Pull with a specific account
git-manager repo pull ~/projects/client-website --account client

# Push with a specific account
git-manager repo push ~/projects/client-website \
  --account client \
  --message "fix: responsive navigation"
```

---

### Headless server setup (no GNOME keyring)

On a headless Linux server, the GNOME keyring is not running. Git Manager automatically falls back to the AES-256-GCM encrypted file store at `~/.git-manager/secrets.enc`.

```bash
# Works exactly the same — no extra configuration needed
export GIT_MANAGER_DB_URL="mysql://git_manager:pass@localhost/git_manager"

git-manager account add \
  --alias deploy \
  --platform github \
  --username deploy-bot \
  --email deploy@company.com

git-manager ssh generate --account deploy --no-agent

# Copy the public key to GitHub
git-manager ssh list --account deploy
# Shows the public key

git-manager ssh test --account deploy
```

---

### Setting up a CI/CD pipeline

```bash
# On the CI machine — run once during setup
git-manager account add \
  --alias ci \
  --platform github \
  --username github-actions-bot \
  --email ci@company.com \
  --auth-method https_pat

# Store a PAT with deployment permissions
git-manager account set-token ci --token "$GITHUB_DEPLOY_TOKEN"

git-manager ssh generate --account ci

# Export the public key for adding to GitHub Deploy Keys
git-manager ssh list --account ci
```

---

## 21. Plugin System

Git Manager uses a plugin architecture for platform integrations. Each platform (GitHub, GitLab, Bitbucket, Azure DevOps) is implemented as a plugin that the kernel loads at startup.

### Plugin types

| Type | Loading mechanism | Use case |
|------|------------------|---------|
| **Static** | Compiled into the binary at build time (`Vec<Box<dyn Plugin>>` in `main.rs`) | Built-in platform plugins |
| **Dynamic** | Loaded from `.so` / `.dylib` files at runtime via `libloading` | Third-party or enterprise plugins |

### How plugins register capabilities

Each plugin implements the `Plugin` trait:

```rust
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;

    /// Called when the plugin is loaded. Register providers and services
    /// in the ServiceRegistry here.
    fn on_load(&self, registry: Arc<ServiceRegistry>) -> Result<(), PluginError>;

    fn on_unload(&self) -> Result<(), PluginError>;

    fn get_event_subscriptions(&self) -> Vec<EventSubscription>;
}
```

During `on_load()`, the plugin receives a `ServiceRegistry` where it can:
- **Get** the `CredentialService` (registered by the infrastructure plugin at priority 0)
- **Register** its own `RepositoryProvider` and `AuthProvider` for other services to look up

### Plugin load priority

Plugins are loaded in ascending priority order:

| Priority | Plugin | Purpose |
|----------|--------|---------|
| 0 | `InfrastructurePlugin` (binary-level) | Registers `CredentialService` so provider plugins can find it |
| 10 | `GitHubPlugin`, `GitLabPlugin`, `BitbucketPlugin`, `AzureDevOpsPlugin` | Register `RepositoryProvider` and `AuthProvider` |
| 20 | `DesktopPlugin`, `CliPlugin`, `WebPlugin` | Register interface-level services |

### Writing a dynamic plugin

A dynamic plugin must be a Rust crate that produces a shared library (`.so` on Linux, `.dylib` on macOS) and exports two C-ABI symbols:

```rust
// In your plugin crate:

#[no_mangle]
pub extern "C" fn _gm_plugin_create() -> *mut std::ffi::c_void {
    let plugin = Box::new(MyPlugin::new());
    let double_boxed = Box::new(plugin as Box<dyn gm_kernel::contracts::plugin::Plugin>);
    Box::into_raw(double_boxed) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn _gm_plugin_destroy(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr as *mut Box<dyn gm_kernel::contracts::plugin::Plugin>);
        }
    }
}
```

Place the compiled `.so` file in the directory pointed to by `GIT_MANAGER_PLUGIN_DIR`. Git Manager will scan that directory at startup and load all valid plugin files.

> ⚠️ **Compatibility requirement:** Dynamic plugins must be compiled with the exact same Rust toolchain version as the kernel binary. Mismatched vtable layouts will cause crashes or undefined behaviour.

---

## 22. Troubleshooting

### SSH connection test fails

**"Connection refused" / "Network unreachable"**
- Check that port 22 is not blocked by a firewall
- For Azure DevOps, verify you are testing against `ssh.dev.azure.com` not `dev.azure.com`

**"Permission denied (publickey)"**
- The public key has not been added to the platform (or was added to the wrong account)
- The key file permissions are wrong: `chmod 600 ~/.ssh/id_ed25519_github_work`
- The SSH agent is holding an old key that is being offered first. Run: `ssh-add -D` to clear the agent, then `git-manager ssh add-to-agent --account work`

**"Host key verification failed"**
- Run `ssh -T git@github.com` once manually to accept the host key into `~/.ssh/known_hosts`, then retry

---

### "No active SSH key" error on clone/pull/push

The account has no SSH key generated yet:

```bash
git-manager ssh list --account work
# If empty:
git-manager ssh generate --account work
```

---

### Credential decryption fails after re-installation

The file fallback (`~/.git-manager/secrets.enc`) is tied to the machine's `/etc/machine-id` and `$HOME`. If either changed (e.g., fresh OS install, user rename), decryption will fail.

**Resolution:** Re-enter your tokens:
```bash
git-manager account set-token work
```

---

### "Duplicate plugin" error at startup

Two plugins with the same name are being loaded. Check `GIT_MANAGER_PLUGIN_DIR` for duplicate `.so` files.

---

### Database migration fails

```
Error: migration 20240101000002_add_repositories has already been applied
  but its checksum does not match the one in the migration file.
```

A migration file was modified after being applied. Do not edit migration files that have already been run. Create a new migration file for schema changes instead.

---

### libsecret D-Bus error on Linux

```
Failed to connect to secret service: Could not connect: No such file or directory
```

The GNOME keyring daemon is not running. Git Manager automatically falls back to the encrypted file store — no action required. This is expected on headless systems.

To explicitly start the keyring on a desktop session:

```bash
gnome-keyring-daemon --start --components=secrets
```

---

### `zig build` fails with "Security/Security.h not found"

This error appears in `zls` (the Zig language server) on Linux — it is **not** a build error. The build itself is correct because `platform/index.zig` uses a comptime OS switch to route to `linux_keyring.zig` on Linux, never importing the macOS file.

The `macos_keychain.zig` file has been updated to wrap the `@cImport` in a comptime OS conditional, which prevents zls from evaluating the Apple header on non-macOS systems.

---

## 23. Developer Guide

### Repository structure

```
git_manager/
├── apps/
│   ├── cli/src/main.rs        ← CLI binary entry point (wires all concrete types)
│   ├── web/src/main.rs        ← Web server binary entry point
│   └── desktop/src/main.rs   ← Tauri desktop binary entry point
├── crates/
│   ├── gm_shared/             ← DTOs, errors, value objects (no dependencies)
│   ├── gm_domain/             ← Business logic (depends only on gm_shared)
│   ├── gm_ports/              ← Trait interfaces for adapters and plugins
│   ├── gm_kernel/             ← Microkernel: plugin loader, event bus, command bus
│   ├── gm_adapters/           ← Concrete adapter implementations (MySQL, SQLite, Zig FFI)
│   ├── gm_plugin_github/      ← GitHub provider plugin
│   ├── gm_plugin_gitlab/      ← GitLab provider plugin
│   ├── gm_plugin_bitbucket/   ← Bitbucket provider plugin
│   ├── gm_plugin_azure_devops/← Azure DevOps provider plugin
│   ├── gm_interface_cli/      ← CLI interface crate
│   ├── gm_interface_web/      ← Axum web server + Tera templates
│   └── gm_interface_desktop/  ← Tauri backend + Svelte frontend
└── zig_native/
    ├── build.zig
    └── src/
        ├── root.zig           ← Library root
        ├── ssh/               ← SSH key generation, agent, config, connection test
        ├── git_exec/          ← git subprocess execution
        ├── filesystem/        ← Atomic writes, permissions
        └── platform/          ← OS keychain backends
            ├── index.zig      ← Comptime platform selector
            ├── linux_keyring.zig   ← GNOME libsecret
            ├── macos_keychain.zig  ← macOS Security.framework
            └── fallback_keyring.zig← AES-256-GCM encrypted file
```

### Dependency direction rule

The enforced dependency direction is:

```
apps/* → gm_interface_* → gm_kernel → gm_ports → gm_domain → gm_shared
                                 ↑
                         gm_adapters (depends inward)
                         gm_plugin_* (depends inward via gm_ports/gm_kernel)
```

**Never break this rule.** The domain must never import adapters, kernel, or interface crates. Violations will make the system difficult to test and extend.

### Adding a new platform

1. Create a new crate: `crates/gm_plugin_<platform>/`
2. Implement the plugin following the GitHub plugin as a reference template:
   - `Cargo.toml` — depends on `gm_shared`, `gm_ports`, `gm_kernel`
   - `src/lib.rs` — exports `create_plugin() -> XxxPlugin`
   - `src/plugin.rs` — implements `Plugin` trait, registers providers in `on_load()`
   - `src/client.rs` — platform HTTP client
   - `src/auth.rs` — authentication helpers
   - `src/auth_provider.rs` — implements `AuthProvider` port
   - `src/repository_provider.rs` — implements `RepositoryProvider` port
3. Add the new platform UUID to the seed migration
4. Add the plugin to the `static_plugins` list in each binary's `main.rs`
5. Add `platform_info()` and `platform_display_name()` entries in each binary

### Running tests

```bash
# Unit tests (no database, no network, no filesystem)
cargo test --lib

# Integration tests (requires a running MySQL or SQLite)
GIT_MANAGER_SQLITE_PATH=":memory:" cargo test --test '*'

# Zig tests
cd zig_native
zig build test
```

### Code style

- Rust: run `cargo fmt` and `cargo clippy --all-targets -- -D warnings` before committing
- Zig: run `zig fmt src/` before committing
- No `unwrap()` outside test code — use `?` or explicit `match`/`if let`
- All public types in `gm_shared` must implement `Debug`, `Clone`, `Serialize`, `Deserialize`
- All domain entities must implement `Debug` (enforced by `#![deny(missing_debug_implementations)]`)

---

*Git Multi-Account Manager — built with Rust, Zig, and Svelte.*