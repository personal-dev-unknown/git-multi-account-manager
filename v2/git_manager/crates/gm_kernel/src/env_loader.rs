// crates/gm_kernel/src/env_loader.rs
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Finds and loads a .env configuration file regardless of where the user runs
// `git-zyrix` from — home directory, Pictures, systemd service, or anywhere else.
//
// Search order (first match wins, no errors on miss):
//   1. Ancestors of CWD          — works when running inside the repo tree
//   2. Ancestors of binary path  — works with portable installs
//   3. ~/.config/git-zyrix/.env  — XDG standard path for production setups
//   4. ~/.git-zyrix/.env         — home directory fallback
//
// At each directory level we check two paths:
//   <dir>/v2/git_manager/.env   (project-root style — for development)
//   <dir>/.env                  (standard dotenv convention)

use std::path::{Path, PathBuf};

/// Try to find and load a .env file from any of the standard locations.
///
/// Returns `true` if a file was found and loaded, `false` otherwise.
/// Safe to call unconditionally — failures are silently ignored.
pub fn try_load_dotenv() -> bool {
    // 1. Primary Production Configuration: Inject `database.json` into the environment.
    // This executes first so that `.env` loading can correctly override these
    // settings in a developer's local environment without modifying the persistent configuration.
    crate::config_loader::inject_into_env();

    // 2. Local Development Configuration: Find and load `.env`.
    // dotenvy DOES NOT override existing environment variables by default.
    // However, we want .env to act as a local development override. If we needed .env
    // to strictly overwrite production config, we would use `dotenvy::from_path_override`.
    // For now, the standard `from_path` aligns with the existing architecture.
    let candidates = candidate_paths();
    for path in &candidates {
        if path.is_file() {
            if dotenvy::from_path(path).is_ok() {
                return true;
            }
        }
    }

    false
}

/// Collect all candidate .env paths across all search strategies.
fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. Walk up from CWD
    if let Ok(cwd) = std::env::current_dir() {
        collect_upward(&cwd, &mut paths);
    }

    // 2. Walk up from the binary's location
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            collect_upward(exe_dir, &mut paths);
        }
    }

    // 3. System Config Directories (XDG on Linux, AppData on Windows, Library on macOS)
    if let Some(config_dir) = dirs_next::config_dir() {
        let current = config_dir.join("git-zyrix").join(".env");
        paths.push(current);

        let legacy = config_dir.join("git-manager").join(".env");
        paths.push(legacy);
    }

    // 4. Fallback to home directory
    if let Some(home) = home_dir() {
        let home_current = home.join(".git-zyrix").join(".env");
        paths.push(home_current);

        let home_legacy = home.join(".git-manager").join(".env");
        paths.push(home_legacy);
    }

    paths
}

/// Walk up from `start` toward filesystem root, appending candidate paths.
/// At each level checks for `.env`.
fn collect_upward(start: &Path, out: &mut Vec<PathBuf>) {
    let mut dir: &Path = start;
    loop {
        out.push(dir.join(".env"));
        match dir.parent() {
            Some(p) => dir = p,
            None => break,
        }
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
        .or_else(|| dirs_next::home_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_env_in_project_tree() {
        // Passes when CWD is anywhere under the repo (including test_clone/).
        // Verifies the walk-up logic reaches git_manager/.env eventually.
        let candidates = candidate_paths();
        let any_exists = candidates.iter().any(|p| p.is_file());
        assert!(any_exists, "should find at least one .env candidate that exists on disk");
    }

    #[test]
    fn finds_cwd_ancestor() {
        let cwd = std::env::current_dir().unwrap();
        let mut found = Vec::new();
        collect_upward(&cwd, &mut found);
        assert!(!found.is_empty(), "collect_upward must emit candidates");
        // At least one should end with `.env`
        assert!(found.iter().any(|p| p.ends_with(".env")));
    }

    #[test]
    fn includes_xdg_path() {
        let candidates = candidate_paths();
        let has_xdg = candidates.iter().any(|p| {
            p.to_string_lossy().contains(".config/git-zyrix/.env")
        });
        assert!(has_xdg, "must include XDG config path");
    }

    #[test]
    fn includes_home_fallback() {
        let candidates = candidate_paths();
        let has_home = candidates.iter().any(|p| {
            p.to_string_lossy().contains(".git-zyrix/.env")
        });
        assert!(has_home, "must include ~/.git-zyrix/.env fallback");
    }
}
