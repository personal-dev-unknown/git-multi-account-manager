// crates/gm_plugin_github/src/auth.rs
//
// Helper utilities for GitHub authentication. GitHub currently supports PATs
// (Personal Access Tokens, both classic and fine-grained) and OAuth apps.
// For v1 Git Manager supports PATs only — they are simpler, more reliable,
// and work identically from the user's perspective across all GitHub account types.


/// Returns true if `token` looks like a GitHub PAT.
/// Classic PATs start with "ghp_"; fine-grained tokens start with "github_pat_".
/// This is a heuristic for early validation before making an API call.
pub fn looks_like_github_token(token: &str) -> bool {
    token.starts_with("ghp_")
        || token.starts_with("gho_")  // OAuth token
        || token.starts_with("github_pat_")
        || token.len() >= 40          // Legacy 40-char hex token
}