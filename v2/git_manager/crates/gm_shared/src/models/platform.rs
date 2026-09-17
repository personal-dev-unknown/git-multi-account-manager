//! Platform DTO and PlatformType enumeration.
//!
//! A platform is a Git hosting service: GitHub, GitLab, Bitbucket, etc.
//! Platforms are seed data — they are inserted into the database at first
//! run and rarely change. Plugins register themselves against a platform
//! by matching on `PlatformType`.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Serializable snapshot of a platform record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformDto {
    pub uuid:            Uuid,
    /// Internal slug: "github", "gitlab", "bitbucket", "azure_devops", "custom"
    pub name:            String,
    /// Display name shown to users: "GitHub", "GitLab", etc.
    pub display_name:    String,
    /// REST API base URL, e.g. "https://api.github.com"
    pub api_base_url:    Option<String>,
    /// Default SSH hostname, e.g. "github.com"
    pub ssh_host:        Option<String>,
    pub supports_oauth:  bool,
    pub supports_pat:    bool,
    pub supports_ssh:    bool,
    pub supports_https:  bool,
    pub is_active:       bool,
    pub is_self_hosted:  bool,
}

/// The known Git hosting platforms that provider plugins support.
///
/// `Custom(String)` accommodates unknown or user-defined platforms,
/// with the inner string containing the base domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformType {
    GitHub,
    GitLab,
    Bitbucket,
    AzureDevOps,
    SourceForge,
    /// Self-hosted Git servers (Gitea, Gogs, Gitolite, etc.).
    /// The inner string is the base domain: "git.mycompany.com".
    #[serde(rename = "self_hosted")]
    SelfHosted(String),
    /// Cloud storage providers (S3, GCS, Azure Blob).
    /// The inner string identifies the provider: "s3", "gcs", "azure".
    #[serde(rename = "cloud_storage")]
    CloudStorage(String),
    /// Local filesystem repositories.
    LocalPath,
    /// Any other platform not covered above.
    /// The inner string is the domain: "git.mycompany.com".
    #[serde(rename = "custom")]
    Custom(String),
}

impl PlatformType {
    /// Returns the internal slug used in database `name` column and plugin identifiers.
    pub fn slug(&self) -> &str {
        match self {
            PlatformType::GitHub           => "github",
            PlatformType::GitLab           => "gitlab",
            PlatformType::Bitbucket        => "bitbucket",
            PlatformType::AzureDevOps      => "azure_devops",
            PlatformType::SourceForge      => "sourceforge",
            PlatformType::SelfHosted(_)    => "self_hosted",
            PlatformType::CloudStorage(_)  => "cloud_storage",
            PlatformType::LocalPath        => "local_path",
            PlatformType::Custom(_)        => "custom",
        }
    }

    /// Returns the default SSH hostname for known platforms.
    /// Returns None for platforms that don't support SSH.
    pub fn default_ssh_host(&self) -> Option<&str> {
        match self {
            PlatformType::GitHub           => Some("github.com"),
            PlatformType::GitLab           => Some("gitlab.com"),
            PlatformType::Bitbucket        => Some("bitbucket.org"),
            PlatformType::AzureDevOps      => Some("ssh.dev.azure.com"),
            PlatformType::SourceForge      => Some("git.code.sf.net"),
            PlatformType::SelfHosted(d)    => Some(d.as_str()),
            PlatformType::CloudStorage(_)  => None,
            PlatformType::LocalPath        => None,
            PlatformType::Custom(d)        => Some(d.as_str()),
        }
    }

    /// Attempts to detect the platform type from a hostname string.
    /// Used during URL parsing to identify the platform without a database lookup.
    pub fn from_hostname(host: &str) -> Self {
        let h = host.trim_start_matches("www.");
        if h.contains("github.com")             { return PlatformType::GitHub; }
        if h.contains("gitlab.com")             { return PlatformType::GitLab; }
        if h.contains("bitbucket.org")          { return PlatformType::Bitbucket; }
        if h.contains("dev.azure.com")
            || h.contains("visualstudio.com")   { return PlatformType::AzureDevOps; }
        if h.contains("sourceforge.net")        { return PlatformType::SourceForge; }
        if h.contains("s3.amazonaws.com")
            || h.contains("storage.googleapis.com")
            || h.contains("blob.core.windows.net") { return PlatformType::CloudStorage("generic".to_string()); }
        PlatformType::Custom(h.to_string())
    }
}

impl std::fmt::Display for PlatformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformType::GitHub            => write!(f, "GitHub"),
            PlatformType::GitLab            => write!(f, "GitLab"),
            PlatformType::Bitbucket         => write!(f, "Bitbucket"),
            PlatformType::AzureDevOps       => write!(f, "Azure DevOps"),
            PlatformType::SourceForge       => write!(f, "SourceForge"),
            PlatformType::SelfHosted(d)     => write!(f, "Self-Hosted ({})", d),
            PlatformType::CloudStorage(p)   => write!(f, "Cloud Storage ({})", p),
            PlatformType::LocalPath         => write!(f, "Local Path"),
            PlatformType::Custom(d)         => write!(f, "Custom ({})", d),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hostname_detects_known_platforms() {
        assert_eq!(PlatformType::from_hostname("github.com"),  PlatformType::GitHub);
        assert_eq!(PlatformType::from_hostname("gitlab.com"),  PlatformType::GitLab);
        assert_eq!(PlatformType::from_hostname("bitbucket.org"), PlatformType::Bitbucket);
        assert_eq!(PlatformType::from_hostname("sourceforge.net"), PlatformType::SourceForge);
    }

    #[test]
    fn from_hostname_custom_for_unknown() {
        let result = PlatformType::from_hostname("git.mycompany.com");
        assert!(matches!(result, PlatformType::Custom(_)));
    }

    #[test]
    fn from_hostname_detects_cloud_storage() {
        let r = PlatformType::from_hostname("s3.amazonaws.com");
        assert!(matches!(r, PlatformType::CloudStorage(_)));
        let r = PlatformType::from_hostname("storage.googleapis.com");
        assert!(matches!(r, PlatformType::CloudStorage(_)));
    }

    #[test]
    fn default_ssh_host_returns_correct_values() {
        assert_eq!(PlatformType::GitHub.default_ssh_host(), Some("github.com"));
        assert_eq!(PlatformType::AzureDevOps.default_ssh_host(), Some("ssh.dev.azure.com"));
        assert_eq!(PlatformType::SourceForge.default_ssh_host(), Some("git.code.sf.net"));
        assert_eq!(PlatformType::CloudStorage("s3".to_string()).default_ssh_host(), None);
        assert_eq!(PlatformType::LocalPath.default_ssh_host(), None);
    }

    #[test]
    fn slug_matches_expected_values() {
        assert_eq!(PlatformType::GitHub.slug(), "github");
        assert_eq!(PlatformType::SourceForge.slug(), "sourceforge");
        assert_eq!(PlatformType::LocalPath.slug(), "local_path");
        assert_eq!(PlatformType::CloudStorage("s3".to_string()).slug(), "cloud_storage");
        assert_eq!(PlatformType::SelfHosted("gitea.local".to_string()).slug(), "self_hosted");
    }

    #[test]
    fn display_formats_cleanly() {
        assert_eq!(PlatformType::GitHub.to_string(), "GitHub");
        assert_eq!(PlatformType::SourceForge.to_string(), "SourceForge");
        assert_eq!(PlatformType::LocalPath.to_string(), "Local Path");
        assert_eq!(PlatformType::SelfHosted("gitea.local".to_string()).to_string(), "Self-Hosted (gitea.local)");
    }
}