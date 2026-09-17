// crates/gm_interface_cli/src/ui/interactive/accounts.rs
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Main menu option [5]: Show all accounts.
//
// Loads all registered accounts via CliServices, renders them in a
// heavy-border table (matching v1's Rich-style layout), and waits for
// the user to press Enter before returning to the main menu.

use crate::services::CliServicesHandle;
use crate::ui::colors;
use super::common::{FlowResult, press_enter, print_accounts_table, section, spinner};

/// [5] Show all accounts — lists every registered account in a bordered table.
pub(super) async fn flow_show_accounts(handle: &CliServicesHandle) -> FlowResult {
    section("All Accounts");

    let pb = spinner("Loading accounts…");
    let accounts = handle.services().list_accounts(None).await?;
    pb.finish_and_clear();

    if accounts.is_empty() {
        println!("{} No accounts configured yet.", colors::info_prefix());
        println!();
        println!("  Use option {}  to add an account and generate an SSH key.",
            colors::bold("[7]"));
    } else {
        print_accounts_table(&accounts);
        println!("  {} account(s) registered.", accounts.len());
    }

    press_enter();
    Ok(())
}