# Git Multi-Account Manager — Complete System Guide

## Overview

Git Multi-Account Manager lets you manage multiple Git hosting accounts
(GitHub, GitLab, Bitbucket, Azure DevOps, SourceForge, and custom/self-hosted)
from a single CLI, web interface, or desktop app. Each account gets its own
SSH key, its own `~/.ssh/gitmanager/config` Host block, and full identity
isolation — no key leakage between accounts.

---

## 1. Accounts

### Commands

| Command | Description |
|---|---|
| `git-manager account add --alias <name> --platform <platform> --username <user> --email <email>` | Create a new account |
| `git-manager account list [--platform <slug>]` | List all accounts (optionally filtered by platform) |
| `git-manager account show <alias>` | Show detailed account information |
| `git-manager account remove <alias> [-y]` | Delete an account (skip confirmation with `-y`) |
| `git-manager account set-default <alias> [--platform <slug>]` | Set as default for its platform |
| `git-manager account test <alias>` | Test SSH connection for account |
| `git-manager setup` | Interactive setup wizard (recommended for new users) |
| `git-manager setup --platform github --alias work --username user --email e@m.com --auth ssh` | Non-interactive setup |

### Platform Slugs

| Slug | Display Name |
|---|---|
| `github` | GitHub |
| `gitlab` | GitLab |
| `bitbucket` | Bitbucket |
| `azure_devops` | Azure DevOps |
| `sourceforge` | SourceForge |
| `self_hosted` | Self-Hosted |
| `cloud_storage` | Cloud Storage |
| `local_path` | Local Path |
| `custom` | Custom |

### Account Lifecycle

1. `git-manager setup` (interactive) or `git-manager account add` (direct):
   - Creates the account record in the database
   - Prompts for SSH key generation and connection test (or HTTPS PAT storage)
2. After creation, SSH keys can be managed via `git-manager ssh *` commands
3. PATs / tokens can be stored via `git-manager account add` with `--auth https` or via interactive menu

---

## 2. SSH Keys

### Commands

| Command | Description |
|---|---|
| `git-manager ssh generate --account <alias> [--key-type ed25519] [--passphrase ...] [--add-to-agent]` | Generate SSH key, write config, optionally add to agent |
| `git-manager ssh list [--account <alias>]` | List SSH keys for an account (or all) |
| `git-manager ssh test --account <alias> [--timeout-ms 10000]` | Test SSH connection against platform |
| `git-manager ssh validate --account <alias> [--timeout-ms 10000]` | Run full 10-check validation suite |
| `git-manager ssh add-to-agent --key <uuid> [--passphrase ...]` | Load a key into the SSH agent |

### SSH Architecture

```
~/.ssh/config
    └── Include ~/.ssh/gitmanager/config    ← managed by app

~/.ssh/gitmanager/config                    ← written by app
    Host github.com-work
      HostName github.com
      User git
      IdentityFile ~/.ssh/id_ed25519_github_work
      IdentitiesOnly yes

    Host gitlab.com-personal
      HostName gitlab.com
      User git
      IdentityFile ~/.ssh/id_ed25519_gitlab_personal
      IdentitiesOnly yes
```

**Key isolation**: Each account has a unique Host alias (`hostname-alias`). Git's
ssh reads `~/.ssh/config` → finds `Include` → reads managed config → matches
Host alias → uses the specific `IdentityFile` with `IdentitiesOnly yes`.
No other keys from the agent are tried.

### SSH Validation (`git-manager ssh validate`)

Runs 10 independent checks:
1. Key record exists in database
2. Private key file exists on disk
3. Private key permissions are 0600
4. Public key permissions are 0644
5. Host config entry exists in database
6. Managed config file (`~/.ssh/gitmanager/config`) exists
7. `Include ~/.ssh/gitmanager/config` directive present in `~/.ssh/config`
8. SSH agent is running (`SSH_AUTH_SOCK` set)
9. Key is loaded in the SSH agent (fingerprint match)
10. SSH connection test against configured host alias

---

## 3. Clone / Repositories

### Commands

| Command | Description |
|---|---|
| `git-manager clone <full_name> --account <alias> [--dest <path>] [--depth <n>]` | Clone a repository with SSH identity isolation |
| `git-manager repos list [--account <uuid>]` | List managed repositories |
| `git-manager repos remote-list --account <alias> [--page 1] [--per-page 15]` | List repositories from the platform API |
| `git-manager repos detect [--path <dir>]` | Detect the current directory's tracked account/repo (auto-detects from CWD) |

### Clone URL Resolution

When you run `git-manager clone owner/repo --account work`:

1. `CloneUrlResolver::build_url()` converts the bare name to a full URL:
   - SSH: `owner/repo` → `git@github.com-work:owner/repo.git`
   - HTTPS: `owner/repo` → `https://github.com/owner/repo.git`
2. Git is invoked with `GIT_SSH_COMMAND="ssh -i <key_path> -o IdentitiesOnly=yes"`
3. The repository is tracked in the database after successful clone

### `repos detect` (Directory-Aware)

When run inside a git repository, `repos detect`:
1. Reads the remote URL from `git remote get-url origin`
2. Looks up the tracked repository in the database by local path
3. Displays which account owns the repo, the account's platform, and the current branch
4. If the repo is not tracked, suggests `repos setup`

### `repos remote-list` (Platform API)

Lists repositories from the remote platform (GitHub/GitLab/Bitbucket etc.) using
the stored PAT. Paginated (default 15 per page). In interactive mode, prompts
which platform before listing.

---

## 4. Git Operations

### Commands

| Command | Description |
|---|---|
| `git-manager git pull --repository <uuid> --account <uuid> [--rebase] [--dry-run]` | Pull latest changes |
| `git-manager git push --repository <uuid> --account <uuid> -m "message" [--branch <name>] [--force] [--dry-run]` | Stage, commit, and push |
| `git-manager git status [path]` | Show working tree status |
| `git-manager git log [--account <alias>] [-n 20]` | View operation history |
| `git-manager git detect [--path <dir>]` | Detect current directory's tracked repo/account |

### Dry-Run Mode

Both `pull --dry-run` and `push --dry-run` show what would happen without
making changes:

- `pull --dry-run`: Shows incoming commits, files changed, and estimates if
  conflicts would occur (based on merge analysis).
- `push --dry-run`: Shows what commits would be pushed, checks if remote
  has diverged, and warns about potential conflicts.

---

## 5. Configuration

### Commands

| Command | Description |
|---|---|
| `git-manager config list` | List all configuration values |
| `git-manager config get <key>` | Get a specific configuration value |
| `git-manager config set <key> <value>` | Set a configuration value (persisted to DB) |
| `git-manager config reset [-y]` | Reset all configuration to factory defaults |

### Configuration Keys

| Key | Default | Description |
|---|---|---|
| `ssh.default_key_type` | `"ed25519"` | Default SSH key type |
| `ssh.connect_timeout_ms` | `"30000"` | SSH connection test timeout (ms) |
| `ssh.auto_add_to_agent` | `"true"` | Auto-add keys to SSH agent |
| `git.max_concurrent_ops` | `"4"` | Max concurrent git operations |
| `git.default_remote` | `"origin"` | Default remote name |
| `ui.show_banner` | `"true"` | Show CLI banner on startup |
| `ui.log_level` | `"info"` | Logging level |
| `ui.theme_active_slug` | `"zyrix"` | Active color theme |
| `web.bind_addr` | `"127.0.0.1:5000"` | Web interface bind address |
| `core.auto_migrate` | `"true"` | Run DB migrations on startup |

Configurations are stored in the `app_configurations` table in the database
(not in JSON files or environment variables).

---

## 6. Theme System

### Commands

| Command | Description |
|---|---|
| `git-manager theme list` | List all available themes (30 built-in) |
| `git-manager theme current` | Show current theme |
| `git-manager theme set <slug>` | Set active theme (persisted to database) |
| `git-manager theme preview` | Preview current theme's colour palette |

### Theme Slugs (30 Built-in)

| Category | Slugs |
|---|---|
| Dark (8) | `zyrix`, `jet_black`, `graphite`, `charcoal`, `dark_navy`, `deep_purple`, `forest_green`, `coffee_brown` |
| Light (8) | `pure_white`, `soft_gray`, `silver`, `ivory`, `warm_beige`, `cream`, `light_blue`, `light_mint` |
| Colored (14) | `royal_blue`, `electric_blue`, `teal`, `emerald_green`, `leaf_green`, `sunset_orange`, `amber`, `crimson_red`, `burgundy`, `purple_orchid`, `magenta`, `rose_pink`, `soft_yellow` |

### Theme Palette (8 semantic colour slots)

`primary`, `secondary`, `accent`, `success`, `error`, `warning`, `info`, `banner`

Theme preference is stored in the database (`app_configurations` table, key:
`ui.theme_active_slug`), not in JSON files.

---

## 7. Operation Logs

### Commands

| Command | Description |
|---|---|
| `git-manager logs [--account <alias>] [-n <limit>]` | View recent git operation history |

Each clone, pull, and push is recorded in the `git_operations` table with:
operation type, status, commit SHA, duration, error messages.

---

## 8. DAG (Architecture Visualisation)

### Commands

| Command | Description |
|---|---|
| `git-manager dag crates` | Show workspace crate dependency graph |
| `git-manager dag workflow` | Show clone-and-configure step DAG |
| `git-manager dag events` | Show domain event flow diagram |

---

## 9. Interactive Menu

### Main Menu

```
╭─ Main Menu ──────────────────────────────────────────╮
│                                                      │
│  ═══ Repository Operations ═══                      │
│  [1] Clone a repository                              │
│  [2] Check current repository account                │
│  [3] Git services (pull/push/status/dry-run/detect)  │
│  [4] Set up repository for specific account          │
│                                                      │
│  ═══ Account Management ═══                        │
│  [5] Show all accounts                               │
│  [6] Test SSH connections                            │
│  [7] Manage keys and authentication                  │
│                                                      │
│  [8] Help / Documentation                            │
│  [9] Exit                                            │
╰──────────────────────────────────────────────────────╯
```

### Sub-menus

**[3] Git Services:**
- [1] Pull (with dry-run option, conflict prediction)
- [2] Push (with dry-run option)
- [3] Status
- [4] Detect current directory
- [5] Remote repo listing
- [6] Back

**[7] Manage Keys and Authentication:**
- [1] Generate new SSH key
- [2] Setup PAT (Personal Access Token)
- [3] Setup OAuth token
- [4] Store password
- [5] SSH validate
- [6] List keys
- [7] Back

**[8] Help / Documentation:**
- [1] Quick start guide
- [2] List all CLI commands
- [3] SSH architecture explanation
- [4] Configuration reference
- [5] Back

### Ctrl+C Contract
Every sub-screen handles Ctrl+C gracefully:
- Inside a flow: returns `FlowError::Cancelled`, prints "Cancelled", returns to menu
- At main menu: exits cleanly

---

## 10. Setup Wizard

### Interactive (`git-manager setup`)

```
Step 1: Choose platform       → GitHub / GitLab / Bitbucket / Azure DevOps / SourceForge
Step 2: Enter alias           → e.g. "work", "personal"
Step 3: Enter username        → Git username on platform
Step 4: Enter email           → for commit author + SSH key comment
Step 5: Choose auth method    → SSH key (recommended) or HTTPS + PAT
Step 6: SSH path:
          → Generate ed25519 key
          → Optionally add to SSH agent
          → Optionally test connection
          → Shows URL to add public key
Step 6: HTTPS path:
          → Prompt for PAT/token
          → Store in OS keychain
Step 7: "Account fully set up!"
```

### Non-interactive (`git-manager setup --platform ... --alias ...`)

All flags: `--platform`, `--alias`, `--username`, `--email`, `--auth`,
`--key-type`, `--passphrase`, `--add-to-agent`, `--token`, `--timeout-ms`

---

## 11. Provider Plugins (8 platforms)

| Plugin | Platform | Auth Methods | API Access |
|---|---|---|---|
| GitHub | `github.com` | SSH, HTTPS+PAT, OAuth | `api.github.com` (repos, orgs, teams, starred) |
| GitLab | `gitlab.com` | SSH, HTTPS+PAT, OAuth | `gitlab.com/api/v4` (repos, groups, starred) |
| Bitbucket | `bitbucket.org` | SSH, HTTPS+PAT | `api.bitbucket.org` (repos, teams) |
| Azure DevOps | `ssh.dev.azure.com` | SSH, HTTPS+PAT | `dev.azure.com` (repos) |
| SourceForge | `git.code.sf.net` | SSH, HTTPS | (URL-based repository access) |
| Self-Hosted | configurable | SSH, HTTPS+PAT | configurable |
| Cloud Storage | — | URL-based | (no API — direct URL access) |
| Local Path | — | filesystem | (no remote — local directories) |

---

## 12. Web & Desktop Interface

Both `git-manager web` (Axum server) and `git-manager desktop` (Tauri/Svelte)
provide the same feature set as the CLI:

- Account CRUD (add, list, remove, set-default, store PAT)
- SSH key management (generate, list, test, add-to-agent, validate)
- Repository operations (clone, list, detect)
- Git operations (pull, push, status)
- Configuration management (list, get, set)
- Operation logs
- Theme management
- Platform API remote repo listing

---

## 13. Data Storage

### Database (MySQL)

All persistent data is stored in a MySQL database with these tables:

| Table | Purpose |
|---|---|
| `accounts` | Git hosting accounts |
| `credentials` | Encrypted PATs and tokens |
| `ssh_keys` | SSH key metadata |
| `ssh_host_configs` | SSH Host block configurations |
| `repositories` | Tracked repositories |
| `git_operations` | Operation audit log |
| `sync_sessions` | Push/pull session tracking |
| `clone_operations` | Clone operation observability |
| `app_configurations` | Key-value settings |
| `platforms` | Platform definitions (seed data) |
| `events` | Domain event store |
| `audit_logs` | Security audit trail |

### File System

- `~/.ssh/config` — contains only `Include ~/.ssh/gitmanager/config`
- `~/.ssh/gitmanager/config` — account-specific Host blocks
- `~/.ssh/id_ed25519_<platform>_<alias>` — private keys (0600 permissions)
- `~/.ssh/id_ed25519_<platform>_<alias>.pub` — public keys (0644 permissions)

### OS Keychain

Credentials (PATs, tokens, passwords) are stored in the OS keychain:
- Linux: libsecret (GNOME Keyring / KDE Wallet)
- macOS: Keychain
- Windows: Credential Manager
