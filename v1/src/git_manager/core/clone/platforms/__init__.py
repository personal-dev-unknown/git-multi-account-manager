"""Platform integrations for clone operations."""

from .base import BasePlatform
from .github import GitHubPlatform
from .gitlab import GitLabPlatform
from .bitbucket import BitbucketPlatform
from .azure_devops import AzureDevOpsPlatform
from .sourceforge import SourceForgePlatform
from .self_hosted import SelfHostedPlatform
from .cloud_storage import CloudStoragePlatform
from .local_path import LocalPathPlatform
from .custom import CustomPlatform

__all__ = [
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
]
