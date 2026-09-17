"""Base platform class for Git service integrations."""

from abc import ABC, abstractmethod
from typing import List, Dict, Optional
from dataclasses import dataclass


@dataclass
class Repository:
    """Normalized repository data."""
    id: int
    name: str
    full_name: str
    owner: str
    description: str
    visibility: str  # 'private' or 'public'
    language: str
    size_kb: int
    stars: int
    updated_at: str
    ssh_url: str
    https_url: str
    web_url: str
    is_fork: bool
    default_branch: str


class BasePlatform(ABC):
    """Base class for platform integrations."""
    
    def __init__(self, api_base: Optional[str] = None, ssh_host: Optional[str] = None):
        self.api_base = api_base
        self.ssh_host = ssh_host
    
    @abstractmethod
    def fetch_repositories(self, account: Dict) -> List[Repository]:
        """Fetch user's repositories from platform API."""
        pass
    
    @abstractmethod
    def test_connection(self, account: Dict) -> bool:
        """Test API connection."""
        pass
    
    @abstractmethod
    def fork_repository(self, repo_url: str, account: Dict) -> Dict:
        """Fork a repository."""
        pass
    
    @abstractmethod
    def get_repository_info(self, owner: str, repo: str, account: Dict) -> Optional[Repository]:
        """Get information about a specific repository."""
        pass
    
    # ==================== Utility Methods ====================
    
    def filter_repositories(
        self,
        repositories: List[Repository],
        visibility: Optional[str] = None,
        language: Optional[str] = None,
        min_stars: int = 0,
        search_term: Optional[str] = None
    ) -> List[Repository]:
        """Filter repositories by various criteria.
        
        Args:
            repositories: List of repositories to filter
            visibility: 'public' or 'private' (None = all)
            language: Programming language filter (None = all)
            min_stars: Minimum number of stars
            search_term: Search in name and description
            
        Returns:
            Filtered list of repositories
        """
        filtered = repositories
        
        if visibility:
            filtered = [r for r in filtered if r.visibility == visibility]
        
        if language:
            filtered = [r for r in filtered if r.language.lower() == language.lower()]
        
        if min_stars > 0:
            filtered = [r for r in filtered if r.stars >= min_stars]
        
        if search_term:
            search_lower = search_term.lower()
            filtered = [
                r for r in filtered
                if search_lower in r.name.lower() or search_lower in r.description.lower()
            ]
        
        return filtered
    
    def sort_repositories(
        self,
        repositories: List[Repository],
        sort_by: str = 'name',
        reverse: bool = False
    ) -> List[Repository]:
        """Sort repositories by various criteria.
        
        Args:
            repositories: List of repositories to sort
            sort_by: 'name', 'stars', 'updated_at', 'size'
            reverse: Reverse sort order
            
        Returns:
            Sorted list of repositories
        """
        sort_key_map = {
            'name': lambda r: r.name.lower(),
            'stars': lambda r: r.stars,
            'updated_at': lambda r: r.updated_at,
            'size': lambda r: r.size_kb,
            'owner': lambda r: r.owner.lower(),
        }
        
        key_func = sort_key_map.get(sort_by, sort_key_map['name'])
        return sorted(repositories, key=key_func, reverse=reverse)
    
    def get_repository_by_name(
        self,
        repositories: List[Repository],
        name: str
    ) -> Optional[Repository]:
        """Find a repository by name.
        
        Args:
            repositories: List of repositories to search
            name: Repository name (case-insensitive)
            
        Returns:
            Repository object or None if not found
        """
        name_lower = name.lower()
        for repo in repositories:
            if repo.name.lower() == name_lower:
                return repo
        return None
    
    def get_repositories_by_owner(
        self,
        repositories: List[Repository],
        owner: str
    ) -> List[Repository]:
        """Get all repositories by a specific owner.
        
        Args:
            repositories: List of repositories to search
            owner: Owner name (case-insensitive)
            
        Returns:
            List of repositories owned by the owner
        """
        owner_lower = owner.lower()
        return [r for r in repositories if r.owner.lower() == owner_lower]
    
    def get_statistics(self, repositories: List[Repository]) -> Dict:
        """Calculate statistics about repositories.
        
        Args:
            repositories: List of repositories
            
        Returns:
            Dictionary with statistics
        """
        if not repositories:
            return {
                'total': 0,
                'public': 0,
                'private': 0,
                'total_stars': 0,
                'total_size_kb': 0,
                'languages': {},
                'avg_stars': 0,
                'avg_size_kb': 0,
            }
        
        public_count = sum(1 for r in repositories if r.visibility == 'public')
        private_count = len(repositories) - public_count
        total_stars = sum(r.stars for r in repositories)
        total_size = sum(r.size_kb for r in repositories)
        
        languages = {}
        for repo in repositories:
            if repo.language and repo.language != 'Unknown':
                languages[repo.language] = languages.get(repo.language, 0) + 1
        
        return {
            'total': len(repositories),
            'public': public_count,
            'private': private_count,
            'total_stars': total_stars,
            'total_size_kb': total_size,
            'languages': languages,
            'avg_stars': total_stars / len(repositories) if repositories else 0,
            'avg_size_kb': total_size / len(repositories) if repositories else 0,
        }
    
    def validate_repository_url(self, url: str) -> bool:
        """Validate if a URL is a valid repository URL.
        
        Args:
            url: Repository URL to validate
            
        Returns:
            True if URL is valid
        """
        import re
        
        # Basic validation for common Git URL patterns
        patterns = [
            r'^git@[^:]+:[^/]+/[^/]+(?:\.git)?$',  # SSH
            r'^https?://[^/]+/[^/]+/[^/]+(?:\.git)?/?$',  # HTTPS
            r'^/[^/]+/[^/]+$',  # Local path
        ]
        
        for pattern in patterns:
            if re.match(pattern, url):
                return True
        return False
