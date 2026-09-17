/// SourceForge authentication helpers.
///
/// SourceForge supports SSH key authentication for git operations.
/// It does not have a PAT or OAuth API — projects are publicly readable.
/// Credentials are only used for SSH key management.

/// SourceForge does not use PATs. This function always returns false.
pub fn looks_like_valid_token(_token: &str) -> bool {
    false
}

/// Returns the SourceForge SSH host for SSH key configuration.
pub fn ssh_host() -> &'static str {
    "git.code.sf.net"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_always_invalid() {
        assert!(!looks_like_valid_token("anything"));
    }
}
