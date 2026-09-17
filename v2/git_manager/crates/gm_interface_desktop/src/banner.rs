// crates/gm_interface_desktop/src/banner.rs
//
// Caller:  apps/desktop/src/main.rs → main() calls print_banner() before bootstrap()
// Purpose: Renders the ZYRIX application banner on desktop app startup.
//          Mirrors the structure and constants of gm_interface_cli/src/ui/banner.rs
//          exactly — same LOGO_LINES, same BOX_INNER_WIDTH, same box helpers.
//          Desktop-specific: shows "Desktop Interface" and the Tauri window title.

use std::io::{self, Write};

/// The minimum terminal/column width needed to render the full ASCII art logo.
const FULL_BANNER_MIN_WIDTH: usize = 60;

/// Reads the terminal width, falling back to COLUMNS env var, then 80.
fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80)
}

/// ZYRIX ASCII art — 6 rows.
/// Identical to gm_interface_cli/src/ui/banner.rs LOGO_LINES.
const LOGO_LINES: &[&str] = &[
    r"████████╗  ██╗   ██╗  ██████╗  ██╗  ██╗  ██╗",
    r"╚══════██╗ ╚██╗ ██╔╝  ██╔══██╗ ██║  ╚██╗██╔╝",
    r"      ██╔╝  ╚████╔╝   ██████╔╝ ██║   ╚███╔╝ ",
    r"    ██╔╝     ╚██╔╝    ██╔══██╗ ██║   ██╔██╗ ",
    r"  ██╔╝        ██║     ██║  ██║ ██║  ██╔╝ ██╗",
    r"████████╗     ╚═╝     ╚═╝  ╚═╝ ╚═╝  ╚═╝  ╚═╝",
];

/// Inner content width of the box — identical to CLI banner.
const BOX_INNER_WIDTH: usize = 50;

/// Prints a centered content line inside the banner box with ║ borders.
fn box_line(content: &str, display_len: usize) {
    let padding   = BOX_INNER_WIDTH.saturating_sub(display_len);
    let left_pad  = padding / 2;
    let right_pad = padding - left_pad;
    println!("  ║ {}{}{} ║", " ".repeat(left_pad), content, " ".repeat(right_pad));
}

/// Prints a blank spacer line inside the banner box.
fn box_empty_line() {
    println!("  ║{}║", " ".repeat(BOX_INNER_WIDTH + 2));
}

/// Renders the full-width banner with ZYRIX logo and desktop-specific lines.
fn print_full_banner(version: &str) {
    let top_border = format!("  ╔{}╗", "═".repeat(BOX_INNER_WIDTH + 2));
    let bot_border = format!("  ╚{}╝", "═".repeat(BOX_INNER_WIDTH + 2));

    println!("{}", top_border);
    box_empty_line();

    for line in LOGO_LINES {
        let logo_w = line.chars().count();
        let pad    = BOX_INNER_WIDTH.saturating_sub(logo_w);
        let left   = pad / 2;
        let right  = pad - left;
        println!("  ║ {}{}{} ║", " ".repeat(left), line, " ".repeat(right));
    }

    box_empty_line();

    let subtitle = format!("ZYRIX  ·  v{}  ·  Desktop Interface", version);
    box_line(&subtitle, subtitle.len());

    box_empty_line();

    let platforms = "GitHub  ·  GitLab  ·  Bitbucket  ·  Azure DevOps";
    box_line(platforms, platforms.len());

    box_empty_line();
    println!("{}", bot_border);
}

/// Compact one-line fallback for narrow terminals.
fn print_compact_banner(version: &str) {
    println!("  ZYRIX v{}  ·  Desktop Interface", version);
    println!("  {}", "─".repeat(60));
}

/// Public entry point — call once in `main()` before `bootstrap()`.
///
/// Prints to stdout via `println!` (not tracing) so the banner always
/// appears regardless of log level and before the tracing subscriber is set up.
pub fn print_banner(version: &str) {
    println!();

    if terminal_width() >= FULL_BANNER_MIN_WIDTH {
        print_full_banner(version);
    } else {
        print_compact_banner(version);
    }

    println!();

    let _ = io::stdout().flush();
}
