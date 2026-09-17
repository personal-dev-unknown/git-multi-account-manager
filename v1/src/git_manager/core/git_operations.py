# src/git_manager/core/git_operations.py
"""Git Operations - Handles Git repository operations."""

import subprocess
from pathlib import Path
from typing import Optional, Dict, List, Tuple
import re

from ..utils.logger import get_logger
from ..utils import validate_url
from ..utils.constants import GitOperation, RepoStatus, DEFAULT_BRANCH, DEFAULT_REMOTE
from ..models.repository import Repository, RepositoryStatus
from .exceptions import GitError, RepositoryError
from .account_manager import AccountManager


logger = get_logger(__name__)


class GitOperations:
    """Handles Git repository operations."""
    
    def __init__(self, account_manager: Optional[AccountManager] = None):
        """Initialize Git Operations.
        
        Args:
            account_manager: AccountManager instance
        """
        self.account_manager = account_manager or AccountManager()
        logger.info("GitOperations initialized")
    
    def clone(
        self,
        url: str,
        account_name: str,
        destination: Optional[Path] = None,
        branch: Optional[str] = None
    ) -> Repository:
        """Clone a repository.
        
        Args:
            url: Repository URL
            account_name: Account to use for cloning
            destination: Clone destination (optional)
            branch: Specific branch to clone (optional)
            
        Returns:
            Repository object
            
        Raises:
            GitError: If clone operation fails
        """
        account = self.account_manager.get_account(account_name)
        
        # Validate and convert URL to SSH
        if not validate_url(url):
            raise GitError(f"Invalid repository URL: {url}")
        
        ssh_url = self._convert_to_ssh_url(url, account)
        
        # Prepare clone command
        cmd = ['git', 'clone', ssh_url]
        if destination:
            cmd.append(str(destination))
        if branch:
            cmd.extend(['-b', branch])
        
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                check=True
            )
            logger.info(f"Cloned repository: {ssh_url}")
            
            # Determine repository path
            if destination:
                repo_path = destination
            else:
                repo_name = self._extract_repo_name(url)
                repo_path = Path.cwd() / repo_name
            
            return Repository(
                name=self._extract_repo_name(url),
                path=repo_path,
                remote_url=ssh_url,
                account=account,
                branch=branch or DEFAULT_BRANCH
            )
            
        except subprocess.CalledProcessError as e:
            logger.error(f"Clone failed: {e.stderr}")
            raise GitError(f"Clone failed: {e.stderr}")
    
    def check_status(self, repo_path: Optional[Path] = None) -> RepositoryStatus:
        """Check repository status.
        
        Args:
            repo_path: Repository path (defaults to current directory)
            
        Returns:
            RepositoryStatus object
            
        Raises:
            RepositoryError: If not a git repository
        """
        repo_path = repo_path or Path.cwd()
        
        if not self._is_git_repo(repo_path):
            raise RepositoryError(f"Not a git repository: {repo_path}")
        
        status = RepositoryStatus()
        
        # Check for uncommitted changes
        result = self._run_git_command(['diff-index', '--quiet', 'HEAD', '--'], repo_path)
        status.has_uncommitted_changes = result.returncode != 0
        
        # Get branch info
        result = self._run_git_command(['rev-parse', '--abbrev-ref', 'HEAD'], repo_path)
        status.current_branch = result.stdout.strip()
        
        # Check ahead/behind
        result = self._run_git_command(
            ['rev-list', '--left-right', '--count', f'HEAD...{DEFAULT_REMOTE}/{status.current_branch}'],
            repo_path
        )
        if result.returncode == 0 and result.stdout.strip():
            ahead, behind = map(int, result.stdout.split())
            status.commits_ahead = ahead
            status.commits_behind = behind
            
            # Set status based on repository state
            if ahead > 0 and behind > 0:
                status.status = RepoStatus.DIVERGED
            elif ahead > 0:
                status.status = RepoStatus.AHEAD
            elif behind > 0:
                status.status = RepoStatus.BEHIND
            elif status.has_uncommitted_changes:
                status.status = RepoStatus.UNCOMMITTED
            else:
                status.status = RepoStatus.CLEAN
        else:
            # If we can't determine remote status, check if we have a remote
            result = self._run_git_command(['remote', 'show', DEFAULT_REMOTE], repo_path)
            if result.returncode != 0:
                status.status = RepoStatus.NO_UPSTREAM
            elif status.has_uncommitted_changes:
                status.status = RepoStatus.UNCOMMITTED
            else:
                status.status = RepoStatus.CLEAN
        
        # Get uncommitted files
        result = self._run_git_command(['status', '--porcelain'], repo_path)
        status.uncommitted_files = result.stdout.strip().split('\n') if result.stdout else []
        
        logger.debug(f"Repository status: {status}")
        return status
    
    def pull(
        self,
        repo_path: Optional[Path] = None,
        rebase: bool = False
    ) -> Tuple[bool, str]:
        """Pull changes from remote.
        
        Args:
            repo_path: Repository path
            rebase: Use rebase instead of merge
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        cmd = ['pull']
        if rebase:
            cmd.append('--rebase')
        
        result = self._run_git_command(cmd, repo_path)
        
        if result.returncode == 0:
            logger.info(f"Pull successful: {repo_path}")
            return True, result.stdout
        else:
            logger.error(f"Pull failed: {result.stderr}")
            return False, result.stderr
    
    def push(
        self,
        repo_path: Optional[Path] = None,
        set_upstream: bool = False,
        branch: Optional[str] = None
    ) -> Tuple[bool, str]:
        """Push changes to remote.
        
        Args:
            repo_path: Repository path
            set_upstream: Set upstream branch
            branch: Branch to push
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        cmd = ['push']
        if set_upstream:
            if not branch:
                result = self._run_git_command(['rev-parse', '--abbrev-ref', 'HEAD'], repo_path)
                branch = result.stdout.strip()
            cmd.extend(['--set-upstream', DEFAULT_REMOTE, branch])
            cmd.extend(['-u', 'origin', branch])
        
        result = self._run_git_command(cmd, repo_path)
        
        if result.returncode == 0:
            logger.info(f"Push successful: {repo_path}")
            return True, result.stdout
        else:
            logger.error(f"Push failed: {result.stderr}")
            return False, result.stderr
    
    def _convert_to_ssh_url(self, url: str, account) -> str:
        """Convert URL to SSH format with account host.
        
        Args:
            url: Repository URL
            account: Account object
            
        Returns:
            SSH URL
        """
        from ..utils.platform_helpers import is_local_path
        
        # Handle local paths - return as-is
        if is_local_path(url):
            return url
        
        # Extract owner and repo from URL
        patterns = [
            # GitHub, GitLab, Bitbucket, SourceForge HTTPS
            r'https://(?:github\.com|gitlab\.com|bitbucket\.org|git\.code\.sf\.net)/([^/]+)/([^/]+?)(?:\.git)?/?$',
            # Azure DevOps HTTPS
            r'https://dev\.azure\.com/([^/]+)/[^/]+/_git/([^/]+?)/?$',
            # Generic HTTPS
            r'https://[^/]+/([^/]+)/([^/]+?)(?:\.git)?/?$',
            # SSH format
            r'git@[^:]+:([^/]+)/(.+?)\.git$',
            # Short format (owner/repo)
            r'^([^/]+)/([^/]+)$'
        ]
        
        for pattern in patterns:
            match = re.match(pattern, url)
            if match:
                owner, repo = match.groups()
                repo = repo.replace('.git', '')
                return f"git@{account.host}:{owner}/{repo}.git"
        
        raise GitError(f"Could not parse repository URL: {url}")
    
    def _extract_repo_name(self, url: str) -> str:
        """Extract repository name from URL."""
        patterns = [
            r'/([^/]+?)(?:\.git)?/?$',
            r':([^/:]+)/[^/]+\.git$',
            r'/([^/]+)$'
        ]
        
        for pattern in patterns:
            match = re.search(pattern, url)
            if match:
                return match.group(1).replace('.git', '')
        
        return 'repository'
    
    def _is_git_repo(self, path: Path) -> bool:
        """Check if path is a git repository."""
        result = self._run_git_command(['rev-parse', '--git-dir'], path)
        return result.returncode == 0
    
    def _run_git_command(
        self,
        args: List[str],
        cwd: Optional[Path] = None
    ) -> subprocess.CompletedProcess:
        """Run git command.
        
        Args:
            args: Git command arguments
            cwd: Working directory
            
        Returns:
            CompletedProcess object
        """
        cmd = ['git'] + args
        return subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True
        )