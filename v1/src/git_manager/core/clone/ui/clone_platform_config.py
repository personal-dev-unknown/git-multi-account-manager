"""Platform configurations for clone operations."""

from dataclasses import dataclass
from typing import Optional


@dataclass
class PlatformConfig:
    """Configuration for a Git platform."""
    platform_id: str
    name: str
    api_base: Optional[str]
    ssh_host: str
    supports_password: bool
    rate_limit: Optional[str]
    auth_header_format: str  # e.g., 'Authorization: token {token}'
    
    def get_auth_header(self, token: str) -> str:
        """Generate authentication header."""
        return self.auth_header_format.format(token=token)


# Platform configurations
GITHUB_CONFIG = PlatformConfig(
    platform_id='github',
    name='GitHub',
    api_base='https://api.github.com',
    ssh_host='github.com',
    supports_password=False,
    rate_limit='5000/hour',
    auth_header_format='Authorization: token {token}'
)

GITLAB_CONFIG = PlatformConfig(
    platform_id='gitlab',
    name='GitLab',
    api_base='https://gitlab.com/api/v4',
    ssh_host='gitlab.com',
    supports_password=True,
    rate_limit='600/minute',
    auth_header_format='PRIVATE-TOKEN: {token}'
)

BITBUCKET_CONFIG = PlatformConfig(
    platform_id='bitbucket',
    name='Bitbucket',
    api_base='https://api.bitbucket.org/2.0',
    ssh_host='bitbucket.org',
    supports_password=False,
    rate_limit='1000/hour',
    auth_header_format='Authorization: Basic {token}'
)

GITEA_CONFIG = PlatformConfig(
    platform_id='gitea',
    name='Gitea',
    api_base=None,  # User configures
    ssh_host=None,  # User configures
    supports_password=True,
    rate_limit=None,
    auth_header_format='Authorization: token {token}'
)

CUSTOM_CONFIG = PlatformConfig(
    platform_id='custom',
    name='Custom Git Server',
    api_base=None,
    ssh_host=None,
    supports_password=True,
    rate_limit=None,
    auth_header_format='Authorization: token {token}'
)

# Platform registry
PLATFORMS = {
    'github': GITHUB_CONFIG,
    'gitlab': GITLAB_CONFIG,
    'bitbucket': BITBUCKET_CONFIG,
    'gitea': GITEA_CONFIG,
    'custom': CUSTOM_CONFIG,
}


def get_platform_config(platform_id: str) -> PlatformConfig:
    """Get configuration for a platform."""
    if platform_id not in PLATFORMS:
        raise ValueError(f"Unknown platform: {platform_id}")
    return PLATFORMS[platform_id]


def get_platform_by_host(host: str) -> Optional[str]:
    """Get platform ID by SSH host."""
    host_lower = host.lower()
    
    for platform_id, config in PLATFORMS.items():
        if config.ssh_host and config.ssh_host.lower() == host_lower:
            return platform_id
    
    return None
