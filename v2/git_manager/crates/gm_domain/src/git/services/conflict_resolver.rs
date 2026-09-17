// crates/gm_domain/src/git/services/conflict_resolver.rs
//
// The ConflictResolver encapsulates the domain logic around what constitutes
// a merge conflict, how to detect one from git status output, and what
// information the system exposes about it. It does not attempt to resolve
// conflicts automatically — that would require understanding the semantics
// of the conflicting code, which is beyond the system's scope. Instead it
// provides the context that the CLI or web UI needs to present to the user.

use crate::git::ports::git_executor::{GitStatus, StatusEntry};

/// A description of a conflict in a single file, extracted from git status output.
#[derive(Debug, Clone)]
pub struct FileConflict {
    /// The path of the conflicted file relative to the repository root.
    pub path: String,
    /// The type of conflict: both-modified, added-by-us, deleted-by-us, etc.
    pub conflict_type: ConflictType,
}

/// The specific way in which two histories conflicted in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// Both the local and remote branches modified the same file.
    BothModified,
    /// The file was added by both branches with different content.
    BothAdded,
    /// The local branch deleted the file but the remote branch modified it.
    DeletedByUs,
    /// The remote branch deleted the file but the local branch modified it.
    DeletedByThem,
    /// Any other conflict type reported by git (indicated by the porcelain status codes).
    Other(String),
}

/// The ConflictResolver is a stateless service — all methods are pure functions
/// over the data from git status. It can be constructed with no dependencies.
pub struct ConflictResolver;

impl ConflictResolver {
    pub fn new() -> Self {
        Self
    }

    /// Returns true if the given git status indicates at least one conflict.
    /// This is the fast path used by the sync service to decide whether to
    /// abort the sync and notify the user.
    pub fn has_conflicts(status: &GitStatus) -> bool {
        status.has_conflicts
    }

    /// Extracts all conflicted file paths from the git status and returns
    /// a list of `FileConflict` structs describing each one.
    ///
    /// Git's `--porcelain` output uses two-character codes. The 'U' character
    /// in either position indicates a conflict. Specific two-char combinations:
    ///   "UU" — both sides modified
    ///   "AA" — both sides added with different content
    ///   "DU" — deleted by us, modified by them
    ///   "UD" — modified by us, deleted by them
    pub fn extract_conflicts(status: &GitStatus) -> Vec<FileConflict> {
        status.entries.iter()
            .filter(|entry| Self::is_conflicted(entry))
            .map(|entry| FileConflict {
                path:          entry.path.clone(),
                conflict_type: Self::classify_conflict(entry),
            })
            .collect()
    }

    /// Returns a single-paragraph human-readable description of all conflicts,
    /// suitable for embedding in error messages or CLI output.
    pub fn describe_conflicts(conflicts: &[FileConflict]) -> String {
        if conflicts.is_empty() {
            return "No conflicts.".to_string();
        }
        let list: Vec<String> = conflicts.iter()
            .map(|c| format!("  {} ({:?})", c.path, c.conflict_type))
            .collect();
        format!("{} conflicted file(s):\n{}", conflicts.len(), list.join("\n"))
    }

    fn is_conflicted(entry: &StatusEntry) -> bool {
        // In git --porcelain output, 'U' in either position means conflict.
        // 'AA' and 'DD' are also conflict states.
        let i = entry.index_status;
        let w = entry.working_status;
        i == 'U' || w == 'U'
            || (i == 'A' && w == 'A')
            || (i == 'D' && w == 'D')
    }

    fn classify_conflict(entry: &StatusEntry) -> ConflictType {
        match (entry.index_status, entry.working_status) {
            ('U', 'U') => ConflictType::BothModified,
            ('A', 'A') => ConflictType::BothAdded,
            ('D', 'U') => ConflictType::DeletedByUs,
            ('U', 'D') => ConflictType::DeletedByThem,
            _          => ConflictType::Other(
                format!("{}{}", entry.index_status, entry.working_status)
            ),
        }
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::ports::git_executor::{GitStatus, StatusEntry};

    fn make_status(entries: Vec<StatusEntry>) -> GitStatus {
        let has_conflicts = entries.iter().any(|e| {
            e.index_status == 'U' || e.working_status == 'U'
                || (e.index_status == 'A' && e.working_status == 'A')
        });
        let conflicted_files = entries.iter()
            .filter(|e| e.index_status == 'U' || e.working_status == 'U')
            .map(|e| e.path.clone())
            .collect();
        GitStatus {
            branch: "main".to_string(),
            upstream: None,
            ahead: 0,
            behind: 0,
            entries,
            is_clean: false,
            has_conflicts,
            conflicted_files,
        }
    }

    #[test]
    fn detects_uu_as_both_modified() {
        let status = make_status(vec![StatusEntry {
            path: "src/main.rs".to_string(),
            index_status: 'U',
            working_status: 'U',
        }]);
        let conflicts = ConflictResolver::extract_conflicts(&status);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::BothModified);
    }

    #[test]
    fn clean_status_has_no_conflicts() {
        let status = make_status(vec![StatusEntry {
            path: "README.md".to_string(),
            index_status: 'M',
            working_status: ' ',
        }]);
        assert!(!ConflictResolver::has_conflicts(&status));
        assert!(ConflictResolver::extract_conflicts(&status).is_empty());
    }
}