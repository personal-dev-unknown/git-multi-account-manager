// crates/gm_interface_cli/src/ui/banner.rs
//
// Caller:  app.rs → dispatch() calls print_banner() before any command
// Purpose: Renders the application banner on CLI startup.
//          Respects NO_COLOR env var and terminal width.
//          Falls back gracefully to a compact banner on narrow terminals.

use owo_colors::OwoColorize;
use std::io::{self, Write};

/// The minimum terminal width needed to render the full ASCII art logo.
/// If the terminal is narrower than this, we render a compact banner instead.
const FULL_BANNER_MIN_WIDTH: usize = 60;

/// Reads the terminal width using the `terminal_size` crate,
/// falling back to the COLUMNS env var, then 80.
fn terminal_width() -> usize {
    use terminal_size::{terminal_size, Width};
    terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or_else(|| {
            std::env::var("COLUMNS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(80)
        })
}

/// Returns true if color output should be rendered.
/// Respects the NO_COLOR standard (https://no-color.org/).
fn colors_enabled() -> bool {
    std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map_or(true, |t| t != "dumb")
        && atty::is(atty::Stream::Stdout)
}

/// ZYRIX ASCII art — 6 rows, each exactly 44 visible chars wide.
const LOGO_LINES: &[&str] = &[
    r"████████╗  ██╗   ██╗  ██████╗  ██╗  ██╗  ██╗",
    r"╚══════██╗ ╚██╗ ██╔╝  ██╔══██╗ ██║  ╚██╗██╔╝",
    r"      ██╔╝  ╚████╔╝   ██████╔╝ ██║   ╚███╔╝ ",
    r"    ██╔╝     ╚██╔╝    ██╔══██╗ ██║   ██╔██╗ ",
    r"  ██╔╝        ██║     ██║  ██║ ██║  ██╔╝ ██╗",
    r"████████╗     ╚═╝     ╚═╝  ╚═╝ ╚═╝  ╚═╝  ╚═╝",
];

/// Inner content width of the box (chars between ║ borders).
/// BOX_INNER_WIDTH ≥ logo line width (44) + 2 side padding.
const BOX_INNER_WIDTH: usize = 50;

/// Prints a centered string inside the banner box with ║ borders.
/// `content_display_len` is the *visible* character count (excluding ANSI codes)
/// used for padding calculations — pass the raw string length before colorizing.
fn box_line(content: &str, content_display_len: usize) {
    let padding = BOX_INNER_WIDTH.saturating_sub(content_display_len);
    let left_pad = padding / 2;
    let right_pad = padding - left_pad;
    println!(
        "  ║ {}{}{} ║",
        " ".repeat(left_pad),
        content,
        " ".repeat(right_pad)
    );
}

/// Prints a blank line inside the banner box.
fn box_empty_line() {
    println!("  ║{}║", " ".repeat(BOX_INNER_WIDTH + 2));
}

/// Renders the full-width banner with ASCII art logo.
/// Called when the terminal is wide enough (≥ FULL_BANNER_MIN_WIDTH columns).
fn print_full_banner(version: &str, color: bool) {
    let top_border = format!("  ╔{}╗", "═".repeat(BOX_INNER_WIDTH + 2));
    let bot_border = format!("  ╚{}╝", "═".repeat(BOX_INNER_WIDTH + 2));

    if color {
        println!("{}", top_border.bright_green());
        box_empty_line();
        for line in LOGO_LINES {
            let logo_w = line.chars().count();
            let pad   = BOX_INNER_WIDTH.saturating_sub(logo_w);
            let left  = pad / 2;
            let right = pad - left;
            println!("  ║ {}{}{} ║", " ".repeat(left), line.bright_green(), " ".repeat(right));
        }
        box_empty_line();

        // Subtitle line: measure the *raw* string for padding, colorize after.
        let raw_subtitle = format!("ZYRIX  ·  v{}", version);
        let colored_subtitle = format!(
            "{}  {}  {}",
            "ZYRIX".bold(),
            "·".cyan(),
            format!("v{}", version).dimmed()
        );
        box_line(&colored_subtitle, raw_subtitle.len());

        box_empty_line();

        // Platform line
        let raw_platforms = "GitHub  ·  GitLab  ·  Bitbucket  ·  Azure DevOps";
        let colored_platforms = format!(
            "{}  {}  {}  {}  {}  {}  {}",
            "GitHub".yellow(),
            "·".cyan(),
            "GitLab".yellow(),
            "·".cyan(),
            "Bitbucket".yellow(),
            "·".cyan(),
            "Azure DevOps".yellow()
        );
        box_line(&colored_platforms, raw_platforms.len());

        box_empty_line();
        println!("{}", bot_border.bright_green());
    } else {
        println!("{}", top_border);
        box_empty_line();
        for line in LOGO_LINES {
            let logo_w = line.chars().count();
            let pad   = BOX_INNER_WIDTH.saturating_sub(logo_w);
            let left  = pad / 2;
            let right = pad - left;
            println!("  ║ {}{}{} ║", " ".repeat(left), line, " ".repeat(right));
        }
        box_empty_line();
        box_line(
            &format!("ZYRIX  ·  v{}", version),
            0,
        );
        box_empty_line();
        box_line("GitHub  ·  GitLab  ·  Bitbucket  ·  Azure DevOps", 0);
        box_empty_line();
        println!("{}", bot_border);
    }
}

/// Renders a compact one-line banner for narrow terminals (< FULL_BANNER_MIN_WIDTH).
/// Ensures the banner never wraps or breaks the layout on small terminals.
fn print_compact_banner(version: &str, color: bool) {
    if color {
        println!(
            "  {} {} {}",
            "◆ ZYRIX".bright_green().bold(),
            format!("v{}", version).dimmed(),
            "· GitHub · GitLab · Bitbucket · Azure DevOps".cyan()
        );
        println!(
            "  {}",
            "─────────────────────────────────────────".dimmed()
        );
    } else {
        println!("  ZYRIX v{} · GitHub · GitLab · Bitbucket · Azure DevOps", version);
        println!("  {}", "─".repeat(60));
    }
}

/// The public entry point. Call this once at the start of CLI dispatch,
/// guarded by the `!no_banner && atty` check in `app.rs`.
///
/// # Example
/// ```rust
/// use gm_interface_cli::ui::banner::print_banner;
///
/// print_banner(env!("CARGO_PKG_VERSION"));
/// ```
pub fn print_banner(version: &str) {
    let color = colors_enabled();
    let width = terminal_width();

    println!();

    if width >= FULL_BANNER_MIN_WIDTH {
        print_full_banner(version, color);
    } else {
        print_compact_banner(version, color);
    }

    println!();

    // Flush immediately so the banner appears before any async work begins.
    let _ = io::stdout().flush();
}
