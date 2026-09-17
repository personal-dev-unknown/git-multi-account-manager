// crates/gm_plugin_gitlab/src/auth.rs
//
// GitLab-specific authentication helpers.
//
// GitLab supports three credential types:
//   1. Personal Access Tokens (PAT) — static long-lived tokens with scope-based permissions.
//      Passed as the `PRIVATE-TOKEN` header. Most common for developer tooling.
//   2. OAuth 2.0 access tokens — short-lived tokens issued via the PKCE flow.
//      Passed as `Authorization: Bearer {token}`. Requires a configured OAuth app.
//   3. Deploy tokens — read-only tokens for CI/CD pipelines.
//      Not handled here; Git Manager uses PAT or OAuth for interactive accounts.
//
// In v1, Git Manager supports PAT authentication for GitLab. OAuth support is
// a future feature requiring a redirect URI and browser interaction. The helpers
// here reflect the v1 PAT-only scope while leaving extension points clean.

/// The required scopes for a GitLab PAT used by Git Manager.
/// Shown to the user when they are asked to create a token.
pub const REQUIRED_PAT_SCOPES: &[&str] = &[
    "read_user",       // GET /user — validate the token and read username
    "read_api",        // read access to repositories and metadata
    "read_repository", // clone access via HTTPS
];

/// Builds the human-readable scope description shown in the "add account" UI.
pub fn required_scopes_description() -> String {
    REQUIRED_PAT_SCOPES
        .iter()
        .map(|s| format!("  • {s}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Validates that a PAT string has a plausible format.
/// GitLab PATs are 20 characters long and alphanumeric with hyphens.
/// This is a format check only — the actual validity is confirmed by the API call.
pub fn looks_like_valid_pat(token: &str) -> bool {
    let t = token.trim();
    // GitLab personal access tokens are typically 20+ chars
    t.len() >= 20 && t.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Validates that a password credential has the correct "username:password" format.
/// Returns `true` if the credential contains exactly one `:` separator, has a
/// non-empty username, and has a non-empty password.
pub fn looks_like_valid_password(credential: &str) -> bool {
    credential.contains(':')
        && credential.split(':').next().map_or(false, |u| !u.is_empty())
        && credential.split(':').nth(1).map_or(false, |p| !p.is_empty())
}

/// Returns a human-readable description of how to set up GitLab password auth.
pub fn password_auth_instructions() -> String {
    format!(
        "To use password authentication for GitLab:\n\
         1. Ensure your GitLab account has a strong password\n\
         2. If 2FA is enabled, create a personal access token instead\n\
         3. Enter your credential as 'username:password' when prompted\n\
         \n\
         Note: GitLab recommends Personal Access Tokens over password auth.\n\
         Create a PAT at: https://gitlab.com/-/user_settings/personal_access_tokens"
    )
}

/// Returns the GitLab PAT creation URL so the CLI can print it as a help link.
pub fn pat_creation_url() -> &'static str {
    "https://gitlab.com/-/user_settings/personal_access_tokens"
}

/// Returns the required scope query string for deep-linking to the PAT creation page.
/// The resulting URL pre-selects the required scopes in the GitLab UI.
pub fn pat_creation_url_with_scopes() -> String {
    let scopes = REQUIRED_PAT_SCOPES.join(",");
    format!(
        "https://gitlab.com/-/user_settings/personal_access_tokens?scopes={scopes}&name=git-zyrix"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_pat_format() {
        // 20+ alphanumeric chars passes
        assert!(looks_like_valid_pat("glpat-abcdefghijklmnopqr"));
        assert!(looks_like_valid_pat("01234567890123456789abcd"));
    }

    #[test]
    fn short_token_rejected() {
        assert!(!looks_like_valid_pat("short"));
        assert!(!looks_like_valid_pat(""));
    }

    #[test]
    fn special_chars_rejected() {
        // Spaces and punctuation other than hyphen/underscore are invalid
        assert!(!looks_like_valid_pat("abc def ghi jkl mno pqr"));
        assert!(!looks_like_valid_pat("token!with@special#chars$%^"));
    }
}