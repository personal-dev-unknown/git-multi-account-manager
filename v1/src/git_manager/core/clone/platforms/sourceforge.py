"""SourceForge Platform Implementation."""

import requests
from typing import Optional, Dict, List
from .base import BasePlatform, Repository


class SourceForgePlatform(BasePlatform):
    """SourceForge platform implementation.
    
    Supports:
    - Repository fetching via SourceForge API
    - Project-based repository structure
    - SSH and HTTPS clone URLs
    - Public repositories
    """
    
    def __init__(self):
        """Initialize SourceForge platform."""
        super().__init__(
            api_base='https://sourceforge.net/api',
            ssh_host='git.code.sf.net'
        )
        self.session = requests.Session()
        self.timeout = 10
        self.platform_name = 'SourceForge'
    
    def fetch_repositories(self, account: Dict) -> List[Repository]:
        """Fetch user's repositories from SourceForge API."""
        
        repositories = []
        username = account.get('username')
        
        try:
            # Get user's projects
            api_url = f"{self.api_base}/projects?username={username}"
            response = self.session.get(
                api_url,
                timeout=self.timeout
            )
            response.raise_for_status()
            
            projects = response.json().get('projects', [])
            
            for proj in projects:
                repositories.append(Repository(
                    id=hash(proj.get('name', '')),
                    name=proj['name'],
                    full_name=f"{username}/{proj['name']}",
                    owner=username,
                    description=proj.get('summary', '') or '',
                    visibility='public',
                    language='',
                    size_kb=0,
                    stars=0,
                    updated_at=proj.get('created', ''),
                    ssh_url=f"git@git.code.sf.net:p/{proj['name']}/git.git",
                    https_url=f"https://git.code.sf.net/p/{proj['name']}/git.git",
                    web_url=proj.get('url', f"https://sourceforge.net/projects/{proj['name']}"),
                    is_fork=False,
                    default_branch='master'
                ))
        
        except requests.exceptions.RequestException as e:
            raise Exception(f"Failed to fetch SourceForge repositories: {str(e)}")
        
        return repositories
    
    def test_connection(self, account: Dict) -> bool:
        """Test SourceForge API connection."""
        try:
            username = account.get('username')
            if not username:
                return False
            
            response = self.session.get(
                f"{self.api_base}/projects?username={username}",
                timeout=self.timeout
            )
            return response.status_code == 200
        except Exception:
            return False
    
    def fork_repository(self, repo_url: str, account: Dict) -> Dict:
        """Fork a repository on SourceForge."""
        raise NotImplementedError("Fork not yet implemented for SourceForge")
    
    def get_repository_info(self, owner: str, repo: str, account: Dict) -> Optional[Repository]:
        """Get information about a specific SourceForge repository."""
        try:
            username = account.get('username')
            api_url = f"{self.api_base}/projects?username={username}"
            response = self.session.get(
                api_url,
                timeout=self.timeout
            )
            response.raise_for_status()
            
            projects = response.json().get('projects', [])
            for proj in projects:
                if proj['name'] == repo:
                    return Repository(
                        id=hash(proj.get('name', '')),
                        name=proj['name'],
                        full_name=f"{username}/{proj['name']}",
                        owner=username,
                        description=proj.get('summary', '') or '',
                        visibility='public',
                        language='',
                        size_kb=0,
                        stars=0,
                        updated_at=proj.get('created', ''),
                        ssh_url=f"git@git.code.sf.net:p/{proj['name']}/git.git",
                        https_url=f"https://git.code.sf.net/p/{proj['name']}/git.git",
                        web_url=proj.get('url', f"https://sourceforge.net/projects/{proj['name']}"),
                        is_fork=False,
                        default_branch='master'
                    )
            return None
        except Exception:
            return None
    
    # ==================== Helper Methods ====================
    
    def get_user_projects(self, username: str) -> List[Dict]:
        """Get all projects for a user.
        
        Args:
            username: SourceForge username
            
        Returns:
            List of project dictionaries
        """
        try:
            api_url = f"{self.api_base}/projects?username={username}"
            response = self.session.get(api_url, timeout=self.timeout)
            response.raise_for_status()
            return response.json().get('projects', [])
        except Exception as e:
            raise Exception(f"Failed to get user projects: {str(e)}")
    
    def get_project_details(self, project_name: str) -> Optional[Dict]:
        """Get detailed information about a project.
        
        Args:
            project_name: SourceForge project name
            
        Returns:
            Project details dictionary or None
        """
        try:
            api_url = f"{self.api_base}/projects/{project_name}"
            response = self.session.get(api_url, timeout=self.timeout)
            response.raise_for_status()
            return response.json()
        except Exception:
            return None
    
    def get_project_git_url(self, project_name: str, auth_type: str = 'https') -> str:
        """Get Git clone URL for a SourceForge project.
        
        Args:
            project_name: SourceForge project name
            auth_type: 'ssh' or 'https'
            
        Returns:
            Git clone URL
        """
        if auth_type == 'ssh':
            return f"git@git.code.sf.net:p/{project_name}/git.git"
        else:
            return f"https://git.code.sf.net/p/{project_name}/git.git"
