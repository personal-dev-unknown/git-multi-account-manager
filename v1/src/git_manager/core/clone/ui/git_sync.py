# src/git_manager/core/git_sync.py
"""Git Sync Operations - Comprehensive sync, push, and pull strategies."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, List
from enum import Enum

from ..utils.logger import get_logger


logger = get_logger(__name__)


class SyncStrategy(str, Enum):
    """Sync strategy options."""
    SAFE_SYNC = "safe_sync"           # Stash → Pull → Pop (recommended)
    QUICK_PULL = "quick_pull"         # Pull only
    QUICK_PUSH = "quick_push"         # Push only
    PULL_REBASE = "pull_rebase"       # Pull with rebase (clean history)
    RESET_TO_REMOTE = "reset_to_remote"  # Force reset (destructive)


class GitSync:
    """Handles Git sync, push, and pull operations with safety checks."""
    
    def __init__(self):
        """Initialize Git Sync."""
        logger.info("GitSync initialized")
    
    def safe_sync(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None
    ) -> Tuple[bool, str, dict]:
        """Safe sync: Stash → Pull → Pop (recommended for most users).
        
        Args:
            repo_path: Repository path (defaults to current directory)
            branch: Branch to sync (defaults to current branch)
            
        Returns:
            Tuple of (success, message, details)
        """
        repo_path = repo_path or Path.cwd()
        details = {
            "stashed": False,
            "pulled": False,
            "conflicts": False,
            "changes_count": 0
        }
        
        try:
            # Step 1: Check for uncommitted changes
            uncommitted = self._get_uncommitted_files(repo_path)
            details["changes_count"] = len(uncommitted)
            
            if uncommitted:
                # Stash changes
                logger.info(f"Stashing {len(uncommitted)} uncommitted changes")
                success, msg = self._run_git_command(
                    ['stash', 'push', '-m', 'Safe sync - work in progress'],
                    repo_path
                )
                if not success:
                    return False, f"Failed to stash changes: {msg}", details
                details["stashed"] = True
            
            # Step 2: Fetch to see what's different
            success, msg = self._run_git_command(['fetch', 'origin'], repo_path)
            if not success:
                return False, f"Failed to fetch: {msg}", details
            
            # Step 3: Check if branches diverged
            current_branch = branch or self._get_current_branch(repo_path)
            diverged, local_commits, remote_commits = self._check_divergence(
                repo_path, current_branch
            )
            
            if diverged:
                # Branches diverged - use rebase (cleaner history)
                logger.info(f"Branches diverged. Using rebase strategy.")
                success, msg = self._run_git_command(
                    ['pull', '--rebase', 'origin', current_branch],
                    repo_path
                )
            else:
                # Simple fast-forward or no changes
                success, msg = self._run_git_command(
                    ['pull', 'origin', current_branch],
                    repo_path
                )
            
            if not success:
                # Pull failed - try to restore stashed changes
                if details["stashed"]:
                    self._run_git_command(['stash', 'pop'], repo_path)
                return False, f"Pull failed: {msg}", details
            
            details["pulled"] = True
            
            # Step 4: Reapply stashed changes if any
            if details["stashed"]:
                success, msg = self._run_git_command(['stash', 'pop'], repo_path)
                if not success:
                    details["conflicts"] = True
                    return False, f"Conflicts when reapplying changes: {msg}", details
            
            return True, "Safe sync completed successfully", details
            
        except Exception as e:
            logger.error(f"Safe sync failed: {e}")
            return False, f"Safe sync error: {str(e)}", details
    
    def quick_pull(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None
    ) -> Tuple[bool, str]:
        """Quick pull: Pull remote changes only.
        
        Args:
            repo_path: Repository path
            branch: Branch to pull
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        success, msg = self._run_git_command(
            ['pull', 'origin', current_branch],
            repo_path
        )
        
        if success:
            logger.info(f"Quick pull successful from {current_branch}")
        else:
            logger.error(f"Quick pull failed: {msg}")
        
        return success, msg
    
    def quick_push(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        set_upstream: bool = False
    ) -> Tuple[bool, str]:
        """Quick push: Push local commits only.
        
        Args:
            repo_path: Repository path
            branch: Branch to push
            set_upstream: Set upstream tracking
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        cmd = ['push', 'origin', current_branch]
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
        branch: Optional[str] = None
    ) -> Tuple[bool, str]:
        """Pull with rebase: For clean linear history.
        
        Args:
            repo_path: Repository path
            branch: Branch to pull
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        success, msg = self._run_git_command(
            ['pull', '--rebase', 'origin', current_branch],
            repo_path
        )
        
        if success:
            logger.info(f"Pull with rebase successful from {current_branch}")
        else:
            logger.error(f"Pull with rebase failed: {msg}")
        
        return success, msg
    
    def reset_to_remote(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None
    ) -> Tuple[bool, str]:
        """Reset to remote: Discard ALL local changes (DESTRUCTIVE).
        
        Args:
            repo_path: Repository path
            branch: Branch to reset to
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        try:
            # Fetch latest
            success, msg = self._run_git_command(['fetch', 'origin'], repo_path)
            if not success:
                return False, f"Fetch failed: {msg}"
            
            # Reset hard to remote
            success, msg = self._run_git_command(
                ['reset', '--hard', f'origin/{current_branch}'],
                repo_path
            )
            if not success:
                return False, f"Reset failed: {msg}"
            
            # Clean untracked files
            success, msg = self._run_git_command(['clean', '-fd'], repo_path)
            if not success:
                return False, f"Clean failed: {msg}"
            
            logger.warning(f"Repository reset to remote/{current_branch}")
            return True, f"Successfully reset to origin/{current_branch}"
            
        except Exception as e:
            logger.error(f"Reset to remote failed: {e}")
            return False, f"Reset error: {str(e)}"
    
    def get_pre_flight_checks(
        self,
        repo_path: Optional[Path] = None
    ) -> dict:
        """Run pre-flight checks before sync operations.
        
        Args:
            repo_path: Repository path
            
        Returns:
            Dictionary with check results
        """
        repo_path = repo_path or Path.cwd()
        checks = {
            "uncommitted_changes": False,
            "uncommitted_count": 0,
            "unpushed_commits": False,
            "unpushed_count": 0,
            "new_remote_commits": False,
            "new_remote_count": 0,
            "branches_diverged": False,
            "can_reach_remote": True,
            "warnings": []
        }
        
        try:
            # Check for uncommitted changes
            uncommitted = self._get_uncommitted_files(repo_path)
            if uncommitted:
                checks["uncommitted_changes"] = True
                checks["uncommitted_count"] = len(uncommitted)
                checks["warnings"].append(
                    f"⚠️  {len(uncommitted)} uncommitted changes detected"
                )
            
            # Fetch to get remote info
            self._run_git_command(['fetch', 'origin'], repo_path)
            
            # Check for unpushed commits
            current_branch = self._get_current_branch(repo_path)
            result = subprocess.run(
                ['git', 'rev-list', '--count', f'HEAD..origin/{current_branch}'],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                unpushed = int(result.stdout.strip() or 0)
                if unpushed > 0:
                    checks["unpushed_commits"] = True
                    checks["unpushed_count"] = unpushed
                    checks["warnings"].append(
                        f"ℹ️  {unpushed} local commits not on remote"
                    )
            
            # Check for new remote commits
            result = subprocess.run(
                ['git', 'rev-list', '--count', f'origin/{current_branch}..HEAD'],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                new_remote = int(result.stdout.strip() or 0)
                if new_remote > 0:
                    checks["new_remote_commits"] = True
                    checks["new_remote_count"] = new_remote
                    checks["warnings"].append(
                        f"ℹ️  {new_remote} new commits on remote"
                    )
            
            # Check if branches diverged
            diverged, local, remote = self._check_divergence(repo_path, current_branch)
            if diverged:
                checks["branches_diverged"] = True
                checks["warnings"].append(
                    f"⚠️  Local and remote have diverged! "
                    f"({local} local, {remote} remote)"
                )
            
        except Exception as e:
            logger.error(f"Pre-flight checks failed: {e}")
            checks["warnings"].append(f"❌ Error during checks: {str(e)}")
        
        return checks
    
    def _get_uncommitted_files(self, repo_path: Path) -> List[str]:
        """Get list of uncommitted files."""
        result = subprocess.run(
            ['git', 'status', '--porcelain'],
            cwd=repo_path,
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0 and result.stdout.strip():
            return result.stdout.strip().split('\n')
        return []
    
    def _get_current_branch(self, repo_path: Path) -> str:
        """Get current branch name."""
        result = subprocess.run(
            ['git', 'rev-parse', '--abbrev-ref', 'HEAD'],
            cwd=repo_path,
            capture_output=True,
            text=True
        )
        return result.stdout.strip() or 'main'
    
    def _check_divergence(
        self,
        repo_path: Path,
        branch: str
    ) -> Tuple[bool, int, int]:
        """Check if local and remote branches have diverged.
        
        Returns:
            Tuple of (diverged, local_commits, remote_commits)
        """
        result = subprocess.run(
            ['git', 'rev-list', '--left-right', '--count',
             f'HEAD...origin/{branch}'],
            cwd=repo_path,
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0 and result.stdout.strip():
            local, remote = map(int, result.stdout.split())
            diverged = local > 0 and remote > 0
            return diverged, local, remote
        
        return False, 0, 0
    
    def _run_git_command(
        self,
        args: List[str],
        cwd: Path
    ) -> Tuple[bool, str]:
        """Run git command and return success status and output.
        
        Returns:
            Tuple of (success, output)
        """
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
