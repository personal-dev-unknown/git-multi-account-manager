// crates/gm_interface_cli/src/ui/colors.rs
//
// Thin wrappers around owo-colors that respect the TTY check. When stdout is
// not a terminal (piped, redirected, CI), every colour function returns the
// plain text so scripts don't get polluted with ANSI escape codes.
//
// Theme integration: functions ending in `_theme` use the currently-active
// theme's colour palette. The theme engine is initialised by CliPlugin::run()
// and stored in a global static for ergonomic access.

use std::sync::{Arc, Mutex, OnceLock};

use owo_colors::OwoColorize;
use terminal_size::{terminal_size, Width};

use crate::ui::theme_engine::{ThemeColors, ThemeRegistry};

// ── Global theme state ─────────────────────────────────────────────────────────
// Set once by CliPlugin::run() early in the boot sequence.
static THEME: OnceLock<Arc<Mutex<ThemeRegistry>>> = OnceLock::new();

/// Initialise the global theme reference. Panics if called more than once.
pub fn init_theme(registry: Arc<Mutex<ThemeRegistry>>) {
    THEME.set(registry).ok();
}

/// Helper: extract the current theme colors, falling back to Zyrix defaults.
fn current_colors() -> ThemeColors {
    THEME.get()
        .and_then(|r| r.lock().ok())
        .map(|g| g.current_colors())
        .unwrap_or(ThemeColors::zyrix())
}

// ── TTY check ─────────────────────────────────────────────────────────────────

fn is_tty() -> bool { atty::is(atty::Stream::Stdout) }

// ── Generic style helpers ─────────────────────────────────────────────────────

pub fn bold(s: &str)    -> String { if is_tty() { s.bold().to_string() } else { s.to_string() } }
pub fn dim(s: &str)     -> String { if is_tty() { s.dimmed().to_string() } else { s.to_string() } }

// ── Named colour functions (hardcoded, for backward compat) ───────────────────

pub fn green(s: &str)   -> String { if is_tty() { s.green().to_string() } else { s.to_string() } }
pub fn yellow(s: &str)  -> String { if is_tty() { s.yellow().to_string() } else { s.to_string() } }
pub fn red(s: &str)     -> String { if is_tty() { s.red().to_string() } else { s.to_string() } }
pub fn cyan(s: &str)    -> String { if is_tty() { s.cyan().to_string() } else { s.to_string() } }
pub fn magenta(s: &str) -> String { if is_tty() { s.magenta().to_string() } else { s.to_string() } }

// ── Prefix helpers (hardcoded) ────────────────────────────────────────────────

pub fn success_prefix() -> String { if is_tty() { "✓".green().bold().to_string() } else { "[ok]".to_string() } }
pub fn error_prefix()   -> String { if is_tty() { "✗".red().bold().to_string()   } else { "[err]".to_string() } }
pub fn warn_prefix()    -> String { if is_tty() { "⚠".yellow().bold().to_string()} else { "[warn]".to_string() } }
pub fn info_prefix()    -> String { if is_tty() { "ℹ".cyan().to_string()         } else { "[info]".to_string() } }
pub fn check_mark()     -> String { if is_tty() { "✓".green().to_string()         } else { "v".to_string() } }

// ── ANSI-aware helpers ───────────────────────────────────────────────────────

/// Visual width of a string, ignoring ANSI escape sequences.
pub fn visual_width(s: &str) -> usize {
    let mut len = 0;
    let mut esc = false;
    for b in s.bytes() {
        if b == 0x1B { esc = true; continue; }
        if esc { if b == b'm' { esc = false; } continue; }
        if b & 0x80 == 0 || b & 0xC0 == 0xC0 { len += 1; }
    }
    len
}

/// Right-pad a string (which may contain ANSI codes) to `width` visual columns.
pub fn pad_vis(s: &str, width: usize) -> String {
    let v = visual_width(s);
    if v >= width { s.to_string() } else { format!("{}{}", s, " ".repeat(width - v)) }
}

/// Detect the current terminal width (columns). Falls back to 120.
pub fn terminal_width() -> usize {
    terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or_else(|| {
            std::env::var("COLUMNS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120)
        })
        .clamp(80, 240)
}

/// Colours a status string based on its value.
pub fn colored_status(status: &str) -> String {
    if !is_tty() { return status.to_string(); }
    match status.to_lowercase().as_str() {
        "active"        => green(status),
        "unverified"    => yellow(status),
        "suspended" | "token_expired" => red(status),
        _               => dim(status),
    }
}

// ── Theme-aware colour functions ─────────────────────────────────────────────
// These follow the current theme palette. Use them for new UI code.

/// Apply the current theme's primary colour to text.
pub fn theme_primary(s: &str) -> String {
    if is_tty() { s.color(current_colors().primary).to_string() } else { s.to_string() }
}

/// Apply the current theme's accent colour to text.
pub fn theme_accent(s: &str) -> String {
    if is_tty() { s.color(current_colors().accent).to_string() } else { s.to_string() }
}

/// Apply the current theme's success colour to text.
pub fn theme_success(s: &str) -> String {
    if is_tty() { s.color(current_colors().success).to_string() } else { s.to_string() }
}

/// Apply the current theme's error colour to text.
pub fn theme_error(s: &str) -> String {
    if is_tty() { s.color(current_colors().error).to_string() } else { s.to_string() }
}

/// Apply the current theme's warning colour to text.
pub fn theme_warning(s: &str) -> String {
    if is_tty() { s.color(current_colors().warning).to_string() } else { s.to_string() }
}

/// Apply the current theme's info colour to text.
pub fn theme_info(s: &str) -> String {
    if is_tty() { s.color(current_colors().info).to_string() } else { s.to_string() }
}

/// Apply the current theme's banner/logo colour.
pub fn theme_banner(s: &str) -> String {
    if is_tty() { s.color(current_colors().banner).to_string() } else { s.to_string() }
}
