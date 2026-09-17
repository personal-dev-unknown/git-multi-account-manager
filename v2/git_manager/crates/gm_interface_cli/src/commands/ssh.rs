// crates/gm_interface_cli/src/commands/ssh.rs
// SSH key management: generate, test, list, add-to-agent.

use std::sync::Arc;
use clap::{Parser, Subcommand};
use uuid::Uuid;
use gm_kernel::kernel::Kernel;
use gm_shared::errors::GitManagerError;
use gm_ports::inbound::commands::{GenerateSshKeyCommand, TestSshConnectionCommand};

use crate::services::{CliServicesHandle, SshValidationResultDto};
use crate::ui::{colors::*, tables};

#[derive(Parser, Debug)]
pub struct SshCmd {
    #[command(subcommand)]
    pub action: SshAction,
}

#[derive(Subcommand, Debug)]
pub enum SshAction {
    /// Generate a new SSH key pair for an account.
    Generate {
        /// Account alias to generate the key for.
        #[arg(long)] account: String,
        /// Key type: ed25519 (recommended) or rsa.
        #[arg(long, default_value = "ed25519")] key_type: String,
        /// Optional key passphrase (leave blank for no passphrase).
        #[arg(long)] passphrase: Option<String>,
        /// Add the key to the SSH agent immediately after generation.
        #[arg(long)] add_to_agent: bool,
    },
    /// List all SSH keys, optionally filtered by account.
    List {
        #[arg(long)] account: Option<String>,
    },
    /// Test the SSH connection for an account.
    Test {
        #[arg(long)] account: String,
        /// Connection timeout in milliseconds.
        #[arg(long, default_value_t = 10_000)] timeout_ms: u32,
    },
    /// Add an existing key to the SSH agent.
    AddToAgent {
        /// UUID of the SSH key to add.
        #[arg(long)] key: String,
        #[arg(long)] passphrase: Option<String>,
    },
    /// Run full SSH setup validation for an account.
    Validate {
        /// Account alias to validate.
        #[arg(long)] account: String,
        /// Connection timeout in milliseconds.
        #[arg(long, default_value_t = 10_000)] timeout_ms: u32,
    },
}

pub async fn handle(cmd: SshCmd, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
    let svc = kernel.get::<CliServicesHandle>()
        .ok_or_else(|| GitManagerError::Other("CLI services not initialised".to_string()))?;

    match cmd.action {
        SshAction::Generate { account, key_type, passphrase, add_to_agent } => {
            let acc = svc.services().get_account_by_alias(&account, None).await?
                .ok_or_else(|| GitManagerError::Other(format!("No account '{account}'")))?;

            use indicatif::{ProgressBar, ProgressStyle};
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")
                .expect("ssh: hardcoded spinner template must be valid"));
            pb.set_message(format!("Generating {key_type} key for '{account}'…"));
            pb.enable_steady_tick(std::time::Duration::from_millis(80));

            let key = svc.services().generate_ssh_key(GenerateSshKeyCommand {
                account_uuid: acc.uuid,
                key_type:     key_type.clone(),
                comment:      Some(acc.email.clone()),
                passphrase,
                add_to_agent,
            }).await?;
            pb.finish_and_clear();

            println!("{} SSH key generated for '{}'", success_prefix(), bold(&account));
            println!("  Type:        {}", key_type);
            println!("  Fingerprint: {}", bold(&key.fingerprint));
            println!("  Public key:  {}", dim(&key.public_key));
            println!();
            println!("  {} Copy the public key above and add it to your {} account:",
                info_prefix(), bold(&account));
            println!("  {}", dim(&format!("https://github.com/settings/ssh/new  (or your platform's SSH settings)")));
            Ok(())
        }
        SshAction::List { account } => {
            let account_uuid = if let Some(ref alias) = account {
                let acc = svc.services().get_account_by_alias(alias, None).await?
                    .ok_or_else(|| GitManagerError::Other(format!("No account '{alias}'")))?;
                acc.uuid
            } else {
                Uuid::nil() // sentinel: list all (handled by CliServices impl)
            };
            let keys = svc.services().list_ssh_keys(account_uuid).await?;
            if keys.is_empty() {
                println!("{} No SSH keys found.", info_prefix());
                if let Some(a) = account {
                    println!("  Generate one: {}", dim(&format!("git-zyrix ssh generate --account {a}")));
                }
            } else {
                tables::print_ssh_keys(&keys);
            }
            Ok(())
        }
        SshAction::Test { account, timeout_ms } => {
            let acc = svc.services().get_account_by_alias(&account, None).await?
                .ok_or_else(|| GitManagerError::Other(format!("No account '{account}'")))?;
            let result = svc.services().test_ssh_connection(TestSshConnectionCommand {
                account_uuid: acc.uuid, timeout_ms: Some(timeout_ms)
            }).await?;
            if result.success {
                println!("{} SSH test passed for '{}' (authenticated as {})",
                    success_prefix(), bold(&account),
                    bold(result.username.as_deref().unwrap_or("(unknown)")));
            } else {
                println!("{} SSH test failed for '{}': {}",
                    error_prefix(), bold(&account),
                    result.error.as_deref().unwrap_or("connection refused"));
            }
            Ok(())
        }
        SshAction::Validate { account, timeout_ms } => {
            let acc = svc.services().get_account_by_alias(&account, None).await?
                .ok_or_else(|| GitManagerError::Other(format!("No account '{account}'")))?;
            let result = svc.services().validate_ssh_setup(acc.uuid, timeout_ms).await?;
            print_validation_result(&result, &account);
            Ok(())
        }
        SshAction::AddToAgent { key, passphrase } => {
            let key_uuid = Uuid::parse_str(&key)
                .map_err(|_| GitManagerError::Other(format!("Invalid key UUID: {key}")))?;
            svc.services().add_key_to_agent(key_uuid, passphrase.as_deref()).await?;
            println!("{} Key {} added to SSH agent.", success_prefix(), dim(&key));
            Ok(())
        }
    }
}

fn print_validation_result(result: &SshValidationResultDto, account: &str) {
    let passed_count = result.checks.iter().filter(|c| c.passed).count();
    let total = result.checks.len();

    println!("\n{} SSH Setup Validation for '{}'", bold("═ SSH VALIDATION ═"), bold(account));
    println!("  Checks: {}/{} passed", passed_count, total);
    println!();

    for check in &result.checks {
        let icon = if check.passed {
            format!("{}", green("✔"))
        } else {
            format!("{}", red("✘"))
        };
        let status = if check.passed {
            format!("{}", green("PASS"))
        } else {
            format!("{}", red("FAIL"))
        };
        println!("  {icon} [{status}] {}", bold(&check.name));
        println!("         {}", dim(&check.detail));
    }

    if result.all_passed {
        println!();
        println!("  {} All checks passed — SSH setup is complete for '{}'",
            success_prefix(), bold(account));
    } else {
        println!();
        println!("  {} Some checks failed — run 'git-zyrix ssh generate --account {}' to fix",
            "⚠", account);
    }
}