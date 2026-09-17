// crates/gm_interface_cli/src/app.rs
//
// The top-level Clap argument structures and the dispatch function that routes
// a parsed CLI to the correct command handler module.
//
// ── Design principle ──────────────────────────────────────────────────────────
// The Clap structs are pure data — they carry no logic and hold no references.
// All business logic lives in the `commands/` handlers. This separation means
// you can unit-test the dispatch logic by constructing CLI values directly,
// without touching the terminal or the kernel.
//
// ── Banner behaviour ──────────────────────────────────────────────────────────
// The banner is printed only when stdout is attached to a terminal (atty check)
// AND the --no-banner flag is not set AND the subcommand is not one of the
// machine-readable ones (config get, which is sometimes piped). The atty check
// prevents the banner from appearing in shell scripts that capture output.

use std::sync::Arc;
use clap::{Parser, Subcommand};

use gm_kernel::kernel::Kernel;
use gm_shared::errors::GitManagerError;

use crate::commands::{account, clone, config, dag, git, logs, setup, ssh, theme};

/// Git Multi-Account Manager — manage multiple GitHub, GitLab, Bitbucket,
/// and Azure DevOps accounts from a single unified CLI.
#[derive(Parser, Debug)]
#[command(
    name    = "git-zyrix",
    version = env!("CARGO_PKG_VERSION"),
    about   = "Multi-account Git manager — GitHub, GitLab, Bitbucket, Azure DevOps",
    long_about = "Manage SSH keys, repositories, and sync operations across multiple\n\
                  Git hosting accounts with a single, consistent interface.",
    propagate_version = true,
)]
pub struct Cli {
    /// Suppress the startup banner. Useful in scripts or piped output.
    #[arg(long, global = true)]
    pub no_banner: bool,

    /// Enable verbose tracing output (equivalent to RUST_LOG=debug).
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage Git hosting accounts (add, list, remove, set-default, test).
    ///
    /// Examples:
    ///   git-zyrix account add --alias work --platform github
    ///   git-zyrix account list
    ///   git-zyrix account set-default work
    #[command(alias = "acc")]
    Account(account::AccountCmd),

    /// Clone a repository from a managed account using its SSH key.
    ///
    /// Examples:
    ///   git-zyrix clone owner/repo --account work
    ///   git-zyrix clone owner/repo --account work --dest ~/projects/repo
    Clone(clone::CloneCmd),

    /// Git operations: pull, push, sync, status, commit.
    ///
    /// Examples:
    ///   git-zyrix git pull
    ///   git-zyrix git push --message "feat: add login"
    ///   git-zyrix git sync
    #[command(name = "git")]
    Git(git::GitCmd),

    /// SSH key operations: generate, test, list, add-to-agent.
    ///
    /// Examples:
    ///   git-zyrix ssh generate --account work
    ///   git-zyrix ssh test --account work
    ///   git-zyrix ssh list
    Ssh(ssh::SshCmd),

    /// Application configuration: get, set, list, reset.
    ///
    /// Examples:
    ///   git-zyrix config list
    ///   git-zyrix config get ssh.default_key_type
    ///   git-zyrix config set ssh.connect_timeout_ms 15000
    Config(config::ConfigCmd),

    /// View operation logs: recent operations, filter by type or account.
    ///
    /// Examples:
    ///   git-zyrix logs
    ///   git-zyrix logs --account work --limit 20
    Logs(logs::LogsCmd),

    /// Colour scheme management: list, switch, preview themes.
    ///
    /// Examples:
    ///   git-zyrix theme list
    ///   git-zyrix theme set jet_black
    ///   git-zyrix theme preview
    Theme(theme::ThemeCmd),

    /// Visualise the system architecture as a Directed Acyclic Graph.
    ///
    /// Examples:
    ///   zyrix dag crates    -- workspace crate dependency graph
    ///   zyrix dag workflow  -- clone_and_configure step DAG
    ///   zyrix dag events    -- domain event flow diagram
    #[command(name = "dag")]
    Dag(dag::DagArgs),

    /// One-command account setup: create account + configure auth + test.
    ///
    /// Interactive by default; all options available as flags for scripting.
    ///
    /// Examples:
    ///   git-zyrix setup                                          # interactive
    ///   git-zyrix setup --platform github --alias work \\        # non-interactive
    ///     --username shakamoses --email moses@example.com \\
    ///     --auth ssh --key-type ed25519 --add-to-agent
    Setup(setup::SetupCmd),

    /// Launch the interactive menu-driven session.
    ///
    /// Equivalent to running git-zyrix with no subcommand.
    #[command(alias = "i")]
    Interactive,
}

/// Routes the parsed CLI struct to the correct command handler and
/// manages the banner lifecycle.
pub async fn dispatch(cli: Cli, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
    match cli.command {
        Commands::Account(cmd) => account::handle(cmd, kernel).await,
        Commands::Clone(cmd)   => clone::handle(cmd, kernel).await,
        Commands::Git(cmd)     => git::handle(cmd, kernel).await,
        Commands::Ssh(cmd)     => ssh::handle(cmd, kernel).await,
        Commands::Setup(cmd)   => setup::handle(cmd, kernel).await,
        Commands::Config(cmd)  => config::handle(cmd, kernel).await,
        Commands::Logs(cmd)    => logs::handle(cmd, kernel).await,
        Commands::Theme(cmd)   => theme::handle(cmd, kernel).await,
        Commands::Dag(args)    => { dag::handle_dag_command(args); Ok(()) }
        Commands::Interactive   => {
            let handle = kernel
                .get::<crate::services::CliServicesHandle>()
                .ok_or_else(|| GitManagerError::Other(
                    "CliServicesHandle not registered — check apps/cli/main.rs bootstrap"
                        .to_string(),
                ))?;
            crate::ui::interactive::run_menu(handle).await
        }
    }
}

/// Returns true for commands whose output is intended for machine consumption,
/// where a banner would corrupt parsing by scripts.
#[allow(dead_code)]
fn is_machine_readable(cmd: &Commands) -> bool {
    matches!(
        cmd,
        Commands::Config(config::ConfigCmd {
            action: config::ConfigAction::Get { .. },
            ..
        })
    )
}