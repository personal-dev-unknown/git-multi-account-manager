"""GitLab platform integration."""

import requests
from typing import List, Dict, Optional
from .base import BasePlatform, Repository


class GitLabPlatform(BasePlatform):
    """GitLab platform integration.
    
    Supports:
    - Repository fetching via GitLab API v4
    - PAT (Personal Access Token) authentication
    - Repository forking
    - SSH and HTTPS clone URLs
    - Group and project management
    """
    
    def __init__(self):
        super().__init__(
            api_base='https://gitlab.com/api/v4',
            ssh_host='gitlab.com'
        )
        self.session = requests.Session()
        self.timeout = 10
        self.platform_name = 'GitLab'
    
    def fetch_repositories(self, account: Dict) -> List[Repository]:
        """Fetch user's repositories from GitLab API."""
        
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
                    f"{self.api_base}/projects",
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
    
    def test_connection(self, account: Dict) -> bool:
        """Test GitLab API connection."""
        try:
            headers = {'PRIVATE-TOKEN': account.get('pat_token')}
            response = self.session.get(
                f"{self.api_base}/user",
                headers=headers,
                timeout=5
            )
            return response.status_code == 200
        except Exception:
            return False
    
    def fork_repository(self, repo_url: str, account: Dict) -> Dict:
        """Fork a repository on GitLab."""
        try:
            from ..parsers import URLParser
            parsed = URLParser.parse(repo_url)
            
            headers = {'PRIVATE-TOKEN': account.get('pat_token')}
            response = self.session.post(
                f"{self.api_base}/projects/{parsed.owner}%2F{parsed.repo}/fork",
                headers=headers,
                timeout=self.timeout
            )
            
            if response.status_code in [200, 201]:
                fork_data = response.json()
                return {
                    'success': True,
                    'fork_url': fork_data['ssh_url_to_repo'],
                    'fork_owner': fork_data['namespace']['path'],
                    'message': f"Successfully forked to {fork_data['namespace']['path']}/{fork_data['name']}"
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
        """Get information about a specific GitLab repository."""
        try:
            headers = {'PRIVATE-TOKEN': account.get('pat_token')}
            response = self.session.get(
                f"{self.api_base}/projects/{owner}%2F{repo}",
                headers=headers,
                timeout=self.timeout
            )
            response.raise_for_status()
            
            repo_data = response.json()
            return Repository(
                id=repo_data['id'],
                name=repo_data['name'],
                full_name=repo_data['path_with_namespace'],
                owner=repo_data['namespace']['path'],
                description=repo_data.get('description', '') or '',
                visibility=repo_data.get('visibility', 'private'),
                language=repo_data.get('language') or 'Unknown',
                size_kb=repo_data.get('statistics', {}).get('repository_size', 0) // 1024,
                stars=repo_data.get('star_count', 0),
                updated_at=repo_data['last_activity_at'],
                ssh_url=repo_data['ssh_url_to_repo'],
                https_url=repo_data['http_url_to_repo'],
                web_url=repo_data['web_url'],
                is_fork='forked_from_project' in repo_data,
                default_branch=repo_data.get('default_branch', 'main')
            )
        except Exception:
            return None
    
    # ==================== Helper Methods ====================
    
    def _create_auth_headers(self, pat_token: str) -> Dict[str, str]:
        """Create GitLab authentication headers.
        
        Args:
            pat_token: Personal Access Token
            
        Returns:
            Dictionary with authorization headers
        """
        return {'PRIVATE-TOKEN': pat_token}
    
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
    
    def get_user_groups(self, account: Dict) -> List[Dict]:
        """Get all groups for the authenticated user.
        
        Args:
            account: Account dictionary with pat_token
            
        Returns:
            List of group dictionaries
        """
        try:
            headers = self._create_auth_headers(account['pat_token'])
            groups = []
            page = 1
            
            while True:
                response = self.session.get(
                    f"{self.api_base}/groups",
                    headers=headers,
                    params={'per_page': 100, 'page': page},
                    timeout=self.timeout
                )
                response.raise_for_status()
                
                page_groups = response.json()
                if not page_groups:
                    break
                
                groups.extend(page_groups)
                page += 1
            
            return groups
        except Exception:
            return []
    
    def get_group_repositories(self, group_id: int, account: Dict) -> List[Repository]:
        """Get all repositories in a group.
        
        Args:
            group_id: GitLab group ID
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
                    f"{self.api_base}/groups/{group_id}/projects",
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
            
            return repos
        except Exception as e:
            raise Exception(f"Failed to get group repositories: {str(e)}")
