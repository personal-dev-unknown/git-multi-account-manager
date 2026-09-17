"""URL parser for Git repositories."""

import re
from typing import Dict, Optional
from dataclasses import dataclass


@dataclass
class ParsedURL:
    """Parsed Git repository URL."""
    format: str  # 'ssh', 'https', 'short'
    host: str
    owner: str
    repo: str
    username: Optional[str] = None
    
    def to_ssh(self) -> str:
        """Convert to SSH URL."""
        return f"git@{self.host}:{self.owner}/{self.repo}.git"
    
    def to_https(self) -> str:
        """Convert to HTTPS URL."""
        return f"https://{self.host}/{self.owner}/{self.repo}.git"
    
    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            'format': self.format,
            'host': self.host,
            'owner': self.owner,
            'repo': self.repo,
            'username': self.username
        }


class URLParser:
    """Parse and normalize Git repository URLs."""
    
    @staticmethod
    def parse(url: str) -> ParsedURL:
        """
        Parse any Git URL format.
        
        Supported formats:
        - github.com/user/repo
        - https://github.com/user/repo
        - https://github.com/user/repo.git
        - git@github.com:user/repo.git
        - user@git.company.com:repo.git
        """
        
        url = url.strip().rstrip('/')
        
        # SSH format: git@host:owner/repo.git or user@host:owner/repo.git
        ssh_pattern = r'(?:git@|([^@]+)@)([^:]+):([^/]+)/(.+?)(?:\.git)?$'
        ssh_match = re.match(ssh_pattern, url)
        if ssh_match:
            username, host, owner, repo = ssh_match.groups()
            return ParsedURL(
                format='ssh',
                host=host,
                owner=owner,
                repo=repo.replace('.git', ''),
                username=username or 'git'
            )
        
        # HTTPS format: https://host/owner/repo.git or https://host/owner/repo
        https_pattern = r'https?://([^/]+)/([^/]+)/(.+?)(?:\.git)?$'
        https_match = re.match(https_pattern, url)
        if https_match:
            host, owner, repo = https_match.groups()
            return ParsedURL(
                format='https',
                host=host,
                owner=owner,
                repo=repo.replace('.git', '')
            )
        
        # Shorthand: host/owner/repo or host/owner/repo.git
        short_pattern = r'([^/]+\.[^/]+)/([^/]+)/(.+?)(?:\.git)?$'
        short_match = re.match(short_pattern, url)
        if short_match:
            host, owner, repo = short_match.groups()
            return ParsedURL(
                format='short',
                host=host,
                owner=owner,
                repo=repo.replace('.git', '')
            )
        
        raise ValueError(f"Invalid Git URL: {url}")
    
    @staticmethod
    def detect_platform(url: str) -> str:
        """
        Auto-detect platform from URL.
        
        Returns: 'github', 'gitlab', 'bitbucket', 'gitea', or 'custom'
        """
        url_lower = url.lower()
        
        if 'github.com' in url_lower:
            return 'github'
        elif 'gitlab.com' in url_lower:
            return 'gitlab'
        elif 'bitbucket.org' in url_lower:
            return 'bitbucket'
        elif 'gitea' in url_lower:
            return 'gitea'
        elif 'gitlab' in url_lower:
            return 'gitlab_selfhosted'
        else:
            return 'custom'
    
    @staticmethod
    def normalize_url(url: str, format: str = 'ssh') -> str:
        """
        Normalize URL to specified format.
        
        Args:
            url: Original URL
            format: Target format ('ssh' or 'https')
        
        Returns:
            Normalized URL
        """
        parsed = URLParser.parse(url)
        
        if format == 'ssh':
            return parsed.to_ssh()
        elif format == 'https':
            return parsed.to_https()
        else:
            raise ValueError(f"Invalid format: {format}")
