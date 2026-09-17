//! Git remote URL parser.
//!
//! Git URLs appear in several incompatible formats across the ecosystem. This
//! module normalises all of them into a single `GitUrl` struct so that the rest
//! of the system can work with typed fields (host, owner, repo) rather than
//! performing ad-hoc string slicing everywhere a URL is needed.
//!
//! The four formats handled:
//!
//! 1. **SSH SCP-like** — `git@github.com:owner/repo.git`
//!    The most common format for SSH access. The colon separates the host from
//!    the path. No slashes before the colon. The `.git` suffix is optional.
//!
//! 2. **SSH SCP with host alias** — `git@github.com-work:owner/repo.git`
//!    Identical structure to format 1, but the hostname portion is an alias
//!    declared in `~/.ssh/config` (e.g. `Host github.com-work`). The alias
//!    encodes the real host plus an account discriminator.
//!
//! 3. **SSH URL** — `ssh://git@github.com/owner/repo.git`
//!    Less common. Uses slashes throughout. The `git@` user prefix is present.
//!
//! 4. **HTTPS URL** — `https://github.com/owner/repo.git`
//!    Used for HTTPS-PAT authentication. No SSH key involved.

use crate::models::platform::PlatformType;
use crate::validation::validators::ValidationError;

/// A parsed Git remote URL decomposed into its structural components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitUrl {
    /// The transport protocol used in the URL.
    pub protocol: GitProtocol,

    /// The actual hostname of the Git hosting platform, e.g. `"github.com"`.
    /// For SSH-alias URLs this is the real host extracted from the alias
    /// by stripping the account suffix (e.g. `"github.com-work"` → `"github.com"`).
    /// For HTTPS and non-alias SSH URLs this is the literal hostname.
    pub host: String,

    /// The SSH host alias if the URL used one (e.g. `"github.com-work"`).
    /// For HTTPS URLs and plain SSH URLs without an alias this is `None`.
    /// The host alias is what appears in `~/.ssh/config` Host blocks.
    pub host_alias: Option<String>,

    /// The repository owner (GitHub user or organization, GitLab namespace, etc.).
    /// E.g. `"shakamoses"` in `git@github.com:shakamoses/project.git`.
    pub owner: String,

    /// The repository name, without the `.git` suffix.
    /// E.g. `"project"` in `git@github.com:shakamoses/project.git`.
    pub repo: String,

    /// The detected platform type, derived from the hostname.
    pub platform: PlatformType,
}

/// The transport protocol used in the URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitProtocol {
    /// SSH key-based authentication. Covers both the SCP-like format
    /// (`git@host:path`) and the `ssh://` URL format.
    Ssh,
    /// HTTPS with username/password or PAT authentication.
    Https,
    /// Unauthenticated git protocol. Rarely used for push access,
    /// mostly for read-only public repositories on self-hosted servers.
    Git,
}

/// Parses a Git remote URL into its structural components.
///
/// Returns `Err(ValidationError::InvalidUrl)` for strings that are clearly
/// not a Git URL (empty, unrecognised scheme, missing owner/repo separator).
/// Returns `Ok(GitUrl)` for any URL that can be unambiguously decomposed.
pub fn parse_git_url(url: &str) -> Result<GitUrl, ValidationError> {
    let url = url.trim();

    if url.is_empty() {
        return Err(ValidationError::InvalidUrl {
            value: url.to_string(),
            reason: "URL cannot be empty".to_string(),
        });
    }

    // Branch on the prefix to choose the correct parser.
    if url.starts_with("git@") {
        parse_ssh_scp(url)
    } else if url.starts_with("ssh://") {
        parse_ssh_url(url)
    } else if url.starts_with("https://") {
        parse_https_url(url)
    } else if url.starts_with("http://") {
        parse_http_url(url)
    } else if url.starts_with("git://") {
        parse_git_protocol_url(url)
    } else {
        Err(ValidationError::InvalidUrl {
            value: url.to_string(),
            reason: "unrecognised URL scheme — expected git@, ssh://, https://, or git://".to_string(),
        })
    }
}

// ── SSH SCP-like format: git@host:owner/repo[.git] ───────────────────────────

fn parse_ssh_scp(url: &str) -> Result<GitUrl, ValidationError> {
    // Strip the "git@" prefix
    let after_at = &url[4..]; // safe: url starts with "git@" (4 bytes, all ASCII)

    // The colon separates host (or host alias) from the path
    let colon_pos = after_at.find(':').ok_or_else(|| ValidationError::InvalidUrl {
        value: url.to_string(),
        reason: "SSH URL is missing ':' between host and path".to_string(),
    })?;

    let host_part = &after_at[..colon_pos];
    let path_part = &after_at[colon_pos + 1..];

    if host_part.is_empty() {
        return Err(ValidationError::InvalidUrl {
            value: url.to_string(),
            reason: "hostname portion is empty".to_string(),
        });
    }

    let (owner, repo) = split_owner_repo(url, path_part)?;
    let (host, host_alias) = resolve_ssh_host(host_part);
    let platform = PlatformType::from_hostname(&host);

    Ok(GitUrl {
        protocol: GitProtocol::Ssh,
        host,
        host_alias,
        owner,
        repo,
        platform,
    })
}

// ── SSH URL format: ssh://git@host/owner/repo[.git] ──────────────────────────

fn parse_ssh_url(url: &str) -> Result<GitUrl, ValidationError> {
    // Strip "ssh://"
    let rest = &url[6..];

    // Optionally strip "git@" user prefix
    let rest = rest.strip_prefix("git@").unwrap_or(rest);

    // The first '/' separates host from path
    let slash_pos = rest.find('/').ok_or_else(|| ValidationError::InvalidUrl {
        value: url.to_string(),
        reason: "SSH URL is missing '/' between host and path".to_string(),
    })?;

    let host_part = &rest[..slash_pos];
    let path_part = &rest[slash_pos + 1..];

    let (owner, repo) = split_owner_repo(url, path_part)?;
    let (host, host_alias) = resolve_ssh_host(host_part);
    let platform = PlatformType::from_hostname(&host);

    Ok(GitUrl {
        protocol: GitProtocol::Ssh,
        host,
        host_alias,
        owner,
        repo,
        platform,
    })
}

// ── HTTPS URL format: https://host/owner/repo[.git] ──────────────────────────

fn parse_https_url(url: &str) -> Result<GitUrl, ValidationError> {
    parse_url_with_scheme(url, "https://", GitProtocol::Https)
}

fn parse_http_url(url: &str) -> Result<GitUrl, ValidationError> {
    parse_url_with_scheme(url, "http://", GitProtocol::Https)
}

fn parse_git_protocol_url(url: &str) -> Result<GitUrl, ValidationError> {
    parse_url_with_scheme(url, "git://", GitProtocol::Git)
}

fn parse_url_with_scheme(
    url:      &str,
    scheme:   &str,
    protocol: GitProtocol,
) -> Result<GitUrl, ValidationError> {
    let rest = &url[scheme.len()..];

    // Optional "user@" prefix (HTTPS URLs may include credentials in the URL)
    let rest = if let Some(at_pos) = rest.find('@') {
        &rest[at_pos + 1..]
    } else {
        rest
    };

    // First '/' separates host from path
    let slash_pos = rest.find('/').ok_or_else(|| ValidationError::InvalidUrl {
        value: url.to_string(),
        reason: "URL is missing path component after hostname".to_string(),
    })?;

    let host = rest[..slash_pos].to_string();
    let path_part = &rest[slash_pos + 1..];

    if host.is_empty() {
        return Err(ValidationError::InvalidUrl {
            value: url.to_string(),
            reason: "hostname is empty".to_string(),
        });
    }

    let (owner, repo) = split_owner_repo(url, path_part)?;
    let platform = PlatformType::from_hostname(&host);

    Ok(GitUrl {
        protocol,
        host: host.to_string(),
        host_alias: None,
        owner,
        repo,
        platform,
    })
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Splits a path like "owner/repo.git" or "owner/repo" into (owner, repo).
/// Strips the `.git` suffix from the repo name if present.
fn split_owner_repo(original_url: &str, path: &str) -> Result<(String, String), ValidationError> {
    // Some platforms include a leading slash in the path — strip it.
    let path = path.trim_start_matches('/');

    let slash_pos = path.find('/').ok_or_else(|| ValidationError::InvalidUrl {
        value: original_url.to_string(),
        reason: "URL path must be in the form 'owner/repository'".to_string(),
    })?;

    let owner = &path[..slash_pos];
    let repo_raw = &path[slash_pos + 1..];

    if owner.is_empty() {
        return Err(ValidationError::InvalidUrl {
            value: original_url.to_string(),
            reason: "repository owner is missing".to_string(),
        });
    }

    if repo_raw.is_empty() {
        return Err(ValidationError::InvalidUrl {
            value: original_url.to_string(),
            reason: "repository name is missing".to_string(),
        });
    }

    // Strip the .git suffix — Git Manager stores the canonical name without it.
    let repo = repo_raw.strip_suffix(".git").unwrap_or(repo_raw);

    if repo.is_empty() {
        return Err(ValidationError::InvalidUrl {
            value: original_url.to_string(),
            reason: "repository name is empty after stripping .git suffix".to_string(),
        });
    }

    Ok((owner.to_string(), repo.to_string()))
}

/// Determines whether a hostname is a plain host or an SSH host alias.
///
/// The heuristic: if the hostname portion contains a hyphen in a segment that
/// looks like an account discriminator suffix on a known platform hostname
/// (e.g. "github.com-work"), we treat everything as a host alias and infer
/// the real hostname by taking the part up to and including the first ".com",
/// ".org", etc. segment.
///
/// For unambiguous aliases like "github.com-work":
///   → real host: "github.com", alias: Some("github.com-work")
///
/// For plain hostnames like "github.com" or "git.mycompany.com":
///   → real host: same string, alias: None
///
/// Returns (real_host, optional_alias).
fn resolve_ssh_host(host_part: &str) -> (String, Option<String>) {
    // Known platforms and their SSH hostnames. If the host_part starts with
    // one of these known prefixes and has additional content after the TLD,
    // it is a host alias.
    const KNOWN_SSH_HOSTS: &[&str] = &[
        "github.com",
        "gitlab.com",
        "bitbucket.org",
        "ssh.dev.azure.com",
        "git.code.sf.net",
    ];

    for &known in KNOWN_SSH_HOSTS {
        if host_part.starts_with(known) && host_part.len() > known.len() {
            // Additional characters after the known hostname — this is a host alias
            // like "github.com-work" where known = "github.com" and the suffix is "-work".
            let suffix = &host_part[known.len()..];
            if suffix.starts_with('-') {
                // Confirmed alias: the suffix starts with a hyphen separator.
                return (known.to_string(), Some(host_part.to_string()));
            }
        }
    }

    // Not a recognised alias pattern — treat as-is.
    (host_part.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_github_ssh_scp() {
        let parsed = parse_git_url("git@github.com:shakamoses/project.git").unwrap();
        assert_eq!(parsed.protocol, GitProtocol::Ssh);
        assert_eq!(parsed.host, "github.com");
        assert_eq!(parsed.host_alias, None);
        assert_eq!(parsed.owner, "shakamoses");
        assert_eq!(parsed.repo, "project");
        assert_eq!(parsed.platform, PlatformType::GitHub);
    }

    #[test]
    fn parses_ssh_host_alias() {
        let parsed = parse_git_url("git@github.com-work:shakamoses/project.git").unwrap();
        assert_eq!(parsed.host, "github.com");
        assert_eq!(parsed.host_alias, Some("github.com-work".to_string()));
        assert_eq!(parsed.owner, "shakamoses");
        assert_eq!(parsed.repo, "project");
    }

    #[test]
    fn parses_ssh_without_git_suffix() {
        let parsed = parse_git_url("git@github.com:owner/repo").unwrap();
        assert_eq!(parsed.repo, "repo");
    }

    #[test]
    fn parses_https_url() {
        let parsed = parse_git_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(parsed.protocol, GitProtocol::Https);
        assert_eq!(parsed.host, "github.com");
        assert_eq!(parsed.host_alias, None);
        assert_eq!(parsed.owner, "owner");
        assert_eq!(parsed.repo, "repo");
    }

    #[test]
    fn parses_ssh_url_format() {
        let parsed = parse_git_url("ssh://git@gitlab.com/owner/repo.git").unwrap();
        assert_eq!(parsed.protocol, GitProtocol::Ssh);
        assert_eq!(parsed.host, "gitlab.com");
        assert_eq!(parsed.platform, PlatformType::GitLab);
    }

    #[test]
    fn parses_gitlab_ssh() {
        let parsed = parse_git_url("git@gitlab.com:owner/repo.git").unwrap();
        assert_eq!(parsed.platform, PlatformType::GitLab);
    }

    #[test]
    fn rejects_empty_url() {
        assert!(parse_git_url("").is_err());
    }

    #[test]
    fn rejects_ssh_without_colon() {
        assert!(parse_git_url("git@github.com/owner/repo").is_err());
    }

    #[test]
    fn rejects_missing_repo() {
        assert!(parse_git_url("git@github.com:owner/").is_err());
    }
}