// crates/gm_interface_cli/src/commands/git.rs
// Git operations dispatched through CliServices.

use std::{path::PathBuf, sync::Arc};
use clap::{Parser, Subcommand};
use gm_kernel::kernel::Kernel;
use gm_shared::errors::GitManagerError;
use gm_ports::inbound::commands::{PullRepositoryCommand, PushRepositoryCommand};

use crate::services::CliServicesHandle;
use crate::ui::colors::*;

#[derive(Parser, Debug)]
pub struct GitCmd {
    #[command(subcommand)]
    pub action: GitAction,
}

#[derive(Subcommand, Debug)]
pub enum GitAction {
    /// Pull latest changes. The command resolves the account and repo by the
    /// path — pass the path of a repository this tool already tracks.
    Pull {
        /// UUID of the repository to pull.
        #[arg(long)] repository: uuid::Uuid,
        /// Account to authenticate with.
        #[arg(long)] account:    uuid::Uuid,
        /// Preview what would happen without making changes.
        #[arg(long)] dry_run:   bool,
    },
    /// Commit all changes and push to the remote.
    Push {
        #[arg(long)] repository:     uuid::Uuid,
        #[arg(long)] account:        uuid::Uuid,
        #[arg(long, short = 'm')]    message: String,
        #[arg(long, default_value = "main")] branch: String,
        /// Preview what would happen without making changes.
        #[arg(long)] dry_run:   bool,
    },
    /// Status of a tracked repository.
    Status {
        /// Local path to the repository.
        #[arg(default_value = ".")] path: PathBuf,
    },
    /// Detect which account and repository are associated with the current directory.
    Detect {
        /// Path to check (defaults to current directory).
        #[arg(default_value = ".")] path: PathBuf,
    },
    /// List repositories from the platform API for a specific account.
    RemoteList {
        /// Account alias to list repositories for.
        #[arg(long)] account: String,
        /// Page number (default 1).
        #[arg(long, default_value = "1")] page: u32,
        /// Results per page (default 15).
        #[arg(long, default_value = "15")] per_page: u32,
    },
}

pub async fn handle(cmd: GitCmd, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
    let svc = kernel.get::<CliServicesHandle>()
        .ok_or_else(|| GitManagerError::Other("CLI services not initialised".to_string()))?;

    match cmd.action {
        GitAction::Pull { repository, account, dry_run } => {
            let msg = if dry_run { "Dry-run: analysing pull..." } else { "Pulling…" };
            let pb = spinner(msg);
            let result = if dry_run {
                let preview = svc.services().dry_run_pull(repository, account).await?;
                pb.finish_and_clear();
                println!("{} {}", info_prefix(), bold("Dry-Run Preview — Pull"));
                println!("  {}", dim(&preview.summary));
                for d in &preview.details {
                    println!("    {}", dim(d));
                }
                if preview.conflicts_predicted {
                    println!("{} Conflicts predicted!", warn_prefix());
                }
                return Ok(());
            } else {
                svc.services().git_pull(PullRepositoryCommand {
                    repository_uuid: repository,
                    account_uuid: account,
                    branch: None,
                    rebase: false,
                    dry_run: false,
                }).await?
            };
            pb.finish_and_clear();
            if result.commits_transferred > 0 {
                println!("{} Pulled {} commit(s)", success_prefix(), bold(&result.commits_transferred.to_string()));
            } else {
                println!("{} Already up to date.", info_prefix());
            }
            if result.had_conflicts {
                println!("{} Merge conflicts detected — resolve manually.", warn_prefix());
            }
            Ok(())
        }
        GitAction::Push { repository, account, message, branch, dry_run } => {
            let msg = if dry_run { "Dry-run: analysing push..." } else { "Pushing…" };
            let pb = spinner(msg);
            let result = if dry_run {
                let preview = svc.services().dry_run_push(repository, account).await?;
                pb.finish_and_clear();
                println!("{} {}", info_prefix(), bold("Dry-Run Preview — Push"));
                println!("  {}", dim(&preview.summary));
                for d in &preview.details {
                    println!("    {}", dim(d));
                }
                return Ok(());
            } else {
                svc.services().git_push(PushRepositoryCommand {
                    repository_uuid: repository,
                    account_uuid: account,
                    commit_message: message.clone(),
                    branch,
                    force: false,
                    dry_run: false,
                }).await?
            };
            pb.finish_and_clear();
            println!("{} Pushed {} commit(s)  \"{}\"", success_prefix(),
                bold(&result.commits_transferred.to_string()), message);
            Ok(())
        }
        GitAction::Status { path } => {
            let entries = svc.services().git_status(&path).await?;
            if entries.is_empty() {
                println!("{} Working tree clean.", info_prefix());
            } else {
                println!("  {} changed file(s):", bold(&entries.len().to_string()));
                for e in &entries {
                    let code   = &e.status;
                    let colour = match code.trim() {
                        s if s.starts_with('M') => yellow(code),
                        s if s.starts_with('A') => green(code),
                        s if s.starts_with('D') => red(code),
                        _                       => dim(code).to_string(),
                    };
                    println!("  {} {}", colour, e.path);
                }
            }
            Ok(())
        }
        GitAction::Detect { path } => {
            println!("{} Detecting repository...", info_prefix());
            let result = svc.services().detect_repository(&path).await?;
            println!();
            if result.tracked {
                println!("{} {}  {}",
                    bold("Repository:"),
                    result.repository_name.as_deref().unwrap_or("(unknown)"),
                    dim("(tracked)"));
                if let Some(alias) = &result.account_alias {
                    println!("  {}  {}", bold("Account:"), alias);
                }
                if let Some(name) = &result.platform_name {
                    println!("  {}  {}", bold("Platform:"), name);
                }
                if let Some(branch) = &result.current_branch {
                    println!("  {}  {}", bold("Branch:"), branch);
                }
                if let Some(url) = &result.remote_url {
                    println!("  {}  {}", bold("Remote:"), dim(url));
                }
            } else {
                println!("{} No tracked repository found at this path.", warn_prefix());
                if let Some(url) = result.remote_url {
                    println!("  Git repo detected: {}", dim(&url));
                    println!("  To track it, use: {} {}",
                        bold("git-zyrix interactive"),
                        dim("→ [4] Set up repository"));
                }
                if let Some(branch) = result.current_branch {
                    println!("  Current branch: {}", branch);
                }
            }
            Ok(())
        }
        GitAction::RemoteList { account, page, per_page } => {
            // Look up the account by alias to get its UUID
            let accounts = svc.services().list_accounts(None).await?;
            let acct = accounts.iter().find(|a| a.alias == account)
                .ok_or_else(|| GitManagerError::Other(format!("Account '{account}' not found")))?;

            let pb = spinner(&format!("Listing remote repositories for '{}'…", account));
            let repos = svc.services().list_remote_repositories(acct.uuid, page, per_page).await;
            pb.finish_and_clear();

            match repos {
                Ok(names) => {
                    if names.is_empty() {
                        println!("{} No repositories found for '{}'.", info_prefix(), account);
                    } else {
                        println!("{} Repositories for {} (page {page}):",
                            success_prefix(), bold(&account));
                        for (i, name) in names.iter().enumerate() {
                            println!("  {}. {}",
                                dim(&format!("{:>3}", i + 1 + ((page - 1) * per_page) as usize)),
                                name);
                        }
                    }
                }
                Err(e) => println!("{} Failed: {}", error_prefix(), e),
            }
            Ok(())
        }
    }
}

fn spinner(msg: &str) -> indicatif::ProgressBar {
    use indicatif::{ProgressBar, ProgressStyle};
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")
        .expect("git: hardcoded spinner template must be valid"));
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}