"""Clone API endpoints."""

from typing import Dict, Optional, List
from ..workflow import CloneWorkflow
from ..parsers import URLParser
from ...database_manager import DatabaseManager


class CloneAPI:
    """API for clone operations."""
    
    def __init__(self, account_manager, config_manager=None, database_manager=None):
        self.database_manager = database_manager or DatabaseManager()
        self.workflow = CloneWorkflow(account_manager, config_manager, self.database_manager)
        self.account_manager = account_manager
    
    def clone_external_repository(
        self,
        repo_url: str,
        account_name: str,
        auth_method: str = 'ssh',
        destination: Optional[str] = None,
        recursive: bool = False,
        shallow: bool = False,
        fork: bool = False
    ) -> Dict:
        """
        Clone external repository.
        
        Args:
            repo_url: Repository URL
            account_name: Account to use
            auth_method: Authentication method
            destination: Clone destination
            recursive: Clone submodules
            shallow: Shallow clone
            fork: Fork before cloning
        
        Returns:
            Clone result
        """
        
        try:
            # Analyze repository
            analysis = self.workflow.analyze_external_repository(repo_url)
            if not analysis['success']:
                return analysis
            
            # Fork if requested
            if fork:
                platform = analysis['platform']
                fork_result = self.workflow.fork_repository(repo_url, account_name, platform)
                if not fork_result['success']:
                    return fork_result
                
                # Use fork URL
                repo_url = fork_result['fork_url']
                upstream_url = analysis['ssh_url']
            else:
                upstream_url = None
            
            # Clone repository
            result = self.workflow.clone_repository(
                repo_url=repo_url,
                account_name=account_name,
                auth_method=auth_method,
                destination=destination,
                recursive=recursive,
                shallow=shallow,
                upstream_url=upstream_url
            )
            
            return result
        
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def clone_personal_repository(
        self,
        platform: str,
        account_name: str,
        repo_name: str,
        auth_method: str = 'ssh',
        destination: Optional[str] = None,
        recursive: bool = False,
        shallow: bool = False
    ) -> Dict:
        """
        Clone personal repository.
        
        Args:
            platform: Platform ID
            account_name: Account name
            repo_name: Repository name
            auth_method: Authentication method
            destination: Clone destination
            recursive: Clone submodules
            shallow: Shallow clone
        
        Returns:
            Clone result
        """
        
        try:
            # Fetch personal repositories
            repositories = self.workflow.fetch_personal_repositories(
                platform,
                account_name,
                force_refresh=False
            )
            
            # Find repository
            repo = None
            for r in repositories:
                if r.name == repo_name:
                    repo = r
                    break
            
            if not repo:
                return {
                    'success': False,
                    'error': f"Repository not found: {repo_name}"
                }
            
            # Clone repository
            result = self.workflow.clone_repository(
                repo_url=repo.ssh_url if auth_method == 'ssh' else repo.https_url,
                account_name=account_name,
                auth_method=auth_method,
                destination=destination,
                recursive=recursive,
                shallow=shallow
            )
            
            return result
        
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def get_clone_status(self, destination: str) -> Dict:
        """
        Get clone status/information.
        
        Args:
            destination: Clone destination
        
        Returns:
            Clone status
        """
        
        import os
        import subprocess
        
        try:
            if not os.path.exists(destination):
                return {
                    'success': False,
                    'error': 'Destination not found'
                }
            
            # Get git status
            result = subprocess.run(
                ['git', '-C', destination, 'status', '--porcelain'],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            # Get remote info
            remote_result = subprocess.run(
                ['git', '-C', destination, 'remote', '-v'],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            # Get branch info
            branch_result = subprocess.run(
                ['git', '-C', destination, 'branch', '-v'],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            return {
                'success': True,
                'destination': destination,
                'status': result.stdout,
                'remotes': remote_result.stdout,
                'branches': branch_result.stdout
            }
        
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
