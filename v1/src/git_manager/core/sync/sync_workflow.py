"""Sync Workflow - Orchestrate push, pull, and sync operations.

Supports all 8 Git platforms:
- GitHub
- GitLab
- Bitbucket
- Azure DevOps
- Self-Hosted
- Cloud Storage
- Local Path
- SourceForge
"""

import subprocess
from pathlib import Path
from typing import Optional, Dict, Tuple, List

from ...utils.logger import get_logger
from .push_manager import PushManager
from .pull_manager import PullManager
from .branch_manager import BranchManager


logger = get_logger(__name__)


class SyncWorkflow:
    """Orchestrate Git sync operations with branch management.
    
    Supports all 8 Git platforms for push, pull, and sync operations.
    """
    
    def __init__(self):
        """Initialize Sync Workflow."""
        self.push_manager = PushManager()
        self.pull_manager = PullManager()
        self.branch_manager = BranchManager()
        logger.info("SyncWorkflow initialized")
    
    def push_feature(
        self,
        repo_path: Optional[Path] = None,
        branch_option: str = 'default',
        new_branch_name: Optional[str] = None,
        remote: str = 'origin'
    ) -> Dict:
        """Push feature with branch selection.
        
        Args:
            repo_path: Repository path
            branch_option: 'default' or 'new'
            new_branch_name: Name for new branch (if branch_option='new')
            remote: Remote name
            
        Returns:
            Dictionary with result
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            if branch_option == 'default':
                # Push to default branch
                success, message, details = self.push_manager.push_to_default_branch(
                    repo_path,
                    remote,
                    set_upstream=False
                )
            elif branch_option == 'new':
                # Create and push to new branch
                if not new_branch_name:
                    return {
                        'success': False,
                        'error': 'New branch name required'
                    }
                
                success, message, details = self.push_manager.push_to_new_branch(
                    repo_path,
                    new_branch_name,
                    remote,
                    set_upstream=True
                )
            else:
                return {
                    'success': False,
                    'error': f"Unknown branch option: {branch_option}"
                }
            
            return {
                'success': success,
                'message': message,
                'details': details
            }
            
        except Exception as e:
            logger.error(f"Push feature error: {e}")
            return {
                'success': False,
                'error': str(e)
            }
    
    def pull_changes(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        strategy: str = 'safe',
        remote: str = 'origin'
    ) -> Dict:
        """Pull changes with strategy selection.
        
        Args:
            repo_path: Repository path
            branch: Branch to pull
            strategy: 'safe', 'smart', 'rebase', 'ff-only', or 'fetch'
            remote: Remote name
            
        Returns:
            Dictionary with result
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            if strategy == 'safe':
                success, message, details = self.pull_manager.safe_pull(repo_path, branch, remote)
                return {
                    'success': success,
                    'message': message,
                    'details': details
                }
            
            elif strategy == 'smart':
                success, message = self.pull_manager.smart_pull(repo_path, branch, remote)
                return {
                    'success': success,
                    'message': message
                }
            
            elif strategy == 'rebase':
                success, message = self.pull_manager.pull_rebase(repo_path, branch, remote)
                return {
                    'success': success,
                    'message': message
                }
            
            elif strategy == 'ff-only':
                success, message = self.pull_manager.pull_ff_only(repo_path, branch, remote)
                return {
                    'success': success,
                    'message': message
                }
            
            elif strategy == 'fetch':
                success, message = self.pull_manager.fetch_only(repo_path, remote)
                return {
                    'success': success,
                    'message': message
                }
            
            else:
                return {
                    'success': False,
                    'error': f"Unknown pull strategy: {strategy}"
                }
            
        except Exception as e:
            logger.error(f"Pull changes error: {e}")
            return {
                'success': False,
                'error': str(e)
            }
    
    def sync_repository(
        self,
        repo_path: Optional[Path] = None,
        remote: str = 'origin'
    ) -> Dict:
        """Sync repository (pull then push).
        
        Args:
            repo_path: Repository path
            remote: Remote name
            
        Returns:
            Dictionary with result
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            current_branch = self.branch_manager.get_current_branch(repo_path)
            
            # Pull first
            pull_result = self.pull_changes(repo_path, current_branch, 'safe', remote)
            
            if not pull_result['success']:
                return {
                    'success': False,
                    'error': f"Pull failed: {pull_result.get('message', 'Unknown error')}",
                    'pull': pull_result
                }
            
            # Then push
            push_result = self.push_feature(repo_path, 'default', None, remote)
            
            return {
                'success': push_result['success'],
                'message': f"Sync complete: {push_result.get('message', 'Done')}",
                'pull': pull_result,
                'push': push_result
            }
            
        except Exception as e:
            logger.error(f"Sync repository error: {e}")
            return {
                'success': False,
                'error': str(e)
            }
    
    def get_branch_options(self, repo_path: Optional[Path] = None) -> Dict:
        """Get available branch options for push.
        
        Args:
            repo_path: Repository path
            
        Returns:
            Dictionary with branch options
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            current_branch = self.branch_manager.get_current_branch(repo_path)
            default_branch = self.branch_manager.get_default_branch(repo_path)
            local_branches = self.branch_manager.list_local_branches(repo_path)
            
            return {
                'current_branch': current_branch,
                'default_branch': default_branch,
                'local_branches': [b.name for b in local_branches],
                'can_push_to_default': current_branch != default_branch,
                'can_create_new': True
            }
            
        except Exception as e:
            logger.error(f"Error getting branch options: {e}")
            return {
                'error': str(e)
            }
    
    def get_sync_status(self, repo_path: Optional[Path] = None, remote: str = 'origin') -> Dict:
        """Get current sync status.
        
        Args:
            repo_path: Repository path
            remote: Remote name
            
        Returns:
            Dictionary with sync status
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            current_branch = self.branch_manager.get_current_branch(repo_path)
            branch_info = self.branch_manager.get_branch_info(repo_path, current_branch)
            
            return {
                'current_branch': current_branch,
                'branch_info': branch_info,
                'is_synced': branch_info.get('commits_ahead', 0) == 0 and branch_info.get('commits_behind', 0) == 0
            }
            
        except Exception as e:
            logger.error(f"Error getting sync status: {e}")
            return {
                'error': str(e)
            }
    
    def safe_sync(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> Tuple[bool, str, Dict]:
        """Safe sync: Stash → Pull → Pop (recommended for most users).
        
        Args:
            repo_path: Repository path
            branch: Branch to sync
            remote: Remote name
            
        Returns:
            Tuple of (success, message, details)
        """
        repo_path = repo_path or Path.cwd()
        details = {
            'stashed': False,
            'pulled': False,
            'conflicts': False,
            'changes_count': 0
        }
        
        try:
            # Step 1: Check for uncommitted changes
            uncommitted = self._get_uncommitted_files(repo_path)
            details['changes_count'] = len(uncommitted)
            
            if uncommitted:
                # Stash changes
                logger.info(f"Stashing {len(uncommitted)} uncommitted changes")
                success, msg = self._run_git_command(['stash', 'push', '-m', 'Safe sync - work in progress'], repo_path)
                if not success:
                    return False, f"Failed to stash changes: {msg}", details
                details['stashed'] = True
            
            # Step 2: Fetch to see what's different
            success, msg = self._run_git_command(['fetch', remote], repo_path)
            if not success:
                return False, f"Failed to fetch: {msg}", details
            
            # Step 3: Check if branches diverged
            current_branch = branch or self.branch_manager.get_current_branch(repo_path)
            diverged, local_commits, remote_commits = self._check_divergence(repo_path, current_branch, remote)
            
            if diverged:
                # Branches diverged - use rebase (cleaner history)
                logger.info(f"Branches diverged. Using rebase strategy.")
                success, msg = self._run_git_command(['pull', '--rebase', remote, current_branch], repo_path)
            else:
                # Simple fast-forward or no changes
                success, msg = self._run_git_command(['pull', remote, current_branch], repo_path)
            
            if not success:
                # Pull failed - try to restore stashed changes
                if details['stashed']:
                    self._run_git_command(['stash', 'pop'], repo_path)
                return False, f"Pull failed: {msg}", details
            
            details['pulled'] = True
            
            # Step 4: Reapply stashed changes if any
            if details['stashed']:
                success, msg = self._run_git_command(['stash', 'pop'], repo_path)
                if not success:
                    details['conflicts'] = True
                    return False, f"Conflicts when reapplying changes: {msg}", details
            
            return True, "Safe sync completed successfully", details
            
        except Exception as e:
            logger.error(f"Safe sync failed: {e}")
            return False, f"Safe sync error: {str(e)}", details
    
    def quick_pull(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> Tuple[bool, str]:
        """Quick pull: Pull remote changes only.
        
        Args:
            repo_path: Repository path
            branch: Branch to pull
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        success, msg = self._run_git_command(['pull', remote, current_branch], repo_path)
        
        if success:
            logger.info(f"Quick pull successful from {current_branch}")
        else:
            logger.error(f"Quick pull failed: {msg}")
        
        return success, msg
    
    def quick_push(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin',
        set_upstream: bool = False
    ) -> Tuple[bool, str]:
        """Quick push: Push local commits only.
        
        Args:
            repo_path: Repository path
            branch: Branch to push
            remote: Remote name
            set_upstream: Set upstream tracking
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        cmd = ['push', remote, current_branch]
        if set_upstream:
            cmd.insert(1, '-u')
        
        success, msg = self._run_git_command(cmd, repo_path)
        
        if success:
            logger.info(f"Quick push successful to {current_branch}")
        else:
            logger.error(f"Quick push failed: {msg}")
        
        return success, msg
    
    def pull_with_rebase(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> Tuple[bool, str]:
        """Pull with rebase: For clean linear history.
        
        Args:
            repo_path: Repository path
            branch: Branch to pull
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        success, msg = self._run_git_command(['pull', '--rebase', remote, current_branch], repo_path)
        
        if success:
            logger.info(f"Pull with rebase successful from {current_branch}")
        else:
            logger.error(f"Pull with rebase failed: {msg}")
        
        return success, msg
    
    def reset_to_remote(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> Tuple[bool, str]:
        """Reset to remote: Discard ALL local changes (DESTRUCTIVE).
        
        Args:
            repo_path: Repository path
            branch: Branch to reset to
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Fetch latest
            success, msg = self._run_git_command(['fetch', remote], repo_path)
            if not success:
                return False, f"Fetch failed: {msg}"
            
            # Reset hard to remote
            success, msg = self._run_git_command(['reset', '--hard', f'{remote}/{current_branch}'], repo_path)
            if not success:
                return False, f"Reset failed: {msg}"
            
            # Clean untracked files
            success, msg = self._run_git_command(['clean', '-fd'], repo_path)
            if not success:
                return False, f"Clean failed: {msg}"
            
            logger.warning(f"Repository reset to {remote}/{current_branch}")
            return True, f"Successfully reset to {remote}/{current_branch}"
            
        except Exception as e:
            logger.error(f"Reset to remote failed: {e}")
            return False, f"Reset error: {str(e)}"
    
    def get_pre_flight_checks(self, repo_path: Optional[Path] = None, remote: str = 'origin') -> Dict:
        """Run pre-flight checks before sync operations.
        
        Args:
            repo_path: Repository path
            remote: Remote name
            
        Returns:
            Dictionary with check results
        """
        repo_path = repo_path or Path.cwd()
        checks = {
            'uncommitted_changes': False,
            'uncommitted_count': 0,
            'unpushed_commits': False,
            'unpushed_count': 0,
            'new_remote_commits': False,
            'new_remote_count': 0,
            'branches_diverged': False,
            'can_reach_remote': True,
            'warnings': []
        }
        
        try:
            # Check for uncommitted changes
            uncommitted = self._get_uncommitted_files(repo_path)
            if uncommitted:
                checks['uncommitted_changes'] = True
                checks['uncommitted_count'] = len(uncommitted)
                checks['warnings'].append(f"⚠️  {len(uncommitted)} uncommitted changes detected")
            
            # Fetch to get remote info
            self._run_git_command(['fetch', remote], repo_path)
            
            # Check for unpushed commits
            current_branch = self.branch_manager.get_current_branch(repo_path)
            result = subprocess.run(
                ['git', 'rev-list', '--count', f'HEAD..{remote}/{current_branch}'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            if result.returncode == 0:
                unpushed = int(result.stdout.strip() or 0)
                if unpushed > 0:
                    checks['unpushed_commits'] = True
                    checks['unpushed_count'] = unpushed
                    checks['warnings'].append(f"ℹ️  {unpushed} local commits not on remote")
            
            # Check for new remote commits
            result = subprocess.run(
                ['git', 'rev-list', '--count', f'{remote}/{current_branch}..HEAD'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            if result.returncode == 0:
                new_remote = int(result.stdout.strip() or 0)
                if new_remote > 0:
                    checks['new_remote_commits'] = True
                    checks['new_remote_count'] = new_remote
                    checks['warnings'].append(f"ℹ️  {new_remote} new commits on remote")
            
            # Check if branches diverged
            diverged, local, remote_count = self._check_divergence(repo_path, current_branch, remote)
            if diverged:
                checks['branches_diverged'] = True
                checks['warnings'].append(f"⚠️  Local and remote have diverged! ({local} local, {remote_count} remote)")
            
        except Exception as e:
            logger.error(f"Pre-flight checks failed: {e}")
            checks['warnings'].append(f"❌ Error during checks: {str(e)}")
        
        return checks
    
    def _get_uncommitted_files(self, repo_path: Path) -> List[str]:
        """Get list of uncommitted files."""
        result = subprocess.run(
            ['git', 'status', '--porcelain'],
            cwd=repo_path,
            capture_output=True,
            text=True,
            timeout=10
        )
        
        if result.returncode == 0 and result.stdout.strip():
            return result.stdout.strip().split('\n')
        return []
    
    def _check_divergence(self, repo_path: Path, branch: str, remote: str = 'origin') -> Tuple[bool, int, int]:
        """Check if local and remote branches have diverged.
        
        Returns:
            Tuple of (diverged, local_commits, remote_commits)
        """
        result = subprocess.run(
            ['git', 'rev-list', '--left-right', '--count', f'HEAD...{remote}/{branch}'],
            cwd=repo_path,
            capture_output=True,
            text=True,
            timeout=10
        )
        
        if result.returncode == 0 and result.stdout.strip():
            local, remote_count = map(int, result.stdout.split())
            diverged = local > 0 and remote_count > 0
            return diverged, local, remote_count
        
        return False, 0, 0
    
    def _run_git_command(self, args: List[str], cwd: Path) -> Tuple[bool, str]:
        """Run git command and return success status and output."""
        try:
            result = subprocess.run(
                ['git'] + args,
                cwd=cwd,
                capture_output=True,
                text=True,
                timeout=30
            )
            
            output = result.stdout or result.stderr
            success = result.returncode == 0
            
            return success, output.strip()
            
        except subprocess.TimeoutExpired:
            return False, "Command timed out"
        except Exception as e:
            return False, str(e)
