//! Filesystem utility functions — path expansion, directory creation, safe reads.
//!
//! These functions handle the gap between "what the user typed" and "what the OS
//! needs". A user types `~/.ssh/id_ed25519_work`; the OS needs the absolute path.
//! A service needs a directory to exist; we create it with the right permissions.
//!
//! All functions are synchronous because they operate on small path strings or
//! small config files — none of them touch large data or block for long enough
//! to justify the complexity of async file I/O.

use std::path::{Path, PathBuf};

/// Expands a path that begins with `~` to the user's home directory.
///
/// The function reads `HOME` from the environment (POSIX) or `USERPROFILE`
/// (Windows fallback) to determine the home directory. If neither is set —
/// an unusual but possible situation in minimal containers — the path is
/// returned unchanged.
///
/// Expansion only occurs for `~` at the very start of the path:
///   - `~/projects` → `/home/shaka/projects`
///   - `~user/bin` is NOT expanded (user-specific tilde, non-portable, ignored)
///   - `/home/shaka/~notilde` → unchanged (tilde in the middle)
pub fn expand_path(path: &str) -> PathBuf {
    if path == "~" {
        return home_dir().unwrap_or_else(|| PathBuf::from("~"));
    }

    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }

    PathBuf::from(path)
}

/// Creates a directory and all missing parent directories.
///
/// Behaves like `mkdir -p` — succeeds if the directory already exists.
/// Sets the mode of the final created directory to `mode` on POSIX systems.
/// On Windows the mode is ignored (Windows uses ACLs, not POSIX permissions).
///
/// Returns the absolute `PathBuf` of the directory on success, or the
/// underlying IO error if creation failed (e.g. permission denied).
pub fn ensure_dir(path: &Path, mode: u32) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(path)?;

    // Set permissions on POSIX systems.
    // On Windows cfg(unix) is false, so this block is compiled out entirely.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(mode);
        std::fs::set_permissions(path, permissions)?;
    }

    // Suppress unused variable warning on Windows where mode is not used.
    #[cfg(not(unix))]
    let _ = mode;

    Ok(path.to_path_buf())
}

/// Reads the entire contents of a file into a String.
///
/// Unlike `std::fs::read_to_string`, this function:
///   - Expands a leading `~` in the path before reading
///   - Returns a clean `std::io::Error` with the path embedded in the message
///     so that error messages in the CLI and web interface show the full path
///     that could not be read, making the error immediately actionable.
pub fn safe_read_to_string(path: &str) -> std::io::Result<String> {
    let expanded = expand_path(path);
    std::fs::read_to_string(&expanded).map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!("failed to read '{}': {}", expanded.display(), e),
        )
    })
}

/// Writes bytes to a file, creating the file's parent directories if needed.
///
/// This is a convenience wrapper that combines `ensure_dir` on the parent
/// with `std::fs::write`. It does NOT use the atomic write pattern (temp file
/// + rename) — that is handled at the Zig layer for security-sensitive files
///   like `~/.ssh/config`. Use this for non-critical configuration snippets.
pub fn safe_write(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, contents)
}

/// Returns the absolute path to the user's home directory.
///
/// Checks the following in order:
///   1. `HOME` environment variable (POSIX — Linux, macOS)
///   2. `USERPROFILE` environment variable (Windows)
///
/// Returns `None` if neither is set.
pub fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

/// Returns the path to the application's configuration directory.
///
/// On all platforms this is `~/.git-zyrix`. Creates the directory with
/// mode 0700 if it does not already exist.
pub fn config_dir() -> std::io::Result<PathBuf> {
    let home = home_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME environment variable is not set — cannot locate config directory",
        )
    })?;
    let dir = home.join(crate::constants::app::CONFIG_DIR_NAME);
    ensure_dir(&dir, 0o700)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_path_leaves_absolute_unchanged() {
        let result = expand_path("/home/shaka/projects");
        assert_eq!(result, PathBuf::from("/home/shaka/projects"));
    }

    #[test]
    fn expand_path_expands_tilde_slash() {
        std::env::set_var("HOME", "/home/testuser");
        let result = expand_path("~/projects");
        assert_eq!(result, PathBuf::from("/home/testuser/projects"));
    }

    #[test]
    fn expand_path_tilde_alone_returns_home() {
        std::env::set_var("HOME", "/home/testuser");
        let result = expand_path("~");
        assert_eq!(result, PathBuf::from("/home/testuser"));
    }

    #[test]
    fn expand_path_middle_tilde_unchanged() {
        let result = expand_path("/projects/~notilde/file");
        assert_eq!(result, PathBuf::from("/projects/~notilde/file"));
    }
}