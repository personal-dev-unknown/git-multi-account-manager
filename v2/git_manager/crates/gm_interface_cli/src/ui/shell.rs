// crates/gm_interface_cli/src/ui/shell.rs
//
// Persistent subcommand REPL shell.
//
// When the user picks "Subcommand" from the mode selector, this loop takes
// over instead of running a single Clap parse-and-exit.  It keeps a prompt
// open, parses every line the user types as if it were passed on the real
// command line, dispatches it, then loops back for the next command.
//
// Exit words: exit / quit / q  or Ctrl+D (EOF).
//
// ── Send safety ───────────────────────────────────────────────────────────────
// StdinLock<'_> is not Send.  To keep the async future Send we acquire the
// lock, read one line, and DROP the lock — all inside a synchronous block —
// before any .await point.

use std::{
    io::{self, Write},
    sync::Arc,
};

use clap::Parser;
use tokio::io::{AsyncBufReadExt, BufReader};
use gm_kernel::kernel::Kernel;
use gm_shared::errors::GitManagerError;

use crate::app::Cli;
use crate::ui::colors;

const PROMPT: &str = "git-zyrix";

pub async fn run_shell(kernel: Arc<Kernel>) -> Result<(), GitManagerError> {
    println!();
    println!(
        "{}  Subcommand shell — type any git-zyrix command below.",
        colors::info_prefix()
    );
    println!(
        "  {}  {}",
        colors::dim("Examples:"),
        colors::dim("account list  |  ssh generate --account work  |  clone owner/repo --account work")
    );
    println!(
        "  {}  {}",
        colors::dim("Quit    :"),
        colors::dim("exit  or  Ctrl+D")
    );
    println!();

    let mut stdin_lines = BufReader::new(tokio::io::stdin()).lines();

    loop {
        // Print prompt, flush so it appears before the user types.
        print!("{} {} ", colors::bold(PROMPT), colors::dim(">"));
        io::stdout().flush().ok();

        // ── Read one line, racing against Ctrl+C ─────────────────────────────
        // Ctrl+C cancels only the current input — the shell stays alive.
        // Ctrl+D (EOF on next_line → Ok(None)) or "exit" ends the session.
        let line: String = tokio::select! {
            // Ctrl+C — cancel this input, loop back to the prompt.
            _ = tokio::signal::ctrl_c() => {
                println!("\n{}  Operation cancelled by user.", colors::warn_prefix());
                continue;
            }
            // Normal line input (or EOF).
            result = stdin_lines.next_line() => {
                match result {
                    Ok(None)    => break,          // EOF / Ctrl+D
                    Err(_)      => break,
                    Ok(Some(l)) => l.trim().to_string(),
                }
            }
        };

        if line.is_empty() {
            continue;
        }

        // Exit words
        if matches!(line.as_str(), "exit" | "quit" | "q") {
            break;
        }

        // Split the line into tokens (handles quoted strings).
        let tokens = match shlex_split(&line) {
            Some(t) => t,
            None => {
                eprintln!("{} Unmatched quote in input.", colors::error_prefix());
                continue;
            }
        };

        // Prepend the binary name so Clap sees a full argv.
        let argv: Vec<String> = std::iter::once(PROMPT.to_string())
            .chain(tokens)
            .collect();

        // ── Dispatch ──────────────────────────────────────────────────────────
        match Cli::try_parse_from(&argv) {
            Ok(cli) => {
                if let Err(e) = crate::app::dispatch(cli, kernel.clone()).await {
                    eprintln!("{} {}", colors::error_prefix(), e);
                }
            }
            Err(e) => {
                eprintln!("{e}");
            }
        }

        println!(); // visual separator between commands
    }

    println!();
    println!("{} Shell session ended.  Goodbye!", colors::success_prefix());
    Ok(())
}

/// Minimal shell-word splitter that handles single and double quoted strings.
/// Returns None if there is an unclosed quote.
fn shlex_split(s: &str) -> Option<Vec<String>> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut chars = s.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            ' ' | '\t' if !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(current.drain(..).collect());
                }
            }
            '\\' if (in_double || (!in_single && !in_double)) => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            _ => current.push(ch),
        }
    }

    if in_single || in_double {
        return None; // unclosed quote
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Some(tokens)
}
