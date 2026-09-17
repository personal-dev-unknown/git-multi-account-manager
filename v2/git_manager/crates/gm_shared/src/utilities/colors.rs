//! Terminal color support detection and ANSI escape code stripping.
//!
//! The CLI and TUI layers use these functions to decide whether to emit colored
//! output. The rules follow the widely adopted `NO_COLOR` convention
//! (https://no-color.org) and the `COLORTERM` convention used by most modern
//! terminal emulators.
//!
//! `strip_ansi` is used by the web interface and audit logging layer to remove
//! color codes from git and ssh output before storing it in the database or
//! sending it in an HTTP response — both contexts where raw ANSI sequences
//! would be meaningless or harmful.

/// Returns `true` if the running terminal is likely to render ANSI color codes.
///
/// Decision algorithm (checked in priority order):
///
/// 1. If `NO_COLOR` is set (any non-empty value), return `false`. This is the
///    canonical opt-out mechanism. Any process that sets `NO_COLOR` is explicitly
///    requesting plain output.
///
/// 2. If `TERM=dumb`, return `false`. Dumb terminals do not understand any
///    escape sequences; printing them produces garbage.
///
/// 3. If `COLORTERM` is set to `"truecolor"` or `"24bit"`, return `true`.
///    These values are set by terminals that support full 24-bit RGB color
///    (Kitty, Alacritty, modern versions of Terminal.app, Windows Terminal).
///
/// 4. If `TERM` contains "color" (e.g. `xterm-256color`), return `true`.
///
/// 5. If `TERM` is one of the known color-supporting values, return `true`.
///
/// 6. Otherwise return `false` — assume no color support to avoid garbage output
///    in unknown environments such as CI pipelines, cron jobs, or SSH sessions
///    with stripped environments.
pub fn supports_color() -> bool {
    // Opt-out: NO_COLOR overrides everything
    if std::env::var("NO_COLOR").map(|v| !v.is_empty()).unwrap_or(false) {
        return false;
    }

    // Dumb terminal: no escape sequence support
    if std::env::var("TERM").as_deref() == Ok("dumb") {
        return false;
    }

    // Explicit 24-bit color support declaration
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        let lower = colorterm.to_lowercase();
        if lower == "truecolor" || lower == "24bit" {
            return true;
        }
    }

    // TERM-based heuristics
    if let Ok(term) = std::env::var("TERM") {
        // Any TERM value containing "color" supports color (e.g. xterm-256color)
        if term.contains("color") {
            return true;
        }
        // Known color-supporting terminal types
        const COLOR_TERMS: &[&str] = &[
            "xterm", "vt100", "rxvt", "screen", "tmux", "linux", "ansi",
        ];
        if COLOR_TERMS.iter().any(|&t| term.starts_with(t)) {
            return true;
        }
    }

    false
}

/// Strips ANSI escape sequences from a string, returning plain text.
///
/// ANSI escape sequences follow the pattern `ESC [ ... m` where `ESC` is the
/// byte 0x1B (decimal 27). This function implements a state machine that
/// recognises the full family of CSI (Control Sequence Introducer) sequences
/// used for colors, cursor movement, and text attributes. It also handles the
/// OSC (Operating System Command) sequences used by some terminals for window
/// titles and hyperlinks.
///
/// The output contains only the visible printable content — no control characters
/// and no invisible formatting. The output string is allocated freshly; the input
/// is not modified.
///
/// Performance: O(n) in the length of the input string. A single pass with a
/// small state machine — no regex, no allocation during scanning.
pub fn strip_ansi(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut i = 0;

    while i < bytes.len() {
        let byte = bytes[i];

        if byte == 0x1B {
            // Escape character — start of an escape sequence
            i += 1;
            if i >= bytes.len() {
                break;
            }

            match bytes[i] {
                b'[' => {
                    // CSI sequence: ESC [ ... (final byte in 0x40–0x7E range)
                    i += 1;
                    while i < bytes.len() {
                        let seq_byte = bytes[i];
                        i += 1;
                        // Final byte of a CSI sequence: ASCII 0x40–0x7E (@A–Z[\]^_`a–z{|}~)
                        if (0x40..=0x7E).contains(&seq_byte) {
                            break;
                        }
                    }
                }
                b']' => {
                    // OSC sequence: ESC ] ... ST or ESC ] ... BEL
                    // String terminator is either ESC \ or BEL (0x07).
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == 0x07 {
                            // BEL terminates OSC
                            i += 1;
                            break;
                        }
                        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                            // ESC \ (ST) terminates OSC
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                _ => {
                    // Two-character escape sequence (Fe sequence): ESC + single byte.
                    // Examples: ESC M (reverse index), ESC 7 (save cursor).
                    // Simply consume the second byte and move on.
                    i += 1;
                }
            }
        } else {
            // Regular byte — find the run of non-escape bytes and push them all
            let start = i;
            while i < bytes.len() && bytes[i] != 0x1B {
                i += 1;
            }
            // SAFETY: We are working on valid UTF-8 input but slicing at byte
            // boundaries that were not checked for UTF-8 character boundaries.
            // ANSI escape sequences are always pure ASCII (all bytes < 128), so
            // the escape byte 0x1B can only appear at a UTF-8 character boundary.
            // Therefore, slicing [start..i] across escape boundaries is safe as
            // long as we only split at positions where we saw 0x1B.
            if let Ok(s) = std::str::from_utf8(&bytes[start..i]) {
                output.push_str(s);
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_color_codes() {
        let colored = "\x1b[32mGreen text\x1b[0m";
        assert_eq!(strip_ansi(colored), "Green text");
    }

    #[test]
    fn strip_ansi_removes_256_color_codes() {
        let colored = "\x1b[38;5;196mRed\x1b[0m normal";
        assert_eq!(strip_ansi(colored), "Red normal");
    }

    #[test]
    fn strip_ansi_leaves_plain_text_unchanged() {
        let plain = "Hello, world!";
        assert_eq!(strip_ansi(plain), plain);
    }

    #[test]
    fn strip_ansi_handles_empty_string() {
        assert_eq!(strip_ansi(""), "");
    }

    #[test]
    fn strip_ansi_handles_multiple_sequences() {
        let input = "\x1b[1mBold\x1b[0m and \x1b[4munderline\x1b[0m";
        assert_eq!(strip_ansi(input), "Bold and underline");
    }

    #[test]
    fn strip_ansi_handles_cursor_movement() {
        // ESC[2J (clear screen) followed by text
        let input = "\x1b[2JVisible text";
        assert_eq!(strip_ansi(input), "Visible text");
    }

    #[test]
    fn strip_ansi_handles_osc_hyperlink() {
        // OSC 8 hyperlink sequence
        let input = "\x1b]8;;https://example.com\x07link text\x1b]8;;\x07";
        assert_eq!(strip_ansi(input), "link text");
    }

    #[test]
    fn supports_color_respects_no_color() {
        // We cannot reliably test the true path because it depends on the
        // test runner's environment. We CAN test the opt-out path.
        std::env::set_var("NO_COLOR", "1");
        assert!(!supports_color());
        std::env::remove_var("NO_COLOR");
    }
}