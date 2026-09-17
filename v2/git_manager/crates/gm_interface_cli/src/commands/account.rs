// crates/gm_interface_cli/src/commands/account.rs

use std::sync::Arc;
use clap::{Parser, Subcommand};
use uuid::Uuid;
use gm_kernel::kernel::Kernel;
use gm_shared::errors::GitManagerError;
use gm_shared::models::platform::PlatformType;
use gm_ports::inbound::commands::AddAccountCommand;

use crate::services::CliServicesHandle;
use crate::ui::{colors::*, tables};

#[derive(Parser, Debug)]
pub struct AccountCmd {
    #[command(subcommand)]
    pub action: AccountAction,
}

#[derive(Subcommand, Debug)]
pub enum AccountAction {
    /// Add a new Git hosting account.
    Add {
        #[arg(long, short = 'a')] alias:       String,
        #[arg(long, short = 'p')] platform:    String,
        #[arg(long, short = 'u')] username:    String,
        #[arg(long, short = 'e')] email:       String,
        #[arg(long, short = 'm', default_value = "ssh")] auth_method: String,
        /// Mark as the default for its platform immediately.
        #[arg(long)] set_default: bool,
    },
    /// List all managed accounts.
    List {
        #[arg(long, short = 'p')] platform: Option<String>,
    },
    /// Show details for one account by alias.
    Show { alias: String },
    /// Remove an account and its SSH keys.
    Remove {
        alias: String,
        /// Skip the confirmation prompt.
        #[arg(long, short = 'y')] yes: bool,
    },
    /// Make an account the platform default.
    SetDefault {
        alias: String,
        #[arg(long, short = 'p')] platform: Option<String>,
    },
    /// Test the SSH connection for an account right now.
    Test { alias: String },
}

pub async fn handle(cmd: AccountCmd, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
    let svc = kernel
        .get::<CliServicesHandle>()
        .ok_or_else(|| GitManagerError::Other("CLI services not initialised — check bootstrap".to_string()))?;

    match cmd.action {
        AccountAction::Add { alias, platform, username, email, auth_method, set_default } => {
            let platform_type = parse_platform(&platform)?;
            let platform_id   = platform_uuid_for(&platform_type);
            let command = AddAccountCommand {
                alias:       alias.clone(),
                platform_id,
                username:    username.clone(),
                email:       email.clone(),
                auth_method: auth_method.clone(),
            };
            let account = svc.services().add_account(command).await?;
            if set_default {
                svc.services().set_default_account(account.uuid, account.platform_id).await?;
            }
            println!("{} Account {} added  ({}  {})",
                success_prefix(), bold(&alias), platform_type, auth_method);
            if set_default {
                println!("  {} Set as default {} account", check_mark(), bold(&platform_type.to_string()));
            }
            println!();
            println!("  Next: {}", dim(&format!("git-zyrix ssh generate --account {alias}")));
            Ok(())
        }
        AccountAction::List { platform } => {
            let platform_id = platform.as_deref().map(|p| {
                parse_platform(p).map(|pt| platform_uuid_for(&pt))
            }).transpose()?;
            let accounts = svc.services().list_accounts(platform_id).await?;
            if accounts.is_empty() {
                println!("{} No accounts yet. Add one:", info_prefix());
                println!("  {}", dim("git-zyrix account add --alias work --platform github --username you --email you@example.com"));
            } else {
                tables::print_accounts(&accounts);
            }
            Ok(())
        }
        AccountAction::Show { alias } => {
            let account = svc.services()
                .get_account_by_alias(&alias, None)
                .await?
                .ok_or_else(|| GitManagerError::Other(format!("No account with alias '{alias}'")))?;
            println!();
            println!("  {:<18} {}", bold("Alias:"),    bold(&account.alias));
            println!("  {:<18} {}", bold("UUID:"),     dim(&account.uuid.to_string()));
            println!("  {:<18} {}", bold("Username:"), account.username);
            println!("  {:<18} {}", bold("Email:"),    account.email);
            println!("  {:<18} {}", bold("Status:"),   colored_status(&account.status.to_string()));
            println!("  {:<18} {}", bold("Default:"),  if account.is_default { green("yes") } else { dim("no").to_string() });
            println!();
            Ok(())
        }
        AccountAction::Remove { alias, yes } => {
            if !yes {
                use dialoguer::Confirm;
                let confirmed = Confirm::new()
                    .with_prompt(format!("Remove account '{alias}' and all its SSH keys?"))
                    .default(false)
                    .interact()
                    .map_err(|e| GitManagerError::Other(format!("prompt: {e}")))?;
                if !confirmed {
                    println!("{} Cancelled.", info_prefix());
                    return Ok(());
                }
            }
            let account = svc.services()
                .get_account_by_alias(&alias, None)
                .await?
                .ok_or_else(|| GitManagerError::Other(format!("No account '{alias}'")))?;
            svc.services().remove_account(account.uuid).await?;
            println!("{} Account '{}' removed.", success_prefix(), bold(&alias));
            Ok(())
        }
        AccountAction::SetDefault { alias, platform } => {
            let platform_id = platform.as_deref().map(|p| {
                parse_platform(p).map(|pt| platform_uuid_for(&pt))
            }).transpose()?;
            let account = svc.services()
                .get_account_by_alias(&alias, platform_id)
                .await?
                .ok_or_else(|| GitManagerError::Other(format!("No account '{alias}'")))?;
            svc.services().set_default_account(account.uuid, account.platform_id).await?;
            println!("{} '{}' is now the default account.", success_prefix(), bold(&alias));
            Ok(())
        }
        AccountAction::Test { alias } => {
            use indicatif::{ProgressBar, ProgressStyle};
            use std::time::Duration;
            use gm_ports::inbound::commands::TestSshConnectionCommand;

            let account = svc.services()
                .get_account_by_alias(&alias, None)
                .await?
                .ok_or_else(|| GitManagerError::Other(format!("No account '{alias}'")))?;

            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .expect("account: hardcoded spinner template must be valid"));
            pb.set_message(format!("Testing SSH connection for '{alias}'…"));
            pb.enable_steady_tick(Duration::from_millis(80));

            let result = svc.services().test_ssh_connection(TestSshConnectionCommand {
                account_uuid: account.uuid,
                timeout_ms:   Some(10_000),
            }).await?;
            pb.finish_and_clear();

            if result.success {
                let who = result.username.as_deref().unwrap_or("(unknown)");
                println!("{} SSH connection succeeded  (authenticated as {})", success_prefix(), bold(who));
            } else {
                println!("{} SSH connection failed: {}", error_prefix(),
                    result.error.as_deref().unwrap_or("unknown error"));
                println!("  Hint: ensure the public key is registered on the platform.");
            }
            Ok(())
        }
    }
}

pub(crate) fn parse_platform(s: &str) -> Result<PlatformType, GitManagerError> {
    match s.to_lowercase().as_str() {
        "github"                          => Ok(PlatformType::GitHub),
        "gitlab"                          => Ok(PlatformType::GitLab),
        "bitbucket"                       => Ok(PlatformType::Bitbucket),
        "azure-devops" | "azure_devops"   => Ok(PlatformType::AzureDevOps),
        "sourceforge"                     => Ok(PlatformType::SourceForge),
        "self-hosted" | "self_hosted"     => Ok(PlatformType::SelfHosted("custom".to_string())),
        "cloud-storage" | "cloud_storage" => Ok(PlatformType::CloudStorage("generic".to_string())),
        "local-path" | "local_path"       => Ok(PlatformType::LocalPath),
        "custom"                          => Ok(PlatformType::Custom("custom".to_string())),
        other => Err(GitManagerError::Other(
            format!("Unknown platform '{other}'. Choose: github, gitlab, bitbucket, azure-devops, sourceforge, self-hosted, cloud-storage, local-path, custom"))),
    }
}

/// Returns the fixed platform UUID seeded in 20240101000001_seed_platforms.sql.
pub(crate) fn platform_uuid_for(pt: &PlatformType) -> Uuid {
    match pt {
        PlatformType::GitHub            => Uuid::parse_str("00000000-0001-0000-0000-000000000001").unwrap(),
        PlatformType::GitLab            => Uuid::parse_str("00000000-0002-0000-0000-000000000001").unwrap(),
        PlatformType::Bitbucket         => Uuid::parse_str("00000000-0003-0000-0000-000000000001").unwrap(),
        PlatformType::AzureDevOps       => Uuid::parse_str("00000000-0004-0000-0000-000000000001").unwrap(),
        PlatformType::SourceForge       => Uuid::parse_str("00000000-0005-0000-0000-000000000001").unwrap(),
        PlatformType::SelfHosted(_)     => Uuid::parse_str("00000000-0006-0000-0000-000000000001").unwrap(),
        PlatformType::CloudStorage(_)   => Uuid::parse_str("00000000-0007-0000-0000-000000000001").unwrap(),
        PlatformType::LocalPath         => Uuid::parse_str("00000000-0008-0000-0000-000000000001").unwrap(),
        PlatformType::Custom(_)         => Uuid::parse_str("00000000-0009-0000-0000-000000000001").unwrap(),
    }
}