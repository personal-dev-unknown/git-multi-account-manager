"""Azure DevOps Platform Implementation."""

import requests
from typing import Optional, Dict, List
from base64 import b64encode
from .base import BasePlatform, Repository


class AzureDevOpsPlatform(BasePlatform):
    """Azure DevOps platform implementation.
    
    Supports:
    - Repository fetching via Azure DevOps API v7.0
    - PAT (Personal Access Token) authentication
    - Project-based repository structure
    - SSH and HTTPS clone URLs
    """
    
    def __init__(self):
        """Initialize Azure DevOps platform."""
        super().__init__(
            api_base='https://dev.azure.com',
            ssh_host='ssh.dev.azure.com'
        )
        self.session = requests.Session()
        self.timeout = 10
        self.api_version = '7.0'
        self.platform_name = 'Azure DevOps'
    
    def fetch_repositories(self, account: Dict) -> List[Repository]:
        """Fetch user's repositories from Azure DevOps API."""
        
        if not account.get('pat_token'):
            raise ValueError("Azure DevOps requires a Personal Access Token (PAT)")
        
        # Create auth header
        auth_string = b64encode(f":{account['pat_token']}".encode()).decode()
        headers = {
            "Authorization": f"Basic {auth_string}",
            "Content-Type": "application/json"
        }
        
        repositories = []
        organization = account.get('username')
        
        try:
            # Get projects
            projects_url = f"{self.api_base}/_apis/projects?api-version=7.0"
            response = self.session.get(
                projects_url,
                headers=headers,
                timeout=self.timeout
            )
            response.raise_for_status()
            projects = response.json().get('value', [])
            
            # Fetch repositories for each project
            for project in projects:
                repos_url = f"{self.api_base}/{organization}/{project['name']}/_apis/git/repositories?api-version=7.0"
                response = self.session.get(
                    repos_url,
                    headers=headers,
                    timeout=self.timeout
                )
                response.raise_for_status()
                
                for repo_data in response.json().get('value', []):
                    repositories.append(Repository(
                        id=hash(repo_data['id']),
                        name=repo_data['name'],
                        full_name=f"{project['name']}/{repo_data['name']}",
                        owner=organization,
                        description=repo_data.get('description', '') or '',
                        visibility='private' if repo_data.get('isPrivate', True) else 'public',
                        language='',
                        size_kb=0,
                        stars=0,
                        updated_at=repo_data.get('pushedAt', ''),
                        ssh_url=repo_data.get('sshUrl', ''),
                        https_url=repo_data.get('webUrl', ''),
                        web_url=repo_data.get('webUrl', ''),
                        is_fork=False,
                        default_branch=repo_data.get('defaultBranch', 'main')
                    ))
        
        except requests.exceptions.RequestException as e:
            raise Exception(f"Failed to fetch Azure DevOps repositories: {str(e)}")
        
        return repositories
    
    def test_connection(self, account: Dict) -> bool:
        """Test Azure DevOps API connection."""
        try:
            if not account.get('pat_token'):
                return False
            
            auth_string = b64encode(f":{account['pat_token']}".encode()).decode()
            headers = {
                "Authorization": f"Basic {auth_string}",
                "Content-Type": "application/json"
            }
            
            response = self.session.get(
                f"{self.api_base}/_apis/projects?api-version=7.0",
                headers=headers,
                timeout=self.timeout
            )
            return response.status_code == 200
        except Exception:
            return False
    
    def fork_repository(self, repo_url: str, account: Dict) -> Dict:
        """Fork a repository on Azure DevOps."""
        # Azure DevOps fork implementation would go here
        raise NotImplementedError("Fork not yet implemented for Azure DevOps")
    
    def get_repository_info(self, owner: str, repo: str, account: Dict) -> Optional[Repository]:
        """Get information about a specific Azure DevOps repository."""
        try:
            if not account.get('pat_token'):
                return None
            
            auth_string = b64encode(f":{account['pat_token']}".encode()).decode()
            headers = {
                "Authorization": f"Basic {auth_string}",
                "Content-Type": "application/json"
            }
            
            # owner is organization, repo is project/repo
            parts = repo.split('/')
            if len(parts) == 2:
                project, repo_name = parts
            else:
                project = owner
                repo_name = repo
            
            url = f"{self.api_base}/{owner}/{project}/_apis/git/repositories/{repo_name}?api-version={self.api_version}"
            response = self.session.get(
                url,
                headers=headers,
                timeout=self.timeout
            )
            response.raise_for_status()
            
            repo_data = response.json()
            return Repository(
                id=hash(repo_data['id']),
                name=repo_data['name'],
                full_name=f"{project}/{repo_data['name']}",
                owner=owner,
                description=repo_data.get('description', '') or '',
                visibility='private' if repo_data.get('isPrivate', True) else 'public',
                language='',
                size_kb=0,
                stars=0,
                updated_at=repo_data.get('pushedAt', ''),
                ssh_url=repo_data.get('sshUrl', ''),
                https_url=repo_data.get('webUrl', ''),
                web_url=repo_data.get('webUrl', ''),
                is_fork=False,
                default_branch=repo_data.get('defaultBranch', 'main')
            )
        except Exception:
            return None
    
    # ==================== Helper Methods ====================
    
    def _create_auth_headers(self, pat_token: str) -> Dict[str, str]:
        """Create Azure DevOps authentication headers.
        
        Args:
            pat_token: Personal Access Token
            
        Returns:
            Dictionary with authorization headers
        """
        auth_string = b64encode(f":{pat_token}".encode()).decode()
        return {
            "Authorization": f"Basic {auth_string}",
            "Content-Type": "application/json"
        }
    
    def get_projects(self, account: Dict) -> List[Dict]:
        """Get all projects in the organization.
        
        Args:
            account: Account dictionary with pat_token
            
        Returns:
            List of project dictionaries
        """
        try:
            headers = self._create_auth_headers(account['pat_token'])
            url = f"{self.api_base}/_apis/projects?api-version={self.api_version}"
            response = self.session.get(url, headers=headers, timeout=self.timeout)
            response.raise_for_status()
            return response.json().get('value', [])
        except Exception as e:
            raise Exception(f"Failed to get projects: {str(e)}")
    
    def get_project_repositories(self, organization: str, project: str, account: Dict) -> List[Repository]:
        """Get all repositories in a specific project.
        
        Args:
            organization: Organization name
            project: Project name
            account: Account dictionary with pat_token
            
        Returns:
            List of Repository objects
        """
        try:
            headers = self._create_auth_headers(account['pat_token'])
            url = f"{self.api_base}/{organization}/{project}/_apis/git/repositories?api-version={self.api_version}"
            response = self.session.get(url, headers=headers, timeout=self.timeout)
            response.raise_for_status()
            
            repositories = []
            for repo_data in response.json().get('value', []):
                repositories.append(Repository(
                    id=hash(repo_data['id']),
                    name=repo_data['name'],
                    full_name=f"{project}/{repo_data['name']}",
                    owner=organization,
                    description=repo_data.get('description', '') or '',
                    visibility='private' if repo_data.get('isPrivate', True) else 'public',
                    language='',
                    size_kb=0,
                    stars=0,
                    updated_at=repo_data.get('pushedAt', ''),
                    ssh_url=repo_data.get('sshUrl', ''),
                    https_url=repo_data.get('webUrl', ''),
                    web_url=repo_data.get('webUrl', ''),
                    is_fork=False,
                    default_branch=repo_data.get('defaultBranch', 'main')
                ))
            return repositories
        except Exception as e:
            raise Exception(f"Failed to get project repositories: {str(e)}")
