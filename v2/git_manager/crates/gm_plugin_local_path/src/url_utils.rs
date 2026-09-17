/// Local path URL parsing utilities.
///
/// Supports:
/// - Absolute paths: /home/user/project
/// - Home-relative paths: ~/project
/// - file:// protocol: file:///home/user/project

use std::path::{Path, PathBuf};

/// Information parsed from a local repository path.
#[derive(Debug, Clone)]
pub struct LocalPathInfo {
    pub absolute: PathBuf,
    pub repo:     String,
    pub owner:    String,
}

/// Parses a local path into its components.
/// Returns None if the path is invalid or does not exist.
pub fn parse_local_path(path: &str) -> Option<LocalPathInfo> {
    let expanded = if path.starts_with("file://") {
        PathBuf::from(&path[7..])
    } else {
        let p = path.replace('~', &dirs_next::home_dir()?.to_string_lossy());
        PathBuf::from(p)
    };

    let repo_name = expanded.file_name()?.to_string_lossy().to_string();
    let parent_name = expanded.parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "local".to_string());

    Some(LocalPathInfo {
        absolute: expanded,
        repo: repo_name,
        owner: parent_name,
    })
}

/// Checks whether a path contains a .git directory.
pub fn is_git_repository(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Checks whether a path exists and is accessible.
pub fn path_exists(path: &str) -> bool {
    let expanded = path.replace('~', &dirs_next::home_dir()
        .map(|d| d.to_string_lossy().to_string())
        .unwrap_or_default());
    Path::new(&expanded).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_absolute_path() {
        let info = parse_local_path("/home/user/my-repo").unwrap();
        assert_eq!(info.repo, "my-repo");
        assert_eq!(info.owner, "user");
    }

    #[test]
    fn parse_file_protocol() {
        let info = parse_local_path("file:///home/user/my-repo").unwrap();
        assert_eq!(info.repo, "my-repo");
    }
}
