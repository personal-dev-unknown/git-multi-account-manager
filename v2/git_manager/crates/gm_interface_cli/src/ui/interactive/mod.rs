// crates/gm_interface_cli/src/ui/interactive/mod.rs
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Public entry point for the interactive TUI session.
//
// run_menu() is called by two code paths:
//   • plugin.rs::run()        — when no subcommand is given and the user picks
//                               "Interactive" at the mode selector
//   • app.rs Commands::Interactive — `git-zyrix interactive` or alias `i`
//
// ── Panel design — matching v1 Python CLI ─────────────────────────────────────
// The main menu is drawn as a full-width bordered panel (╭─ Main Menu ─╮),
// filling the terminal width up to 120 columns.  Section headers
// (═══ Repository Operations ═══) are rendered inside the panel as static text.
// After the panel, a v1-style typed number prompt collects the user's choice.
//
// ── Ctrl+C contract ───────────────────────────────────────────────────────────
// Every prompt wrapper in common.rs maps IO errors to FlowError::Cancelled.
// run_menu() catches Cancelled at two levels:
//
//   FLOW LEVEL  — user presses Ctrl+C inside a sub-screen
//                 ⚠  Operation cancelled by user
//                 → loop back to main menu (no exit)
//
//   MENU LEVEL  — user presses Ctrl+C / Esc at the main menu itself
//                 ⚠  Operation cancelled by user
//                 → exit the interactive session cleanly
//
// ── Module layout ─────────────────────────────────────────────────────────────
// mod accounts  — flow_show_accounts()
// mod common    — FlowError, prompt wrappers, shared helpers, table renderers
// mod git_ops   — flow_git_ops() + pull / push / status sub-flows
// mod repos     — flow_clone(), flow_check_repo(), flow_setup_repo()
// mod ssh       — flow_test_ssh(), flow_manage_keys(), generate, pat

mod accounts;
pub(super) mod common;   // pub(super) so app.rs / plugin.rs can't see FlowError
mod git_ops;
mod repos;
mod settings;
mod ssh;

use std::sync::Arc;

use gm_shared::errors::GitManagerError;

use crate::services::CliServicesHandle;
use crate::ui::colors;
use common::{FlowError, press_enter, section, typed_menu};

// ─────────────────────────────────────────────────────────────────────────────
// Original helpers — kept exactly as they were (callers import from this path)
// ─────────────────────────────────────────────────────────────────────────────

/// Presents a multi-select list and returns the indices of chosen items.
pub fn multi_select(label: &str, options: &[String]) -> Result<Vec<usize>, GitManagerError> {
    use dialoguer::MultiSelect;
    MultiSelect::new()
        .with_prompt(label)
        .items(options)
        .interact()
        .map_err(|e| GitManagerError::Other(format!("multi-select: {e}")))
}

/// Fuzzy-search a list and return the selected index.
pub fn fuzzy_select(label: &str, options: &[String]) -> Result<Option<usize>, GitManagerError> {
    use dialoguer::FuzzySelect;
    FuzzySelect::new()
        .with_prompt(label)
        .items(options)
        .interact_opt()
        .map_err(|e| GitManagerError::Other(format!("fuzzy-select: {e}")))
}

// ─────────────────────────────────────────────────────────────────────────────
// Panel renderer
// ─────────────────────────────────────────────────────────────────────────────

/// Detects the terminal width (capped at 120, minimum 72).
fn term_width() -> usize {
    use terminal_size::{terminal_size, Width};
    terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80)
        .min(120)
        .max(72)
}

/// Draws the v1-style bordered panel containing the main menu.
/// Border colour: cyan (consistent with section headers throughout the app).
fn draw_main_panel() {
    let w      = term_width();
    let inner  = w.saturating_sub(2);   // width between │ │
    let title  = " Main Menu ";

    // ── Top border with centred title ──
    let title_len  = title.len();
    let left_dashes  = (inner.saturating_sub(title_len)) / 2;
    let right_dashes = inner.saturating_sub(left_dashes + title_len);
    let top = format!(
        "╭{}{}{}╮",
        "─".repeat(left_dashes),
        colors::bold(title),
        "─".repeat(right_dashes),
    );
    println!("{}", colors::cyan(&top));

    // ── Helper: pad one line to fill the panel ──
    let line = |text: &str| {
        // Compute printable width (strip ANSI codes for the pad calculation)
        // Simple heuristic: the raw text length without ANSI overhead
        let visible: usize = text.chars().filter(|c| !c.is_control()).count();
        let pad = inner.saturating_sub(visible + 1); // +1 for leading space
        println!("{}  {}{} {}",
            colors::cyan("│"),
            text,
            " ".repeat(pad),
            colors::cyan("│"));
    };

    // ── Content ──
    line("");

    // Section: Repository Operations
    let repo_hdr = format!("{} Repository Operations {}",
        colors::cyan("  ═══"), colors::cyan("═══  "));
    line(&repo_hdr);

    line(&format!("  {}  Clone a repository",               colors::yellow("[1]")));
    line(&format!("  {}  Check current repository account", colors::yellow("[2]")));
    line(&format!("  {}  Git operations (pull/push/status/dry-run/detect)", colors::yellow("[3]")));
    line(&format!("  {}  Set up repository for specific account", colors::yellow("[4]")));

    line("");

    // Section: Account Management
    let acc_hdr = format!("{} Account Management {}",
        colors::cyan("  ═══"), colors::cyan("═══  "));
    line(&acc_hdr);

    line(&format!("  {}  Show all accounts",                colors::yellow("[5]")));
    line(&format!("  {}  Test SSH connections",             colors::yellow("[6]")));
    line(&format!("  {}  Git auth (keys/PATs/OAuth)",   colors::yellow("[7]")));

    line("");
    line(&format!("  {}  Settings", colors::yellow("[8]")));
    line(&format!("  {}  Help / Documentation", colors::yellow("[9]")));
    line(&format!("  {}  Exit", colors::dim("[0]")));
    line("");

    // ── Bottom border ──
    let bot = format!("╰{}╯", "─".repeat(inner));
    println!("{}", colors::cyan(&bot));
}

// ─────────────────────────────────────────────────────────────────────────────
// Main loop
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// Help panel
// ─────────────────────────────────────────────────────────────────────────────

pub(super) fn print_help() {
    section("Help & Documentation");
    println!();
    println!("  {}  Quick Start", colors::bold("[1]"));
    println!("      Use {} to set up your first account.", colors::yellow("git-zyrix setup"));
    println!("      Follow the interactive wizard to create an account,");
    println!("      generate SSH keys, and add them to the SSH agent.");
    println!();
    println!("  {}  SSH Architecture", colors::bold("[2]"));
    println!("      Each account has an isolated Host entry in");
    println!("      {}.", colors::dim("~/.ssh/gitzyrix/config"));
    println!("      The {} directive in {} includes it.", colors::dim("Include"), colors::dim("~/.ssh/config"));
    println!("      {} ensures only that key is tried.", colors::dim("IdentitiesOnly yes"));
    println!();
    println!("  {}  Clone Workflow", colors::bold("[3]"));
    println!("      {} → Pick account → Enter owner/repo", colors::yellow("[1] Clone"));
    println!("      The URL is rewritten with the account's SSH host alias.");
    println!();
    println!("  {}  All CLI Commands", colors::bold("[4]"));
    println!("      See SYSTEM_GUIDE.md or run {} --help", colors::yellow("git-zyrix"));
    println!();
    press_enter();
}

/// Interactive session entry point.
/// Loops until the user selects [8] Exit or Ctrl+C at the main menu.
pub async fn run_menu(handle: Arc<CliServicesHandle>) -> Result<(), GitManagerError> {
    loop {
        println!();
        draw_main_panel();
        println!();

        // v1-style typed number input (10 options, default = 10 for Exit)
        let sel = match typed_menu(10, 10) {
            Ok(idx) => idx,
            Err(FlowError::Cancelled) => {
                // Ctrl+C / Esc at the main menu → exit cleanly
                println!();
                println!("  {}  Operation cancelled by user", colors::yellow("⚠"));
                println!();
                return Ok(());
            }
            Err(FlowError::Service(e)) => return Err(e),
        };

        println!();

        // Dispatch to the chosen flow.
        // Each flow function returns Result<(), FlowError>.
        let result: Result<(), FlowError> = match sel {
            0 => repos::flow_clone(&handle).await,
            1 => repos::flow_check_repo(&handle).await,
            2 => git_ops::flow_git_ops(&handle).await,
            3 => repos::flow_setup_repo(&handle).await,
            4 => accounts::flow_show_accounts(&handle).await,
            5 => ssh::flow_test_ssh(&handle).await,
            6 => ssh::flow_manage_keys(&handle).await,
            7 => settings::flow_settings(),
            8 => {
                // [9] Help / Documentation
                print_help();
                Ok(())
            }
            9 => {
                // [0] Exit
                println!();
                println!("  Goodbye!\n");
                return Ok(());
            }
            _ => unreachable!(),
        };

        // ── Error handling — colour and message depend on the failure type ──
        match result {
            Ok(()) => {
                // Flow completed normally — redraw main menu on next iteration.
            }
            Err(FlowError::Cancelled) => {
                // Ctrl+C / Esc inside a sub-screen.
                // Match v1's exact wording and warning icon.
                println!();
                println!("  {}  Operation cancelled by user", colors::yellow("⚠"));
                // No press_enter here — loop immediately so the user sees the
                // main menu again without an extra keypress.
            }
            Err(FlowError::Service(e)) => {
                // Unexpected domain/service error — show in red and wait.
                println!("{} {}", colors::error_prefix(), e);
                press_enter();
            }
        }
    }
}