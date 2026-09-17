//! Platform detection utilities — OS identification, WSL detection, home directory.
//!
//! These functions answer "what environment am I running in?" without pulling in
//! external crates. The information is used by the adapter layer to make decisions
//! like which credential backend to prefer and how to format path separators
//! in diagnostic messages.

/// A simplified categorisation of the host operating system.
/// Covers the platforms where Git Manager is expected to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux,
    MacOs,
    Windows,
    /// Any other OS that Rust can target (FreeBSD, NetBSD, etc.)
    Other(String),
}

impl std::fmt::Display for OperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatingSystem::Linux       => write!(f, "Linux"),
            OperatingSystem::MacOs       => write!(f, "macOS"),
            OperatingSystem::Windows     => write!(f, "Windows"),
            OperatingSystem::Other(name) => write!(f, "{name}"),
        }
    }
}

/// Returns the current operating system as a typed enum.
///
/// This is a compile-time constant — the Rust compiler's `cfg!` macros are
/// evaluated at build time, so there is no runtime overhead. The result is
/// inlined by the optimizer wherever it is called.
pub fn get_current_os() -> OperatingSystem {
    if cfg!(target_os = "linux") {
        OperatingSystem::Linux
    } else if cfg!(target_os = "macos") {
        OperatingSystem::MacOs
    } else if cfg!(target_os = "windows") {
        OperatingSystem::Windows
    } else {
        OperatingSystem::Other(std::env::consts::OS.to_string())
    }
}

/// Returns the user's home directory path as a `String`.
///
/// On POSIX systems: reads the `HOME` environment variable.
/// On Windows: reads `USERPROFILE`, falling back to `HOMEDRIVE` + `HOMEPATH`.
///
/// Returns `None` if no home directory can be determined — this can happen
/// in minimal container environments where environment variables are stripped.
pub fn get_home_dir() -> Option<String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .or_else(|_| {
            // Windows fallback: HOMEDRIVE + HOMEPATH (e.g. "C:" + "\Users\shaka")
            let drive = std::env::var("HOMEDRIVE").unwrap_or_default();
            let path  = std::env::var("HOMEPATH").unwrap_or_default();
            if drive.is_empty() && path.is_empty() {
                Err(std::env::VarError::NotPresent)
            } else {
                Ok(format!("{drive}{path}"))
            }
        })
        .ok()
}

/// Detects whether the process is running inside Windows Subsystem for Linux.
///
/// WSL detection is important because:
///   - The credential backend should prefer the Linux keyring (libsecret)
///     rather than attempting Windows Credential Manager calls
///   - Path display should use POSIX separators even when the user may be
///     used to Windows path conventions
///   - Some SSH agent socket paths differ between native Linux and WSL
///
/// Detection method: reads `/proc/version` and checks whether it contains
/// the string "microsoft" or "WSL" (case-insensitive). This is the canonical
/// method used by most Linux tools for WSL detection and works on WSL 1 and WSL 2.
///
/// Returns `false` on any error (file not found, permission denied) since
/// those conditions indicate native Linux or another non-WSL environment.
pub fn is_wsl() -> bool {
    let Ok(version) = std::fs::read_to_string("/proc/version") else {
        return false;
    };
    let lower = version.to_lowercase();
    lower.contains("microsoft") || lower.contains("wsl")
}

/// Returns a human-readable string describing the current platform and
/// any special context (e.g. "Linux (WSL2)").
///
/// Used in diagnostic messages, log output, and the `git-zyrix --version`
/// display to help debug environment-specific issues.
pub fn platform_description() -> String {
    let os = get_current_os();
    if os == OperatingSystem::Linux && is_wsl() {
        return "Linux (WSL)".to_string();
    }
    os.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_current_os_returns_a_variant() {
        // We can not assert a specific value since the test runs on the build
        // machine's OS, but we confirm the function returns something meaningful.
        let os = get_current_os();
        let display = os.to_string();
        assert!(!display.is_empty());
    }

    #[test]
    fn platform_description_is_non_empty() {
        let desc = platform_description();
        assert!(!desc.is_empty());
    }
}