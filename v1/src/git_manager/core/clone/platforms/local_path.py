"""Local Path Platform Implementation."""

from typing import Optional, Dict, List
from pathlib import Path
from .base import BasePlatform, Repository


class LocalPathPlatform(BasePlatform):
    """Local path platform implementation for local and network repositories."""
    
    def __init__(self):
        """Initialize Local Path platform."""
        super().__init__(
            api_base=None,
            ssh_host=None
        )
        self.platform_name = 'Local Path'
    
    def fetch_repositories(self, account: Dict) -> List[Repository]:
        """Fetch repositories from local path.
        
        Args:
            account: Account dictionary with 'username' as base path
            
        Returns:
            List of Repository objects
        """
        repositories = []
        
        try:
            base_path = Path(account.get('username', '.')).expanduser()
            
            if not base_path.exists():
                raise ValueError(f"Path does not exist: {base_path}")
            
            # Look for Git repositories in the path
            for item in base_path.iterdir():
                if item.is_dir() and (item / '.git').exists():
                    repositories.append(Repository(
                        id=hash(str(item)),
                        name=item.name,
                        full_name=item.name,
                        owner='local',
                        description=f"Local repository at {item}",
                        visibility='private',
                        language='',
                        size_kb=0,
                        stars=0,
                        updated_at='',
                        ssh_url='',
                        https_url=str(item),
                        web_url=str(item),
                        is_fork=False,
                        default_branch='main'
                    ))
        
        except Exception as e:
            raise Exception(f"Failed to fetch local repositories: {str(e)}")
        
        return repositories
    
    def test_connection(self, account: Dict) -> bool:
        """Test local path accessibility.
        
        Args:
            account: Account dictionary with 'username' as path
            
        Returns:
            True if path is accessible
        """
        try:
            path = Path(account.get('username', '.')).expanduser()
            return path.exists()
        except Exception:
            return False
    
    def fork_repository(self, repo_url: str, account: Dict) -> Dict:
        """Fork a repository (not applicable for local paths).
        
        Args:
            repo_url: Repository URL
            account: Account dictionary
            
        Returns:
            Error dictionary
        """
        return {
            'success': False,
            'error': 'Forking is not supported for local paths'
        }
    
    def get_repository_info(self, owner: str, repo: str, account: Dict) -> Optional[Repository]:
        """Get information about a specific local repository.
        
        Args:
            owner: Owner/directory name
            repo: Repository name
            account: Account dictionary
            
        Returns:
            Repository object or None
        """
        try:
            base_path = Path(account.get('username', '.')).expanduser()
            repo_path = base_path / repo
            
            if repo_path.exists() and (repo_path / '.git').exists():
                return Repository(
                    id=hash(str(repo_path)),
                    name=repo,
                    full_name=repo,
                    owner='local',
                    description=f"Local repository at {repo_path}",
                    visibility='private',
                    language='',
                    size_kb=0,
                    stars=0,
                    updated_at='',
                    ssh_url='',
                    https_url=str(repo_path),
                    web_url=str(repo_path),
                    is_fork=False,
                    default_branch='main'
                )
            return None
        except Exception:
            return None
    
    def get_clone_url(
        self,
        owner: str,
        repo: str,
        auth_type: str = "file",
        account_name: Optional[str] = None,
        base_path: Optional[str] = None
    ) -> str:
        """Get clone URL for local repository.
        
        Args:
            owner: Owner/directory name
            repo: Repository name
            auth_type: Not used for local paths
            account_name: Not used
            base_path: Base path for repository
            
        Returns:
            Local file path
        """
        if base_path:
            path = Path(base_path).expanduser() / repo
        else:
            path = Path.home() / owner / repo
        
        return str(path)
    
    def parse_url(self, url: str) -> Dict:
        """Parse local path URL.
        
        Args:
            url: Local path (absolute or relative)
            
        Returns:
            Dictionary with path and other info
        """
        import re
        
        # Expand user home directory
        expanded_url = url.replace('~', str(Path.home()))
        
        # Handle file:// protocol
        if url.startswith('file://'):
            expanded_url = url[7:]
        
        # Validate path exists or parent exists
        path = Path(expanded_url).expanduser()
        
        if not path.exists() and not path.parent.exists():
            raise ValueError(f"Invalid local path: {url}")
        
        # Extract repository name from path
        repo_name = path.name
        owner = path.parent.name if path.parent != path.parent.parent else 'local'
        
        return {
            'path': str(path),
            'owner': owner,
            'repo': repo_name,
            'full_name': f"{owner}/{repo_name}",
            'type': 'local'
        }
    
    def validate_path(self, path: str) -> bool:
        """Validate if path is accessible.
        
        Args:
            path: Local path to validate
            
        Returns:
            True if path is valid and accessible
        """
        try:
            expanded_path = Path(path).expanduser()
            return expanded_path.exists() or expanded_path.parent.exists()
        except (ValueError, OSError):
            return False
    
    def is_git_repository(self, path: str) -> bool:
        """Check if path is a Git repository.
        
        Args:
            path: Local path to check
            
        Returns:
            True if path contains a .git directory
        """
        try:
            repo_path = Path(path).expanduser()
            return (repo_path / '.git').exists()
        except (ValueError, OSError):
            return False
