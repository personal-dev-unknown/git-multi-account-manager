use std::path::PathBuf;
use std::sync::Arc;
use clap::Parser;
use gm_kernel::Kernel;
use gm_kernel::security::kill_current_child;
use gm_shared::errors::GitManagerError;
use gm_ports::inbound::commands::CloneRepositoryCommand;
use crate::services::CliServicesHandle;
use crate::ui::colors::*;

#[derive(Parser, Debug)]
pub struct CloneCmd {
    pub full_name: String,
    #[arg(long, short = 'a')]
    pub account: Option<String>,
    #[arg(long, short = 'd')]
    pub dest: Option<PathBuf>,
    #[arg(long, default_value_t = 0)]
    pub depth: u32,
}

pub async fn handle(cmd: CloneCmd, kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
    let svc = kernel.get::<CliServicesHandle>()
        .ok_or_else(|| GitManagerError::Other("CLI services not initialised".to_string()))?;

    let account_uuid = if let Some(alias) = &cmd.account {
        let acc = svc.services().get_account_by_alias(alias, None).await?
            .ok_or_else(|| GitManagerError::Other(format!("No account '{}'", alias)))?;
        Some(acc.uuid)
    } else { None };

    let repo_name = cmd.full_name.split('/').next_back().unwrap_or(&cmd.full_name);
    let dest_path = cmd.dest.unwrap_or_else(|| PathBuf::from(repo_name));

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(indicatif::ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .expect("valid template"));
    pb.set_message(format!("Cloning {}…", cmd.full_name));
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let pb_arc = Arc::new(pb);
    let pb_clone = pb_arc.clone();

    let clone_fut = svc.services().clone_repository(CloneRepositoryCommand {
        account_uuid,
        url: cmd.full_name.clone(),
        destination: dest_path.to_str().map(|s| s.to_string()),
        branch: None,
        depth: cmd.depth,
        ..Default::default()
    });

    let result = tokio::select! {
        result = clone_fut => result,
        _ = tokio::signal::ctrl_c() => {
            pb_clone.finish_and_clear();
            println!();
            println!("{} Cancelling clone…", yellow("⚠"));
            kill_current_child();
            let _ = std::fs::remove_dir_all(&dest_path);
            return Err(GitManagerError::Other("Clone cancelled by user".into()));
        }
    };

    let pb = Arc::into_inner(pb_arc).expect("only one ref");
    pb.finish_and_clear();

    match result {
        Ok(_repo) => {
            let account_tag = if let Some(ref alias) = cmd.account {
                format!(" for account '{}'", alias)
            } else {
                String::new()
            };
            println!("{} Cloned {} to {}{}", success_prefix(), bold(&cmd.full_name),
                bold(&dest_path.display().to_string()), dim(&account_tag));
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.to_lowercase().contains("cancelled") {
                let _ = std::fs::remove_dir_all(&dest_path);
            }
            println!("{} Clone failed: {}", error_prefix(), e);
        }
    }

    Ok(())
}
