"""GitHub platform integration."""

import requests
from typing import List, Dict, Optional
from .base import BasePlatform, Repository


class GitHubPlatform(BasePlatform):
    """GitHub platform integration.
    
    Supports:
    - Repository fetching via GitHub API v3
    - PAT (Personal Access Token) authentication
    - Repository forking
    - SSH and HTTPS clone URLs
    """
    
    def __init__(self):
        super().__init__(
            api_base='https://api.github.com',
            ssh_host='github.com'
        )
        self.session = requests.Session()
        self.timeout = 10
        self.platform_name = 'GitHub'
    
    def fetch_repositories(self, account: Dict) -> List[Repository]:
        """Fetch user's repositories from GitHub API."""
        
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
                    f"{self.api_base}/user/repos",
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
    
    def test_connection(self, account: Dict) -> bool:
        """Test GitHub API connection."""
        try:
            headers = {'Authorization': f"token {account.get('pat_token')}"}
            response = self.session.get(
                f"{self.api_base}/user",
                headers=headers,
                timeout=5
            )
            return response.status_code == 200
        except Exception:
            return False
    
    def fork_repository(self, repo_url: str, account: Dict) -> Dict:
        """Fork a repository on GitHub."""
        try:
            # Parse owner/repo from URL
            from ..parsers import URLParser
            parsed = URLParser.parse(repo_url)
            
            headers = {'Authorization': f"token {account.get('pat_token')}"}
            response = self.session.post(
                f"{self.api_base}/repos/{parsed.owner}/{parsed.repo}/forks",
                headers=headers,
                timeout=self.timeout
            )
            
            if response.status_code in [200, 201]:
                fork_data = response.json()
                return {
                    'success': True,
                    'fork_url': fork_data['ssh_url'],
                    'fork_owner': fork_data['owner']['login'],
                    'message': f"Successfully forked to {fork_data['owner']['login']}/{fork_data['name']}"
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
        """Get information about a specific GitHub repository."""
        try:
            headers = {'Authorization': f"token {account.get('pat_token')}"}
            response = self.session.get(
                f"{self.api_base}/repos/{owner}/{repo}",
                headers=headers,
                timeout=self.timeout
            )
            response.raise_for_status()
            
            repo_data = response.json()
            return Repository(
                id=repo_data['id'],
                name=repo_data['name'],
                full_name=repo_data['full_name'],
                owner=repo_data['owner']['login'],
                description=repo_data.get('description', '') or '',
                visibility='private' if repo_data['private'] else 'public',
                language=repo_data.get('language') or 'Unknown',
                size_kb=repo_data['size'],
                stars=repo_data['stargazers_count'],
                updated_at=repo_data['updated_at'],
                ssh_url=repo_data['ssh_url'],
                https_url=repo_data['clone_url'],
                web_url=repo_data['html_url'],
                is_fork=repo_data['fork'],
                default_branch=repo_data['default_branch']
            )
        except Exception:
            return None
    
    # ==================== Helper Methods ====================
    
    def _create_auth_headers(self, pat_token: str) -> Dict[str, str]:
        """Create GitHub authentication headers.
        
        Args:
            pat_token: Personal Access Token
            
        Returns:
            Dictionary with authorization headers
        """
        return {
            'Authorization': f"token {pat_token}",
            'Accept': 'application/vnd.github.v3+json'
        }
    
    def get_user_info(self, account: Dict) -> Optional[Dict]:
        """Get authenticated user information.
        
        Args:
            account: Account dictionary with pat_token
            
        Returns:
            User information dictionary or None
        """
        try:
            headers = self._create_auth_headers(account['pat_token'])
            response = self.session.get(
                f"{self.api_base}/user",
                headers=headers,
                timeout=self.timeout
            )
            response.raise_for_status()
            return response.json()
        except Exception:
            return None
    
    def get_user_organizations(self, account: Dict) -> List[Dict]:
        """Get all organizations for the authenticated user.
        
        Args:
            account: Account dictionary with pat_token
            
        Returns:
            List of organization dictionaries
        """
        try:
            headers = self._create_auth_headers(account['pat_token'])
            orgs = []
            page = 1
            
            while True:
                response = self.session.get(
                    f"{self.api_base}/user/orgs",
                    headers=headers,
                    params={'per_page': 100, 'page': page},
                    timeout=self.timeout
                )
                response.raise_for_status()
                
                page_orgs = response.json()
                if not page_orgs:
                    break
                
                orgs.extend(page_orgs)
                page += 1
            
            return orgs
        except Exception:
            return []
    
    def get_organization_repositories(self, org: str, account: Dict) -> List[Repository]:
        """Get all repositories in an organization.
        
        Args:
            org: Organization name
            account: Account dictionary with pat_token
            
        Returns:
            List of Repository objects
        """
        try:
            headers = self._create_auth_headers(account['pat_token'])
            repos = []
            page = 1
            
            while True:
                response = self.session.get(
                    f"{self.api_base}/orgs/{org}/repos",
                    headers=headers,
                    params={
                        'per_page': 100,
                        'page': page,
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
            
            return repos
        except Exception as e:
            raise Exception(f"Failed to get organization repositories: {str(e)}")
    
    def get_starred_repositories(self, account: Dict) -> List[Repository]:
        """Get all repositories starred by the user.
        
        Args:
            account: Account dictionary with pat_token
            
        Returns:
            List of Repository objects
        """
        try:
            headers = self._create_auth_headers(account['pat_token'])
            repos = []
            page = 1
            
            while True:
                response = self.session.get(
                    f"{self.api_base}/user/starred",
                    headers=headers,
                    params={'per_page': 100, 'page': page},
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
            
            return repos
        except Exception as e:
            raise Exception(f"Failed to get starred repositories: {str(e)}")
