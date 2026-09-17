use std::path::Path;
use std::sync::Arc;

/// Stages of a clone lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneStage {
    Connecting,
    ReceivingObjects,
    ResolvingDeltas,
    CheckingOut,
    UpdatingSparseCheckout,
    Done,
}

impl std::fmt::Display for CloneStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connecting => write!(f, "connecting"),
            Self::ReceivingObjects => write!(f, "receiving_objects"),
            Self::ResolvingDeltas => write!(f, "resolving_deltas"),
            Self::CheckingOut => write!(f, "checking_out"),
            Self::UpdatingSparseCheckout => write!(f, "sparse_checkout"),
            Self::Done => write!(f, "done"),
        }
    }
}

/// Immutable snapshot of clone progress at a point in time.
#[derive(Debug, Clone)]
pub struct CloneProgress {
    /// Percentage completed, from 0.00 to 100.00 inclusive.
    pub percentage: f64,
    /// Current clone stage.
    pub stage: CloneStage,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    /// Transfer speed in bytes per second (0 if unknown).
    pub speed_bps: u64,
    /// Human-readable status line.
    pub message: String,
}

impl CloneProgress {
    pub fn new(percentage: f64) -> Self {
        let stage = if percentage < 1.0 {
            CloneStage::Connecting
        } else if percentage < 90.0 {
            CloneStage::ReceivingObjects
        } else if percentage < 99.0 {
            CloneStage::ResolvingDeltas
        } else if percentage < 100.0 {
            CloneStage::CheckingOut
        } else {
            CloneStage::Done
        };
        Self {
            percentage: (percentage * 100.0).round() / 100.0,
            stage,
            bytes_transferred: 0,
            total_bytes: 0,
            speed_bps: 0,
            message: String::new(),
        }
    }

    pub fn done() -> Self {
        Self {
            percentage: 100.00,
            stage: CloneStage::Done,
            bytes_transferred: 0,
            total_bytes: 0,
            speed_bps: 0,
            message: "completed".into(),
        }
    }
}

/// Callback for reporting clone progress.
/// The closure is called with progress snapshots during a clone operation.
pub type ProgressCallback = Arc<dyn Fn(CloneProgress) + Send + Sync>;

/// A no-op progress reporter that discards all updates.
pub fn noop_progress() -> ProgressCallback {
    Arc::new(|_| {})
}

/// Format bytes into a human-readable string (KB, MB, GB).
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_idx = 0;
    while value >= 1024.0 && unit_idx < UNITS.len() - 1 {
        value /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", value, UNITS[unit_idx])
    }
}

/// Estimate remaining time based on bytes remaining and current speed.
/// Returns a human-readable string like "~2 minutes 30 seconds".
pub fn eta_remaining(bytes_remaining: u64, speed_bps: u64) -> String {
    if speed_bps == 0 || bytes_remaining == 0 {
        return "calculating…".into();
    }
    let secs = bytes_remaining / speed_bps;
    if secs < 60 {
        format!("~{} seconds", secs.max(1))
    } else if secs < 3600 {
        format!("~{} minutes {} seconds", secs / 60, secs % 60)
    } else {
        format!("~{} hours {} minutes", secs / 3600, (secs % 3600) / 60)
    }
}

/// Checks available disk space on a given path.
pub trait DiskSpaceChecker: Send + Sync {
    fn available_bytes(&self, path: &Path) -> Result<u64, String>;
    fn ensure_space(&self, path: &Path, required_bytes: u64) -> Result<(), String> {
        let avail = self.available_bytes(path)?;
        if avail < required_bytes {
            return Err(format!(
                "insufficient disk space: need {} ({} bytes), only {} ({} bytes) available",
                human_bytes(required_bytes), required_bytes,
                human_bytes(avail), avail,
            ));
        }
        Ok(())
    }
}

/// Memory protection guard for clone operations.
#[derive(Debug, Clone)]
pub struct MemoryGuard {
    /// Maximum allowed memory for a single clone in megabytes.
    pub max_clone_memory_mb: u64,
}

impl Default for MemoryGuard {
    fn default() -> Self {
        Self { max_clone_memory_mb: 1024 }
    }
}

impl MemoryGuard {
    pub fn new(max_mb: u64) -> Self {
        Self { max_clone_memory_mb: max_mb }
    }

    /// Validates that a clone operation of `estimated_size_bytes` is within
    /// memory limits. Returns Ok or an error message.
    pub fn check(&self, estimated_size_bytes: u64) -> Result<(), String> {
        let max_bytes = self.max_clone_memory_mb * 1024 * 1024;
        if estimated_size_bytes > max_bytes {
            return Err(format!(
                "clone requires ~{} MB, but memory limit is {} MB",
                estimated_size_bytes / (1024 * 1024),
                self.max_clone_memory_mb,
            ));
        }
        Ok(())
    }
}
