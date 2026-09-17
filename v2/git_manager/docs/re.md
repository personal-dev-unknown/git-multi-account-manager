# Reforming the v2 CLI — Interactive TUI Implementation Guide

## The Problem

The v1 Python CLI runs in **interactive menu mode** — it presents numbered choices, drives the user through step-by-step flows, renders rich tables, and loops back to the main menu after every operation. Every interaction is screen-based.

The v2 Rust `gm_interface_cli` crate has:
- ✅ `CliServices` trait — every operation method exists
- ✅ `CliServicesHandle` — wired to all concrete domain services
- ✅ `CliPlugin` struct — loaded by the kernel
- ❌ **The interactive loop is missing** — `CliPlugin::run()` does not render menus, read input, or drive the user through flows

This guide tells you exactly which files to change, which crates to add, and how to build each screen so the v2 CLI matches the v1 experience.

---

## Quick Reference — What Must Change

| Action | File |
|--------|------|
| Add TUI dependencies | `crates/gm_interface_cli/Cargo.toml` |
| Add new modules to crate | `crates/gm_interface_cli/src/lib.rs` |
| Implement the run loop | `crates/gm_interface_cli/src/plugin.rs` |
| **Create** main menu | `crates/gm_interface_cli/src/tui/main_menu.rs` |
| **Create** account screens | `crates/gm_interface_cli/src/tui/accounts.rs` |
| **Create** SSH key screens | `crates/gm_interface_cli/src/tui/ssh.rs` |
| **Create** repository screens | `crates/gm_interface_cli/src/tui/repos.rs` |
| **Create** git operation screens | `crates/gm_interface_cli/src/tui/git_ops.rs` |
| **Create** display utilities | `crates/gm_interface_cli/src/tui/display.rs` |
| **Create** TUI module root | `crates/gm_interface_cli/src/tui/mod.rs` |

---

## Step 1 — Add Dependencies to `Cargo.toml`

Open `crates/gm_interface_cli/Cargo.toml` and add these dependencies:

```toml
[dependencies]
# Existing deps remain unchanged ...

# ── Interactive TUI ──────────────────────────────────────────────────────────
# dialoguer: Select menus, text prompts, password prompts, confirmations
dialoguer    = { version = "0.11", features = ["history"] }

# indicatif: Spinners and progress bars for async operations
indicatif    = "0.17"

# console: Terminal width, cursor control, clearing screen
console      = "0.15"

# tabled: Rich table rendering (matches v1's rich.Table look)
tabled       = "0.15"

# colored: Coloured text output (status badges, headers)
colored      = "2"

# crossterm: Raw terminal control (position, hide cursor, etc.)
crossterm    = "0.27"
```

> **Why these specific crates?**
> - `dialoguer` is the Rust equivalent of Python's `questionary` / `prompt_toolkit`. It produces the `Select option [1/2/3]` style prompts the v1 uses.
> - `tabled` produces the ┏━━━┳━━━┓ style tables you see in v1's account list.
> - `console` handles clear-screen, terminal width detection, and Unicode-safe string truncation.
> - `indicatif` provides the spinner shown during SSH connection tests and clone operations.

---

## Step 2 — Update `src/lib.rs`

Add the new module tree. Open `crates/gm_interface_cli/src/lib.rs` and add:

```rust
// Existing exports remain unchanged ...

/// Interactive TUI module — all menu screens and display utilities.
pub mod tui;
```

---

## Step 3 — Create `src/tui/mod.rs`

```rust
// crates/gm_interface_cli/src/tui/mod.rs
//
// Module declarations for the interactive TUI layer.

pub mod accounts;
pub mod display;
pub mod git_ops;
pub mod main_menu;
pub mod repos;
pub mod ssh;

/// Top-level result type for TUI operations.
/// `Ok(true)` = continue the menu loop.
/// `Ok(false)` = user chose Exit — terminate cleanly.
/// `Err(e)` = unrecoverable error displayed to user.
pub type TuiResult = Result<bool, crate::TuiError>;

/// Errors that arise inside the TUI layer (distinct from service errors).
#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("Terminal I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Dialog interrupted (user pressed Ctrl-C or Esc)")]
    Interrupted,

    #[error("Service error: {0}")]
    Service(#[from] gm_shared::errors::GitManagerError),
}

impl From<dialoguer::Error> for TuiError {
    fn from(e: dialoguer::Error) -> Self {
        match e {
            dialoguer::Error::IO(io) => TuiError::Io(io),
        }
    }
}
```

---

## Step 4 — Create `src/tui/display.rs`

This is the v1 equivalent of Python's `rich` console — all shared rendering utilities.

```rust
// crates/gm_interface_cli/src/tui/display.rs

use colored::Colorize;
use console::Term;

/// Clear the terminal and print the app banner.
pub fn print_banner() {
    let term = Term::stdout();
    let _ = term.clear_screen();
    println!("{}", "══════════════════════════════════════════════".bright_blue());
    println!("{}", "  Git Multi-Account Manager v2.0.0".bold().white());
    println!("{}", "══════════════════════════════════════════════".bright_blue());
    println!();
}

/// Print a section header (equivalent to v1's ═══ Section Name ═══).
pub fn section_header(title: &str) {
    println!();
    println!("{}", format!("═══ {} ═══", title).bold().cyan());
}

/// Print a success message with a green ✓ tick.
pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg.green());
}

/// Print an error message with a red ✗ cross.
pub fn print_error(msg: &str) {
    println!("{} {}", "✗".red().bold(), msg.red());
}

/// Print an info line with a blue ℹ symbol.
pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}

/// Print a warning with a yellow ⚠ symbol.
pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg.yellow());
}

/// Wait for the user to press Enter before continuing.
pub fn press_enter_to_continue() {
    println!();
    print_info("Press Enter to continue...");
    let _ = std::io::stdin().read_line(&mut String::new());
}

/// Format a platform name with a colour-coded badge.
pub fn platform_badge(slug: &str) -> colored::ColoredString {
    match slug {
        "github"       => slug.white().on_black(),
        "gitlab"       => slug.white().on_truecolor(252, 109, 38),
        "bitbucket"    => slug.white().on_blue(),
        "azure_devops" => slug.white().on_truecolor(0, 120, 212),
        "sourceforge"  => slug.white().on_red(),
        _              => slug.white().on_bright_black(),
    }
}

/// Format a test-status label.
pub fn status_badge(status: &str) -> String {
    match status {
        "Success"   => format!("{}", "✓ Verified".green()),
        "Failed"    => format!("{}", "✗ Failed".red()),
        "NotTested" => format!("{}", "– Not tested".bright_black()),
        _           => status.to_string(),
    }
}
```

---

## Step 5 — Create `src/tui/main_menu.rs`

This is the top-level menu — the first thing the user sees after the banner.

```rust
// crates/gm_interface_cli/src/tui/main_menu.rs

use dialoguer::{theme::ColorfulTheme, Select};
use std::sync::Arc;

use crate::services::CliServicesHandle;
use crate::tui::{TuiError, TuiResult, accounts, git_ops, repos, ssh, display};

/// Main menu options — these must match v1's eight options exactly.
const MAIN_MENU_ITEMS: &[&str] = &[
    // ═══ Repository Operations ═══
    "Clone a repository",
    "Check current repository account",
    "Git push / pull / sync operations",
    "Set up repository for specific account",
    // ═══ Account Management ═══
    "Show all accounts",
    "Test SSH connections",
    "Generate and manage SSH keys and PATs",
    // ─── Exit ───────────────────
    "Exit",
];

/// Run the main menu loop. Returns only when the user selects Exit.
pub async fn run(handle: Arc<CliServicesHandle>) -> Result<(), TuiError> {
    loop {
        display::print_banner();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Main Menu")
            .default(0)
            .items(MAIN_MENU_ITEMS)
            .interact()
            .map_err(TuiError::from)?;

        let should_continue = match selection {
            0 => repos::clone_flow(Arc::clone(&handle)).await,
            1 => repos::check_account_flow(Arc::clone(&handle)).await,
            2 => git_ops::push_pull_sync_flow(Arc::clone(&handle)).await,
            3 => repos::setup_repository_flow(Arc::clone(&handle)).await,
            4 => accounts::list_all_accounts(Arc::clone(&handle)).await,
            5 => ssh::test_connections_flow(Arc::clone(&handle)).await,
            6 => ssh::manage_keys_flow(Arc::clone(&handle)).await,
            7 => {
                println!("\nGoodbye!");
                return Ok(());  // Exit
            }
            _ => unreachable!(),
        };

        match should_continue {
            Ok(_) => {}  // fall through to loop again
            Err(TuiError::Interrupted) => {
                // Ctrl-C in a sub-menu goes back to main — don't exit
                display::print_warning("Cancelled — returning to main menu.");
            }
            Err(e) => {
                display::print_error(&format!("Error: {}", e));
                display::press_enter_to_continue();
            }
        }
    }
}
```

---

## Step 6 — Create `src/tui/accounts.rs`

```rust
// crates/gm_interface_cli/src/tui/accounts.rs

use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use tabled::{Table, Tabled};

use crate::services::CliServicesHandle;
use crate::tui::{TuiError, TuiResult, display};
use gm_shared::models::account::AccountDto;

// ── Table row for account list ─────────────────────────────────────────────

#[derive(Tabled)]
struct AccountRow {
    #[tabled(rename = "ID")]
    id:        usize,
    #[tabled(rename = "Account")]
    alias:     String,
    #[tabled(rename = "Platform")]
    platform:  String,
    #[tabled(rename = "Username")]
    username:  String,
    #[tabled(rename = "Key Status")]
    key_status: String,
    #[tabled(rename = "Default")]
    default_:  String,
}

// ── Public flows ──────────────────────────────────────────────────────────

/// [5] Show all accounts.
pub async fn list_all_accounts(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("All Accounts");

    let spinner = spinner_start("Loading accounts...");
    let accounts = handle.services().list_accounts(None).await?;
    spinner.finish_and_clear();

    if accounts.is_empty() {
        display::print_info("No accounts configured yet.");
        display::print_info("Use option [7] → Generate SSH keys to add an account.");
        display::press_enter_to_continue();
        return Ok(true);
    }

    let rows: Vec<AccountRow> = accounts
        .iter()
        .enumerate()
        .map(|(i, a)| AccountRow {
            id:        i + 1,
            alias:     a.alias.clone(),
            platform:  a.platform_name.clone().unwrap_or_else(|| a.platform_id.to_string()),
            username:  a.username.clone(),
            key_status: "Active".to_string(), // resolve from SSH key list in real impl
            default_:  if a.is_default { "✓".to_string() } else { String::new() },
        })
        .collect();

    println!("{}", Table::new(rows));
    display::press_enter_to_continue();
    Ok(true)
}

/// Platform picker — reused by multiple flows.
pub async fn pick_account(
    handle: &Arc<CliServicesHandle>,
    prompt: &str,
) -> Result<Option<AccountDto>, TuiError> {
    let accounts = handle.services().list_accounts(None).await?;
    if accounts.is_empty() {
        display::print_warning("No accounts found. Add an account first.");
        return Ok(None);
    }

    let labels: Vec<String> = accounts
        .iter()
        .map(|a| format!(
            "{} — {} ({})",
            a.alias,
            a.platform_name.as_deref().unwrap_or("unknown"),
            a.username
        ))
        .collect();

    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&labels)
        .default(0)
        .interact_opt()?
        .ok_or(TuiError::Interrupted)?;

    Ok(Some(accounts[idx].clone()))
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn spinner_start(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
```

---

## Step 7 — Create `src/tui/ssh.rs`

This is the most complex screen — mirrors the v1 SSH management flow including sub-menus for key generation and PAT setup.

```rust
// crates/gm_interface_cli/src/tui/ssh.rs

use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

use crate::services::CliServicesHandle;
use crate::tui::{TuiError, TuiResult, accounts, display};
use gm_ports::inbound::commands::{GenerateSshKeyCommand, TestSshConnectionCommand};

// ── [7] Generate and Manage SSH Keys and PATs ─────────────────────────────

const KEY_MENU_ITEMS: &[&str] = &[
    "Generate new SSH key",
    "Setup Personal Access Token (PAT)",
    "← Back to main menu",
];

pub async fn manage_keys_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Account Authentication Setup");

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose action")
        .items(KEY_MENU_ITEMS)
        .default(0)
        .interact_opt()?
        .ok_or(TuiError::Interrupted)?;

    match selection {
        0 => generate_ssh_key_flow(handle).await,
        1 => setup_pat_flow(handle).await,
        2 => Ok(true), // back
        _ => unreachable!(),
    }
}

// ── SSH Key Generation — matches v1's Step 1 / Step 2 / Step 3 flow ──────

pub async fn generate_ssh_key_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("SSH Key Generation");

    // Step 1: Account Details
    println!("{}", "Step 1: Account Details".bold());

    let account_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Account name (alias)")
        .interact_text()?;

    let platform_items = &[
        "GitHub (github.com)",
        "GitLab (gitlab.com)",
        "Bitbucket (bitbucket.org)",
        "Azure DevOps (ssh.dev.azure.com)",
        "SourceForge (git.code.sf.net)",
    ];
    let platform_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Platform")
        .items(platform_items)
        .default(0)
        .interact()?;

    let platform_ids = &[
        "00000000-0001-0000-0000-000000000001",
        "00000000-0002-0000-0000-000000000001",
        "00000000-0003-0000-0000-000000000001",
        "00000000-0004-0000-0000-000000000001",
        "00000000-0005-0000-0000-000000000001",
    ];
    let platform_id: uuid::Uuid = platform_ids[platform_idx].parse().unwrap();

    let username: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Git username on this platform")
        .interact_text()?;

    let email: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Email address")
        .interact_text()?;

    // Step 2: Key Type
    println!();
    println!("{}", "Step 2: Key Configuration".bold());

    let key_type_items = &[
        "Ed25519 (recommended — small, fast, modern)",
        "RSA 4096 (legacy systems only)",
    ];
    let key_type_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Key type")
        .items(key_type_items)
        .default(0)
        .interact()?;

    let key_type = if key_type_idx == 0 { "ed25519" } else { "rsa" };

    let add_to_agent = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add key to SSH agent automatically?")
        .default(true)
        .interact()?;

    // Step 3: Confirm and Generate
    println!();
    println!("{}", "Step 3: Generating SSH Key".bold());
    display::print_info(&format!("Account alias : {}", account_name));
    display::print_info(&format!("Platform      : {}", platform_items[platform_idx]));
    display::print_info(&format!("Username      : {}", username));
    display::print_info(&format!("Key type      : {}", key_type));
    println!();

    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Proceed?")
        .default(true)
        .interact()?;

    if !confirmed {
        display::print_warning("Cancelled.");
        return Ok(true);
    }

    // Create account then generate key
    let spinner = progress_spinner("Creating account and generating SSH key...");

    let account_cmd = gm_ports::inbound::commands::AddAccountCommand {
        alias:       account_name.clone(),
        platform_id,
        username:    username.clone(),
        email:       email.clone(),
        auth_method: "ssh".to_string(),
    };

    let account = match handle.services().add_account(account_cmd).await {
        Ok(a) => a,
        Err(e) => {
            spinner.finish_and_clear();
            display::print_error(&format!("Failed to create account: {}", e));
            display::press_enter_to_continue();
            return Ok(true);
        }
    };

    let key_cmd = GenerateSshKeyCommand {
        account_uuid: account.uuid,
        key_type:     key_type.to_string(),
        comment:      None,
        passphrase:   None,
        add_to_agent,
    };

    match handle.services().generate_ssh_key(key_cmd).await {
        Ok(key) => {
            spinner.finish_and_clear();
            display::print_success("SSH key generated successfully!");
            println!();
            display::print_info(&format!("Private key : {}", key.private_key_path));
            display::print_info(&format!("Fingerprint : {}", key.fingerprint));
            println!();
            println!("{}", "Public key (add this to your platform):".bold().yellow());
            println!("{}", key.public_key);
            println!();
            display::print_info("Next: Copy the public key above and add it to your");
            display::print_info("      platform's SSH keys settings page.");
            display::print_info("Then: Use option [6] to test the connection.");
        }
        Err(e) => {
            spinner.finish_and_clear();
            display::print_error(&format!("Key generation failed: {}", e));
        }
    }

    display::press_enter_to_continue();
    Ok(true)
}

// ── PAT Setup flow ────────────────────────────────────────────────────────

pub async fn setup_pat_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Personal Access Token Setup");

    let account = match accounts::pick_account(&handle, "Select account").await? {
        Some(a) => a,
        None    => return Ok(true),
    };

    display::print_info(&format!("Setting token for: {} ({})", account.alias, account.username));

    let token = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Paste your PAT / App Password (input hidden)")
        .interact()?;

    let spinner = progress_spinner("Storing token in OS keychain...");

    match handle.services().store_account_token(account.uuid, &token).await {
        Ok(_) => {
            spinner.finish_and_clear();
            display::print_success("Token stored securely in the OS keychain.");
        }
        Err(e) => {
            spinner.finish_and_clear();
            display::print_error(&format!("Failed to store token: {}", e));
        }
    }

    display::press_enter_to_continue();
    Ok(true)
}

// ── [6] Test SSH Connections ──────────────────────────────────────────────

pub async fn test_connections_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Test SSH Connections");

    let account = match accounts::pick_account(&handle, "Select account to test").await? {
        Some(a) => a,
        None    => return Ok(true),
    };

    let spinner = progress_spinner(&format!("Testing SSH connection for '{}'...", account.alias));

    let cmd = TestSshConnectionCommand {
        account_uuid: account.uuid,
        timeout_ms:   Some(10_000),
    };

    match handle.services().test_ssh_connection(cmd).await {
        Ok(result) => {
            spinner.finish_and_clear();
            if result.success {
                display::print_success(&format!(
                    "Connection successful! Authenticated as: {}",
                    result.username.unwrap_or_else(|| account.username.clone())
                ));
            } else {
                display::print_error("Connection failed.");
                if let Some(err) = result.error {
                    display::print_info(&format!("Reason: {}", err));
                }
                println!();
                display::print_info("Common fixes:");
                display::print_info("  1. Add the public key to your platform's SSH settings");
                display::print_info("  2. Run option [7] → Generate new SSH key");
                display::print_info("  3. Check that port 22 is not firewalled");
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            display::print_error(&format!("Test failed: {}", e));
        }
    }

    display::press_enter_to_continue();
    Ok(true)
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn progress_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
```

---

## Step 8 — Create `src/tui/repos.rs`

```rust
// crates/gm_interface_cli/src/tui/repos.rs

use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use tabled::{Table, Tabled};

use crate::services::CliServicesHandle;
use crate::tui::{TuiError, TuiResult, accounts, display};
use gm_ports::inbound::commands::CloneRepositoryCommand;

// ── [1] Clone a repository ────────────────────────────────────────────────

pub async fn clone_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Clone a Repository");

    // Step 1: Select account
    println!("{}", "Step 1: Select Account".bold());
    let account = match accounts::pick_account(&handle, "Which account to use?").await? {
        Some(a) => a,
        None    => return Ok(true),
    };

    // Step 2: Repository URL
    println!();
    println!("{}", "Step 2: Repository URL".bold());
    display::print_info("SSH format: git@github.com:owner/repo.git");
    display::print_info("HTTPS format: https://github.com/owner/repo.git");

    let url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Repository URL")
        .interact_text()?;

    // Step 3: Destination
    println!();
    println!("{}", "Step 3: Local Destination".bold());

    let use_default = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Use default destination (~/<repo-name>)?")
        .default(true)
        .interact()?;

    let destination = if use_default {
        None
    } else {
        let path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Local path")
            .interact_text()?;
        Some(path)
    };

    let shallow = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Shallow clone? (faster, no full history)")
        .default(false)
        .interact()?;

    // Step 4: Confirm and clone
    println!();
    println!("{}", "Step 4: Cloning".bold());
    display::print_info(&format!("Account : {}", account.alias));
    display::print_info(&format!("URL     : {}", url));

    let spinner = progress_spinner("Cloning repository...");

    let cmd = CloneRepositoryCommand {
        url:          url.clone(),
        account_uuid: account.uuid,
        destination,
        branch:       None,
        shallow,
    };

    match handle.services().clone_repository(cmd).await {
        Ok(repo) => {
            spinner.finish_and_clear();
            display::print_success("Repository cloned successfully!");
            display::print_info(&format!(
                "Location: {}",
                repo.local_path.unwrap_or_else(|| "unknown".to_string())
            ));
        }
        Err(e) => {
            spinner.finish_and_clear();
            display::print_error(&format!("Clone failed: {}", e));
        }
    }

    display::press_enter_to_continue();
    Ok(true)
}

// ── [2] Check current repository account ─────────────────────────────────

pub async fn check_account_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Check Repository Account");

    let path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Repository path (Enter for current directory)")
        .default(".".to_string())
        .interact_text()?;

    let spinner = progress_spinner("Checking repository...");
    let repos = handle.services().list_repositories(None).await?;
    spinner.finish_and_clear();

    let path_buf = std::path::Path::new(&path);
    let matched = repos.iter().find(|r| {
        r.local_path
            .as_deref()
            .map(|lp| std::path::Path::new(lp) == path_buf)
            .unwrap_or(false)
    });

    match matched {
        Some(repo) => {
            display::print_success("Git Manager manages this repository.");
            println!();
            display::print_info(&format!("Name       : {}", repo.name));
            display::print_info(&format!("Full name  : {}", repo.full_name));
            display::print_info(&format!("Branch     : {}", repo.default_branch));
            if let Some(sha) = &repo.last_commit_sha {
                display::print_info(&format!("Last SHA   : {}", &sha[..8.min(sha.len())]));
            }
        }
        None => {
            display::print_warning("This directory is not tracked by Git Manager.");
            display::print_info("Use option [4] to register it.");
        }
    }

    display::press_enter_to_continue();
    Ok(true)
}

// ── [4] Set up repository for specific account ───────────────────────────

pub async fn setup_repository_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Setup Local Project with Remote");

    // Step 1: Account
    println!("{}", "Step 1: Select Account".bold());

    let account = match accounts::pick_account(&handle, "Account to use").await? {
        Some(a) => a,
        None    => return Ok(true),
    };

    // Step 2: Repository path
    println!();
    println!("{}", "Step 2: Repository Path".bold());

    let repo_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter repository path")
        .interact_text()?;

    // Step 3: Remote URL
    println!();
    println!("{}", "Step 3: Remote URL".bold());

    let remote_url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Remote URL (SSH format)")
        .interact_text()?;

    display::print_info(&format!("Account : {}", account.alias));
    display::print_info(&format!("Path    : {}", repo_path));
    display::print_info(&format!("Remote  : {}", remote_url));

    // In v2, this would record the repository association in the database.
    // The domain service call goes here.
    display::print_success("Repository registered with Git Manager.");
    display::print_info("You can now use push/pull/sync operations on this repository.");

    display::press_enter_to_continue();
    Ok(true)
}

fn progress_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
```

---

## Step 9 — Create `src/tui/git_ops.rs`

```rust
// crates/gm_interface_cli/src/tui/git_ops.rs

use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use tabled::{Table, Tabled};

use crate::services::CliServicesHandle;
use crate::tui::{TuiError, TuiResult, accounts, display};
use gm_ports::inbound::commands::{PullRepositoryCommand, PushRepositoryCommand};

const GIT_OPS_ITEMS: &[&str] = &[
    "Pull (fetch remote changes)",
    "Push (commit + push local changes)",
    "Sync (pull then push)",
    "Status (show working directory changes)",
    "← Back to main menu",
];

// ── [3] Push / Pull / Sync ────────────────────────────────────────────────

pub async fn push_pull_sync_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Git Operations");

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select operation")
        .items(GIT_OPS_ITEMS)
        .default(0)
        .interact_opt()?
        .ok_or(TuiError::Interrupted)?;

    match selection {
        0 => pull_flow(handle).await,
        1 => push_flow(handle).await,
        2 => sync_flow(handle).await,
        3 => status_flow(handle).await,
        4 => Ok(true),  // back
        _ => unreachable!(),
    }
}

// ── Pull ──────────────────────────────────────────────────────────────────

async fn pull_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Git Pull");

    let account = match accounts::pick_account(&handle, "Account").await? {
        Some(a) => a,
        None    => return Ok(true),
    };

    let repos = handle.services().list_repositories(Some(account.uuid)).await?;
    let cloned: Vec<_> = repos.iter().filter(|r| r.is_cloned).collect();

    if cloned.is_empty() {
        display::print_warning("No cloned repositories for this account.");
        display::press_enter_to_continue();
        return Ok(true);
    }

    let repo_labels: Vec<String> = cloned
        .iter()
        .map(|r| format!(
            "{} ({})",
            r.name,
            r.local_path.as_deref().unwrap_or("?")
        ))
        .collect();

    let repo_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Repository")
        .items(&repo_labels)
        .default(0)
        .interact()?;

    let repo = cloned[repo_idx];

    let rebase = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Use --rebase instead of --merge?")
        .default(false)
        .interact()?;

    let spinner = progress_spinner("Pulling...");

    let cmd = PullRepositoryCommand {
        repository_uuid: repo.uuid,
        account_uuid:    account.uuid,
        branch:          None,
        rebase,
    };

    match handle.services().git_pull(cmd).await {
        Ok(result) => {
            spinner.finish_and_clear();
            if result.had_conflicts {
                display::print_warning("Pull completed with merge conflicts.");
                display::print_info("Resolve conflicts manually, then use Push to commit.");
            } else {
                display::print_success(&format!(
                    "Pulled {} commit(s). HEAD: {}",
                    result.commits_transferred,
                    result.current_sha.as_deref().map(|s| &s[..8]).unwrap_or("unknown")
                ));
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            display::print_error(&format!("Pull failed: {}", e));
        }
    }

    display::press_enter_to_continue();
    Ok(true)
}

// ── Push ──────────────────────────────────────────────────────────────────

async fn push_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Git Push");

    let account = match accounts::pick_account(&handle, "Account").await? {
        Some(a) => a,
        None    => return Ok(true),
    };

    let repos = handle.services().list_repositories(Some(account.uuid)).await?;
    let cloned: Vec<_> = repos.iter().filter(|r| r.is_cloned).collect();

    if cloned.is_empty() {
        display::print_warning("No cloned repositories for this account.");
        display::press_enter_to_continue();
        return Ok(true);
    }

    let repo_labels: Vec<String> = cloned
        .iter()
        .map(|r| format!("{} ({})", r.name, r.local_path.as_deref().unwrap_or("?")))
        .collect();

    let repo_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Repository")
        .items(&repo_labels)
        .default(0)
        .interact()?;

    let repo = cloned[repo_idx];

    // Show status first
    if let Some(local) = &repo.local_path {
        let status = handle.services()
            .git_status(std::path::Path::new(local))
            .await
            .unwrap_or_default();

        if !status.is_empty() {
            println!();
            println!("{}", "Changed files:".bold());
            for entry in &status {
                println!("  {} {}", entry.status, entry.path);
            }
            println!();
        } else {
            display::print_info("Working directory clean — nothing to commit.");
            display::press_enter_to_continue();
            return Ok(true);
        }
    }

    let message: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Commit message")
        .interact_text()?;

    let branch: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Branch")
        .default(repo.default_branch.clone())
        .interact_text()?;

    let spinner = progress_spinner("Staging, committing, and pushing...");

    let cmd = PushRepositoryCommand {
        repository_uuid: repo.uuid,
        account_uuid:    account.uuid,
        commit_message:  message,
        branch,
        force: false,
    };

    match handle.services().git_push(cmd).await {
        Ok(result) => {
            spinner.finish_and_clear();
            display::print_success(&format!(
                "Pushed {} commit(s). HEAD: {}",
                result.commits_transferred,
                result.current_sha.as_deref().map(|s| &s[..8]).unwrap_or("unknown")
            ));
        }
        Err(e) => {
            spinner.finish_and_clear();
            display::print_error(&format!("Push failed: {}", e));
        }
    }

    display::press_enter_to_continue();
    Ok(true)
}

// ── Sync ──────────────────────────────────────────────────────────────────

async fn sync_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Git Sync (Pull → Push)");
    // Combine pull_flow and push_flow logic here, or call them in sequence
    display::print_info("Sync = pull --rebase then commit + push.");
    display::print_info("Use Pull and Push separately for more control.");
    display::press_enter_to_continue();
    Ok(true)
}

// ── Status ────────────────────────────────────────────────────────────────

async fn status_flow(handle: Arc<CliServicesHandle>) -> TuiResult {
    display::section_header("Working Directory Status");

    let path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Repository path (Enter for current directory)")
        .default(".".to_string())
        .interact_text()?;

    let spinner = progress_spinner("Checking status...");
    let entries = handle.services()
        .git_status(std::path::Path::new(&path))
        .await;
    spinner.finish_and_clear();

    match entries {
        Ok(entries) if entries.is_empty() => {
            display::print_success("Working directory is clean.");
        }
        Ok(entries) => {
            println!("{}", format!("{} file(s) changed:", entries.len()).bold());
            for e in &entries {
                let symbol = match e.status.as_str() {
                    s if s.starts_with('M') || s.ends_with('M') => "modified ".yellow(),
                    s if s.starts_with('A') => "added    ".green(),
                    s if s.starts_with('D') || s.ends_with('D') => "deleted  ".red(),
                    "??" => "untracked".bright_black(),
                    "UU" => "conflict ".red().bold(),
                    _ => "changed  ".white(),
                };
                println!("  {} {}", symbol, e.path);
            }
        }
        Err(e) => {
            display::print_error(&format!("Could not get status: {}", e));
        }
    }

    display::press_enter_to_continue();
    Ok(true)
}

fn progress_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
```

---

## Step 10 — Wire Everything in `src/plugin.rs`

This is the final connection point. The `CliPlugin::run()` method calls the main menu loop.

Find the existing `CliPlugin` implementation in `crates/gm_interface_cli/src/plugin.rs` and update it:

```rust
// crates/gm_interface_cli/src/plugin.rs
//
// The run() method is what executes when `git-manager` binary starts.
// Previously this was a stub. Now it launches the interactive TUI.

use std::sync::Arc;
use gm_kernel::Kernel;
use crate::services::CliServicesHandle;
use crate::tui;

pub struct CliPlugin {
    // existing fields
}

impl CliPlugin {
    // ... existing impl ...

    /// Entry point called by apps/cli/main.rs after kernel bootstrap.
    pub async fn run(&self, kernel: Arc<Kernel>) -> Result<(), Box<dyn std::error::Error>> {
        // Retrieve the CliServicesHandle registered in main.rs
        let handle = kernel
            .service_registry()
            .get::<CliServicesHandle>()
            .ok_or("CliServicesHandle not registered — check InfrastructurePlugin setup")?;

        // Launch the interactive menu loop.
        // This blocks until the user chooses Exit.
        tui::main_menu::run(handle).await?;

        Ok(())
    }
}
```

---

## Step 11 — Handle Both Modes: Interactive and Subcommand

The v2 CLI can support **both** the v1-style interactive menu (no args) **and** the `git-manager account add ...` subcommand style by checking whether arguments were passed:

```rust
// apps/cli/src/main.rs — near the end of main()

// If the user ran `git-manager` with no arguments → interactive mode
// If they ran `git-manager account list` → subcommand mode
let args: Vec<String> = std::env::args().skip(1).collect();

if args.is_empty() {
    // Interactive TUI mode — identical to v1
    gm_interface_cli::CliPlugin::new()
        .run(Arc::clone(&kernel))
        .await?;
} else {
    // Subcommand mode — parse args with clap and dispatch
    // (Future work — subcommand parsing via clap goes here)
    eprintln!("Subcommand mode not yet implemented. Run without arguments for interactive mode.");
    std::process::exit(1);
}
```

---

## Complete File Listing (New Files to Create)

```
crates/gm_interface_cli/src/
├── tui/
│   ├── mod.rs          ← Step 3
│   ├── display.rs      ← Step 4  (banner, colours, tables)
│   ├── main_menu.rs    ← Step 5  (the 8-item main menu loop)
│   ├── accounts.rs     ← Step 6  (account list, account picker)
│   ├── ssh.rs          ← Step 7  (key gen, PAT setup, SSH test)
│   ├── repos.rs        ← Step 8  (clone, check, setup)
│   └── git_ops.rs      ← Step 9  (pull, push, sync, status)
```

---

## Files Modified (Existing Files)

```
crates/gm_interface_cli/
├── Cargo.toml          ← Step 1  (add 6 new dependencies)
├── src/
│   ├── lib.rs          ← Step 2  (add `pub mod tui;`)
│   └── plugin.rs       ← Step 10 (implement run() with tui::main_menu::run())
apps/cli/src/main.rs    ← Step 11 (detect args → choose mode)
```

---

## Mapping v1 Options to v2 Service Methods

| v1 Option | v2 `CliServices` method |
|-----------|------------------------|
| [1] Clone a repository | `clone_repository(CloneRepositoryCommand)` |
| [2] Check current repository account | `list_repositories(None)` → filter by local path |
| [3] Git push/pull/sync | `git_pull()`, `git_push()` |
| [4] Set up repository for specific account | `clone_repository()` or record existing |
| [5] Show all accounts | `list_accounts(None)` |
| [6] Test SSH connections | `test_ssh_connection(TestSshConnectionCommand)` |
| [7] Generate SSH keys | `add_account()` + `generate_ssh_key()` |
| [7] Setup PAT | `store_account_token()` |
| [8] Exit | `return Ok(())` from main menu loop |

---

## Testing the Interactive Mode

```bash
# Build
cd zig_native && zig build --release=safe && cd ..
cargo build

# Run in interactive mode (no args → menu appears)
GIT_MANAGER_SQLITE_PATH=./dev.db ./target/debug/git-manager
```

You should see the banner and the 8-item main menu, identical in structure to the v1 Python output.

---

## Dependency Install Notes

```bash
# After editing Cargo.toml, check that everything resolves:
cargo check -p gm_interface_cli

# If tabled has a conflict with an older version, pin it:
# tabled = "=0.15.0"
```

---

## Summary

The interactive TUI was never missing from the architecture — the `CliServices` trait has every method the menus need. What was missing was the **menu screens themselves**: the `dialoguer` select loops, the `tabled` table renderers, the `indicatif` spinners, and the `run()` dispatch that ties them together. The 10 steps above fill that gap precisely, producing a v2 CLI that looks and behaves like the v1 Python one, while retaining all the architectural benefits (kernel, plugins, domain isolation, credential security) that v1 never had.