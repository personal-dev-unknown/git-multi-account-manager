CODE !

# src/git_manager/api/github_client.py
"""
GitHub API Client for managing repositories, SSH keys, and user information.
"""

import requests
from typing import List, Dict, Optional, Any
from dataclasses import dataclass
from pathlib import Path
import time


@dataclass
class GitHubRepo:
    """GitHub repository information."""
    id: int
    name: str
    full_name: str
    private: bool
    html_url: str
    ssh_url: str
    clone_url: str
    description: Optional[str]
    created_at: str
    updated_at: str
    pushed_at: str
    size: int
    language: Optional[str]
    default_branch: str


@dataclass
class GitHubSSHKey:
    """GitHub SSH key information."""
    id: int
    key: str
    title: str
    created_at: str
    verified: bool
    read_only: bool


class GitHubAPIClient:
    """Client for GitHub REST API."""
    
    BASE_URL = "https://api.github.com"
    API_VERSION = "2022-11-28"
    
    def __init__(self, token: str):
        """
        Initialize GitHub API client.
        
        Args:
            token: Personal access token
        """
        self.token = token
        self.session = requests.Session()
        self.session.headers.update({
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": self.API_VERSION
        })
    
    def _request(
        self, 
        method: str, 
        endpoint: str, 
        params: Optional[Dict] = None,
        json_data: Optional[Dict] = None,
        retry_count: int = 3
    ) -> requests.Response:
        """
        Make API request with error handling and retries.
        
        Args:
            method: HTTP method
            endpoint: API endpoint
            params: Query parameters
            json_data: JSON body data
            retry_count: Number of retries
            
        Returns:
            Response object
            
        Raises:
            requests.HTTPError: On API error
        """
        url = f"{self.BASE_URL}{endpoint}"
        
        for attempt in range(retry_count):
            try:
                response = self.session.request(
                    method=method,
                    url=url,
                    params=params,
                    json=json_data
                )
                
                # Handle rate limiting
                if response.status_code == 429:
                    reset_time = int(response.headers.get('X-RateLimit-Reset', 0))
                    wait_time = max(reset_time - time.time(), 60)
                    if attempt < retry_count - 1:
                        time.sleep(wait_time)
                        continue
                
                response.raise_for_status()
                return response
                
            except requests.exceptions.RequestException as e:
                if attempt == retry_count - 1:
                    raise
                time.sleep(2 ** attempt)  # Exponential backoff
        
        raise requests.exceptions.RetryError("Max retries exceeded")
    
    # ========== Repository Operations ==========
    
    def list_repos(
        self, 
        type: str = "all",
        sort: str = "updated",
        direction: str = "desc",
        per_page: int = 100,
        page: int = 1
    ) -> List[GitHubRepo]:
        """
        List repositories for authenticated user.
        
        Args:
            type: all, owner, public, private, member
            sort: created, updated, pushed, full_name
            direction: asc, desc
            per_page: Results per page (max 100)
            page: Page number
            
        Returns:
            List of repositories
        """
        params = {
            "type": type,
            "sort": sort,
            "direction": direction,
            "per_page": per_page,
            "page": page
        }
        
        response = self._request("GET", "/user/repos", params=params)
        repos_data = response.json()
        
        return [
            GitHubRepo(
                id=repo["id"],
                name=repo["name"],
                full_name=repo["full_name"],
                private=repo["private"],
                html_url=repo["html_url"],
                ssh_url=repo["ssh_url_to_repo"],
                clone_url=repo["clone_url"],
                description=repo.get("description"),
                created_at=repo["created_at"],
                updated_at=repo["updated_at"],
                pushed_at=repo["pushed_at"],
                size=repo["size"],
                language=repo.get("language"),
                default_branch=repo["default_branch"]
            )
            for repo in repos_data
        ]
    
    def list_user_repos(self, username: str, per_page: int = 100) -> List[GitHubRepo]:
        """
        List public repositories for a specific user.
        
        Args:
            username: GitHub username
            per_page: Results per page
            
        Returns:
            List of repositories
        """
        response = self._request(
            "GET", 
            f"/users/{username}/repos",
            params={"per_page": per_page}
        )
        repos_data = response.json()
        
        return [
            GitHubRepo(
                id=repo["id"],
                name=repo["name"],
                full_name=repo["full_name"],
                private=repo["private"],
                html_url=repo["html_url"],
                ssh_url=repo["ssh_url_to_repo"],
                clone_url=repo["clone_url"],
                description=repo.get("description"),
                created_at=repo["created_at"],
                updated_at=repo["updated_at"],
                pushed_at=repo["pushed_at"],
                size=repo["size"],
                language=repo.get("language"),
                default_branch=repo["default_branch"]
            )
            for repo in repos_data
        ]
    
    def get_repo(self, owner: str, repo: str) -> GitHubRepo:
        """
        Get specific repository details.
        
        Args:
            owner: Repository owner
            repo: Repository name
            
        Returns:
            Repository information
        """
        response = self._request("GET", f"/repos/{owner}/{repo}")
        data = response.json()
        
        return GitHubRepo(
            id=data["id"],
            name=data["name"],
            full_name=data["full_name"],
            private=data["private"],
            html_url=data["html_url"],
            ssh_url=data["ssh_url_to_repo"],
            clone_url=data["clone_url"],
            description=data.get("description"),
            created_at=data["created_at"],
            updated_at=data["updated_at"],
            pushed_at=data["pushed_at"],
            size=data["size"],
            language=data.get("language"),
            default_branch=data["default_branch"]
        )
    
    def create_repo(
        self,
        name: str,
        description: Optional[str] = None,
        private: bool = False,
        auto_init: bool = True,
        gitignore_template: Optional[str] = None,
        license_template: Optional[str] = None
    ) -> GitHubRepo:
        """
        Create a new repository.
        
        Args:
            name: Repository name
            description: Repository description
            private: Make repository private
            auto_init: Create initial commit with README
            gitignore_template: .gitignore template (e.g., "Python")
            license_template: License template (e.g., "mit")
            
        Returns:
            Created repository information
        """
        data = {
            "name": name,
            "description": description,
            "private": private,
            "auto_init": auto_init,
            "has_issues": True,
            "has_projects": True,
            "has_wiki": True
        }
        
        if gitignore_template:
            data["gitignore_template"] = gitignore_template
        
        if license_template:
            data["license_template"] = license_template
        
        response = self._request("POST", "/user/repos", json_data=data)
        repo_data = response.json()
        
        return GitHubRepo(
            id=repo_data["id"],
            name=repo_data["name"],
            full_name=repo_data["full_name"],
            private=repo_data["private"],
            html_url=repo_data["html_url"],
            ssh_url=repo_data["ssh_url_to_repo"],
            clone_url=repo_data["clone_url"],
            description=repo_data.get("description"),
            created_at=repo_data["created_at"],
            updated_at=repo_data["updated_at"],
            pushed_at=repo_data["pushed_at"],
            size=repo_data["size"],
            language=repo_data.get("language"),
            default_branch=repo_data["default_branch"]
        )
    
    # ========== SSH Key Operations ==========
    
    def list_ssh_keys(self) -> List[GitHubSSHKey]:
        """
        List SSH keys for authenticated user.
        
        Returns:
            List of SSH keys
        """
        response = self._request("GET", "/user/keys")
        keys_data = response.json()
        
        return [
            GitHubSSHKey(
                id=key["id"],
                key=key["key"],
                title=key["title"],
                created_at=key["created_at"],
                verified=key.get("verified", False),
                read_only=key.get("read_only", False)
            )
            for key in keys_data
        ]
    
    def add_ssh_key(self, title: str, key: str) -> GitHubSSHKey:
        """
        Add SSH key to user account.
        
        Args:
            title: Key title/name
            key: Public key content
            
        Returns:
            Created SSH key information
        """
        data = {
            "title": title,
            "key": key
        }
        
        response = self._request("POST", "/user/keys", json_data=data)
        key_data = response.json()
        
        return GitHubSSHKey(
            id=key_data["id"],
            key=key_data["key"],
            title=key_data["title"],
            created_at=key_data["created_at"],
            verified=key_data.get("verified", False),
            read_only=key_data.get("read_only", False)
        )
    
    def delete_ssh_key(self, key_id: int) -> bool:
        """
        Delete SSH key from user account.
        
        Args:
            key_id: SSH key ID
            
        Returns:
            True if successful
        """
        self._request("DELETE", f"/user/keys/{key_id}")
        return True
    
    # ========== User Operations ==========
    
    def get_user(self) -> Dict[str, Any]:
        """
        Get authenticated user information.
        
        Returns:
            User information dictionary
        """
        response = self._request("GET", "/user")
        return response.json()
    
    def get_user_emails(self) -> List[Dict[str, Any]]:
        """
        Get user email addresses.
        
        Returns:
            List of email information
        """
        response = self._request("GET", "/user/emails")
        return response.json()
    
    def get_rate_limit(self) -> Dict[str, Any]:
        """
        Get current rate limit status.
        
        Returns:
            Rate limit information
        """
        response = self._request("GET", "/rate_limit")
        return response.json()


# Example usage
if __name__ == "__main__":
    import os
    
    # Initialize client
    token = os.getenv("GITHUB_TOKEN")
    if not token:
        print("Error: GITHUB_TOKEN environment variable not set")
        exit(1)
    
    client = GitHubAPIClient(token)
    
    # List repositories
    print("Fetching repositories...")
    repos = client.list_repos(per_page=10)
    for repo in repos:
        print(f"  - {repo.name} ({repo.ssh_url})")
    
    # Get user info
    print("\nUser information:")
    user = client.get_user()
    print(f"  Username: {user['login']}")
    print(f"  Name: {user.get('name', 'N/A')}")
    print(f"  Email: {user.get('email', 'N/A')}")
    
    # List SSH keys
    print("\nSSH Keys:")
    keys = client.list_ssh_keys()
    for key in keys:
        print(f"  - {key.title} (ID: {key.id})")



CODE 2

# src/git_manager/api/api_manager.py
"""
Unified API manager for both GitHub and GitLab.
"""

from typing import List, Dict, Optional, Union, Any
from pathlib import Path
import os

from .github_client import GitHubAPIClient, GitHubRepo, GitHubSSHKey
from .gitlab_client import GitLabAPIClient, GitLabProject, GitLabSSHKey


class APIManager:
    """Unified manager for GitHub and GitLab APIs."""
    
    def __init__(
        self,
        github_token: Optional[str] = None,
        gitlab_token: Optional[str] = None
    ):
        """
        Initialize API manager with tokens.
        
        Args:
            github_token: GitHub personal access token
            gitlab_token: GitLab personal access token
        """
        # Try to load tokens from environment if not provided
        self.github_token = github_token or os.getenv('GITHUB_TOKEN')
        self.gitlab_token = gitlab_token or os.getenv('GITLAB_TOKEN')
        
        # Initialize clients
        self.github = GitHubAPIClient(self.github_token) if self.github_token else None
        self.gitlab = GitLabAPIClient(self.gitlab_token) if self.gitlab_token else None
    
    def is_github_configured(self) -> bool:
        """Check if GitHub is configured."""
        return self.github is not None
    
    def is_gitlab_configured(self) -> bool:
        """Check if GitLab is configured."""
        return self.gitlab is not None
    
    # ========== Repository/Project Operations ==========
    
    def list_repos(
        self,
        platform: str,
        **kwargs
    ) -> Union[List[GitHubRepo], List[GitLabProject]]:
        """
        List repositories for specified platform.
        
        Args:
            platform: 'github' or 'gitlab'
            **kwargs: Platform-specific parameters
            
        Returns:
            List of repositories/projects
        """
        if platform.lower() == 'github':
            if not self.github:
                raise ValueError("GitHub not configured")
            return self.github.list_repos(**kwargs)
        
        elif platform.lower() == 'gitlab':
            if not self.gitlab:
                raise ValueError("GitLab not configured")
            return self.gitlab.list_projects(**kwargs)
        
        else:
            raise ValueError(f"Unknown platform: {platform}")
    
    def list_user_repos(
        self,
        platform: str,
        username: str,
        **kwargs
    ) -> Union[List[GitHubRepo], List[GitLabProject]]:
        """
        List repositories for a specific user.
        
        Args:
            platform: 'github' or 'gitlab'
            username: Username
            **kwargs: Platform-specific parameters
            
        Returns:
            List of repositories/projects
        """
        if platform.lower() == 'github':
            if not self.github:
                raise ValueError("GitHub not configured")
            return self.github.list_user_repos(username, **kwargs)
        
        elif platform.lower() == 'gitlab':
            if not self.gitlab:
                raise ValueError("GitLab not configured")
            return self.gitlab.list_user_projects(username, **kwargs)
        
        else:
            raise ValueError(f"Unknown platform: {platform}")
    
    def create_repo(
        self,
        platform: str,
        name: str,
        **kwargs
    ) -> Union[GitHubRepo, GitLabProject]:
        """
        Create a new repository.
        
        Args:
            platform: 'github' or 'gitlab'
            name: Repository name
            **kwargs: Platform-specific parameters
            
        Returns:
            Created repository/project
        """
        if platform.lower() == 'github':
            if not self.github:
                raise ValueError("GitHub not configured")
            return self.github.create_repo(name, **kwargs)
        
        elif platform.lower() == 'gitlab':
            if not self.gitlab:
                raise ValueError("GitLab not configured")
            return self.gitlab.create_project(name, **kwargs)
        
        else:
            raise ValueError(f"Unknown platform: {platform}")
    
    # ========== SSH Key Operations ==========
    
    def list_ssh_keys(
        self,
        platform: str
    ) -> Union[List[GitHubSSHKey], List[GitLabSSHKey]]:
        """
        List SSH keys for specified platform.
        
        Args:
            platform: 'github' or 'gitlab'
            
        Returns:
            List of SSH keys
        """
        if platform.lower() == 'github':
            if not self.github:
                raise ValueError("GitHub not configured")
            return self.github.list_ssh_keys()
        
        elif platform.lower() == 'gitlab':
            if not self.gitlab:
                raise ValueError("GitLab not configured")
            return self.gitlab.list_ssh_keys()
        
        else:
            raise ValueError(f"Unknown platform: {platform}")
    
    def add_ssh_key(
        self,
        platform: str,
        title: str,
        key: str,
        **kwargs
    ) -> Union[GitHubSSHKey, GitLabSSHKey]:
        """
        Add SSH key to platform.
        
        Args:
            platform: 'github' or 'gitlab'
            title: Key title
            key: Public key content
            **kwargs: Platform-specific parameters
            
        Returns:
            Created SSH key
        """
        if platform.lower() == 'github':
            if not self.github:
                raise ValueError("GitHub not configured")
            return self.github.add_ssh_key(title, key)
        
        elif platform.lower() == 'gitlab':
            if not self.gitlab:
                raise ValueError("GitLab not configured")
            return self.gitlab.add_ssh_key(title, key, **kwargs)
        
        else:
            raise ValueError(f"Unknown platform: {platform}")
    
    def delete_ssh_key(self, platform: str, key_id: int) -> bool:
        """
        Delete SSH key from platform.
        
        Args:
            platform: 'github' or 'gitlab'
            key_id: SSH key ID
            
        Returns:
            True if successful
        """
        if platform.lower() == 'github':
            if not self.github:
                raise ValueError("GitHub not configured")
            return self.github.delete_ssh_key(key_id)
        
        elif platform.lower() == 'gitlab':
            if not self.gitlab:
                raise ValueError("GitLab not configured")
            return self.gitlab.delete_ssh_key(key_id)
        
        else:
            raise ValueError(f"Unknown platform: {platform}")
    
    # ========== User Operations ==========
    
    def get_user(self, platform: str) -> Dict[str, Any]:
        """
        Get user information for platform.
        
        Args:
            platform: 'github' or 'gitlab'
            
        Returns:
            User information
        """
        if platform.lower() == 'github':
            if not self.github:
                raise ValueError("GitHub not configured")
            return self.github.get_user()
        
        elif platform.lower() == 'gitlab':
            if not self.gitlab:
                raise ValueError("GitLab not configured")
            return self.gitlab.get_user()
        
        else:
            raise ValueError(f"Unknown platform: {platform}")
    
    # ========== Utility Methods ==========
    
    def sync_ssh_key_to_platforms(
        self,
        title: str,
        key_path: Path,
        platforms: Optional[List[str]] = None
    ) -> Dict[str, bool]:
        """
        Sync SSH key to multiple platforms.
        
        Args:
            title: Key title
            key_path: Path to public key file
            platforms: List of platforms (default: all configured)
            
        Returns:
            Dict of platform -> success status
        """
        # Read public key
        with open(key_path, 'r') as f:
            public_key = f.read().strip()
        
        # Determine which platforms to sync to
        if platforms is None:
            platforms = []
            if self.is_github_configured():
                platforms.append('github')
            if self.is_gitlab_configured():
                platforms.append('gitlab')
        
        results = {}
        
        for platform in platforms:
            try:
                self.add_ssh_key(platform, title, public_key)
                results[platform] = True
            except Exception as e:
                print(f"Failed to add key to {platform}: {e}")
                results[platform] = False
        
        return results


# ========== CLI Integration ==========

def setup_api_cli_commands():
    """Setup CLI commands for API operations."""
    import click
    from rich.console import Console
    from rich.table import Table
    
    console = Console()
    
    @click.group()
    def api():
        """API operations for GitHub and GitLab."""
        pass
    
    @api.command()
    @click.option('--platform', '-p', 
                  type=click.Choice(['github', 'gitlab'], case_sensitive=False),
                  required=True,
                  help='Platform to query')
    @click.option('--username', '-u',
                  help='Username (leave empty for authenticated user)')
    def list_repos(platform: str, username: Optional[str]):
        """List repositories from GitHub or GitLab."""
        api_manager = APIManager()
        
        try:
            if username:
                repos = api_manager.list_user_repos(platform, username, per_page=50)
                console.print(f"\n[cyan]Repositories for {username} on {platform.upper()}:[/cyan]\n")
            else:
                repos = api_manager.list_repos(platform, per_page=50)
                console.print(f"\n[cyan]Your repositories on {platform.upper()}:[/cyan]\n")
            
            table = Table(show_header=True)
            table.add_column("Name", style="green")
            table.add_column("Visibility", style="yellow")
            table.add_column("SSH URL", style="blue")
            table.add_column("Updated", style="cyan")
            
            for repo in repos:
                if platform.lower() == 'github':
                    table.add_row(
                        repo.name,
                        "Private" if repo.private else "Public",
                        repo.ssh_url,
                        repo.updated_at[:10]
                    )
                else:  # gitlab
                    table.add_row(
                        repo.name,
                        repo.visibility.capitalize(),
                        repo.ssh_url_to_repo,
                        repo.last_activity_at[:10]
                    )
            
            console.print(table)
            console.print(f"\n[info]Total: {len(repos)} repositories[/info]")
            
        except Exception as e:
            console.print(f"[error]Error: {e}[/error]")
    
    @api.command()
    @click.option('--platform', '-p',
                  type=click.Choice(['github', 'gitlab'], case_sensitive=False),
                  required=True,
                  help='Platform to create on')
    @click.option('--name', '-n', required=True, help='Repository name')
    @click.option('--description', '-d', help='Repository description')
    @click.option('--private', is_flag=True, help='Make repository private')
    def create_repo(platform: str, name: str, description: Optional[str], private: bool):
        """Create a new repository."""
        api_manager = APIManager()
        
        try:
            console.print(f"\n[cyan]Creating repository '{name}' on {platform.upper()}...[/cyan]")
            
            repo = api_manager.create_repo(
                platform,
                name,
                description=description,
                private=private
            )
            
            console.print(f"[success]✓ Repository created successfully![/success]")
            console.print(f"\n[info]Name:[/info] {repo.name}")
            
            if platform.lower() == 'github':
                console.print(f"[info]URL:[/info] {repo.html_url}")
                console.print(f"[info]SSH:[/info] {repo.ssh_url}")
                console.print(f"[info]Clone:[/info] {repo.clone_url}")
            else:  # gitlab
                console.print(f"[info]URL:[/info] {repo.web_url}")
                console.print(f"[info]SSH:[/info] {repo.ssh_url_to_repo}")
                console.print(f"[info]HTTP:[/info] {repo.http_url_to_repo}")
            
        except Exception as e:
            console.print(f"[error]Error: {e}[/error]")
    
    @api.command()
    @click.option('--platform', '-p',
                  type=click.Choice(['github', 'gitlab'], case_sensitive=False),
                  required=True,
                  help='Platform to query')
    def list_keys(platform: str):
        """List SSH keys."""
        api_manager = APIManager()
        
        try:
            keys = api_manager.list_ssh_keys(platform)
            
            console.print(f"\n[cyan]SSH Keys on {platform.upper()}:[/cyan]\n")
            
            table = Table(show_header=True)
            table.add_column("ID", style="cyan")
            table.add_column("Title", style="green")
            table.add_column("Fingerprint", style="yellow")
            table.add_column("Created", style="blue")
            
            for key in keys:
                # Extract fingerprint (last part of key)
                fingerprint = key.key.split()[-1][:20] + "..."
                table.add_row(
                    str(key.id),
                    key.title,
                    fingerprint,
                    key.created_at[:10]
                )
            
            console.print(table)
            console.print(f"\n[info]Total: {len(keys)} SSH keys[/info]")
            
        except Exception as e:
            console.print(f"[error]Error: {e}[/error]")
    
    @api.command()
    @click.option('--platform', '-p',
                  type=click.Choice(['github', 'gitlab'], case_sensitive=False),
                  required=True,
                  help='Platform to add key to')
    @click.option('--title', '-t', required=True, help='Key title')
    @click.option('--key-file', '-k', type=click.Path(exists=True),
                  required=True, help='Path to public key file')
    def add_key(platform: str, title: str, key_file: str):
        """Add SSH key to platform."""
        api_manager = APIManager()
        
        try:
            key_path = Path(key_file)
            with open(key_path, 'r') as f:
                public_key = f.read().strip()
            
            console.print(f"\n[cyan]Adding SSH key to {platform.upper()}...[/cyan]")
            
            key = api_manager.add_ssh_key(platform, title, public_key)
            
            console.print(f"[success]✓ SSH key added successfully![/success]")
            console.print(f"\n[info]ID:[/info] {key.id}")
            console.print(f"[info]Title:[/info] {key.title}")
            
        except Exception as e:
            console.print(f"[error]Error: {e}[/error]")
    
    @api.command()
    @click.option('--platform', '-p',
                  type=click.Choice(['github', 'gitlab'], case_sensitive=False),
                  required=True,
                  help='Platform')
    def user_info(platform: str):
        """Get user information."""
        api_manager = APIManager()
        
        try:
            user = api_manager.get_user(platform)
            
            console.print(f"\n[cyan]User Information on {platform.upper()}:[/cyan]\n")
            
            if platform.lower() == 'github':
                console.print(f"[info]Username:[/info] {user.get('login')}")
                console.print(f"[info]Name:[/info] {user.get('name', 'N/A')}")
                console.print(f"[info]Email:[/info] {user.get('email', 'N/A')}")
                console.print(f"[info]Public Repos:[/info] {user.get('public_repos')}")
                console.print(f"[info]Followers:[/info] {user.get('followers')}")
            else:  # gitlab
                console.print(f"[info]Username:[/info] {user.get('username')}")
                console.print(f"[info]Name:[/info] {user.get('name', 'N/A')}")
                console.print(f"[info]Email:[/info] {user.get('email', 'N/A')}")
                console.print(f"[info]Projects:[/info] {user.get('projects_limit')}")
            
        except Exception as e:
            console.print(f"[error]Error: {e}[/error]")
    
    return api


if __name__ == "__main__":
    # Example usage
    api_manager = APIManager()
    
    print("API Manager initialized")
    print(f"GitHub configured: {api_manager.is_github_configured()}")
    print(f"GitLab configured: {api_manager.is_gitlab_configured()}")