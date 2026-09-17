"""Repository Manager Module - Consolidated repository management and setup.

This module consolidates all repository management functionality including:
- Repository tracking and metadata
- Repository initialization and setup
- Git configuration management
- Integration with Clone, SSH, and Sync modules

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

from .manager import RepositoryManager
from .git_config import GitConfig

__all__ = [
    'RepositoryManager',
    'GitConfig',
]
