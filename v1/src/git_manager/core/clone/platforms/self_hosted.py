"""Self-Hosted Git Platform Implementation."""

import requests
from typing import Optional, Dict, List
from .base import BasePlatform, Repository


class SelfHostedPlatform(BasePlatform):
    """Self-hosted Git server implementation (Gitea, Gitolite, Gogs, etc.)."""
    
    def __init__(self):
        """Initialize Self-Hosted platform."""
        super().__init__(
            api_base=None,  # Custom per instance
            ssh_host=None  # Custom per instance
        )
        self.session = requests.Session()
        self.timeout = 10
    
    def fetch_repositories(self, account: Dict) -> List[Repository]:
        """Fetch user's repositories from self-hosted server."""
        
        # Self-hosted servers vary widely, so this is a generic implementation
        # Actual implementation depends on the specific Git server software
        repositories = []
        
        try:
            # This would need to be customized based on the actual server
            # (Gitea, Gitolite, Gogs, etc.)
            # For now, return empty list as self-hosted requires custom configuration
            pass
        except Exception as e:
            raise Exception(f"Failed to fetch self-hosted repositories: {str(e)}")
        
        return repositories
    
    def test_connection(self, account: Dict) -> bool:
        """Test self-hosted server connection."""
        try:
            host = account.get('host')
            if not host:
                return False
            
            response = self.session.get(
                f"https://{host}/api/v1/version",
                timeout=self.timeout
            )
            return response.status_code == 200
        except Exception:
            return False
    
    def fork_repository(self, repo_url: str, account: Dict) -> Dict:
        """Fork a repository on self-hosted server."""
        raise NotImplementedError("Fork not yet implemented for self-hosted")
    
    def get_repository_info(self, owner: str, repo: str, account: Dict) -> Optional[Repository]:
        """Get information about a specific self-hosted repository."""
        try:
            host = account.get('host')
            if not host:
                return None
            
            # Try Gitea API format
            response = self.session.get(
                f"https://{host}/api/v1/repos/{owner}/{repo}",
                timeout=self.timeout
            )
            
            if response.status_code == 200:
                repo_data = response.json()
                return Repository(
                    id=repo_data.get('id', 0),
                    name=repo_data.get('name', repo),
                    full_name=f"{owner}/{repo}",
                    owner=owner,
                    description=repo_data.get('description', '') or '',
                    visibility='private' if repo_data.get('private', True) else 'public',
                    language=repo_data.get('language', '') or '',
                    size_kb=repo_data.get('size', 0),
                    stars=repo_data.get('stars_count', 0),
                    updated_at=repo_data.get('updated_at', ''),
                    ssh_url=f"git@{host}:{owner}/{repo}.git",
                    https_url=f"https://{host}/{owner}/{repo}.git",
                    web_url=repo_data.get('html_url', f"https://{host}/{owner}/{repo}"),
                    is_fork=repo_data.get('fork', False),
                    default_branch=repo_data.get('default_branch', 'main')
                )
            return None
        except Exception:
            return None
