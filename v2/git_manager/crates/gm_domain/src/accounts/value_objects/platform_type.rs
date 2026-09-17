//! Platform type — which Git hosting service an account belongs to.

use gm_shared::models::platform::PlatformType as SharedPlatformType;

/// The hosting platform for a Git account.
/// Each variant drives which provider plugin handles API operations for accounts
/// of that type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlatformType {
    GitHub,
    GitLab,
    Bitbucket,
    AzureDevOps,
    SourceForge,
    /// Self-hosted Git servers (Gitea, Gogs, Gitolite, etc.).
    SelfHosted(String),
    /// Cloud storage providers (S3, GCS, Azure Blob).
    CloudStorage(String),
    /// Local filesystem repositories.
    LocalPath,
    /// Any self-hosted or unrecognised Git hosting platform.
    /// The inner string is the platform's domain name, e.g. "git.mycompany.com".
    Custom(String),
}

impl PlatformType {
    /// Returns the internal slug used as the platform's primary identifier
    /// in the database `platforms.name` column and in plugin registrations.
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

    /// Returns the canonical SSH hostname for this platform, or the custom
    /// domain for `Custom` variants. Used when generating SSH config entries.
    pub fn default_ssh_host(&self) -> &str {
        match self {
            PlatformType::GitHub           => "github.com",
            PlatformType::GitLab           => "gitlab.com",
            PlatformType::Bitbucket        => "bitbucket.org",
            PlatformType::AzureDevOps      => "ssh.dev.azure.com",
            PlatformType::SourceForge      => "git.code.sf.net",
            PlatformType::SelfHosted(d)    => d.as_str(),
            PlatformType::CloudStorage(_)  => "",
            PlatformType::LocalPath        => "",
            PlatformType::Custom(d)        => d.as_str(),
        }
    }

    /// Converts to the shared DTO variant.
    pub fn to_shared(&self) -> SharedPlatformType {
        match self {
            PlatformType::GitHub            => SharedPlatformType::GitHub,
            PlatformType::GitLab            => SharedPlatformType::GitLab,
            PlatformType::Bitbucket         => SharedPlatformType::Bitbucket,
            PlatformType::AzureDevOps       => SharedPlatformType::AzureDevOps,
            PlatformType::SourceForge       => SharedPlatformType::SourceForge,
            PlatformType::SelfHosted(d)     => SharedPlatformType::SelfHosted(d.clone()),
            PlatformType::CloudStorage(p)   => SharedPlatformType::CloudStorage(p.clone()),
            PlatformType::LocalPath         => SharedPlatformType::LocalPath,
            PlatformType::Custom(d)         => SharedPlatformType::Custom(d.clone()),
        }
    }

    /// Reconstructs from the shared DTO variant.
    pub fn from_shared(shared: &SharedPlatformType) -> Self {
        match shared {
            SharedPlatformType::GitHub            => PlatformType::GitHub,
            SharedPlatformType::GitLab            => PlatformType::GitLab,
            SharedPlatformType::Bitbucket         => PlatformType::Bitbucket,
            SharedPlatformType::AzureDevOps       => PlatformType::AzureDevOps,
            SharedPlatformType::SourceForge       => PlatformType::SourceForge,
            SharedPlatformType::SelfHosted(d)     => PlatformType::SelfHosted(d.clone()),
            SharedPlatformType::CloudStorage(p)   => PlatformType::CloudStorage(p.clone()),
            SharedPlatformType::LocalPath         => PlatformType::LocalPath,
            SharedPlatformType::Custom(d)         => PlatformType::Custom(d.clone()),
        }
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