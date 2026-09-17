//! Pure validation functions for user-supplied strings.
//!
//! All functions return `Result<(), ValidationError>` where the Ok variant
//! carries no data (the input was valid as-is) and the Err variant carries
//! a human-readable explanation of what was wrong. This pattern lets callers
//! use the `?` operator to propagate validation failures cleanly.
//!
//! None of these functions allocate memory on the heap — they work over
//! string slices using only iterator and pattern-matching operations.

use thiserror::Error;

/// Describes why a piece of user input failed validation.
/// The message is always written in terms the end user can act on:
/// not "regex failed" but "alias must be 2–50 characters".
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ValidationError {
    #[error("Alias '{value}': {reason}")]
    InvalidAlias { value: String, reason: String },

    #[error("Email '{value}': {reason}")]
    InvalidEmail { value: String, reason: String },

    #[error("URL '{value}': {reason}")]
    InvalidUrl { value: String, reason: String },

    #[error("{message}")]
    Generic { message: String },
}

/// Validates an account alias.
///
/// Rules enforced:
///   - Length must be 2–50 characters (inclusive on both ends)
///   - Every character must be an ASCII lowercase letter, ASCII digit, or hyphen
///   - Must not start or end with a hyphen (a hyphen-only alias "---" is caught
///     by the character check and looks wrong regardless)
///
/// These rules are chosen so that the alias can be embedded safely in:
///   - SSH host aliases: "github.com-{alias}" needs to be a valid SSH hostname component
///   - Database `alias` columns: no escaping or quoting needed
///   - CLI display: no terminal special characters
///
/// Valid examples: "work", "personal", "client-acme", "freelance2024"
/// Invalid examples: "Work" (uppercase), "my_project" (underscore), "a" (too short)
pub fn validate_alias(alias: &str) -> Result<(), ValidationError> {
    if alias.len() < 2 || alias.len() > 50 {
        return Err(ValidationError::InvalidAlias {
            value: alias.to_string(),
            reason: format!(
                "must be 2–50 characters, got {}",
                alias.len()
            ),
        });
    }

    if alias.starts_with('-') || alias.ends_with('-') {
        return Err(ValidationError::InvalidAlias {
            value: alias.to_string(),
            reason: "must not start or end with a hyphen".to_string(),
        });
    }

    for ch in alias.chars() {
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '-' {
            return Err(ValidationError::InvalidAlias {
                value: alias.to_string(),
                reason: format!(
                    "character '{ch}' is not allowed — use only lowercase letters, digits, and hyphens"
                ),
            });
        }
    }

    Ok(())
}

/// Validates an email address with a pragmatic subset of RFC 5322 rules.
///
/// We deliberately do NOT implement full RFC 5322 validation because:
///   - Full RFC 5322 is surprisingly complex (allows quoted strings, comments, etc.)
///   - The email is only embedded in SSH key comments and platform API calls;
///     it is never used as a mail delivery address by this application
///   - False negatives (rejecting valid unusual emails) are worse than the
///     alternative of accepting a slightly malformed address that the platform
///     API will reject anyway
///
/// Rules we do enforce:
///   - Must not be empty
///   - Must contain exactly one '@' character
///   - The local part (before '@') must not be empty
///   - The domain part (after '@') must contain at least one '.'
///   - The domain part must not start or end with '.' or '-'
///   - The TLD (after the last '.') must be at least 2 characters
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    let email = email.trim();

    if email.is_empty() {
        return Err(ValidationError::InvalidEmail {
            value: email.to_string(),
            reason: "cannot be empty".to_string(),
        });
    }

    // Must have exactly one @
    let at_count = email.chars().filter(|&c| c == '@').count();
    if at_count != 1 {
        return Err(ValidationError::InvalidEmail {
            value: email.to_string(),
            reason: "must contain exactly one '@' character".to_string(),
        });
    }

    // Split at the single @
    let at_pos = email.find('@').unwrap();
    let local  = &email[..at_pos];
    let domain = &email[at_pos + 1..];

    if local.is_empty() {
        return Err(ValidationError::InvalidEmail {
            value: email.to_string(),
            reason: "the part before '@' cannot be empty".to_string(),
        });
    }

    if domain.is_empty() {
        return Err(ValidationError::InvalidEmail {
            value: email.to_string(),
            reason: "the part after '@' cannot be empty".to_string(),
        });
    }

    // Domain must contain at least one '.'
    if !domain.contains('.') {
        return Err(ValidationError::InvalidEmail {
            value: email.to_string(),
            reason: "domain must contain at least one '.' (e.g. 'github.com')".to_string(),
        });
    }

    // Domain must not start or end with '.' or '-'
    let domain_start = domain.chars().next().unwrap();
    let domain_end   = domain.chars().last().unwrap();
    if domain_start == '.' || domain_start == '-' || domain_end == '.' || domain_end == '-' {
        return Err(ValidationError::InvalidEmail {
            value: email.to_string(),
            reason: "domain cannot start or end with '.' or '-'".to_string(),
        });
    }

    // TLD must be at least 2 characters
    let tld = domain.rsplit('.').next().unwrap_or("");
    if tld.len() < 2 {
        return Err(ValidationError::InvalidEmail {
            value: email.to_string(),
            reason: "top-level domain must be at least 2 characters".to_string(),
        });
    }

    Ok(())
}

/// Validates a Git remote URL by confirming it matches one of the known formats.
///
/// Accepted formats:
///   - `git@hostname:owner/repo` or `git@hostname:owner/repo.git` (SSH SCP-like)
///   - `git@hostname-alias:owner/repo` (SSH with host alias)
///   - `ssh://git@hostname/owner/repo` (SSH URL)
///   - `https://hostname/owner/repo` (HTTPS)
///   - `http://hostname/owner/repo` (HTTP, legacy/internal servers)
///
/// This function is intentionally permissive about the host and path parts —
/// it only checks that the URL scheme is recognizable. Detailed structural
/// validation (owner/repo format) is performed by `parse_git_url()`.
pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    let url = url.trim();

    if url.is_empty() {
        return Err(ValidationError::InvalidUrl {
            value: url.to_string(),
            reason: "cannot be empty".to_string(),
        });
    }

    // SSH SCP-like format: git@host:path
    if url.starts_with("git@") {
        if !url.contains(':') {
            return Err(ValidationError::InvalidUrl {
                value: url.to_string(),
                reason: "SSH URL must contain ':' separating host from path (e.g. git@github.com:owner/repo)".to_string(),
            });
        }
        return Ok(());
    }

    // URL scheme formats
    if url.starts_with("https://")
        || url.starts_with("http://")
        || url.starts_with("ssh://")
        || url.starts_with("git://")
    {
        // Confirm there is something after the scheme
        let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or("");
        if after_scheme.is_empty() || !after_scheme.contains('/') {
            return Err(ValidationError::InvalidUrl {
                value: url.to_string(),
                reason: "URL must include a host and a path (e.g. https://github.com/owner/repo)".to_string(),
            });
        }
        return Ok(());
    }

    Err(ValidationError::InvalidUrl {
        value: url.to_string(),
        reason: "unrecognized URL format — expected git@host:path or https://host/path".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── alias tests ───────────────────────────────────────────────────────────

    #[test]
    fn alias_valid_lowercase_letters() {
        assert!(validate_alias("work").is_ok());
        assert!(validate_alias("personal").is_ok());
    }

    #[test]
    fn alias_valid_with_hyphens_and_digits() {
        assert!(validate_alias("client-acme").is_ok());
        assert!(validate_alias("freelance2024").is_ok());
        assert!(validate_alias("ab").is_ok()); // minimum length
    }

    #[test]
    fn alias_rejects_uppercase() {
        assert!(validate_alias("Work").is_err());
        assert!(validate_alias("WORK").is_err());
    }

    #[test]
    fn alias_rejects_underscore() {
        assert!(validate_alias("my_project").is_err());
    }

    #[test]
    fn alias_rejects_too_short() {
        assert!(validate_alias("a").is_err());
        assert!(validate_alias("").is_err());
    }

    #[test]
    fn alias_rejects_too_long() {
        let long = "a".repeat(51);
        assert!(validate_alias(&long).is_err());
    }

    #[test]
    fn alias_rejects_leading_or_trailing_hyphen() {
        assert!(validate_alias("-work").is_err());
        assert!(validate_alias("work-").is_err());
    }

    // ── email tests ───────────────────────────────────────────────────────────

    #[test]
    fn email_valid_common_formats() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("user+tag@subdomain.example.co.uk").is_ok());
    }

    #[test]
    fn email_rejects_missing_at() {
        assert!(validate_email("userexample.com").is_err());
    }

    #[test]
    fn email_rejects_multiple_at() {
        assert!(validate_email("user@@example.com").is_err());
    }

    #[test]
    fn email_rejects_missing_domain_dot() {
        assert!(validate_email("user@localhost").is_err());
    }

    #[test]
    fn email_rejects_empty() {
        assert!(validate_email("").is_err());
        assert!(validate_email("   ").is_err());
    }

    // ── URL tests ─────────────────────────────────────────────────────────────

    #[test]
    fn url_valid_ssh_scp_format() {
        assert!(validate_url("git@github.com:owner/repo.git").is_ok());
        assert!(validate_url("git@github.com-work:owner/repo").is_ok());
    }

    #[test]
    fn url_valid_https() {
        assert!(validate_url("https://github.com/owner/repo.git").is_ok());
        assert!(validate_url("https://github.com/owner/repo").is_ok());
    }

    #[test]
    fn url_rejects_empty() {
        assert!(validate_url("").is_err());
    }

    #[test]
    fn url_rejects_bare_hostname() {
        assert!(validate_url("github.com/owner/repo").is_err());
    }

    #[test]
    fn url_rejects_ssh_without_colon() {
        assert!(validate_url("git@github.com/owner/repo").is_err());
    }
}