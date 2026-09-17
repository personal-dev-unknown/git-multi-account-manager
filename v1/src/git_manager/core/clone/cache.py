"""Repository cache for clone operations."""

import json
import time
from pathlib import Path
from typing import Optional, List, Dict
from datetime import datetime, timedelta


class RepositoryCache:
    """Cache for repository listings."""
    
    def __init__(self, cache_dir: Optional[str] = None, ttl: int = 300):
        """
        Initialize cache.
        
        Args:
            cache_dir: Cache directory (default: ~/.config/git-manager/cache)
            ttl: Time to live in seconds (default: 300 = 5 minutes)
        """
        if cache_dir:
            self.cache_dir = Path(cache_dir)
        else:
            self.cache_dir = Path.home() / '.config' / 'git-manager' / 'cache'
        
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        self.ttl = ttl
    
    def _get_cache_file(self, account_id: str, platform: str) -> Path:
        """Get cache file path for account/platform."""
        filename = f"{account_id}_{platform}_repos.json"
        return self.cache_dir / filename
    
    def get(self, account_id: str, platform: str) -> Optional[Dict]:
        """
        Get cached repositories.
        
        Args:
            account_id: Account ID
            platform: Platform ID
        
        Returns:
            Cached data or None if expired/not found
        """
        cache_file = self._get_cache_file(account_id, platform)
        
        if not cache_file.exists():
            return None
        
        try:
            with open(cache_file, 'r') as f:
                data = json.load(f)
            
            # Check if cache is expired
            cached_at = datetime.fromisoformat(data['cached_at'])
            if datetime.now() - cached_at > timedelta(seconds=self.ttl):
                # Cache expired
                cache_file.unlink()
                return None
            
            return data
        
        except Exception:
            return None
    
    def set(self, account_id: str, platform: str, repositories: List[Dict]) -> bool:
        """
        Cache repositories.
        
        Args:
            account_id: Account ID
            platform: Platform ID
            repositories: List of repository dictionaries
        
        Returns:
            True if successful
        """
        cache_file = self._get_cache_file(account_id, platform)
        
        try:
            data = {
                'account_id': account_id,
                'platform': platform,
                'repositories': repositories,
                'cached_at': datetime.now().isoformat(),
                'count': len(repositories)
            }
            
            with open(cache_file, 'w') as f:
                json.dump(data, f, indent=2)
            
            return True
        
        except Exception:
            return False
    
    def clear(self, account_id: Optional[str] = None, platform: Optional[str] = None) -> bool:
        """
        Clear cache.
        
        Args:
            account_id: Account ID (clear all if None)
            platform: Platform ID (clear all if None)
        
        Returns:
            True if successful
        """
        try:
            if account_id and platform:
                # Clear specific cache
                cache_file = self._get_cache_file(account_id, platform)
                if cache_file.exists():
                    cache_file.unlink()
            else:
                # Clear all cache
                for cache_file in self.cache_dir.glob('*_repos.json'):
                    cache_file.unlink()
            
            return True
        
        except Exception:
            return False
    
    def is_expired(self, cached_data: Dict) -> bool:
        """
        Check if cached data is expired.
        
        Args:
            cached_data: Cached data dictionary
        
        Returns:
            True if expired
        """
        try:
            cached_at = datetime.fromisoformat(cached_data['cached_at'])
            return datetime.now() - cached_at > timedelta(seconds=self.ttl)
        except Exception:
            return True
