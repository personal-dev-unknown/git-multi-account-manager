"""Interactive manager for desktop - bridges CLI interactive features to desktop UI."""

from typing import Dict, List, Any

from ..core.interactive_service import InteractiveService
from ..core.platform_config import PlatformManager


class InteractiveDesktopManager:
    """Manages interactive workflows for desktop application."""
    
    def __init__(self):
        """Initialize interactive manager with shared service."""
        self.service = InteractiveService()
        self.platform_manager = PlatformManager()
    
    # ============ ACCOUNT MANAGEMENT ============
    
    def list_all_accounts(self) -> List[Dict[str, Any]]:
        """Get all configured accounts with platform info."""
        return self.service.list_all_accounts()
    
    def test_ssh_connection(self, account_name: str) -> Dict[str, Any]:
        """Test SSH connection for an account."""
        return self.service.test_ssh_connection(account_name)
    
    # ============ CLONE OPERATIONS ============
    
    def analyze_repository_url(self, url: str) -> Dict[str, Any]:
        """Analyze a repository URL."""
        return self.service.analyze_repository_url(url)
    
    def get_personal_repositories(
        self,
        platform: str,
        account_name: str
    ) -> Dict[str, Any]:
        """Fetch personal repositories for an account."""
        return self.service.get_personal_repositories(platform, account_name)
    
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
        return self.service.clone_repository(
            repo_url,
            account_name,
            destination,
            auth_method,
            recursive,
            shallow
        )
    
    # ============ GIT OPERATIONS ============
    
    def check_repository_status(self, repo_path: str) -> Dict[str, Any]:
        """Check repository status."""
        return self.service.check_repository_status(repo_path)
    
    def git_push(self, repo_path: str, message: str = None) -> Dict[str, Any]:
        """Push changes to remote."""
        return self.service.git_push(repo_path)
    
    def git_pull(self, repo_path: str) -> Dict[str, Any]:
        """Pull changes from remote."""
        return self.service.git_pull(repo_path)
    
    def git_sync(self, repo_path: str) -> Dict[str, Any]:
        """Sync repository (pull then push)."""
        return self.service.git_sync(repo_path)
    
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
        return self.service.setup_new_repository(
            repo_path,
            account_name,
            repo_name,
            description,
            branch
        )
    
    # ============ SSH KEY MANAGEMENT ============
    
    def generate_ssh_key(
        self,
        account_name: str,
        email: str,
        platform: str,
        account_type: str = "personal",
        key_type: str = "ed25519",
        passphrase: str = None
    ) -> Dict[str, Any]:
        """Generate SSH key for account."""
        return self.service.generate_ssh_key(
            account_name,
            email,
            platform,
            account_type,
            key_type,
            passphrase
        )
    
    def test_pat_token(self, platform: str, pat_token: str) -> Dict[str, Any]:
        """Test if a PAT token is valid."""
        return self.service.test_pat_token(platform, pat_token)
    
    def get_platform_info(self, platform: str) -> Dict[str, Any]:
        """Get platform information."""
        return self.service.get_platform_info(platform)
