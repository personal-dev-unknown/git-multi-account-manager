"""Shared interactive service - eliminates duplication between desktop and web."""

from pathlib import Path
from typing import Dict, List, Any, Optional
import subprocess
import requests

from .account_manager import AccountManager
from .ssh import SSHWorkflowOrchestrator, SSHIntegrationLayer
from .git_operations import GitOperations
from .database_manager import DatabaseManager
from .config_manager import ConfigManager
from .clone import CloneWorkflow
from .platform_config import PlatformManager
from .repository_manager import RepositoryManager
from ..utils.log_config import get_logger as get_advanced_logger, LogCategory
from ..utils.log_utils import log_git_operation, log_ssh_operation, ContextLogger


class InteractiveService:
    """Centralized service for all interactive operations."""
    
    def __init__(self):
        """Initialize service with core modules."""
        self.account_manager = AccountManager()
        self.ssh_orchestrator = SSHWorkflowOrchestrator()
        self.git_operations = GitOperations(self.account_manager)
        self.database_manager = DatabaseManager()
        self.config_manager = ConfigManager()
        self.platform_manager = PlatformManager()
        self.repository_manager = RepositoryManager()
        
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
        
        # Initialize SSH integration
        self.ssh_integration = SSHIntegrationLayer(
            self.database_manager,
            self.account_manager,
            self.config_manager
        )
    
    # ============ ACCOUNT MANAGEMENT ============
    
    def list_all_accounts(self) -> List[Dict[str, Any]]:
        """Get all configured accounts with platform info."""
        try:
            accounts = self.account_manager.list_accounts()
            result = []
            
            for account in accounts:
                platform_info = self.platform_manager.PLATFORMS.get(
                    account.platform.value
                )
                
                result.append({
                    'name': account.name,
                    'platform': account.platform.value,
                    'username': getattr(account, 'username', 'N/A'),
                    'email': account.email or 'N/A',
                    'host': getattr(account, 'host', 'N/A'),
                    'ssh_support': platform_info.supports_ssh if platform_info else False,
                    'has_ssh_key': bool(getattr(account, 'ssh_key_path', None)),
                    'has_pat': bool(getattr(account, 'pat_token', None))
                })
            
            return result
        except Exception as e:
            self.logger.error(f"Error listing accounts: {e}")
            return []
    
    def test_ssh_connection(self, account_name: str) -> Dict[str, Any]:
        """Test SSH connection for an account."""
        try:
            accounts = self.account_manager.list_accounts()
            account = next((a for a in accounts if a.name == account_name), None)
            
            if not account:
                return {'success': False, 'message': f'Account {account_name} not found'}
            
            # Test connection
            success, message, username = self.ssh_orchestrator._test_connection(
                platform=account.host,
                key_path=Path(account.ssh_key_path) if account.ssh_key_path else None
            )
            
            log_ssh_operation(
                "test_connection",
                account=account_name,
                success=success,
                error_msg=message if not success else None
            )
            
            return {
                'success': success,
                'message': message,
                'username': username if success else None
            }
        except Exception as e:
            self.logger.error(f"SSH test failed: {e}")
            return {'success': False, 'message': str(e)}
    
    # ============ CLONE OPERATIONS ============
    
    def analyze_repository_url(self, url: str) -> Dict[str, Any]:
        """Analyze a repository URL."""
        try:
            clone_workflow = CloneWorkflow(self.account_manager, self.config_manager)
            analysis = clone_workflow.analyze_external_repository(url)
            return analysis
        except Exception as e:
            self.logger.error(f"URL analysis failed: {e}")
            return {'success': False, 'error': str(e)}
    
    def get_personal_repositories(
        self,
        platform: str,
        account_name: str
    ) -> Dict[str, Any]:
        """Fetch personal repositories for an account."""
        try:
            clone_workflow = CloneWorkflow(self.account_manager, self.config_manager)
            repositories = clone_workflow.fetch_personal_repositories(
                platform,
                account_name
            )
            
            return {
                'success': True,
                'repositories': [
                    {
                        'name': repo.name,
                        'visibility': repo.visibility,
                        'description': repo.description,
                        'updated_at': repo.updated_at,
                        'ssh_url': repo.ssh_url,
                        'https_url': repo.https_url
                    }
                    for repo in repositories
                ]
            }
        except Exception as e:
            self.logger.error(f"Repository fetch failed: {e}")
            return {'success': False, 'error': str(e)}
    
    def clone_repository(
        self,
        repo_url: str,
        account_name: str,
        destination: str,
        auth_method: str = 'ssh',
        recursive: bool = False,
        shallow: bool = False
    ) -> Dict[str, Any]:
        """Clone a repository."""
        try:
            clone_workflow = CloneWorkflow(self.account_manager, self.config_manager)
            
            result = clone_workflow.clone_repository(
                repo_url,
                account_name,
                auth_method=auth_method,
                destination=destination,
                recursive=recursive,
                shallow=shallow
            )
            
            if result['success']:
                log_git_operation(
                    "clone",
                    repository=repo_url,
                    success=True
                )
            else:
                log_git_operation(
                    "clone",
                    repository=repo_url,
                    success=False,
                    error_msg=result.get('error')
                )
            
            return result
        except Exception as e:
            self.logger.error(f"Clone failed: {e}")
            log_git_operation("clone", success=False, error_msg=str(e))
            return {'success': False, 'error': str(e)}
    
    # ============ GIT OPERATIONS ============
    
    def check_repository_status(self, repo_path: str) -> Dict[str, Any]:
        """Check repository status."""
        try:
            repo_path = Path(repo_path)
            status = self.git_operations.check_status()
            
            # Get remote URL
            result = subprocess.run(
                ['git', 'config', '--get', 'remote.origin.url'],
                capture_output=True,
                text=True,
                cwd=repo_path
            )
            remote_url = result.stdout.strip() if result.returncode == 0 else "Unknown"
            
            # Match with account
            matched_account = None
            for account in self.account_manager.list_accounts():
                if (account.username in remote_url or 
                    account.host in remote_url):
                    matched_account = account
                    break
            
            return {
                'success': True,
                'branch': status.current_branch,
                'remote_url': remote_url,
                'account': matched_account.name if matched_account else None,
                'uncommitted_changes': status.has_uncommitted_changes,
                'uncommitted_count': len(status.uncommitted_files or []),
                'commits_ahead': status.commits_ahead,
                'commits_behind': status.commits_behind
            }
        except Exception as e:
            self.logger.error(f"Status check failed: {e}")
            return {'success': False, 'error': str(e)}
    
    def git_push(self, repo_path: str) -> Dict[str, Any]:
        """Push changes to remote."""
        try:
            result = self.git_operations.push(repo_path)
            log_git_operation("push", success=result.get('success'))
            return result
        except Exception as e:
            self.logger.error(f"Push failed: {e}")
            log_git_operation("push", success=False, error_msg=str(e))
            return {'success': False, 'error': str(e)}
    
    def git_pull(self, repo_path: str) -> Dict[str, Any]:
        """Pull changes from remote."""
        try:
            result = self.git_operations.pull(repo_path)
            log_git_operation("pull", success=result.get('success'))
            return result
        except Exception as e:
            self.logger.error(f"Pull failed: {e}")
            log_git_operation("pull", success=False, error_msg=str(e))
            return {'success': False, 'error': str(e)}
    
    def git_sync(self, repo_path: str) -> Dict[str, Any]:
        """Sync repository (pull then push)."""
        try:
            result = self.git_operations.sync(repo_path)
            log_git_operation("sync", success=result.get('success'))
            return result
        except Exception as e:
            self.logger.error(f"Sync failed: {e}")
            log_git_operation("sync", success=False, error_msg=str(e))
            return {'success': False, 'error': str(e)}
    
    # ============ REPOSITORY SETUP ============
    
    def setup_new_repository(
        self,
        repo_path: str,
        account_name: str,
        repo_name: str,
        description: str = "",
        branch: str = "main"
    ) -> Dict[str, Any]:
        """Setup a new local repository."""
        try:
            accounts = self.account_manager.list_accounts()
            account = next((a for a in accounts if a.name == account_name), None)
            
            if not account:
                return {'success': False, 'error': f'Account {account_name} not found'}
            
            repo_path = Path(repo_path)
            repo_path.mkdir(parents=True, exist_ok=True)
            
            result = self.repository_manager.setup_new_repository(
                repo_path=repo_path,
                account=account,
                repo_name=repo_name,
                description=description,
                branch=branch,
                initialize_git=True
            )
            
            log_git_operation(
                "setup_repository",
                repository=str(repo_path),
                success=result.get('success')
            )
            
            return result
        except Exception as e:
            self.logger.error(f"Repository setup failed: {e}")
            log_git_operation("setup_repository", success=False, error_msg=str(e))
            return {'success': False, 'error': str(e)}
    
    # ============ SSH KEY MANAGEMENT ============
    
    def generate_ssh_key(
        self,
        account_name: str,
        email: str,
        platform: str,
        account_type: str = "personal",
        key_type: str = "ed25519",
        passphrase: Optional[str] = None
    ) -> Dict[str, Any]:
        """Generate SSH key for account."""
        try:
            with ContextLogger("generate_ssh_key", category=LogCategory.SSH_OPERATION):
                platform_info = self.platform_manager.PLATFORMS.get(platform)
                
                if not platform_info or not platform_info.supports_ssh:
                    return {
                        'success': False,
                        'error': f'Platform {platform} does not support SSH'
                    }
                
                platform_domain = platform_info.api_url or f"{platform}.com"
                
                result = self.ssh_orchestrator.setup_account(
                    name=account_name,
                    email=email,
                    platform=platform_domain,
                    account_type=account_type,
                    key_type=key_type,
                    passphrase=passphrase
                )
                
                log_ssh_operation(
                    "generate_key",
                    account=account_name,
                    success=result.get('success')
                )
                
                return result
        except Exception as e:
            self.logger.error(f"Key generation failed: {e}")
            log_ssh_operation("generate_key", account=account_name, success=False, error_msg=str(e))
            return {'success': False, 'error': str(e)}
    
    def test_pat_token(self, platform: str, pat_token: str) -> Dict[str, Any]:
        """Test if a PAT token is valid."""
        try:
            if platform == 'github':
                response = requests.get(
                    'https://api.github.com/user',
                    headers={
                        'Authorization': f'token {pat_token}',
                        'Accept': 'application/vnd.github.v3+json'
                    },
                    timeout=5
                )
                is_valid = response.status_code == 200
            
            elif platform == 'gitlab':
                response = requests.get(
                    'https://gitlab.com/api/v4/user',
                    headers={'PRIVATE-TOKEN': pat_token},
                    timeout=5
                )
                is_valid = response.status_code == 200
            
            elif platform == 'bitbucket':
                response = requests.get(
                    'https://api.bitbucket.org/2.0/user',
                    auth=('x-token-auth', pat_token),
                    timeout=5
                )
                is_valid = response.status_code == 200
            
            else:
                is_valid = False
            
            return {
                'success': True,
                'valid': is_valid,
                'message': 'PAT is valid' if is_valid else 'PAT is invalid'
            }
        except Exception as e:
            self.logger.error(f"PAT test failed: {e}")
            return {'success': False, 'error': str(e)}
    
    # ============ PLATFORM INFO ============
    
    def get_all_platforms(self) -> Dict[str, Any]:
        """Get all available platforms."""
        try:
            platforms = []
            
            for platform_key, platform_info in self.platform_manager.PLATFORMS.items():
                platforms.append({
                    'name': platform_key,
                    'supports_ssh': platform_info.supports_ssh,
                    'api_url': platform_info.api_url,
                    'supports_pat': True,
                    'ssh_host': platform_info.api_url or f"git@{platform_key}.com"
                })
            
            return {'success': True, 'platforms': platforms}
        except Exception as e:
            self.logger.error(f"Platform fetch failed: {e}")
            return {'success': False, 'error': str(e)}
    
    def get_platform_info(self, platform_name: str) -> Dict[str, Any]:
        """Get platform information."""
        try:
            platform_info = self.platform_manager.PLATFORMS.get(platform_name)
            
            if not platform_info:
                return {
                    'success': False,
                    'error': f'Platform {platform_name} not found'
                }
            
            return {
                'success': True,
                'name': platform_name,
                'supports_ssh': platform_info.supports_ssh,
                'api_url': platform_info.api_url,
                'supports_pat': True,
                'ssh_host': platform_info.api_url or f"git@{platform_name}.com"
            }
        except Exception as e:
            self.logger.error(f"Platform info fetch failed: {e}")
            return {'success': False, 'error': str(e)}
