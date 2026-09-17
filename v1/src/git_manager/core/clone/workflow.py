"""Main clone workflow orchestration."""

import os
from pathlib import Path
from typing import Dict, Optional, List
from datetime import datetime

from .parsers import URLParser
from .platforms import (
    GitHubPlatform, GitLabPlatform, BitbucketPlatform,
    AzureDevOpsPlatform, SourceForgePlatform, SelfHostedPlatform,
    CloudStoragePlatform, LocalPathPlatform, CustomPlatform
)
from .auth import SSHAuth, HTTPSPATAuth, HTTPSPasswordAuth, AnonymousAuth
from .cache import RepositoryCache
from ..database_manager import DatabaseManager


class CloneWorkflow:
    """Orchestrate clone operations."""
    
    def __init__(self, account_manager, config_manager=None, database_manager=None):
        self.account_manager = account_manager
        self.config_manager = config_manager
        self.database_manager = database_manager or DatabaseManager()
        self.cache = RepositoryCache()
        self.default_clone_dir = Path.home() / 'projects'
        
        # Initialize platforms (8 total)
        self.platforms = {
            'github': GitHubPlatform(),
            'gitlab': GitLabPlatform(),
            'bitbucket': BitbucketPlatform(),
            'azure_devops': AzureDevOpsPlatform(),
            'sourceforge': SourceForgePlatform(),
            'self_hosted': SelfHostedPlatform(),
            'cloud_storage': CloudStoragePlatform(),
            'local_path': LocalPathPlatform(),
            'custom': CustomPlatform(),
        }
    
    def get_accounts_for_platform(self, platform: str) -> List[Dict]:
        """Get all accounts for a platform."""
        all_accounts = self.account_manager.list_accounts()
        return [
            acc for acc in all_accounts
            if hasattr(acc, 'platform') and acc.platform.value == platform
        ]
    
    def get_platform(self, platform_id: str):
        """Get platform instance."""
        return self.platforms.get(platform_id, self.platforms['custom'])
    
    def fetch_personal_repositories(
        self,
        platform: str,
        account_name: str,
        force_refresh: bool = False
    ):
        """
        Fetch user's personal repositories.
        
        Args:
            platform: Platform ID ('github', 'gitlab', etc.)
            account_name: Account name
            force_refresh: Ignore cache
        
        Returns:
            List of Repository objects
        """
        
        # Get account
        account = self.account_manager.get_account(account_name)
        if not account:
            raise ValueError(f"Account not found: {account_name}")
        
        # Convert account object to dict
        account_dict = {
            'username': account.username,
            'pat_token': getattr(account, 'pat_token', None),
            'email': account.email,
        }
        
        # Check cache
        if not force_refresh:
            cached = self.cache.get(account_name, platform)
            if cached and not self.cache.is_expired(cached):
                # Return cached repositories
                from .platforms.base import Repository
                repos = []
                for repo_data in cached['repositories']:
                    repos.append(Repository(**repo_data))
                return repos
        
        # Fetch from platform
        platform_obj = self.get_platform(platform)
        repositories = platform_obj.fetch_repositories(account_dict)
        
        # Cache results
        repo_dicts = [repo.__dict__ for repo in repositories]
        self.cache.set(account_name, platform, repo_dicts)
        
        return repositories
    
    def analyze_external_repository(self, repo_url: str) -> Dict:
        """
        Analyze an external repository URL.
        
        Args:
            repo_url: Repository URL
        
        Returns:
            Dictionary with repository info
        """
        
        try:
            # Parse URL
            parsed = URLParser.parse(repo_url)
            
            # Detect platform
            platform = URLParser.detect_platform(repo_url)
            
            return {
                'success': True,
                'url': repo_url,
                'parsed': parsed.to_dict(),
                'platform': platform,
                'ssh_url': parsed.to_ssh(),
                'https_url': parsed.to_https(),
            }
        
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def prepare_clone_destination(
        self,
        repo_name: str,
        account_name: Optional[str] = None,
        custom_path: Optional[str] = None
    ) -> str:
        """
        Prepare clone destination directory.
        
        Args:
            repo_name: Repository name
            account_name: Account name (for organizing)
            custom_path: Custom destination path
        
        Returns:
            Full path to clone destination
        """
        
        if custom_path:
            # Use custom path
            dest = Path(custom_path).expanduser().absolute()
        else:
            # Use default: ~/projects/account/repo
            if account_name:
                dest = self.default_clone_dir / account_name / repo_name
            else:
                dest = self.default_clone_dir / repo_name
        
        # Create parent directories
        dest.parent.mkdir(parents=True, exist_ok=True)
        
        return str(dest)
    
    def clone_repository(
        self,
        repo_url: str,
        account_name: str,
        auth_method: str = 'ssh',
        destination: Optional[str] = None,
        recursive: bool = False,
        shallow: bool = False,
        upstream_url: Optional[str] = None
    ) -> Dict:
        """
        Clone a repository.
        
        Args:
            repo_url: Repository URL
            account_name: Account to use for authentication
            auth_method: Authentication method ('ssh', 'pat', 'password', 'anonymous')
            destination: Clone destination (auto-generated if not provided)
            recursive: Clone submodules recursively
            shallow: Shallow clone (--depth=1)
            upstream_url: Optional upstream remote URL (for forks)
        
        Returns:
            Dictionary with clone result
        """
        
        try:
            # Get account
            account = self.account_manager.get_account(account_name)
            if not account:
                return {
                    'success': False,
                    'error': f"Account not found: {account_name}"
                }
            
            # Parse repository URL
            parsed = URLParser.parse(repo_url)
            
            # Prepare destination
            if not destination:
                destination = self.prepare_clone_destination(
                    parsed.repo,
                    account_name
                )
            else:
                destination = str(Path(destination).expanduser().absolute())
            
            # Check if destination already exists
            if os.path.exists(destination):
                return {
                    'success': False,
                    'error': f"Destination already exists: {destination}"
                }
            
            # Perform clone based on auth method
            if auth_method == 'ssh':
                ssh_key_path = getattr(account, 'ssh_key_path', None)
                if not ssh_key_path:
                    return {
                        'success': False,
                        'error': f"SSH key not configured for account: {account_name}"
                    }
                
                success, message = SSHAuth.clone(
                    repo_url,
                    ssh_key_path,
                    destination,
                    recursive=recursive,
                    shallow=shallow
                )
            
            elif auth_method == 'pat':
                pat_token = getattr(account, 'pat_token', None)
                if not pat_token:
                    return {
                        'success': False,
                        'error': f"PAT not configured for account: {account_name}"
                    }
                
                platform = URLParser.detect_platform(repo_url)
                success, message = HTTPSPATAuth.clone(
                    repo_url,
                    pat_token,
                    destination,
                    platform=platform,
                    recursive=recursive,
                    shallow=shallow
                )
            
            elif auth_method == 'password':
                success, message = HTTPSPasswordAuth.clone(
                    repo_url,
                    account.username,
                    destination,
                    recursive=recursive,
                    shallow=shallow
                )
            
            elif auth_method == 'anonymous':
                success, message = AnonymousAuth.clone(
                    repo_url,
                    destination,
                    recursive=recursive,
                    shallow=shallow
                )
            
            else:
                return {
                    'success': False,
                    'error': f"Unknown auth method: {auth_method}"
                }
            
            if not success:
                # Log failed clone operation
                platform = URLParser.detect_platform(repo_url)
                self.database_manager.log_clone_operation(
                    clone_url=repo_url,
                    destination=destination,
                    method=auth_method,
                    platform_id=platform,
                    account_id=None,
                    status='failed',
                    error_message=message
                )
                
                return {
                    'success': False,
                    'error': message
                }
            
            # Post-clone setup
            account_dict = {
                'username': account.username,
                'email': account.email,
            }
            
            self._setup_post_clone(
                destination,
                account_dict,
                upstream_url=upstream_url
            )
            
            # Log successful clone operation
            platform = URLParser.detect_platform(repo_url)
            self.database_manager.log_clone_operation(
                clone_url=repo_url,
                destination=destination,
                method=auth_method,
                platform_id=platform,
                account_id=None,
                status='success'
            )
            
            # Add repository to database
            try:
                self.database_manager.add_repository(
                    path=destination,
                    platform_id=platform,
                    account_id=None,
                    owner=parsed.owner,
                    name=parsed.repo,
                    full_name=f"{parsed.owner}/{parsed.repo}",
                    clone_url=repo_url,
                    ssh_url=parsed.to_ssh(),
                    https_url=parsed.to_https(),
                    repository_type='external',
                    visibility='unknown',
                    clone_method=auth_method
                )
            except Exception:
                pass  # Ignore database errors
            
            return {
                'success': True,
                'destination': destination,
                'message': message,
                'auth_method': auth_method,
                'account': account_name,
                'timestamp': datetime.now().isoformat()
            }
        
        except Exception as e:
            return {
                'success': False,
                'error': f"Clone operation failed: {str(e)}"
            }
    
    def _setup_post_clone(
        self,
        repo_path: str,
        account: Dict,
        upstream_url: Optional[str] = None
    ):
        """Configure repository after clone."""
        import subprocess
        
        try:
            # Set git user config
            if account.get('email'):
                subprocess.run(
                    ['git', '-C', repo_path, 'config', 'user.email', account['email']],
                    capture_output=True,
                    timeout=10
                )
            
            if account.get('username'):
                subprocess.run(
                    ['git', '-C', repo_path, 'config', 'user.name', account['username']],
                    capture_output=True,
                    timeout=10
                )
            
            # Add upstream remote if provided
            if upstream_url:
                subprocess.run(
                    ['git', '-C', repo_path, 'remote', 'add', 'upstream', upstream_url],
                    capture_output=True,
                    timeout=10
                )
        
        except Exception:
            pass  # Ignore post-clone setup errors
    
    def fork_repository(
        self,
        repo_url: str,
        account_name: str,
        platform: str
    ) -> Dict:
        """
        Fork a repository (requires API access).
        
        Args:
            repo_url: Repository URL
            account_name: Account to fork to
            platform: Platform ID
        
        Returns:
            Dictionary with fork result
        """
        
        try:
            # Get account
            account = self.account_manager.get_account(account_name)
            if not account:
                return {
                    'success': False,
                    'error': f"Account not found: {account_name}"
                }
            
            # Convert account to dict
            account_dict = {
                'username': account.username,
                'pat_token': getattr(account, 'pat_token', None),
            }
            
            # Get platform and fork
            platform_obj = self.get_platform(platform)
            return platform_obj.fork_repository(repo_url, account_dict)
        
        except Exception as e:
            return {
                'success': False,
                'error': f"Fork operation failed: {str(e)}"
            }
