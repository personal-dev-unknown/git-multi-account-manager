// crates/gm_interface_cli/src/ui/tables.rs
// Formatted table rendering. Works with DTOs from gm_shared.

use gm_shared::models::{account::AccountDto, repository::RepositoryDto, ssh_key::SshKeyDto};
use crate::services::{GitStatusEntry, OperationLogEntry};
use crate::ui::colors::*;

pub fn print_accounts(accounts: &[AccountDto]) {
    if accounts.is_empty() { return; }
    println!();
    println!("  {:<20} {:<14} {:<24} {:<14} {}",
        bold("ALIAS"), bold("PLATFORM"), bold("USERNAME"), bold("STATUS"), bold("DEFAULT"));
    println!("  {}", "─".repeat(80));
    for a in accounts {
        let platform = a.platform_name.as_deref().unwrap_or("—");
        let status   = a.status.to_string();
        let default_marker = if a.is_default { check_mark() } else { String::new() };
        println!("  {:<20} {:<14} {:<24} {:<14} {}",
            cyan(&a.alias), dim(platform), a.username, colored_status(&status), default_marker);
    }
    println!();
}

pub fn print_ssh_keys(keys: &[SshKeyDto]) {
    if keys.is_empty() { return; }
    println!();
    println!("  {:<30} {:<10} {:<30} {:<10} {}",
        bold("NAME"), bold("TYPE"), bold("FINGERPRINT"), bold("TESTED"), bold("AGENT"));
    println!("  {}", "─".repeat(90));
    for k in keys {
        let fp_short     = &k.fingerprint[..k.fingerprint.len().min(28)];
        let test_status  = k.last_test_status.to_string();
        let agent        = if k.is_added_to_agent { green("yes") } else { dim("no").to_string() };
        println!("  {:<30} {:<10} {:<30} {:<10} {}",
            cyan(&k.name), dim(&k.key_type.to_string()), fp_short, colored_status(&test_status), agent);
    }
    println!();
}

pub fn print_repositories(repos: &[RepositoryDto]) {
    if repos.is_empty() { return; }
    println!();
    println!("  {:<42} {:<16} {:<12} {}",
        bold("REPOSITORY"), bold("BRANCH"), bold("STATUS"), bold("LOCAL PATH"));
    println!("  {}", "─".repeat(90));
    for r in repos {
        let status = if r.is_cloned { green("cloned") } else { dim("not cloned").to_string() };
        let path   = r.local_path.as_deref().unwrap_or("—");
        println!("  {:<42} {:<16} {:<12} {}",
            cyan(&r.full_name), dim(&r.default_branch), status, dim(path));
    }
    println!();
}

pub fn print_operation_logs(logs: &[OperationLogEntry]) {
    if logs.is_empty() { return; }
    println!();
    println!("  {:<20} {:<14} {:<10} {:<22} {}",
        bold("STARTED"), bold("TYPE"), bold("STATUS"), bold("ACCOUNT/REPO"), bold("ERROR"));
    println!("  {}", "─".repeat(90));
    for e in logs {
        let ts  = e.started_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let ctx = e.repository.as_deref().or(e.account.as_deref()).unwrap_or("—");
        let st  = match e.status.as_str() {
            "success" => green(&e.status),
            "failed"  => red(&e.status),
            _         => yellow(&e.status),
        };
        let err = e.error.as_deref().unwrap_or("");
        let err_short = &err[..err.len().min(28)];
        println!("  {:<20} {:<14} {:<10} {:<22} {}",
            dim(&ts), bold(&e.op_type), st, dim(ctx), red(err_short));
    }
    println!();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Paginated status table helpers (book‑spread layout)
// ═══════════════════════════════════════════════════════════════════════════════

/// Fixed chars consumed by one row aside from the file column:
/// " St " + "  " + " Change  " + "  " + " Area " = 20
const ROW_FIXED: usize = 20;

/// Map the leading path component to a short area badge.
fn path_area(path: &str) -> &'static str {
    let clean = path.trim_start_matches('…');
    match clean.split('/').next().unwrap_or("") {
        "src" | "crates" | "lib"   => "src",
        "docs" | "doc"             => "docs",
        "test" | "tests"           => "test",
        "scripts" | "script"       => "scr",
        "config" | "cfg"           => "cfg",
        "backup"                   => "bkp",
        "docker" | "Docker"        => "dock",
        "snap"                     => "snap",
        "examples" | "example"     => "ex",
        s if s.starts_with('.')    => "cfg",
        ""                         => "root",
        _                          => "·",
    }
}

/// Colored, left‑padded‑to‑4 area badge.
fn col_area(path: &str) -> String {
    let a = path_area(path);
    let p = format!("{:<4}", a);
    match a {
        "src"             => cyan(&p),
        "docs"            => dim(&p),
        "test"            => green(&p),
        "scr" | "cfg"     => yellow(&p),
        "bkp" | "dock" | "snap" => bold(&p),
        "root"            => bold(&p),
        _                 => dim(&p),
    }
}

/// Colored status code, padded to 2 visual chars.
fn col_code(raw: &str) -> String {
    let t = raw.trim();
    let s = format!("{:<2}", t);
    if      t.contains('M')       { yellow(&s) }
    else if t.starts_with('A')    { green(&s)  }
    else if t.contains('D')       { red(&s)    }
    else if t == "??"             { dim(&s)    }
    else if t == "UU"             { red(&s)    }
    else                          { dim(&s)    }
}

/// Colored 8‑visual‑char change label.
fn col_change(raw: &str) -> String {
    let label = match raw.trim() {
        t if t.contains('M')       => "modified",
        t if t.starts_with('A')    => "added   ",
        t if t.contains('D')       => "deleted ",
        "??"                       => "new file",
        "UU"                       => "conflict",
        _                          => "changed ",
    };
    match raw.trim() {
        t if t.contains('M')       => yellow(label),
        t if t.starts_with('A')    => green(label),
        t if t.contains('D')       => red(label),
        "??"                       => cyan(label),
        "UU"                       => red(label),
        _                          => dim(label),
    }
}

/// Build one data row string (no padding — caller pads for 2‑col layout).
pub fn data_row(e: &GitStatusEntry, w_file: usize) -> String {
    let raw = e.path.trim_start_matches('…');
    let file = if raw.len() > w_file {
        format!("…{}", &raw[raw.len() - (w_file - 1)..])
    } else {
        raw.to_string()
    };
    format!(
        "{}  {:<w_file$}  {}  {}",
        pad_vis(&col_code(&e.status), 2),
        file,
        pad_vis(&col_change(&e.status), 8),
        col_area(&e.path),
    )
}

/// Column header line (plain text — caller applies bold etc.).
pub fn header_line(w_file: usize) -> String {
    format!("{:<2}  {:<w_file$}  {:<8}  {:<4}",
        "St", "File", "Change  ", "Area")
}

/// Column divider line (plain text — caller applies dim).
pub fn divider_line(w_file: usize) -> String {
    format!("{:<2}  {:<w_file$}  {:<8}  {:<4}",
        "──", "─".repeat(w_file), "────────", "────")
}

/// Print a single status page inside a rounded‑corner box.
/// Returns the number of lines printed (for cursor‑up overwriting).
pub fn render_status_box(
    slice:    &[&GitStatusEntry],
    total:    usize,
    n_vis:    usize,
    page:     usize,
    n_pages:  usize,
    two_cols: bool,
    filter:   Option<char>,
    sort_asc: bool,
) -> usize {
    let tw     = terminal_width();
    let border = tw - 2; // ╭─…─╮ / ╰─…─╯ horizontal
    let cont_w = tw.saturating_sub(6); // │  content  │

    // Column geometry: 2‑col → split cont_w in half with " ║ " divider
    let (col_l, col_r) = if two_cols {
        let each  = (cont_w.saturating_sub(3)) / 2;
        let extra = cont_w.saturating_sub(3) - 2 * each;
        (each, each + extra)
    } else {
        (cont_w, cont_w)
    };
    let w_file = col_l.saturating_sub(ROW_FIXED);

    // ── Row printer ───────────────────────────────────────────────────
    macro_rules! box_row {
        ($content:expr) => {
            println!("{}  {}  {}",
                cyan("│"),
                pad_vis($content, cont_w),
                cyan("│"),
            );
        };
    }

    // ── Top border ────────────────────────────────────────────────────
    let filter_name = match filter {
        None      => "all",
        Some('M') => "modified",
        Some('A') => "added",
        Some('D') => "deleted",
        Some('?') => "untracked",
        _         => "all",
    };
    let sort_label = if sort_asc { "asc ↑" } else { "desc ↓" };
    let count_str  = if n_vis == total {
        format!("{} changes", total)
    } else {
        format!("{}/{} changes", n_vis, total)
    };
    let title = format!(" {} · Page {}/{} · {} · filter: {}  ",
        count_str, page + 1, n_pages, sort_label, filter_name);
    let t_len  = visual_width(&title);
    let dash_l = border.saturating_sub(t_len) / 2;
    let dash_r = border.saturating_sub(t_len).saturating_sub(dash_l);
    println!("{}{}{}{}{}",
        cyan("╭"),
        cyan(&"─".repeat(dash_l)),
        bold(&title),
        cyan(&"─".repeat(dash_r)),
        cyan("╮"),
    );

    // ── Column headers ────────────────────────────────────────────────
    let hdr = bold(&header_line(w_file));
    let div = dim(&divider_line(w_file));
    let sep = format!(" {} ", dim("║"));

    if two_cols {
        box_row!(&format!("{}{}{}",
            pad_vis(&hdr, col_l), sep, pad_vis(&hdr, col_r)));
        box_row!(&format!("{}{}{}",
            pad_vis(&div, col_l), sep, pad_vis(&div, col_r)));
    } else {
        box_row!(&hdr);
        box_row!(&div);
    }

    // ── Data rows ─────────────────────────────────────────────────────
    let col_h  = if two_cols { (slice.len() + 1) / 2 } else { slice.len() };
    let split  = col_h.min(slice.len());
    let lhs    = &slice[..split];
    let rhs    = &slice[split..];

    for i in 0..col_h {
        let left    = data_row(lhs[i], w_file);
        let content = if two_cols {
            let right = rhs.get(i).copied()
                .map(|e| data_row(e, w_file))
                .unwrap_or_default();
            format!("{}{}{}",
                pad_vis(&left, col_l), sep, pad_vis(&right, col_r))
        } else {
            pad_vis(&left, cont_w).to_string()
        };
        box_row!(&content);
    }

    // ── Navigation separator ──────────────────────────────────────────
    println!("{}{}{}",
        cyan("╞"),
        cyan(&"═".repeat(border)),
        cyan("╡"),
    );

    // ── Navigation row ────────────────────────────────────────────────
    let mut parts: Vec<String> = Vec::new();
    if page > 0           { parts.push(yellow("[p] Prev")); }
    if page + 1 < n_pages { parts.push(yellow("[n] Next")); }
    parts.push(dim("[q] Quit"));
    if filter.is_some()   { parts.push(dim("[c] Clear filter")); }
    else                  { parts.push(dim("[f] Filter")); }
    if sort_asc           { parts.push(dim("[s] Sort ↓")); }
    else                  { parts.push(dim("[s] Sort ↑")); }
    parts.push(dim("or page #"));

    box_row!(&format!("  {}", parts.join("  ")));

    // ── Bottom border ─────────────────────────────────────────────────
    println!("{}{}{}",
        cyan("╰"),
        cyan(&"─".repeat(border)),
        cyan("╯"),
    );
    println!("  › ");

    // Lines printed: top-border + headers(2) + data(col_h) + separator + nav + bottom + prompt = 7 + col_h
    7 + col_h
}