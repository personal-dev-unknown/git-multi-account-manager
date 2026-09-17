"""Fetch repositories from platform APIs."""

import requests
import json
from typing import List, Dict, Optional
from datetime import datetime, timedelta
from dataclasses import dataclass, asdict
from .clone_platform_config import get_platform_config


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
    
    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return asdict(self)


class RepositoryFetcher:
    """Fetch user's repositories from platform APIs."""
    
    def __init__(self, timeout: int = 10):
        self.timeout = timeout
        self.session = requests.Session()
    
    def fetch_repositories(
        self,
        platform: str,
        account: Dict,
        force_refresh: bool = False
    ) -> List[Repository]:
        """
        Fetch repositories from platform API.
        
        Args:
            platform: Platform ID ('github', 'gitlab', etc.)
            account: Account dictionary with 'pat_token' or 'username'
            force_refresh: Ignore cache
        
        Returns:
            List of Repository objects
        """
        
        if platform == 'github':
            return self._fetch_github(account)
        elif platform == 'gitlab':
            return self._fetch_gitlab(account)
        elif platform == 'bitbucket':
            return self._fetch_bitbucket(account)
        else:
            raise ValueError(f"Unsupported platform: {platform}")
    
    def _fetch_github(self, account: Dict) -> List[Repository]:
        """Fetch repositories from GitHub API."""
        config = get_platform_config('github')
        
        if not account.get('pat_token'):
            raise ValueError("GitHub requires a Personal Access Token (PAT)")
        
        headers = {
            'Authorization': f"token {account['pat_token']}",
            'Accept': 'application/vnd.github.v3+json'
        }
        
        repos = []
        page = 1
        
        while True:
            try:
                response = self.session.get(
                    f"{config.api_base}/user/repos",
                    headers=headers,
                    params={
                        'per_page': 100,
                        'page': page,
                        'sort': 'updated',
                        'type': 'all'
                    },
                    timeout=self.timeout
                )
                response.raise_for_status()
                
                page_repos = response.json()
                if not page_repos:
                    break
                
                for repo in page_repos:
                    repos.append(Repository(
                        id=repo['id'],
                        name=repo['name'],
                        full_name=repo['full_name'],
                        owner=repo['owner']['login'],
                        description=repo.get('description', '') or '',
                        visibility='private' if repo['private'] else 'public',
                        language=repo.get('language') or 'Unknown',
                        size_kb=repo['size'],
                        stars=repo['stargazers_count'],
                        updated_at=repo['updated_at'],
                        ssh_url=repo['ssh_url'],
                        https_url=repo['clone_url'],
                        web_url=repo['html_url'],
                        is_fork=repo['fork'],
                        default_branch=repo['default_branch']
                    ))
                
                page += 1
                
            except requests.exceptions.RequestException as e:
                raise Exception(f"Failed to fetch GitHub repositories: {str(e)}")
        
        return repos
    
    def _fetch_gitlab(self, account: Dict) -> List[Repository]:
        """Fetch repositories from GitLab API."""
        config = get_platform_config('gitlab')
        
        if not account.get('pat_token'):
            raise ValueError("GitLab requires a Personal Access Token (PAT)")
        
        headers = {
            'PRIVATE-TOKEN': account['pat_token']
        }
        
        repos = []
        page = 1
        
        while True:
            try:
                response = self.session.get(
                    f"{config.api_base}/projects",
                    headers=headers,
                    params={
                        'membership': True,
                        'per_page': 100,
                        'page': page,
                        'order_by': 'updated_at',
                        'sort': 'desc'
                    },
                    timeout=self.timeout
                )
                response.raise_for_status()
                
                page_repos = response.json()
                if not page_repos:
                    break
                
                for repo in page_repos:
                    repos.append(Repository(
                        id=repo['id'],
                        name=repo['name'],
                        full_name=repo['path_with_namespace'],
                        owner=repo['namespace']['path'],
                        description=repo.get('description', '') or '',
                        visibility=repo.get('visibility', 'private'),
                        language=repo.get('language') or 'Unknown',
                        size_kb=repo.get('statistics', {}).get('repository_size', 0) // 1024,
                        stars=repo.get('star_count', 0),
                        updated_at=repo['last_activity_at'],
                        ssh_url=repo['ssh_url_to_repo'],
                        https_url=repo['http_url_to_repo'],
                        web_url=repo['web_url'],
                        is_fork='forked_from_project' in repo,
                        default_branch=repo.get('default_branch', 'main')
                    ))
                
                page += 1
                
            except requests.exceptions.RequestException as e:
                raise Exception(f"Failed to fetch GitLab repositories: {str(e)}")
        
        return repos
    
    def _fetch_bitbucket(self, account: Dict) -> List[Repository]:
        """Fetch repositories from Bitbucket API."""
        config = get_platform_config('bitbucket')
        
        if not account.get('pat_token'):
            raise ValueError("Bitbucket requires an App Password")
        
        # Bitbucket uses Basic auth
        import base64
        credentials = base64.b64encode(
            f"{account['username']}:{account['pat_token']}".encode()
        ).decode()
        
        headers = {
            'Authorization': f'Basic {credentials}'
        }
        
        repos = []
        page = 1
        
        while True:
            try:
                response = self.session.get(
                    f"{config.api_base}/repositories/{account['username']}",
                    headers=headers,
                    params={
                        'pagelen': 100,
                        'page': page,
                        'sort': '-updated_on'
                    },
                    timeout=self.timeout
                )
                response.raise_for_status()
                
                data = response.json()
                page_repos = data.get('values', [])
                
                if not page_repos:
                    break
                
                for repo in page_repos:
                    # Get SSH and HTTPS URLs from links
                    ssh_url = ''
                    https_url = ''
                    
                    for link in repo.get('links', {}).get('clone', []):
                        if link['name'] == 'ssh':
                            ssh_url = link['href']
                        elif link['name'] == 'http':
                            https_url = link['href']
                    
                    repos.append(Repository(
                        id=repo['id'],
                        name=repo['name'],
                        full_name=repo['full_slug'],
                        owner=repo['project']['key'],
                        description=repo.get('description', '') or '',
                        visibility='private' if repo['is_private'] else 'public',
                        language='Unknown',
                        size_kb=0,
                        stars=0,
                        updated_at=repo['updated_on'],
                        ssh_url=ssh_url,
                        https_url=https_url,
                        web_url=repo['links']['html']['href'],
                        is_fork=False,
                        default_branch=repo.get('mainbranch', {}).get('name', 'main')
                    ))
                
                # Check if there are more pages
                if 'next' not in data:
                    break
                
                page += 1
                
            except requests.exceptions.RequestException as e:
                raise Exception(f"Failed to fetch Bitbucket repositories: {str(e)}")
        
        return repos
    
    def test_connection(self, platform: str, account: Dict) -> bool:
        """Test API connection."""
        try:
            if platform == 'github':
                config = get_platform_config('github')
                headers = {'Authorization': f"token {account.get('pat_token')}"}
                response = self.session.get(
                    f"{config.api_base}/user",
                    headers=headers,
                    timeout=5
                )
            elif platform == 'gitlab':
                config = get_platform_config('gitlab')
                headers = {'PRIVATE-TOKEN': account.get('pat_token')}
                response = self.session.get(
                    f"{config.api_base}/user",
                    headers=headers,
                    timeout=5
                )
            elif platform == 'bitbucket':
                config = get_platform_config('bitbucket')
                import base64
                credentials = base64.b64encode(
                    f"{account['username']}:{account.get('pat_token')}".encode()
                ).decode()
                headers = {'Authorization': f'Basic {credentials}'}
                response = self.session.get(
                    f"{config.api_base}/user",
                    headers=headers,
                    timeout=5
                )
            else:
                return False
            
            return response.status_code == 200
            
        except Exception:
            return False
