// crates/gm_interface_cli/src/commands/config.rs
// Application configuration management.

use std::sync::Arc;
use clap::{Parser, Subcommand};
use gm_kernel::kernel::Kernel;
use gm_shared::errors::GitManagerError;

use crate::services::CliServicesHandle;
use crate::ui::colors::*;

#[derive(Parser, Debug)]
pub struct ConfigCmd {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// List all configuration values.
    List,
    /// Get a single configuration value (machine-readable output).
    Get { key: String },
    /// Set a configuration value.
    Set { key: String, value: String },
    /// Reset all configuration to defaults.
    Reset {
        /// Skip the confirmation prompt.
        #[arg(long, short = 'y')] yes: bool,
    },
}

pub async fn handle(cmd: ConfigCmd, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
    let svc = kernel.get::<CliServicesHandle>()
        .ok_or_else(|| GitManagerError::Other("CLI services not initialised".to_string()))?;

    match cmd.action {
        ConfigAction::List => {
            let entries = svc.services().config_list().await?;
            println!();
            for (key, value) in &entries {
                println!("  {:<40} {}", bold(key), value);
            }
            println!();
            Ok(())
        }
        ConfigAction::Get { key } => {
            // Machine-readable: print only the value, no decoration.
            match svc.services().config_get(&key).await? {
                Some(value) => println!("{value}"),
                None        => return Err(GitManagerError::Other(format!("Key '{key}' not found"))),
            }
            Ok(())
        }
        ConfigAction::Set { key, value } => {
            svc.services().config_set(&key, &value).await?;
            println!("{} {} = {}", success_prefix(), bold(&key), value);
            Ok(())
        }
        ConfigAction::Reset { yes } => {
            if !yes {
                use dialoguer::Confirm;
                let ok = Confirm::new()
                    .with_prompt("Reset all configuration to defaults?")
                    .default(false)
                    .interact()
                    .map_err(|e| GitManagerError::Other(e.to_string()))?;
                if !ok { println!("{} Cancelled.", info_prefix()); return Ok(()); }
            }
            // Set all keys to their defaults.
            let defaults = [
                ("ssh.default_key_type",   "ed25519"),
                ("ssh.connect_timeout_ms", "10000"),
                ("ssh.auto_add_to_agent",  "true"),
                ("git.max_concurrent_ops", "4"),
                ("ui.show_banner",         "true"),
            ];
            for (k, v) in &defaults {
                svc.services().config_set(k, v).await?;
            }
            println!("{} Configuration reset to defaults.", success_prefix());
            Ok(())
        }
    }
}