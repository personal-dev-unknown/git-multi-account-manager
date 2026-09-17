use gm_domain::git::ports::CloneProgress;

/// A progress reporter that logs progress at INFO level with structured fields.
pub fn tracing_progress() -> impl Fn(CloneProgress) + Send + Sync {
    |progress: CloneProgress| {
        tracing::info!(
            percentage = format_args!("{:.2}", progress.percentage),
            stage = %progress.stage,
            bytes_transferred = progress.bytes_transferred,
            total_bytes = progress.total_bytes,
            speed_bps = progress.speed_bps,
            message = %progress.message,
            "clone progress",
        );
    }
}
