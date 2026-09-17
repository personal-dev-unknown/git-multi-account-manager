use std::path::{Path, PathBuf};
use std::sync::Arc;

use gm_kernel::security::kill_current_child;
use gm_ports::inbound::commands::CloneRepositoryCommand;
use gm_shared::models::account::AccountDto;

use crate::services::CliServicesHandle;
use crate::ui::colors;
use super::common::{
    FlowResult, FlowError, confirm, input, press_enter,
    section, spinner, step, typed_menu,
};

pub(super) async fn flow_clone(handle: &CliServicesHandle) -> FlowResult {
    section("Clone a Repository");

    let accounts = handle.services().list_accounts(None).await.unwrap_or_default();

// ── Clone a Repository ──

// ─── Step 1: Clone method ───────────────────────────────────────
step(1, "Clone Method");
println!();
println!("  {}  Anonymous clone  (paste a public URL)", colors::yellow("[1]"));
println!("  {}  Clone with an account", colors::yellow("[2]"));
println!("  {}  Back", colors::dim("[3]"));
println!();

let method_sel = typed_menu(3, 1)?;

match method_sel {
    // ── 1. Anonymous ─────────────────────────────────────────────
    0 => return clone_anonymous(handle).await,

    // ── 2. Authenticated ─────────────────────────────────────────
    1 => {
        if accounts.is_empty() {
            println!();
            println!("{} No accounts configured yet.", colors::warn_prefix());
            println!("  Use option [7] from the main menu to add one first.");
            press_enter();
            return Ok(());
        }

        // Only one account → skip selection step, use it directly
        let account = if accounts.len() == 1 {
            &accounts[0]
        } else {
            // ─── Step 2: Select account ──────────────────────────────
            step(2, "Select Account");
            println!();
            for (i, a) in accounts.iter().enumerate() {
                let plat = a.platform_name.as_deref().unwrap_or("?");
                println!(
                    "  {}  {:20}  {}  ({})",
                    colors::yellow(&format!("[{}]", i + 1)),
                    a.alias,
                    colors::dim(plat),
                    a.username
                );
            }
            println!(
                "  {}  Back",
                colors::dim(&format!("[{}]", accounts.len() + 1))
            );
            println!();

            let acct_sel = typed_menu(accounts.len() + 1, 1)?;

            if acct_sel >= accounts.len() {
                return Ok(()); // Back
            }

            &accounts[acct_sel]
        };
        // ─── Step 3: Clone type ──────────────────────────────────
        // Step number shifts depending on whether account selection was shown
        let clone_type_step = if accounts.len() == 1 { 2 } else { 3 };
        step(clone_type_step, "Clone Type");
        println!();
        println!(
            "  {}  Clone from my repositories  (requires PAT)",
            colors::yellow("[1]")
        );
        println!(
            "  {}  Clone a collaborator repository  (paste URL)",
            colors::yellow("[2]")
        );
        println!("  {}  Back", colors::dim("[3]"));
        println!();

        let clone_type = typed_menu(3, 1)?;

        match clone_type {
            // ── My repositories — listed 1..n, paginated by 15 ──
            0 => return clone_with_pat(handle, account).await,

            // ── Collaborator repo via URL ────────────────────────
            1 => return clone_collaborator_url(handle, account).await,

            _ => return Ok(()), // Back
        }
    }

    _ => return Ok(()), // Back
} // end match method_sel
} // end fn flow_clone

// ─────────────────────────────────────────────────────────────────────────────
// Anonymous clone — public repo URL, no account needed
// ─────────────────────────────────────────────────────────────────────────────

async fn clone_anonymous(handle: &CliServicesHandle) -> FlowResult {
    println!();
    step(2, "Repository URL");
    println!("  {}  SSH:   {}", colors::dim("hint"), colors::dim("git@github.com:owner/repo.git"));
    println!("  {}  HTTPS: {}", colors::dim("hint"), colors::dim("https://github.com/owner/repo.git"));
    println!();
    let url = {
        let raw = input("Repository URL", None)?;
        if raw.trim().is_empty() { return Ok(()); }
        raw.trim().to_string()
    };

    println!();
    step(3, "Local Destination");
    let raw_dest = input("Local path  (blank = current directory)", Some(""))?;
    let dest_string = if raw_dest.trim().is_empty() {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(url.split('/').last().unwrap_or("repo"))
            .to_string_lossy()
            .to_string()
    } else {
        expand_home_dir(&PathBuf::from(raw_dest.trim()))
            .to_string_lossy()
            .to_string()
    };
    let destination = Some(dest_string.clone());
    let dest_path = Path::new(&dest_string);

    if dest_path.exists() {
        let overwrite = confirm(
            &format!("'{}' already exists — remove and re-clone?", dest_string),
            false,
        )?;
        if overwrite {
            if let Err(e) = std::fs::remove_dir_all(dest_path) {
                println!("{} Failed to remove existing directory: {}", colors::error_prefix(), e);
                press_enter();
                return Ok(());
            }
        } else {
            press_enter();
            return Ok(());
        }
    }

    let shallow = confirm("Shallow clone? (faster, no full history)", false)?;

    println!();
    let pb = Arc::new(spinner(format!("Cloning {}…", url)));
    let pb_clone = pb.clone();

    let clone_fut = handle.services().clone_repository(CloneRepositoryCommand {
        url:          url.clone(),
        account_uuid: None,
        destination:  destination.clone(),
        branch:       None,
        depth:        if shallow { 1 } else { 0 },
        ..Default::default()
    });

    let result = tokio::select! {
        result = clone_fut => result,
        _ = tokio::signal::ctrl_c() => {
            pb_clone.finish_and_clear();
            println!();
            println!("  {}  Cancelling clone…", colors::yellow("⚠"));
            kill_current_child();
            let _ = std::fs::remove_dir_all(dest_path);
            return Err(FlowError::Cancelled);
        }
    };

    pb.finish_and_clear();

    match result {
        Ok(repo) => {
            println!("{} Cloned successfully!", colors::success_prefix());
            println!();
            if let Some(path) = &repo.local_path {
                println!("  Location : {}", colors::dim(path));
            }
            println!("  Account  : {} (anonymous)", colors::dim("none"));
            println!();
            println!("  {}  Use option {} to push, pull, and manage.",
                colors::dim("→"), colors::bold("[3]"));
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.to_lowercase().contains("cancelled") {
                let _ = std::fs::remove_dir_all(dest_path);
            }
            println!("{} Clone failed: {}", colors::error_prefix(), e);
        }
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// [1a] Clone with PAT — list user's remote repos and pick one
// ─────────────────────────────────────────────────────────────────────────────

async fn clone_with_pat(handle: &CliServicesHandle, account: &AccountDto) -> FlowResult {
    let mut page = 1u32;
    let per_page = 15u32;

    loop {
        println!();
        let pb = spinner(format!("Fetching repositories (page {page})…"));
        let result = handle.services()
            .list_remote_repositories(account.uuid, page, per_page)
            .await;
        pb.finish_and_clear();

        let repos = match result {
            Ok(r) => r,
            Err(e) => {
                let msg = e.to_string();
                if msg.to_lowercase().contains("token") {
                    println!("{} No Personal Access Token configured for this account.", colors::warn_prefix());
                    println!("  Use option [7] → [2] Setup PAT to configure one.");
                } else {
                    println!("{} Failed to list repositories: {}", colors::error_prefix(), e);
                }
                press_enter();
                return Ok(());
            }
        };

        if repos.is_empty() && page == 1 {
            println!("  No repositories found for this account.");
            press_enter();
            return Ok(());
        }

        if repos.is_empty() {
            page = page.saturating_sub(1);
            continue;
        }

        let has_next = repos.len() == per_page as usize;
        let has_prev = page > 1;

        println!();
        for (i, name) in repos.iter().enumerate() {
            println!("  {}  {}", colors::yellow(&format!("[{}]", i + 1)), name);
        }
        println!();

        let mut nav_labels: Vec<(&str, bool)> = Vec::new();
        if has_next { nav_labels.push(("Next page",     false)); }
        if has_prev { nav_labels.push(("Previous page", false)); }
        nav_labels.push(("Back", true));

        let total_options = repos.len() + nav_labels.len();

        for (i, (label, dim)) in nav_labels.iter().enumerate() {
            let n = repos.len() + i + 1;
            if *dim {
                println!("  {}  {}", colors::dim(&format!("[{}]", n)), label);
            } else {
                println!("  {}  {}", colors::yellow(&format!("[{}]", n)), label);
            }
        }

        let sel = typed_menu(total_options, 1)?;

        if sel < repos.len() {
            return do_clone(handle, account, &repos[sel]).await;
        }

        let cursor = sel - repos.len();

        if has_next && cursor == 0 { page += 1; continue; }
        if has_prev && cursor == (if has_next { 1 } else { 0 }) { page = page.saturating_sub(1); continue; }

        return Ok(());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// [1b] Collaborator URL — paste HTTPS URL, convert to SSH, clone
// ─────────────────────────────────────────────────────────────────────────────

async fn clone_collaborator_url(handle: &CliServicesHandle, account: &AccountDto) -> FlowResult {
    let alias = account.ssh_host_alias.as_deref().unwrap_or(&account.alias).to_string();

    println!();
    println!("{}", colors::bold("  Repository URL"));
    println!("  {}  Paste the HTTPS URL of the collaborator's repository.", colors::dim("hint"));
    println!("  {}  It will be converted to SSH for authentication with your account.", colors::dim("hint"));
    println!();
    let raw = input("Repository URL", None)?;
    if raw.trim().is_empty() { return Ok(()); }
    let raw_url = raw.trim().to_string();

    let url = rewrite_ssh_url(&raw_url, &alias);
    if url != raw_url {
        println!("  {}  Converted to SSH: {}", colors::dim("→"), colors::cyan(&url));
    }

    do_clone(handle, account, &url).await
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared clone execution for account-based clones
// ─────────────────────────────────────────────────────────────────────────────

async fn do_clone(
    handle: &CliServicesHandle,
    account: &AccountDto,
    url: &str,
) -> FlowResult {
    println!();
    println!("{}", colors::bold("  Local Destination"));
    let raw_dest = input("Local path  (blank = current directory)", Some(""))?;
    let repo_dir = url.split('/').last().unwrap_or("repo").trim_end_matches(".git");
    let dest_string = if raw_dest.trim().is_empty() {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(repo_dir)
            .to_string_lossy()
            .to_string()
    } else {
        expand_home_dir(&PathBuf::from(raw_dest.trim()))
            .to_string_lossy()
            .to_string()
    };
    let destination = Some(dest_string.clone());
    let dest_path = Path::new(&dest_string);

    if dest_path.exists() {
        let overwrite = confirm(
            &format!("'{}' already exists — remove and re-clone?", dest_string),
            false,
        )?;
        if overwrite {
            if let Err(e) = std::fs::remove_dir_all(dest_path) {
                println!("{} Failed to remove existing directory: {}", colors::error_prefix(), e);
                press_enter();
                return Ok(());
            }
        } else {
            press_enter();
            return Ok(());
        }
    }

    let shallow = confirm("Shallow clone? (faster, no full history)", false)?;

    println!();
    let pb = Arc::new(spinner(format!("Cloning {}…", url)));
    let pb_clone = pb.clone();

    let clone_fut = handle.services().clone_repository(CloneRepositoryCommand {
        url:          url.to_string(),
        account_uuid: Some(account.uuid),
        destination:  destination.clone(),
        branch:       None,
        depth:        if shallow { 1 } else { 0 },
        ..Default::default()
    });

    let result = tokio::select! {
        result = clone_fut => result,
        _ = tokio::signal::ctrl_c() => {
            pb_clone.finish_and_clear();
            println!();
            println!("  {}  Cancelling clone…", colors::yellow("⚠"));
            kill_current_child();
            let _ = std::fs::remove_dir_all(dest_path);
            return Err(FlowError::Cancelled);
        }
    };

    pb.finish_and_clear();

    match result {
        Ok(repo) => {
            println!("{} Cloned successfully!", colors::success_prefix());
            println!();
            if let Some(path) = &repo.local_path {
                println!("  Location : {}", colors::dim(path));
            }
            println!("  Account  : {}  ({})", account.alias, account.username);
            println!();
            println!("  {}  Use option {} to push, pull, and manage.",
                colors::dim("→"), colors::bold("[3]"));
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.to_lowercase().contains("cancelled") {
                let _ = std::fs::remove_dir_all(dest_path);
            }
            println!("{} Clone failed: {}", colors::error_prefix(), e);
        }
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// [2] Check current repository account
// ─────────────────────────────────────────────────────────────────────────────

pub(super) async fn flow_check_repo(handle: &CliServicesHandle) -> FlowResult {
    section("Check Repository Account");

    let path_str = {
        let raw = input("Repository path", Some("."))?;
        if raw.trim().is_empty() { ".".to_string() } else { raw.trim().to_string() }
    };

    let pb = spinner("Looking up repository…");
    let repos = handle.services().list_repositories(None).await?;
    pb.finish_and_clear();

    // Match by canonical path so symlinks and `./` prefixes work.
    let canonical = std::fs::canonicalize(&path_str).ok();
    let found = repos.iter().find(|r| {
        r.local_path.as_deref().map(|lp| {
            std::fs::canonicalize(lp).ok().as_ref() == canonical.as_ref()
                || lp == path_str.as_str()
        }).unwrap_or(false)
    });

    match found {
        Some(repo) => {
            println!("{} Git Manager is tracking this repository.",
                colors::success_prefix());
            println!();
            println!("  Name     : {}", colors::bold(&repo.name));
            println!("  Branch   : {}", colors::dim(&repo.default_branch));
            if let Some(sha) = &repo.last_commit_sha {
                println!("  Last SHA : {}", colors::dim(&sha[..sha.len().min(8)]));
            }
            if let Some(ts) = &repo.last_synced_at {
                println!("  Synced   : {}", colors::dim(&ts.to_string()));
            }
        }
        None => {
            println!("{} This path is not tracked by Git Manager.",
                colors::warn_prefix());
            println!();
            println!("  Use option {} to register an existing local repository.",
                colors::bold("[4]"));
            println!("  Use option {} to clone from remote with tracking.",
                colors::bold("[1]"));
        }
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// [4] Set up repository for specific account
// ─────────────────────────────────────────────────────────────────────────────

pub(super) async fn flow_setup_repo(handle: &CliServicesHandle) -> FlowResult {
    section("Setup Local Project with Remote");

    // Step 1: Account
    step(1, "Select Account");
    println!();
    let accounts = handle.services().list_accounts(None).await.unwrap_or_default();
    if accounts.is_empty() {
        println!("{} No accounts found — use option [7] to add one.",
            colors::warn_prefix());
        press_enter();
        return Ok(());
    }
    for (i, a) in accounts.iter().enumerate() {
        let plat = a.platform_name.as_deref().unwrap_or("?");
        println!("  {}  {:20}  {}  ({})",
            colors::yellow(&format!("[{}]", i + 1)),
            a.alias,
            colors::dim(plat),
            a.username);
    }
    println!("  {}  Back", colors::dim(&format!("[{}]", accounts.len() + 1)));
    println!();
    let acct_sel = typed_menu(accounts.len() + 1, 1)?;
    let account = if acct_sel < accounts.len() {
        &accounts[acct_sel]
    } else {
        return Ok(());
    };

    // Step 2: Path
    println!();
    step(2, "Repository Path");
    let local_path = {
        let raw = input("Enter repository path", None)?;
        if raw.trim().is_empty() { return Ok(()); }
        raw.trim().to_string()
    };

    if !std::path::Path::new(&local_path).join(".git").exists() {
        println!("{} No .git directory found at '{}' — is this a git repo?",
            colors::warn_prefix(), local_path);
        press_enter();
        return Ok(());
    }

    // Step 3: Remote URL
    println!();
    step(3, "Remote URL");
    let remote_url = {
        let raw = input("Remote URL", None)?;
        if raw.trim().is_empty() { return Ok(()); }
        raw.trim().to_string()
    };

    // Rewrite: git@github.com:owner/repo → git@github.com-alias:owner/repo
    let alias_url = rewrite_ssh_url(&remote_url, &account.alias);

    println!();
    println!("{} Run these commands to connect the repository to '{}':",
        colors::success_prefix(), colors::bold(&account.alias));
    println!();
    println!("    cd {}", colors::dim(&local_path));
    println!("    {}",
        colors::bold(&format!("git remote set-url origin {}", alias_url)));
    println!();
    println!("  SSH config entry  : {}",
        colors::cyan(&format!("~/.ssh/config → Host {}", alias_url
            .split('@').nth(1).unwrap_or("?")
            .split(':').next().unwrap_or("?"))));
    println!("  Authenticated as  : {} ({})",
        account.alias, account.username);
    println!();
    println!("  {}  All future push/pull operations will use the '{}' key.",
        colors::dim("→"), account.alias);

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Expand a leading `~` to the user's home directory.
fn expand_home_dir(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with("~/") || s == "~" {
        if let Some(home) = std::env::var("HOME").ok().map(PathBuf::from) {
            if s == "~" {
                return home;
            }
            return PathBuf::from(s.replacen('~', &home.to_string_lossy(), 1));
        }
    }
    if !path.is_absolute() {
        if let Ok(cwd) = std::env::current_dir() {
            return cwd.join(path);
        }
    }
    path.to_path_buf()
}

/// Rewrites a remote URL to use the account SSH host alias.
///
/// HTTPS URLs are converted to SSH:
///   `https://github.com/owner/repo.git` → `git@github.com-{alias}:owner/repo.git`
///
/// SSH URLs get the alias injected:
///   `git@github.com:owner/repo.git` → `git@github.com-{alias}:owner/repo.git`
///
/// Bare names (`owner/repo`) are returned unchanged — the service layer handles
/// them via `CloneUrlResolver::build_url()`.
fn rewrite_ssh_url(url: &str, alias: &str) -> String {
    // HTTPS → SSH with alias
    if url.starts_with("https://") || url.starts_with("http://") {
        let rest = url.trim_start_matches("https://")
            .trim_start_matches("http://");
        if let Some(slash) = rest.find('/') {
            let host = &rest[..slash];
            let path = &rest[slash + 1..];
            return format!("git@{}-{}:{}", host, alias, path);
        }
        // Malformed URL — return as-is
        return url.to_string();
    }

    // Already SSH → inject alias into host
    if let Some(at) = url.find('@') {
        if let Some(rel_colon) = url[at..].find(':') {
            let host_end = at + rel_colon;
            let host     = &url[at + 1..host_end];
            return format!("git@{}-{}:{}", host, alias, &url[host_end + 1..]);
        }
    }

    url.to_string() // bare name or unknown format: return unchanged
}
