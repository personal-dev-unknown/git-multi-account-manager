// crates/gm_interface_cli/src/ui/interactive/common.rs
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Everything shared by every flow module in ui/interactive/.
//
//  FlowError        — unified error type with a Cancelled variant so that
//                     pressing Esc / Ctrl+C inside any flow returns gracefully
//                     to the parent menu instead of crashing.
//
//  Prompt wrappers  — thin façades over dialoguer that map any IO error to
//                     FlowError::Cancelled.  All flow modules call these
//                     instead of dialoguer directly, keeping the Ctrl+C
//                     contract consistent everywhere.
//
//  Section helpers  — section(), press_enter(), level_header() for visual
//                     consistency across all screens.
//
//  Pickers          — pick_account() and pick_cloned_repo() used by multiple
//                     flow modules; defined once here.
//
//  PLATFORM_OPTIONS — canonical (slug, display) pairs that feed into the
//                     pub(crate) helpers in commands/account.rs.  Single
//                     source of truth: no duplication with that file.


use uuid::Uuid;
use gm_shared::{
    errors::GitManagerError,
    models::{account::AccountDto, repository::RepositoryDto},
};
use gm_ports::inbound::commands::AddAccountCommand;

use crate::commands::account::{parse_platform, platform_uuid_for};
use crate::services::CliServicesHandle;
use crate::ui::{colors, progress};

// ─────────────────────────────────────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────────────────────────────────────

/// All interactive flow functions return this instead of GitManagerError so
/// that Ctrl+C / Esc can be distinguished from genuine service failures.
#[derive(Debug)]
pub(super) enum FlowError {
    /// User cancelled the current operation — show the warning and loop.
    Cancelled,
    /// A domain service call (SSH, git, DB) returned an error.
    Service(GitManagerError),
}

impl From<GitManagerError> for FlowError {
    fn from(e: GitManagerError) -> Self { Self::Service(e) }
}

impl std::fmt::Display for FlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled  => write!(f, "Operation cancelled by user"),
            Self::Service(e) => write!(f, "{e}"),
        }
    }
}

/// Convenience alias used as the return type of every flow function.
pub(super) type FlowResult = Result<(), FlowError>;

// ─────────────────────────────────────────────────────────────────────────────
// Platform constants
// ─────────────────────────────────────────────────────────────────────────────

/// (slug, display label) pairs for the platform picker.
/// Slugs are forwarded directly to `parse_platform` + `platform_uuid_for`
/// from commands/account.rs — UUID mapping is NOT duplicated here.
pub(super) const PLATFORM_OPTIONS: &[(&str, &str)] = &[
    ("github",       "GitHub       (github.com)"),
    ("gitlab",       "GitLab       (gitlab.com)"),
    ("bitbucket",    "Bitbucket    (bitbucket.org)"),
    ("azure_devops", "Azure DevOps (ssh.dev.azure.com)"),
    ("sourceforge",  "SourceForge  (git.code.sf.net)"),
    ("self_hosted",  "Self-Hosted  (custom server)"),
    ("cloud_storage","Cloud Storage (S3/GCS/Azure Blob)"),
    ("local_path",   "Local Path   (filesystem)"),
];

// ─────────────────────────────────────────────────────────────────────────────
// Prompt wrappers — all map IO errors to FlowError::Cancelled
// ─────────────────────────────────────────────────────────────────────────────

/// Arrow-key Select.  Esc / Ctrl+C → FlowError::Cancelled.
pub(super) fn select_menu(label: &str, items: &[&str]) -> Result<usize, FlowError> {
    dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(label)
        .items(items)
        .default(0)
        .interact_opt()
        .map_err(|_| FlowError::Cancelled)?
        .ok_or(FlowError::Cancelled)
}

/// Text input.  Empty input returns the default if one is supplied.
/// Any IO error (including Ctrl+C) → FlowError::Cancelled.
pub(super) fn input(label: &str, default: Option<&str>) -> Result<String, FlowError> {
    let theme = dialoguer::theme::ColorfulTheme::default();
    let mut b = dialoguer::Input::<String>::with_theme(&theme)
        .with_prompt(label)
        .allow_empty(true);

    if let Some(d) = default {
        b = b.default(d.to_string());
    }

    b.interact_text().map_err(|_| FlowError::Cancelled)
}

/// Hidden password prompt.  IO error → FlowError::Cancelled.
pub(super) fn password(label: &str) -> Result<String, FlowError> {
    dialoguer::Password::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(label)
        .interact()
        .map_err(|_| FlowError::Cancelled)
}

/// Yes/No confirmation.  Esc → FlowError::Cancelled.
pub(super) fn confirm(label: &str, default: bool) -> Result<bool, FlowError> {
    dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(label)
        .default(default)
        .interact_opt()
        .map_err(|_| FlowError::Cancelled)?
        .ok_or(FlowError::Cancelled)
}

/// v1-style typed number menu: "Select option [1/2/3] (default): "
/// Wraps `input()` so Ctrl+C propagates as FlowError::Cancelled.
pub(super) fn typed_menu(max: usize, default: usize) -> Result<usize, FlowError> {
    let opts: String = (1..=max).map(|n| n.to_string()).collect::<Vec<_>>().join("/");
    let def = default.to_string();
    let raw = input(&format!("Select option [{opts}] ({default})"), Some(&def))?;
    let n = raw.trim().parse::<usize>().unwrap_or(default);
    if n >= 1 && n <= max { Ok(n - 1) } else { Ok(default - 1) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Progress helper (re-exported so flow modules don't need to import ui::progress)
// ─────────────────────────────────────────────────────────────────────────────

pub(super) fn spinner(msg: impl Into<String>) -> indicatif::ProgressBar {
    progress::spinner(msg)
}

// ─────────────────────────────────────────────────────────────────────────────
// Visual helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Cyan `═══ Title ═══` section divider.
pub(super) fn section(title: &str) {
    println!();
    println!("  {} {} {}",
        colors::cyan("═══"),
        colors::bold(title),
        colors::cyan("═══"));
    println!();
}

/// Step header inside a multi-step flow — slightly narrower indentation.
pub(super) fn step(n: usize, label: &str) {
    println!("{}", colors::bold(&format!("  Step {n}: {label}")));
}

/// Waits for Enter before returning to the parent menu.
///
/// Uses dialoguer's Input prompt which properly manages terminal raw-mode state
/// (enter + exit) so residual bytes don't leak from previous interactive steps.
pub(super) fn press_enter() {
    println!();
    println!("  {}  Press Enter to return…", colors::dim("→"));
    let _ = dialoguer::Input::<String>::new()
        .allow_empty(true)
        .interact_text();
}

// ─────────────────────────────────────────────────────────────────────────────
// Account / repository pickers — used by repos, git_ops, ssh modules
// ─────────────────────────────────────────────────────────────────────────────

/// List all accounts and let the user pick one with an arrow-key Select.
/// Returns None (and prints a hint) if no accounts exist yet.
pub(super) async fn pick_account(
    handle: &CliServicesHandle,
) -> Result<Option<AccountDto>, FlowError> {
    let pb = spinner("Loading accounts…");
    let accounts = handle.services().list_accounts(None).await?;
    pb.finish_and_clear();

    if accounts.is_empty() {
        println!("{} No accounts found — use option [7] to add one.",
            colors::info_prefix());
        press_enter();
        return Ok(None);
    }

    let labels: Vec<String> = accounts.iter()
        .map(|a| format!("  {:<20}  {:<14}  {}",
            a.alias,
            a.platform_name.as_deref().unwrap_or("?"),
            colors::dim(&a.username)))
        .collect();
    let opts: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

    let idx = select_menu("Select account", &opts)?;
    Ok(Some(accounts[idx].clone()))
}

/// List cloned repositories for an account and let the user pick one.
/// Returns None (and prints a hint) if none are cloned yet.
pub(super) async fn pick_cloned_repo(
    handle: &CliServicesHandle,
    account_uuid: Uuid,
) -> Result<Option<RepositoryDto>, FlowError> {
    let pb = spinner("Loading repositories…");
    let all = handle.services().list_repositories(Some(account_uuid)).await?;
    pb.finish_and_clear();

    let cloned: Vec<_> = all.into_iter().filter(|r| r.is_cloned).collect();

    if cloned.is_empty() {
        println!("{} No cloned repositories — use option [1] to clone one.",
            colors::info_prefix());
        press_enter();
        return Ok(None);
    }

    let labels: Vec<String> = cloned.iter()
        .map(|r| format!("  {:<30}  {}",
            r.name,
            colors::dim(r.local_path.as_deref().unwrap_or("?"))))
        .collect();
    let opts: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

    let idx = select_menu("Select repository", &opts)?;
    Ok(Some(cloned[idx].clone()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Account creation wizard — used by ssh.rs when no accounts exist
// ─────────────────────────────────────────────────────────────────────────────

/// Walks the user through creating a new account and returns the stored DTO.
pub(super) async fn create_account_interactive(
    handle: &CliServicesHandle,
) -> Result<AccountDto, FlowError> {
    section("New Account Details");

    let alias = {
        let raw = input("Account alias  (e.g. work, personal, client-acme)", None)?;
        if raw.trim().is_empty() {
            return Err(FlowError::Cancelled);
        }
        raw.trim().to_string()
    };

    let plat_labels: Vec<&str> = PLATFORM_OPTIONS.iter().map(|(_, l)| *l).collect();
    let plat_idx               = select_menu("Platform", &plat_labels)?;
    let (slug, _)              = PLATFORM_OPTIONS[plat_idx];
    let platform_type          = parse_platform(slug)?;
    let platform_id            = platform_uuid_for(&platform_type);

    let username = {
        let raw = input("Git username on this platform", None)?;
        if raw.trim().is_empty() { return Err(FlowError::Cancelled); }
        raw.trim().to_string()
    };
    let email = {
        let raw = input("Email (used in git commits and SSH key comment)", None)?;
        if raw.trim().is_empty() { return Err(FlowError::Cancelled); }
        raw.trim().to_string()
    };

    let pb = spinner(format!("Creating account '{alias}'…"));
    let result = handle.services().add_account(AddAccountCommand {
        alias: alias.clone(),
        platform_id,
        username,
        email,
        auth_method: "ssh".to_string(),
    }).await;
    pb.finish_and_clear();

    let account = result?;
    println!("{} Account '{}' created.", colors::success_prefix(), account.alias);
    Ok(account)
}

// ─────────────────────────────────────────────────────────────────────────────
// Table rendering
// ─────────────────────────────────────────────────────────────────────────────

/// Draws the account table with heavy-border box-drawing chars, matching v1.
/// Status cells are colorised; ANSI codes are added AFTER padding so column
/// widths remain correct.
pub(super) fn print_accounts_table(accounts: &[AccountDto]) {
    if accounts.is_empty() { return; }

    // Dynamic column widths based on data
    let w_alias = accounts.iter().map(|a| a.alias.len()).max().unwrap_or(7).max(7);
    let w_plat  = accounts.iter()
        .map(|a| a.platform_name.as_deref().unwrap_or("").len())
        .max().unwrap_or(8).max(8);
    let w_user  = accounts.iter().map(|a| a.username.len()).max().unwrap_or(8).max(8);
    let w_st    = 10usize;

    let _sep = |_mid: &str, fill: &str| {
        format!("  ┃{f}┃{f}┃{f}┃{f}┃{f}┃",
            f = format!("{}", fill.repeat(0))) // placeholder
    };

    // Build separator strings
    let bar = |w: usize| "━".repeat(w + 2);
    let top = format!("  ┏{}┳{}┳{}┳{}┳{:━<4}┓",
        bar(3), bar(w_alias), bar(w_plat), bar(w_user), bar(w_st));
    let mid = format!("  ┡{}╇{}╇{}╇{}╇{:━<4}┩",
        bar(3), bar(w_alias), bar(w_plat), bar(w_user), bar(w_st));
    let bot = format!("  └{}┴{}┴{}┴{}┴{:─<4}┘",
        "─".repeat(5), "─".repeat(w_alias + 2), "─".repeat(w_plat + 2),
        "─".repeat(w_user + 2), "─".repeat(w_st + 2));

    println!();
    println!("{}", colors::bold(&top));
    println!("  ┃ {:<3} ┃ {:<w_alias$} ┃ {:<w_plat$} ┃ {:<w_user$} ┃ {:<w_st$} ┃",
        colors::bold(" ID"), colors::bold("Account"),
        colors::bold("Platform"), colors::bold("Username"), colors::bold("Status"));
    println!("{}", colors::bold(&mid));

    for (i, a) in accounts.iter().enumerate() {
        let id       = format!("{:>3}", i + 1);
        let alias_p  = format!("{:<w_alias$}", a.alias);
        let plat_p   = format!("{:<w_plat$}", a.platform_name.as_deref().unwrap_or("—"));
        let user_p   = format!("{:<w_user$}", a.username);
        let st_raw   = a.status.to_string();
        let st_p     = format!("{:<w_st$}", st_raw);

        println!("  │ {} │ {} │ {} │ {} │ {} │",
            colors::dim(&id),
            colors::cyan(&alias_p),
            colors::dim(&plat_p),
            user_p,
            colors::colored_status(&st_p));
    }

    println!("{}", bot);
    println!();
}

/// Draws the repository table.
#[allow(dead_code)]
pub(super) fn print_repos_table(repos: &[RepositoryDto]) {
    if repos.is_empty() { return; }

    let w_name   = repos.iter().map(|r| r.name.len()).max().unwrap_or(10).max(10);
    let w_branch = repos.iter().map(|r| r.default_branch.len()).max().unwrap_or(6).max(6);
    let w_path   = repos.iter()
        .map(|r| r.local_path.as_deref().unwrap_or("—").len())
        .max().unwrap_or(4).max(4).min(40);

    let bar = |w: usize| "━".repeat(w + 2);
    let top = format!("  ┏{}┳{}┳{}┳{:━<8}┓", bar(3), bar(w_name), bar(w_branch), bar(w_path));
    let mid = format!("  ┡{}╇{}╇{}╇{:━<8}┩", bar(3), bar(w_name), bar(w_branch), bar(w_path));
    let bot = format!("  └{}┴{}┴{}┴{:─<8}┘",
        "─".repeat(5), "─".repeat(w_name + 2), "─".repeat(w_branch + 2), "─".repeat(w_path + 2));

    println!();
    println!("{}", colors::bold(&top));
    println!("  ┃ {:<3} ┃ {:<w_name$} ┃ {:<w_branch$} ┃ {:<w_path$} ┃",
        colors::bold(" ID"), colors::bold("Repository"),
        colors::bold("Branch"), colors::bold("Path"));
    println!("{}", colors::bold(&mid));

    for (i, r) in repos.iter().enumerate() {
        let id     = format!("{:>3}", i + 1);
        let name_p = format!("{:<w_name$}", r.name);
        let br_p   = format!("{:<w_branch$}", r.default_branch);
        let path   = r.local_path.as_deref().unwrap_or("—");
        let path_t = if path.len() > w_path { &path[path.len() - w_path..] } else { path };
        let path_p = format!("{:<w_path$}", path_t);

        let status = if r.is_cloned { colors::green("cloned") } else { colors::dim("remote") };
        println!("  │ {} │ {} │ {} │ {} {} │",
            colors::dim(&id), colors::cyan(&name_p), colors::dim(&br_p),
            path_p, status);
    }

    println!("{}", bot);
    println!();
}

/// Draws a git status diff table (porcelain codes, file paths, change type labels).
pub(super) fn print_status_table(entries: &[crate::services::GitStatusEntry]) {
    if entries.is_empty() { return; }

    use crate::ui::colors::*;

    let w_path = entries.iter().map(|e| e.path.len()).max().unwrap_or(10).max(10).min(60);

    println!();
    println!("  {} {:<4} │ {:<w_path$} │ {}", bold(""),
        bold("Code"), bold("File"), bold("Change"));
    println!("  {}", "─".repeat(w_path + 24));

    for e in entries {
        let code  = &e.status;
        let label = match e.status.trim() {
            s if s.starts_with('M') || s.ends_with('M') => yellow("  modified"),
            s if s.starts_with('A')                     => green("  added   "),
            s if s.starts_with('D') || s.ends_with('D') => red("  deleted "),
            "??"                                         => dim("  untracked"),
            "UU"                                         => red("  conflict "),
            _                                            => dim("  changed  "),
        };
        let path = if e.path.len() > w_path {
            format!("…{}", &e.path[e.path.len() - (w_path - 1)..])
        } else {
            e.path.clone()
        };
        println!("    {} {:<2}  │ {:<w_path$} │ {}",
            dim("│"), bold(code), path, label);
    }

    println!("  {}", "─".repeat(w_path + 24));
    println!();
}

/// Render entries in a paginated book‑spread layout (2 columns per page).
///
/// Navigation: `[p]` previous, `[n]` next, `[q]` quit, or type a page number.
/// Filter: `[f]` cycles through all → modified → added → deleted → untracked.
/// Sort: `[s]` toggles ascending / descending by path.
///
/// If all entries fit on one page, pagination is hidden.
/// Uses in‑place overwrite (ANSI cursor‑up) — no terminal clearing.
pub(super) fn print_status_table_paginated(entries: &mut Vec<crate::services::GitStatusEntry>) {
    use crate::ui::tables::render_status_box;

    if entries.is_empty() { return; }

    const ROWS_PER_PAGE: usize = 20;

    let mut sort_asc = true;
    let mut filter: Option<char> = None;
    let mut page = 0usize;
    let mut prev_lines = 0usize;

    loop {
        let mut vis: Vec<&crate::services::GitStatusEntry> = entries
            .iter()
            .filter(|e| match filter {
                None      => true,
                Some('M') => e.status.trim().contains('M'),
                Some('A') => e.status.trim().starts_with('A'),
                Some('D') => e.status.trim().contains('D'),
                Some('?') => e.status.trim() == "??",
                _         => true,
            })
            .collect();

        if sort_asc { vis.sort_by(|a, b| a.path.cmp(&b.path)); }
        else        { vis.sort_by(|a, b| b.path.cmp(&a.path)); }

        let n_vis    = vis.len();
        let two_cols = n_vis > ROWS_PER_PAGE;
        let per_page = ROWS_PER_PAGE * if two_cols { 2 } else { 1 };
        let n_pages  = n_vis.div_ceil(per_page).max(1);
        if page >= n_pages { page = n_pages - 1; }

        let start = page * per_page;
        let end   = n_vis.min(start + per_page);
        let slice = &vis[start..end];

        if prev_lines > 0 {
            print!("\x1B[{}A\x1B[J", prev_lines);
        }
        let lines = render_status_box(
            slice, entries.len(), n_vis, page, n_pages,
            two_cols, filter, sort_asc,
        );

        if n_pages == 1 && filter.is_none() {
            return;
        }
        prev_lines = lines;

        use std::io::{self, Write, BufRead};
        let _ = io::stdout().flush();
        let mut line = String::new();
        if io::stdin().lock().read_line(&mut line).is_err() { return; }
        let cmd = line.trim();

        match cmd {
            "q" => return,
            "n" if page + 1 < n_pages => page += 1,
            "p" if page > 0           => page -= 1,
            "s" => sort_asc = !sort_asc,
            "f" => {
                filter = match filter {
                    None      => Some('M'),
                    Some('M') => Some('A'),
                    Some('A') => Some('D'),
                    Some('D') => Some('?'),
                    _         => None,
                };
                page = 0;
            }
            "c" => { filter = None; page = 0; }
            _   => {
                if let Ok(n) = cmd.parse::<usize>() {
                    if n >= 1 && n <= n_pages { page = n - 1; }
                }
            }
        }
    }
}