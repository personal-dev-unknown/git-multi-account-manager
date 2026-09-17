// crates/gm_ports/src/inbound/commands/clone_repository.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_domain::git::ports::git_executor::TagsMode;
use gm_shared::errors::GitManagerError;

/// Command: clone a remote repository.
/// When account_uuid is provided, the account's SSH key is used.
/// When None, anonymous HTTPS clone is attempted (public repos only).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloneRepositoryCommand {
    /// The remote git URL (SSH or HTTPS).
    pub url:               String,
    /// The account whose SSH key or credentials to use for the clone.
    pub account_uuid:      Option<Uuid>,
    /// Optional absolute path on disk. If None, inferred from repo name.
    pub destination:       Option<String>,
    /// If Some, clone only this branch. If None, clone the default branch.
    pub branch:            Option<String>,
    /// Clone depth: 0 = full history, N > 0 = shallow with N commits of history.
    #[serde(default)]
    pub depth:             u32,
    /// Partial clone filter (e.g. "blob:none", "tree:0", "blob:limit=1m").
    #[serde(default)]
    pub filter:            Option<String>,
    /// If true, create a bare repository (no working directory).
    #[serde(default)]
    pub bare:              bool,
    /// If true, create a mirror repository (bare + all refs as-is).
    #[serde(default)]
    pub mirror:            bool,
    /// If set, initialize sparse checkout with only the specified paths (comma-separated).
    #[serde(default)]
    pub sparse_checkout:   Option<String>,
    /// If true, clone only the tip of the requested branch.
    #[serde(default)]
    pub single_branch:     bool,
    /// If true, do not checkout HEAD after clone.
    #[serde(default)]
    pub no_checkout:       bool,
    /// If true, initialize and clone submodules recursively.
    #[serde(default)]
    pub recurse_submodules: bool,
    /// How to handle tags: "none", "reachable" (default), "all".
    #[serde(default = "default_tags_mode")]
    pub tags_mode:         String,
    /// Custom upload pack executable.
    #[serde(default)]
    pub upload_pack:       Option<String>,
}

fn default_tags_mode() -> String {
    "reachable".to_string()
}

impl CloneRepositoryCommand {
    pub fn validate(&self) -> Result<(), GitManagerError> {
        if self.url.trim().is_empty() {
            return Err(GitManagerError::Other("URL cannot be empty".to_string()));
        }
        Ok(())
    }

    pub fn to_tags_mode(&self) -> TagsMode {
        match self.tags_mode.as_str() {
            "none"      => TagsMode::None,
            "all"       => TagsMode::All,
            _           => TagsMode::Reachable,
        }
    }
}