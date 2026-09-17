// crates/gm_domain/src/repositories/value_objects/clone_status.rs
//
// The clone lifecycle for a repository: the system discovers a repository via
// the platform API before it is cloned. Once cloning starts it may fail, then
// succeed. The status drives what operations are available in the UI.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloneStatus {
    /// The repository is known (discovered via API) but no local copy exists.
    NotCloned,
    /// A clone operation is currently in progress.
    Cloning,
    /// The repository has a local copy. Git operations can be performed.
    Cloned,
    /// The last clone attempt failed. The `reason` describes what went wrong.
    Failed(String),
}

impl CloneStatus {
    pub fn is_cloned(&self) -> bool {
        matches!(self, CloneStatus::Cloned)
    }

    pub fn can_start_clone(&self) -> bool {
        matches!(self, CloneStatus::NotCloned | CloneStatus::Failed(_))
    }
}

impl std::fmt::Display for CloneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloneStatus::NotCloned    => write!(f, "not_cloned"),
            CloneStatus::Cloning      => write!(f, "cloning"),
            CloneStatus::Cloned       => write!(f, "cloned"),
            CloneStatus::Failed(r)    => write!(f, "failed: {r}"),
        }
    }
}