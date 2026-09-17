use indicatif::{ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gm_domain::git::ports::{CloneProgress, human_bytes, eta_remaining};

/// Wraps an indicatif ProgressBar and reports CloneProgress updates.
pub struct CliProgressReporter {
    bar:   Arc<Mutex<Option<ProgressBar>>>,
    msg:   String,
    start: Instant,
}

impl CliProgressReporter {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { bar: Arc::new(Mutex::new(None)), msg: msg.into(), start: Instant::now() }
    }

    /// Returns a closure suitable as a `ProgressCallback` for `CloneFallbackEngine`.
    pub fn callback(&self) -> Arc<dyn Fn(CloneProgress) + Send + Sync> {
        let bar = self.bar.clone();
        let msg = self.msg.clone();
        let start = self.start;
        Arc::new(move |progress: CloneProgress| {
            let mut guard = bar.lock().unwrap();
            let percentage = progress.percentage;
            let pct_str = format!("{:.2}%", percentage);

            match percentage {
                p if p >= 100.0 => {
                    if let Some(pb) = guard.take() {
                        pb.finish_with_message(format!("{} — {} complete", msg, pct_str));
                    }
                }
                p if p > 0.0 && guard.is_none() => {
                    let pb = ProgressBar::new(10000);
                    pb.set_style(ProgressStyle::default_bar()
                        .template("{spinner:.cyan} {msg} [{bar:40.cyan/dim}] {pos}/{len} ({eta})")
                        .expect("valid template")
                        .progress_chars("█▉▊▋▌▍▎▏  "));
                    pb.set_message(build_msg(&msg, &progress, &start, false));
                    pb.enable_steady_tick(Duration::from_millis(100));
                    *guard = Some(pb);
                }
                _ => {}
            }

            if let Some(ref pb) = *guard {
                pb.set_position((percentage * 100.0) as u64);
                pb.set_message(build_msg(&msg, &progress, &start, true));
            }
        })
    }
}

fn build_msg(base: &str, progress: &CloneProgress, _start: &Instant, show_speed: bool) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Stage + percentage
    parts.push(format!("{} — {:.2}%", base, progress.percentage));

    // Speed and ETA
    if show_speed && progress.speed_bps > 0 {
        let speed_str = human_bytes(progress.speed_bps);
        parts.push(format!("@ {}/s", speed_str));

        if progress.total_bytes > 0 {
            let remaining = progress.total_bytes.saturating_sub(progress.bytes_transferred);
            let eta = eta_remaining(remaining, progress.speed_bps);
            parts.push(format!("ETA {}", eta));
        }
    }

    // Transferred / total
    if progress.bytes_transferred > 0 || progress.total_bytes > 0 {
        let transferred = human_bytes(progress.bytes_transferred);
        let total = human_bytes(progress.total_bytes);
        parts.push(format!("({} / {})", transferred, total));
    }

    // Git progress message
    if !progress.message.is_empty() {
        parts.push(progress.message.clone());
    }

    parts.join(" ")
}
