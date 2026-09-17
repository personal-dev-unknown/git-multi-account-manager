"""Clone workflow orchestration."""

import os
from pathlib import Path
from typing import Dict, Optional, List
from datetime import datetime

from .clone_url_parser import URLParser
from .clone_platform_config import get_platform_config
from .clone_repository_fetcher import RepositoryFetcher, Repository
from .clone_auth_handler import AuthenticationHandler


class CloneWorkflow:
    """Orchestrate clone operations."""
    
    def __init__(self, account_manager, config_manager=None):
        self.account_manager = account_manager
        self.config_manager = config_manager
        self.fetcher = RepositoryFetcher()
        self.auth_handler = AuthenticationHandler()
        self.default_clone_dir = Path.home() / 'projects'
    
    def get_accounts_for_platform(self, platform: str) -> List[Dict]:
        """Get all accounts for a platform."""
        all_accounts = self.account_manager.list_accounts()
        return [
            acc for acc in all_accounts
            if hasattr(acc, 'platform') and acc.platform.value == platform
        ]
    
    def fetch_personal_repositories(
        self,
        platform: str,
        account_name: str
    ) -> List[Repository]:
        """
        Fetch user's personal repositories.
        
        Args:
            platform: Platform ID ('github', 'gitlab', etc.)
            account_name: Account name
        
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
        
        # Fetch repositories
        return self.fetcher.fetch_repositories(platform, account_dict)
    
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
                
                success, message = self.auth_handler.clone_with_ssh(
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
                success, message = self.auth_handler.clone_with_pat(
                    repo_url,
                    pat_token,
                    destination,
                    platform=platform,
                    recursive=recursive,
                    shallow=shallow
                )
            
            elif auth_method == 'password':
                success, message = self.auth_handler.clone_with_password(
                    repo_url,
                    account.username,
                    destination,
                    recursive=recursive,
                    shallow=shallow
                )
            
            elif auth_method == 'anonymous':
                success, message = self.auth_handler.clone_anonymous(
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
                return {
                    'success': False,
                    'error': message
                }
            
            # Post-clone setup
            account_dict = {
                'username': account.username,
                'email': account.email,
            }
            
            setup_success, setup_message = self.auth_handler.setup_post_clone(
                destination,
                account_dict,
                upstream_url=upstream_url
            )
            
            return {
                'success': True,
                'destination': destination,
                'message': message,
                'setup_message': setup_message,
                'auth_method': auth_method,
                'account': account_name,
                'timestamp': datetime.now().isoformat()
            }
        
        except Exception as e:
            return {
                'success': False,
                'error': f"Clone operation failed: {str(e)}"
            }
    
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
            
            # Parse URL
            parsed = URLParser.parse(repo_url)
            
            # Get platform config
            config = get_platform_config(platform)
            
            if not config.api_base:
                return {
                    'success': False,
                    'error': f"Platform {platform} does not support forking"
                }
            
            # Get PAT
            pat_token = getattr(account, 'pat_token', None)
            if not pat_token:
                return {
                    'success': False,
                    'error': f"PAT not configured for account: {account_name}"
                }
            
            # Make fork request
            import requests
            
            if platform == 'github':
                headers = {'Authorization': f"token {pat_token}"}
                response = requests.post(
                    f"{config.api_base}/repos/{parsed.owner}/{parsed.repo}/forks",
                    headers=headers,
                    timeout=10
                )
            
            elif platform == 'gitlab':
                headers = {'PRIVATE-TOKEN': pat_token}
                # GitLab uses project ID, need to get it first
                response = requests.post(
                    f"{config.api_base}/projects/{parsed.owner}%2F{parsed.repo}/fork",
                    headers=headers,
                    timeout=10
                )
            
            else:
                return {
                    'success': False,
                    'error': f"Forking not supported for {platform}"
                }
            
            if response.status_code in [200, 201]:
                fork_data = response.json()
                
                if platform == 'github':
                    fork_url = fork_data['ssh_url']
                else:
                    fork_url = fork_data['ssh_url_to_repo']
                
                return {
                    'success': True,
                    'fork_url': fork_url,
                    'fork_owner': account.username,
                    'message': f"Successfully forked to {account.username}/{parsed.repo}"
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
