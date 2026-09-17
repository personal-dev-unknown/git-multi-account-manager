"""Bitbucket platform integration."""

import requests
import base64
from typing import List, Dict, Optional
from .base import BasePlatform, Repository


class BitbucketPlatform(BasePlatform):
    """Bitbucket platform integration.
    
    Supports:
    - Repository fetching via Bitbucket API v2.0
    - App Password authentication
    - Repository forking
    - SSH and HTTPS clone URLs
    - Team and workspace management
    """
    
    def __init__(self):
        super().__init__(
            api_base='https://api.bitbucket.org/2.0',
            ssh_host='bitbucket.org'
        )
        self.session = requests.Session()
        self.timeout = 10
        self.platform_name = 'Bitbucket'
    
    def fetch_repositories(self, account: Dict) -> List[Repository]:
        """Fetch user's repositories from Bitbucket API."""
        
        if not account.get('pat_token'):
            raise ValueError("Bitbucket requires an App Password")
        
        # Bitbucket uses Basic auth
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
                    f"{self.api_base}/repositories/{account['username']}",
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
    
    def test_connection(self, account: Dict) -> bool:
        """Test Bitbucket API connection."""
        try:
            credentials = base64.b64encode(
                f"{account['username']}:{account.get('pat_token')}".encode()
            ).decode()
            headers = {'Authorization': f'Basic {credentials}'}
            response = self.session.get(
                f"{self.api_base}/user",
                headers=headers,
                timeout=5
            )
            return response.status_code == 200
        except Exception:
            return False
    
    def fork_repository(self, repo_url: str, account: Dict) -> Dict:
        """Fork a repository on Bitbucket."""
        try:
            from ..parsers import URLParser
            parsed = URLParser.parse(repo_url)
            
            credentials = base64.b64encode(
                f"{account['username']}:{account.get('pat_token')}".encode()
            ).decode()
            headers = {'Authorization': f'Basic {credentials}'}
            
            response = self.session.post(
                f"{self.api_base}/repositories/{parsed.owner}/{parsed.repo}/forks",
                headers=headers,
                timeout=self.timeout
            )
            
            if response.status_code in [200, 201]:
                fork_data = response.json()
                # Get SSH URL from clone links
                ssh_url = ''
                for link in fork_data.get('links', {}).get('clone', []):
                    if link['name'] == 'ssh':
                        ssh_url = link['href']
                
                return {
                    'success': True,
                    'fork_url': ssh_url,
                    'fork_owner': fork_data['project']['key'],
                    'message': f"Successfully forked to {fork_data['project']['key']}/{fork_data['name']}"
                }
            else:
                return {
                    'success': False,
                    'error': f"Fork failed: {response.text}"
                }
        except Exception as e:
            return {
                'success': False,
                'error': f"Fork operation failed: {str(e)}"
            }
    
    def get_repository_info(self, owner: str, repo: str, account: Dict) -> Optional[Repository]:
        """Get information about a specific Bitbucket repository."""
        try:
            credentials = base64.b64encode(
                f"{account['username']}:{account.get('pat_token')}".encode()
            ).decode()
            headers = {'Authorization': f'Basic {credentials}'}
            
            response = self.session.get(
                f"{self.api_base}/repositories/{owner}/{repo}",
                headers=headers,
                timeout=self.timeout
            )
            response.raise_for_status()
            
            repo_data = response.json()
            
            # Get SSH and HTTPS URLs
            ssh_url = ''
            https_url = ''
            for link in repo_data.get('links', {}).get('clone', []):
                if link['name'] == 'ssh':
                    ssh_url = link['href']
                elif link['name'] == 'http':
                    https_url = link['href']
            
            return Repository(
                id=repo_data['id'],
                name=repo_data['name'],
                full_name=repo_data['full_slug'],
                owner=repo_data['project']['key'],
                description=repo_data.get('description', '') or '',
                visibility='private' if repo_data['is_private'] else 'public',
                language='Unknown',
                size_kb=0,
                stars=0,
                updated_at=repo_data['updated_on'],
                ssh_url=ssh_url,
                https_url=https_url,
                web_url=repo_data['links']['html']['href'],
                is_fork=False,
                default_branch=repo_data.get('mainbranch', {}).get('name', 'main')
            )
        except Exception:
            return None
    
    # ==================== Helper Methods ====================
    
    def _create_auth_headers(self, username: str, app_password: str) -> Dict[str, str]:
        """Create Bitbucket authentication headers.
        
        Args:
            username: Bitbucket username
            app_password: App password
            
        Returns:
            Dictionary with authorization headers
        """
        credentials = base64.b64encode(
            f"{username}:{app_password}".encode()
        ).decode()
        return {'Authorization': f'Basic {credentials}'}
    
    def get_user_info(self, account: Dict) -> Optional[Dict]:
        """Get authenticated user information.
        
        Args:
            account: Account dictionary with username and pat_token
            
        Returns:
            User information dictionary or None
        """
        try:
            headers = self._create_auth_headers(account['username'], account['pat_token'])
            response = self.session.get(
                f"{self.api_base}/user",
                headers=headers,
                timeout=self.timeout
            )
            response.raise_for_status()
            return response.json()
        except Exception:
            return None
    
    def get_user_teams(self, account: Dict) -> List[Dict]:
        """Get all teams for the authenticated user.
        
        Args:
            account: Account dictionary with username and pat_token
            
        Returns:
            List of team dictionaries
        """
        try:
            headers = self._create_auth_headers(account['username'], account['pat_token'])
            teams = []
            page = 1
            
            while True:
                response = self.session.get(
                    f"{self.api_base}/teams",
                    headers=headers,
                    params={'pagelen': 100, 'page': page},
                    timeout=self.timeout
                )
                response.raise_for_status()
                
                data = response.json()
                page_teams = data.get('values', [])
                
                if not page_teams:
                    break
                
                teams.extend(page_teams)
                
                if 'next' not in data:
                    break
                
                page += 1
            
            return teams
        except Exception:
            return []
    
    def get_team_repositories(self, team_slug: str, account: Dict) -> List[Repository]:
        """Get all repositories in a team.
        
        Args:
            team_slug: Team slug/username
            account: Account dictionary with username and pat_token
            
        Returns:
            List of Repository objects
        """
        try:
            headers = self._create_auth_headers(account['username'], account['pat_token'])
            repos = []
            page = 1
            
            while True:
                response = self.session.get(
                    f"{self.api_base}/repositories/{team_slug}",
                    headers=headers,
                    params={'pagelen': 100, 'page': page, 'sort': '-updated_on'},
                    timeout=self.timeout
                )
                response.raise_for_status()
                
                data = response.json()
                page_repos = data.get('values', [])
                
                if not page_repos:
                    break
                
                for repo in page_repos:
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
                
                if 'next' not in data:
                    break
                
                page += 1
            
            return repos
        except Exception as e:
            raise Exception(f"Failed to get team repositories: {str(e)}")
