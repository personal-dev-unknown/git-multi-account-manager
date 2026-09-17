// crates/gm_interface_cli/src/commands/logs.rs
// View operation history logs.

use std::sync::Arc;
use clap::Parser;
use gm_kernel::kernel::Kernel;
use gm_shared::errors::GitManagerError;

use crate::services::CliServicesHandle;
use crate::ui::{colors::*, tables};

#[derive(Parser, Debug)]
pub struct LogsCmd {
    /// Filter by account alias.
    #[arg(long)] account: Option<String>,
    /// Maximum number of entries to show (default 20).
    #[arg(long, short = 'n', default_value_t = 20)] limit: u32,
}

pub async fn handle(cmd: LogsCmd, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
    let svc = kernel.get::<CliServicesHandle>()
        .ok_or_else(|| GitManagerError::Other("CLI services not initialised".to_string()))?;

    let account_uuid = if let Some(ref alias) = cmd.account {
        svc.services().get_account_by_alias(alias, None).await?
            .map(|a| a.uuid)
    } else {
        None
    };

    let entries = svc.services().list_recent_operations(account_uuid, cmd.limit).await?;

    if entries.is_empty() {
        println!("{} No operations recorded yet.", info_prefix());
    } else {
        tables::print_operation_logs(&entries);
    }
    Ok(())
}