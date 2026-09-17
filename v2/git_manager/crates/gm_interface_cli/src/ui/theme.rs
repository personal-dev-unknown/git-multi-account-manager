// crates/gm_interface_cli/src/ui/theme.rs
//
// Application banner and global visual theme constants.
// print_banner() is the sole public export — called once in app.rs::dispatch().

use owo_colors::OwoColorize;
use std::io::{self, Write};

use super::colors;

/// Minimum terminal width for the full box banner. Below this the compact
/// one-liner is shown instead.
const FULL_BANNER_MIN_WIDTH: usize = 60;

/// Inner content width of the box (chars between ║ borders).
const BOX_INNER_WIDTH: usize = 50;

/// ZYRIX ASCII art — 6 rows, each exactly 44 visible chars wide.
///
///   Z  — top bar + descending diagonal + bottom bar
///   Y  — two arms converging to a stem
///   R  — left post + curved top + diagonal kick
///   I  — vertical bar
///   X  — two crossing diagonals
const LOGO_LINES: &[&str] = &[
    r"████████╗  ██╗   ██╗  ██████╗  ██╗  ██╗  ██╗",
    r"╚══════██╗ ╚██╗ ██╔╝  ██╔══██╗ ██║  ╚██╗██╔╝",
    r"      ██╔╝  ╚████╔╝   ██████╔╝ ██║   ╚███╔╝ ",
    r"    ██╔╝     ╚██╔╝    ██╔══██╗ ██║   ██╔██╗ ",
    r"  ██╔╝        ██║     ██║  ██║ ██║  ██╔╝ ██╗",
    r"████████╗     ╚═╝     ╚═╝  ╚═╝ ╚═╝  ╚═╝  ╚═╝",
];

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

fn colors_enabled() -> bool {
    std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map_or(true, |t| t != "dumb")
        && atty::is(atty::Stream::Stdout)
}

/// Prints a line centered inside the ║ box.
/// `display_len` = visible char count of `content` (no ANSI codes included).
fn box_line(content: &str, display_len: usize) {
    let padding = BOX_INNER_WIDTH.saturating_sub(display_len);
    let left  = padding / 2;
    let right = padding - left;
    println!("  ║ {}{}{} ║", " ".repeat(left), content, " ".repeat(right));
}

fn box_empty_line() {
    println!("  ║{}║", " ".repeat(BOX_INNER_WIDTH + 2));
}

fn print_full_banner(version: &str, color: bool) {
    let top = format!("  ╔{}╗", "═".repeat(BOX_INNER_WIDTH + 2));
    let bot = format!("  ╚{}╝", "═".repeat(BOX_INNER_WIDTH + 2));

    if color {
        let banner_fg = colors::theme_banner;
        println!("{}", banner_fg(&top));
        box_empty_line();

        for line in LOGO_LINES {
            let logo_w = line.chars().count();
            let pad = BOX_INNER_WIDTH.saturating_sub(logo_w);
            let left  = pad / 2;
            let right = pad - left;
            println!(
                "  ║ {}{}{} ║",
                " ".repeat(left),
                banner_fg(line),
                " ".repeat(right)
            );
        }

        box_empty_line();

        // Subtitle — measure raw string for padding, colorize after.
        let raw_sub = format!("Multi-Account Git Manager  ·  v{}", version);
        let colored_sub = format!(
            "{}  {}  {}",
            colors::theme_primary("Multi-Account Git Manager"),
            "·".cyan(),
            colors::dim(&format!("v{}", version))
        );
        box_line(&colored_sub, raw_sub.len());

        box_empty_line();

        let raw_plat = "GitHub  ·  GitLab  ·  Bitbucket  ·  Azure DevOps";
        let colored_plat = format!(
            "{}  {}  {}  {}  {}  {}  {}",
            colors::theme_accent("GitHub"), "·".cyan(),
            colors::theme_accent("GitLab"), "·".cyan(),
            colors::theme_accent("Bitbucket"), "·".cyan(),
            colors::theme_accent("Azure DevOps")
        );
        box_line(&colored_plat, raw_plat.len());

        box_empty_line();
        println!("{}", colors::theme_banner(&bot));
    } else {
        println!("{}", top);
        box_empty_line();
        for line in LOGO_LINES {
            let logo_w = line.chars().count();
            let pad = BOX_INNER_WIDTH.saturating_sub(logo_w);
            let left  = pad / 2;
            let right = pad - left;
            println!("  ║ {}{}{} ║", " ".repeat(left), line, " ".repeat(right));
        }
        box_empty_line();
        box_line(
            &format!("Multi-Account Git Manager  ·  v{}", version),
            0,
        );
        box_empty_line();
        box_line("GitHub  ·  GitLab  ·  Bitbucket  ·  Azure DevOps", 0);
        box_empty_line();
        println!("{}", bot);
    }
}

fn print_compact_banner(version: &str, color: bool) {
    if color {
        println!(
            "  {} {} {}",
            colors::theme_banner("◆ ZYRIX").bold(),
            colors::dim(&format!("v{}", version)),
            colors::theme_info("· GitHub · GitLab · Bitbucket · Azure DevOps")
        );
        println!("  {}", "─────────────────────────────────────────".dimmed());
    } else {
        println!("  ZYRIX v{} · GitHub · GitLab · Bitbucket · Azure DevOps", version);
        println!("  {}", "─".repeat(60));
    }
}

/// The public entry point. Called once at the start of CLI dispatch.
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
    let _ = io::stdout().flush();
}