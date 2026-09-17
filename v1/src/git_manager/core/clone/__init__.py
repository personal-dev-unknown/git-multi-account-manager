"""Clone repository system for GitManager.

Supports all 8 Git platforms:
- GitHub
- GitLab
- Bitbucket
- Azure DevOps
- Self-Hosted
- Cloud Storage
- Local Path
- SourceForge
"""

from .workflow import CloneWorkflow
from .parsers import URLParser, ParsedURL
from .errors import (
    CloneError,
    AuthenticationError,
    PermissionError,
    RepositoryNotFoundError,
    NetworkError,
    DiskSpaceError,
    InvalidURLError,
    PlatformError,
    SSHError,
    APIError,
)
from .cache import RepositoryCache
from .platforms import (
    BasePlatform, GitHubPlatform, GitLabPlatform, BitbucketPlatform,
    AzureDevOpsPlatform, SourceForgePlatform, SelfHostedPlatform,
    CloudStoragePlatform, LocalPathPlatform, CustomPlatform
)
from .auth import SSHAuth, HTTPSPATAuth, HTTPSPasswordAuth, AnonymousAuth

__all__ = [
    'CloneWorkflow',
    'URLParser',
    'ParsedURL',
    'RepositoryCache',
    'BasePlatform',
    'GitHubPlatform',
    'GitLabPlatform',
    'BitbucketPlatform',
    'AzureDevOpsPlatform',
    'SourceForgePlatform',
    'SelfHostedPlatform',
    'CloudStoragePlatform',
    'LocalPathPlatform',
    'CustomPlatform',
    'SSHAuth',
    'HTTPSPATAuth',
    'HTTPSPasswordAuth',
    'AnonymousAuth',
    'CloneError',
    'AuthenticationError',
    'PermissionError',
    'RepositoryNotFoundError',
    'NetworkError',
    'DiskSpaceError',
    'InvalidURLError',
    'PlatformError',
    'SSHError',
    'APIError',
]
