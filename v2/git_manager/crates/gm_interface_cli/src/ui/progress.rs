// crates/gm_interface_cli/src/ui/progress.rs
// Progress bar factory functions for long-running operations.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Creates a spinner for indeterminate operations (network calls, ssh ops).
pub fn spinner(msg: impl Into<String>) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .expect("valid template"));
    pb.set_message(msg.into());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Creates a determinate progress bar (for clone with known size, etc.).
pub fn progress_bar(len: u64, msg: impl Into<String>) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.cyan} {msg} [{bar:40.cyan/dim}] {pos}/{len}")
        .expect("valid template")
        .progress_chars("█▉▊▋▌▍▎▏  "));
    pb.set_message(msg.into());
    pb
}