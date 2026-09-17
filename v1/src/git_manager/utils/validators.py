# src/git_manager/utils/validators.py
"""Input validators."""

import re
from pathlib import Path
from typing import Optional, Tuple

from .constants import Platform

class Validators:
    """Collection of validation functions."""

    @classmethod
    def validate_username(cls, username: str) -> bool:
        """Validate Git username.
        
        Args:
            username: Username to validate
            
        Returns:
            True if valid
        """
        if not username or not isinstance(username, str):
            return False
        pattern = r'^[a-zA-Z0-9]([a-zA-Z0-9-]){0,38}$'
        return bool(re.match(pattern, username)) and len(username) <= 39


    @classmethod
    def validate_email(cls, email: str) -> bool:
        """Validate email address.  
        
        Args:
            email: Email to validate
            
        Returns:
            True if valid
        """
        if not email or not isinstance(email, str):
            return False
        pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
        return bool(re.match(pattern, email)) and len(email) <= 254


    @classmethod
    def validate_ssh_key(cls, key_path: str) -> bool:
        """Validate SSH key path.
        
        Args:
            key_path: Path to SSH key
            
        Returns:
            True if key exists
        """
        path = Path(key_path).expanduser()
        return path.exists() and path.is_file()


    @classmethod
    def validate_url(cls, url: str) -> bool:
        """Validate repository URL.
        
        Args:
            url: Repository URL
            
        Returns:
            True if valid format
        """
        if not url or not isinstance(url, str):
            return False
        
        patterns = [
            # GitHub
            r'^https://github\.com/[^/]+/[^/]+/?$',
            r'^git@github\.com:[^/]+/[^/]+\.git$',
            r'^git@github\.com-[^:]+:[^/]+/[^/]+\.git$',
            # GitLab
            r'^https://gitlab\.com/[^/]+/[^/]+/?$',
            r'^git@gitlab\.com:[^/]+/[^/]+\.git$',
            r'^git@gitlab\.com-[^:]+:[^/]+/[^/]+\.git$',
            # Bitbucket
            r'^https://bitbucket\.org/[^/]+/[^/]+/?$',
            r'^git@bitbucket\.org:[^/]+/[^/]+\.git$',
            r'^git@bitbucket\.org-[^:]+:[^/]+/[^/]+\.git$',
            # Azure DevOps
            r'^https://dev\.azure\.com/[^/]+/[^/]+/_git/[^/]+/?$',
            r'^git@ssh\.dev\.azure\.com:[^/]+/[^/]+/_git/[^/]+\.git$',
            # SourceForge
            r'^https://git\.code\.sf\.net/p/[^/]+/[^/]+/?$',
            r'^git@git\.code\.sf\.net:[^/]+/[^/]+\.git$',
            # Self-hosted (generic SSH and HTTPS)
            r'^https://[^/]+/[^/]+/[^/]+/?$',
            r'^git@[^:]+:[^/]+/[^/]+\.git$',
            # Local paths
            r'^/[^/]+.*$',
            r'^~/.*$',
            r'^file://.*$',
            # Short format (username/repo)
            r'^[^/]+/[^/]+$'
        ]
        
        return any(re.match(pattern, url) for pattern in patterns)

    def validate_path(path: str) -> bool:
        """Validate path.
        
        Args:
            path: Path to validate
            
        Returns:
            True if valid
        """
        if not path or not isinstance(path, str):
            return False
        return Path(path).expanduser().exists()

    def validate_port(port: int) -> bool:
        """Validate port.
        
        Args:
            port: Port to validate
            
        Returns:
            True if valid
        """
        if not port or not isinstance(port, int):
            return False
        return 0 <= port <= 65535

    def extract_repo_info(url: str) -> Optional[Tuple[str, str, Platform]]:
        """Extract username, repo, and platform from URL."""
        patterns = {
            Platform.GITHUB: [
                (r'^https://github\.com/([^/]+)/([^/]+)/?$', None),
                (r'^git@github\.com:([^/]+)/(.+)\.git$', None),
                (r'^git@github\.com-[^:]+:([^/]+)/(.+)\.git$', None),
            ],
            Platform.GITLAB: [
                (r'^https://gitlab\.com/([^/]+)/([^/]+)/?$', None),
                (r'^git@gitlab\.com:([^/]+)/(.+)\.git$', None),
                (r'^git@gitlab\.com-[^:]+:([^/]+)/(.+)\.git$', None),
            ],
            Platform.BITBUCKET: [
                (r'^https://bitbucket\.org/([^/]+)/([^/]+)/?$', None),
                (r'^git@bitbucket\.org:([^/]+)/(.+)\.git$', None),
                (r'^git@bitbucket\.org-[^:]+:([^/]+)/(.+)\.git$', None),
            ],
            Platform.AZURE_DEVOPS: [
                (r'^https://dev\.azure\.com/([^/]+)/[^/]+/_git/([^/]+)/?$', None),
                (r'^git@ssh\.dev\.azure\.com:([^/]+)/[^/]+/_git/(.+)\.git$', None),
            ],
            Platform.SOURCEFORGE: [
                (r'^https://git\.code\.sf\.net/p/([^/]+)/([^/]+)/?$', None),
                (r'^git@git\.code\.sf\.net:([^/]+)/(.+)\.git$', None),
            ]
        }
        
        for platform, platform_patterns in patterns.items():
            for pattern, _ in platform_patterns:
                match = re.match(pattern, url)
                if match:
                    username, repo = match.groups()
                    repo = repo.replace('.git', '')
                    return username, repo, platform
        
        # Short format (username/repo)
        match = re.match(r'^([^/]+)/([^/]+)$', url)
        if match:
            username, repo = match.groups()
            return username, repo, None  # Platform needs to be specified
        
        return None
