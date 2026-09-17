"""Repository Setup - Complete repository initialization workflow."""

import subprocess
from pathlib import Path
from typing import Optional, Dict
from datetime import datetime

from ...models.account import Account
from ...utils.logger import get_logger
from ..exceptions import RepositorySetupError, RepositoryInitializationError


logger = get_logger(__name__)


class RepositorySetup:
    """Complete repository setup workflow."""
    
    def setup_new_repository(
        self,
        repo_path: Path,
        account: Account,
        repo_name: Optional[str] = None,
        description: Optional[str] = None,
        branch: str = "main",
        initialize_git: bool = True
    ) -> Dict:
        """Setup a new repository with complete initialization.
        
        Args:
            repo_path: Path to repository directory
            account: Account to use for this repository
            repo_name: Name of the repository (defaults to directory name)
            description: Repository description
            branch: Default branch name (default: main)
            initialize_git: Whether to initialize Git (default: True)
            
        Returns:
            Repository configuration dictionary
            
        Raises:
            RepositorySetupError: If setup fails
        """
        try:
            repo_name = repo_name or repo_path.name
            
            # Step 1: Create directory if it doesn't exist
            repo_path.mkdir(parents=True, exist_ok=True)
            logger.info(f"Repository directory ready: {repo_path}")
            
            # Step 2: Initialize Git repository
            if initialize_git:
                self._initialize_git(repo_path)
            
            # Step 3: Configure Git user settings
            self._configure_git_user(repo_path, account)
            
            # Step 4: Configure SSH for repository
            self._configure_git_ssh(repo_path, account)
            
            # Step 5: Create .gitignore
            self._create_gitignore(repo_path)
            
            # Step 6: Create initial commit
            self._create_initial_commit(repo_path, repo_name, description)
            
            # Step 7: Setup branch
            self._setup_branch(repo_path, branch)
            
            # Step 8: Track repository
            repo_config = self._track_repository(
                repo_path, repo_name, account, branch, description
            )
            
            logger.info(f"Successfully setup repository: {repo_name}")
            return repo_config
            
        except subprocess.CalledProcessError as e:
            error_msg = f"Repository setup failed: {e.stderr}"
            logger.error(error_msg)
            raise RepositorySetupError(error_msg)
        except Exception as e:
            error_msg = f"Unexpected error during setup: {str(e)}"
            logger.error(error_msg)
            raise RepositorySetupError(error_msg)
    
    def _initialize_git(self, repo_path: Path) -> None:
        """Initialize Git repository."""
        try:
            subprocess.run(
                ['git', 'init'],
                cwd=repo_path,
                capture_output=True,
                check=True
            )
            logger.info(f"Initialized git repository: {repo_path}")
        except subprocess.CalledProcessError as e:
            raise RepositoryInitializationError(f"Failed to initialize git: {e.stderr}")
    
    def _configure_git_user(self, repo_path: Path, account: Account) -> None:
        """Configure Git user settings for repository."""
        try:
            subprocess.run(
                ['git', 'config', 'user.email', account.email or ''],
                cwd=repo_path,
                capture_output=True,
                check=True
            )
            
            subprocess.run(
                ['git', 'config', 'user.name', account.username],
                cwd=repo_path,
                capture_output=True,
                check=True
            )
            logger.info(f"Configured git user for {account.username}")
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to configure git user: {e.stderr}")
            raise
    
    def _configure_git_ssh(self, repo_path: Path, account: Account) -> None:
        """Configure SSH for Git operations."""
        try:
            if hasattr(account, 'ssh_key_path') and account.ssh_key_path:
                subprocess.run(
                    ['git', 'config', 'core.sshCommand',
                     f'ssh -i {account.ssh_key_path}'],
                    cwd=repo_path,
                    capture_output=True,
                    check=True
                )
                logger.info(f"Configured SSH key: {account.ssh_key_path}")
        except subprocess.CalledProcessError as e:
            logger.warning(f"Failed to configure SSH: {e.stderr}")
    
    def _create_gitignore(self, repo_path: Path) -> None:
        """Create .gitignore file with universal patterns."""
        gitignore_path = repo_path / '.gitignore'
        
        gitignore_content = """# Environment & Configuration
.env
.env.local
.env.*.local
config.local.*
secrets.*
credentials.*

# IDE & Editor Files
.vscode/
.idea/
*.swp
*.swo
*~
.DS_Store
Thumbs.db

# Build & Dependencies
node_modules/
dist/
build/
*.min.js
*.min.css
__pycache__/
*.pyc
*.egg-info/
.venv/
venv/

# Version Control
.git/
.gitignore

# OS Files
.DS_Store
Thumbs.db
"""
        
        try:
            gitignore_path.write_text(gitignore_content)
            logger.info("Created .gitignore")
        except Exception as e:
            logger.warning(f"Failed to create .gitignore: {e}")
    
    def _create_initial_commit(
        self,
        repo_path: Path,
        repo_name: str,
        description: Optional[str] = None
    ) -> None:
        """Create initial commit with README."""
        try:
            readme_path = repo_path / 'README.md'
            readme_content = f"# {repo_name}\n\n{description or 'Project repository'}\n"
            readme_path.write_text(readme_content)
            
            subprocess.run(
                ['git', 'add', '.'],
                cwd=repo_path,
                capture_output=True,
                check=True
            )
            
            subprocess.run(
                ['git', 'commit', '-m', 'Initial commit'],
                cwd=repo_path,
                capture_output=True,
                check=True
            )
            logger.info("Created initial commit")
        except subprocess.CalledProcessError as e:
            logger.warning(f"Failed to create initial commit: {e.stderr}")
    
    def _setup_branch(self, repo_path: Path, branch: str) -> None:
        """Setup default branch."""
        try:
            subprocess.run(
                ['git', 'branch', '-M', branch],
                cwd=repo_path,
                capture_output=True,
                check=True
            )
            logger.info(f"Created branch: {branch}")
        except subprocess.CalledProcessError as e:
            logger.warning(f"Failed to setup branch: {e.stderr}")
    
    def _track_repository(
        self,
        repo_path: Path,
        repo_name: str,
        account: Account,
        branch: str,
        description: Optional[str] = None
    ) -> Dict:
        """Track repository in database."""
        repo_config = {
            'name': repo_name,
            'path': str(repo_path),
            'account': account.name,
            'account_platform': account.platform.value,
            'account_username': account.username,
            'account_email': account.email,
            'default_branch': branch,
            'description': description,
            'created_at': datetime.now().isoformat(),
            'ssh_key_path': str(account.ssh_key_path) if hasattr(account, 'ssh_key_path') else None,
            'host_alias': account.host if hasattr(account, 'host') else None
        }
        
        return repo_config
