"""Custom/Self-hosted Git platform integration."""

import subprocess
from typing import List, Dict, Optional
from .base import BasePlatform, Repository


class CustomPlatform(BasePlatform):
    """Custom/Self-hosted Git platform integration.
    
    Supports:
    - Generic custom Git platforms
    - Self-hosted Gitea, Gitolite, Gogs instances
    - SSH connection testing
    - Manual repository management
    """
    
    def __init__(self, api_base: Optional[str] = None, ssh_host: Optional[str] = None):
        super().__init__(api_base=api_base, ssh_host=ssh_host)
        self.platform_name = 'Custom'
    
    def fetch_repositories(self, account: Dict) -> List[Repository]:
        """
        Fetch repositories from custom platform.
        
        Custom platforms may not have a standard API, so this is a placeholder
        that can be overridden for specific implementations.
        """
        raise NotImplementedError(
            "Custom platforms require manual repository listing. "
            "Use manual URL entry instead."
        )
    
    def test_connection(self, account: Dict) -> bool:
        """Test connection to custom platform via SSH."""
        if not self.ssh_host:
            return False
        
        try:
            result = subprocess.run(
                ['ssh', '-T', f'git@{self.ssh_host}'],
                capture_output=True,
                text=True,
                timeout=5
            )
            return result.returncode == 0
        except Exception:
            return False
    
    def fork_repository(self, repo_url: str, account: Dict) -> Dict:
        """
        Fork a repository on custom platform.
        
        Most custom platforms don't support forking via API.
        """
        return {
            'success': False,
            'error': 'Forking is not supported for custom platforms. '
                     'Please fork manually on the platform website.'
        }
    
    def get_repository_info(self, owner: str, repo: str, account: Dict) -> Optional[Repository]:
        """Get information about a repository on custom platform."""
        # Custom platforms don't have a standard way to get repo info
        return None
    
    # ==================== Helper Methods ====================
    
    def test_ssh_connection(self, host: str, timeout: int = 5) -> bool:
        """Test SSH connection to a custom host.
        
        Args:
            host: SSH host address
            timeout: Connection timeout in seconds
            
        Returns:
            True if connection successful
        """
        try:
            result = subprocess.run(
                ['ssh', '-T', f'git@{host}'],
                capture_output=True,
                text=True,
                timeout=timeout
            )
            return result.returncode in [0, 1]  # 0 or 1 both indicate successful connection
        except Exception:
            return False
    
    def validate_custom_url(self, url: str) -> bool:
        """Validate if URL is a valid custom Git URL.
        
        Args:
            url: Repository URL to validate
            
        Returns:
            True if URL is valid
        """
        import re
        
        # SSH format: git@host:owner/repo.git
        ssh_pattern = r'^git@[^:]+:[^/]+/[^/]+(?:\.git)?$'
        if re.match(ssh_pattern, url):
            return True
        
        # HTTPS format: https://host/owner/repo.git
        https_pattern = r'^https?://[^/]+/[^/]+/[^/]+(?:\.git)?/?$'
        if re.match(https_pattern, url):
            return True
        
        return False
    
    def parse_custom_url(self, url: str) -> Optional[Dict]:
        """Parse a custom Git URL.
        
        Args:
            url: Repository URL
            
        Returns:
            Dictionary with parsed components or None
        """
        import re
        
        # SSH format: git@host:owner/repo.git
        ssh_pattern = r'^git@([^:]+):([^/]+)/([^/]+?)(?:\.git)?$'
        match = re.match(ssh_pattern, url)
        if match:
            host, owner, repo = match.groups()
            return {
                'type': 'ssh',
                'host': host,
                'owner': owner,
                'repo': repo,
                'full_name': f"{owner}/{repo}",
                'url': url
            }
        
        # HTTPS format: https://host/owner/repo.git
        https_pattern = r'^https?://([^/]+)/([^/]+)/([^/]+?)(?:\.git)?/?$'
        match = re.match(https_pattern, url)
        if match:
            host, owner, repo = match.groups()
            return {
                'type': 'https',
                'host': host,
                'owner': owner,
                'repo': repo,
                'full_name': f"{owner}/{repo}",
                'url': url
            }
        
        return None
    
    def extract_host_from_url(self, url: str) -> Optional[str]:
        """Extract host from a custom Git URL.
        
        Args:
            url: Repository URL
            
        Returns:
            Host address or None
        """
        parsed = self.parse_custom_url(url)
        return parsed.get('host') if parsed else None
