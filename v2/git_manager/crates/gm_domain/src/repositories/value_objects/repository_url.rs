// crates/gm_domain/src/repositories/value_objects/repository_url.rs
//
// A validated, typed remote URL for a Git repository.
// The newtype pattern ensures that a RepositoryUrl can only be constructed
// from a string that passed validation, preventing raw unvalidated strings
// from flowing through the domain logic.

use gm_shared::validation::validators::{validate_url, ValidationError};

/// A validated Git remote URL. Once constructed, the inner string is guaranteed
/// to match one of the accepted Git URL formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryUrl(String);

impl RepositoryUrl {
    /// Constructs a RepositoryUrl from a raw string after validating its format.
    pub fn new(raw: String) -> Result<Self, ValidationError> {
        validate_url(&raw)?;
        Ok(Self(raw))
    }

    /// Returns the raw URL string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if this URL uses SSH (git@ or ssh://).
    pub fn is_ssh(&self) -> bool {
        self.0.starts_with("git@") || self.0.starts_with("ssh://")
    }

    /// Returns true if this URL uses HTTPS.
    pub fn is_https(&self) -> bool {
        self.0.starts_with("https://") || self.0.starts_with("http://")
    }
}

impl std::fmt::Display for RepositoryUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for RepositoryUrl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}