// crates/gm_interface_cli/src/ui/interactive/git_ops.rs
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Main menu option [3]: Git push / pull / sync operations.
//
// Presents a sub-menu then delegates to the chosen operation.
// Each sub-flow collects the minimum required input, shows a progress
// spinner during the git subprocess, and displays a color-coded result.

use gm_ports::inbound::commands::{PullRepositoryCommand, PushRepositoryCommand};

use crate::services::CliServicesHandle;
use crate::ui::colors;
use super::common::{
    FlowResult, confirm, input, pick_account, pick_cloned_repo, press_enter,
    print_status_table, print_status_table_paginated, section, spinner, typed_menu,
};

// ─────────────────────────────────────────────────────────────────────────────
// Sub-menu constants
// ─────────────────────────────────────────────────────────────────────────────

const GIT_OPS_ITEMS: &[&str] = &[
    "[1] Pull         — fetch and integrate remote changes",
    "[2] Push         — stage all, commit, and push",
    "[3] Status       — show working directory changes",
    "[4] Detect       — detect current directory's account/repo",
    "[5] Dry-run pull — preview what a pull would do",
    "[6] Dry-run push — preview what a push would do",
    "[7] Remote repos — list repos from platform API",
    "[8] ← Back to main menu",
];

// ─────────────────────────────────────────────────────────────────────────────
// Entry point — [3] Git operations
// ─────────────────────────────────────────────────────────────────────────────

pub(super) async fn flow_git_ops(handle: &CliServicesHandle) -> FlowResult {
    section("Git Operations");

    for (i, item) in GIT_OPS_ITEMS.iter().enumerate() {
        println!("    {}", if i == 7 { colors::dim(item) } else { item.to_string() });
    }
    println!();

    let sel = typed_menu(8, 8)?; // default = 8 (Back)

    println!();
    match sel {
        0 => flow_git_pull(handle).await,
        1 => flow_git_push(handle).await,
        2 => flow_git_status(handle).await,
        3 => flow_git_detect(handle).await,
        4 => flow_dry_run_pull(handle).await,
        5 => flow_dry_run_push(handle).await,
        6 => flow_remote_repos(handle).await,
        7 => Ok(()), // Back
        _ => Ok(()),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pull
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_git_pull(handle: &CliServicesHandle) -> FlowResult {
    section("Git Pull");

    let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };
    let repo    = match pick_cloned_repo(handle, account.uuid).await? { Some(r) => r, None => return Ok(()) };
    let rebase  = confirm("Use --rebase instead of --merge?", false)?;

    let pb = spinner("Pulling…");
    let result = handle.services().git_pull(PullRepositoryCommand {
        repository_uuid: repo.uuid,
        account_uuid:    account.uuid,
        branch:          None,
        rebase,
        dry_run: false,
    }).await;
    pb.finish_and_clear();

    match result {
        Ok(r) if r.had_conflicts => {
            println!("{} Pull completed with merge conflicts.",
                colors::warn_prefix());
            println!("  Resolve conflicts, then push with option {}.",
                colors::bold("[3] → [2]"));
        }
        Ok(r) => {
            let sha = r.current_sha.as_deref()
                .map(|s| &s[..s.len().min(8)]).unwrap_or("?");
            println!("{} Pulled {} new commit(s).  HEAD: {}",
                colors::success_prefix(),
                colors::bold(&r.commits_transferred.to_string()),
                colors::dim(sha));
        }
        Err(e) => println!("{} Pull failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Push
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_git_push(handle: &CliServicesHandle) -> FlowResult {
    section("Git Push");

    let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };
    let repo    = match pick_cloned_repo(handle, account.uuid).await? { Some(r) => r, None => return Ok(()) };

    // Show working directory changes before asking for a commit message.
    if let Some(ref local) = repo.local_path {
        let pb = spinner("Checking working directory…");
        let entries = handle.services()
            .git_status(std::path::Path::new(local)).await
            .unwrap_or_default();
        pb.finish_and_clear();

        if entries.is_empty() {
            println!("{} Working directory is clean — nothing to commit.",
                colors::info_prefix());
            press_enter();
            return Ok(());
        }

        println!("  {} file(s) will be committed:", entries.len());
        print_status_table(&entries);
    }

    let message = {
        let raw = input("Commit message", None)?;
        if raw.trim().is_empty() { return Ok(()); }
        raw.trim().to_string()
    };
    let branch = {
        let raw = input("Branch", Some(&repo.default_branch))?;
        if raw.trim().is_empty() { repo.default_branch.clone() } else { raw.trim().to_string() }
    };

    let pb = spinner("Staging → committing → pushing…");
    let result = handle.services().git_push(PushRepositoryCommand {
        repository_uuid: repo.uuid,
        account_uuid:    account.uuid,
        commit_message:  message.clone(),
        branch,
        force: false,
        dry_run: false,
    }).await;
    pb.finish_and_clear();

    match result {
        Ok(r) => {
            let sha = r.current_sha.as_deref()
                .map(|s| &s[..s.len().min(8)]).unwrap_or("?");
            println!("{} Pushed {} commit(s).  HEAD: {}",
                colors::success_prefix(),
                colors::bold(&r.commits_transferred.to_string()),
                colors::dim(sha));
            println!("  Message : {}", colors::dim(&message));
        }
        Err(e) => println!("{} Push failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Status
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_git_status(handle: &CliServicesHandle) -> FlowResult {
    section("Working Directory Status");

    let path_str = {
        let raw = input("Repository path", Some("."))?;
        if raw.trim().is_empty() { ".".to_string() } else { raw.trim().to_string() }
    };

    let pb = spinner("Checking status…");
    let result = handle.services()
        .git_status(std::path::Path::new(&path_str)).await;
    pb.finish_and_clear();

    match result {
        Ok(entries) if entries.is_empty() => {
            println!("{} Working directory is clean — nothing to commit.",
                colors::success_prefix());
        }
        Ok(mut entries) => {
            print_status_table_paginated(&mut entries);

            // Prompt to push if there are changes
            if !entries.is_empty() {
                let do_push = confirm(
                    &format!("Push {} change(s) now?", entries.len()), false
                )?;
                if do_push {
                    return flow_git_push(handle).await;
                }
            }
        }
        Err(e) => println!("{} Could not get status: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Detect
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_git_detect(handle: &CliServicesHandle) -> FlowResult {
    section("Repository Detection");

    let raw = input("Repository path", Some("."))?;
    let path = if raw.trim().is_empty() { std::path::PathBuf::from(".") } else { std::path::PathBuf::from(raw.trim()) };

    let pb = spinner("Detecting...");
    let result = handle.services().detect_repository(&path).await;
    pb.finish_and_clear();

    match result {
        Ok(r) if r.tracked => {
            println!("  {}  {}", colors::bold("Repository:"), colors::green(r.repository_name.as_deref().unwrap_or("(unknown)")));
            println!("  Account: {}", r.account_alias.as_deref().unwrap_or("(unknown)"));
            if let Some(name) = &r.platform_name { println!("  Platform: {name}"); }
            if let Some(branch) = &r.current_branch { println!("  Branch: {branch}"); }
            if let Some(url) = &r.remote_url { println!("  Remote: {}", colors::dim(url)); }
        }
        Ok(r) => {
            println!("{} No tracked repository found.", colors::warn_prefix());
            if let Some(url) = r.remote_url {
                println!("  Git repo detected at: {}", colors::dim(&url));
                println!("  Use {} -> {} to track it.", colors::bold("[4]"), colors::bold("Set up repository"));
            } else {
                println!("  No git repository found at this path.");
            }
        }
        Err(e) => println!("{} Detection failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Dry-run pull
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_dry_run_pull(handle: &CliServicesHandle) -> FlowResult {
    section("Dry-Run: Pull Preview");

    let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };
    let repo    = match pick_cloned_repo(handle, account.uuid).await? { Some(r) => r, None => return Ok(()) };

    let pb = spinner("Analysing pull...");
    let result = handle.services().dry_run_pull(repo.uuid, account.uuid).await;
    pb.finish_and_clear();

    match result {
        Ok(preview) => {
            println!("  {}", colors::bold(&preview.summary));
            for d in &preview.details {
                println!("    {}", colors::dim(d));
            }
            if preview.conflicts_predicted {
                println!("{} Conflicts predicted!", colors::warn_prefix());
            }
        }
        Err(e) => println!("{} Analysis failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Dry-run push
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_dry_run_push(handle: &CliServicesHandle) -> FlowResult {
    section("Dry-Run: Push Preview");

    let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };
    let repo    = match pick_cloned_repo(handle, account.uuid).await? { Some(r) => r, None => return Ok(()) };

    let pb = spinner("Analysing push...");
    let result = handle.services().dry_run_push(repo.uuid, account.uuid).await;
    pb.finish_and_clear();

    match result {
        Ok(preview) => {
            println!("  {}", colors::bold(&preview.summary));
            for d in &preview.details {
                println!("    {}", colors::dim(d));
            }
        }
        Err(e) => println!("{} Analysis failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Remote repository listing from platform API
// ─────────────────────────────────────────────────────────────────────────────

async fn flow_remote_repos(handle: &CliServicesHandle) -> FlowResult {
    section("Remote Repository Listing");

    let account = match pick_account(handle).await? { Some(a) => a, None => return Ok(()) };

    let pb = spinner(&format!("Fetching remote repos for '{}'...", account.alias));
    let result = handle.services().list_remote_repositories(account.uuid, 1, 15).await;
    pb.finish_and_clear();

    match result {
        Ok(names) => {
            if names.is_empty() {
                println!("{} No remote repositories found.", colors::info_prefix());
            } else {
                println!("{} Repositories for {}:", colors::success_prefix(), colors::bold(&account.alias));
                for (i, name) in names.iter().enumerate() {
                    println!("  {:>3}. {}", i + 1, name);
                }
            }
        }
        Err(e) => println!("{} Failed: {}", colors::error_prefix(), e),
    }

    press_enter();
    Ok(())
}