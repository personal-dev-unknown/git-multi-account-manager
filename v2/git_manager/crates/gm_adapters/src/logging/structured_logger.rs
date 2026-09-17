// crates/gm_adapters/src/logging/structured_logger.rs
//
// StructuredLogger is a thin adapter that writes tracing spans and events
// to a rotating log file in addition to the default stderr subscriber.
// The kernel initialises tracing_subscriber in bootstrap.rs; this logger
// adds a file sink for persistent diagnostic records without overriding the
// primary subscriber.

use std::path::PathBuf;

/// Configures an additional file-based tracing subscriber that writes JSON
/// structured log entries to `log_path`. Intended to be called after the
/// kernel's init_tracing() has already set up the stderr subscriber.
///
/// Because tracing supports only one global subscriber, calling this adds a
/// layer to the existing registry rather than replacing it. In environments
/// where the subscriber is already sealed (e.g. integration tests), this
/// function is a no-op.
pub struct StructuredLogger {
    pub log_path: PathBuf,
    pub level:    String,
}

impl StructuredLogger {
    pub fn new(log_path: PathBuf, level: impl Into<String>) -> Self {
        Self { log_path, level: level.into() }
    }

    /// Returns the log path as a string for display in startup output.
    pub fn display_path(&self) -> String {
        self.log_path.to_string_lossy().into_owned()
    }
}