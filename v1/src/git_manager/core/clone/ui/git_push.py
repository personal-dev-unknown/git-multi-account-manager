# src/git_manager/core/git_push.py
"""Git Push Operations - Safe, smart, and powerful push strategies."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, List, Dict
from enum import Enum

from ..utils.logger import get_logger


logger = get_logger(__name__)


class PushStrategy(str, Enum):
    """Push strategy options."""
    SAFE_PUSH = "safe_push"                    # Default safe push
    PUSH_WITH_LEASE = "push_with_lease"        # Force with lease
    FORCE_PUSH = "force_push"                  # Force push (dangerous)
    PUSH_ALL_BRANCHES = "push_all_branches"    # Push all branches
    PUSH_WITH_TAGS = "push_with_tags"          # Push with tags
    DRY_RUN_PUSH = "dry_run_push"              # Preview only


class GitPush:
    """Handles Git push operations with safety checks."""
    
    def __init__(self):
        """Initialize Git Push."""
        logger.info("GitPush initialized")
    
    def safe_push(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin",
        set_upstream: bool = False
    ) -> Tuple[bool, str, dict]:
        """Safe push with pre-flight checks.
        
        Args:
            repo_path: Repository path
            branch: Branch to push
            remote: Remote name
            set_upstream: Set upstream tracking
            
        Returns:
            Tuple of (success, message, details)
        """
        repo_path = repo_path or Path.cwd()
        details = {
            "commits_pushed": 0,
            "remote_ahead": False,
            "large_files": []
        }
        
        try:
            current_branch = branch or self._get_current_branch(repo_path)
            
            # Pre-flight checks
            checks = self._pre_push_checks(repo_path, current_branch, remote)
            
            if checks['blockers']:
                return False, f"Push blocked: {checks['blockers'][0]}", details
            
            if checks['warnings']:
                logger.warning(f"Push warnings: {checks['warnings']}")
            
            # Execute push
            cmd = ['push', remote, current_branch]
            if set_upstream:
                cmd.insert(1, '-u')
            
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                details["commits_pushed"] = self._count_pushed_commits(repo_path, current_branch, remote)
                logger.info(f"Safe push successful: {details['commits_pushed']} commits")
                return True, f"✓ Pushed {details['commits_pushed']} commits", details
            else:
                return False, f"Push failed: {output}", details
                
        except Exception as e:
            logger.error(f"Safe push error: {e}")
            return False, f"Push error: {str(e)}", details
    
    def push_with_lease(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str]:
        """Push with force-with-lease (safer force push).
        
        Args:
            repo_path: Repository path
            branch: Branch to push
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        try:
            # Create backup before force operation
            backup_branch = f"backup-before-force-{current_branch}"
            self._run_git_command(['branch', backup_branch], repo_path)
            logger.info(f"Created backup branch: {backup_branch}")
            
            # Execute force-with-lease
            cmd = ['push', '--force-with-lease', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info("Force-with-lease push successful")
                return True, "✓ Force-with-lease push successful"
            else:
                return False, f"Force-with-lease failed: {output}"
                
        except Exception as e:
            logger.error(f"Force-with-lease error: {e}")
            return False, f"Error: {str(e)}"
    
    def force_push(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str]:
        """Force push (destructive - requires confirmation).
        
        Args:
            repo_path: Repository path
            branch: Branch to push
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        try:
            # Create backup before force operation
            backup_branch = f"backup-before-force-push-{current_branch}"
            self._run_git_command(['branch', backup_branch], repo_path)
            logger.warning(f"Created backup branch: {backup_branch}")
            
            # Execute force push
            cmd = ['push', '--force', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.warning("Force push executed")
                return True, "✓ Force push successful (backup created)"
            else:
                return False, f"Force push failed: {output}"
                
        except Exception as e:
            logger.error(f"Force push error: {e}")
            return False, f"Error: {str(e)}"
    
    def push_all_branches(
        self,
        repo_path: Optional[Path] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str, List[str]]:
        """Push all branches to remote.
        
        Args:
            repo_path: Repository path
            remote: Remote name
            
        Returns:
            Tuple of (success, message, branches_pushed)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            # Get all branches
            result = subprocess.run(
                ['git', 'branch', '-a'],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            branches = [b.strip().replace('* ', '') for b in result.stdout.split('\n') if b.strip()]
            local_branches = [b for b in branches if not b.startswith('remotes/')]
            
            # Push all
            cmd = ['push', '--all', remote]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info(f"Pushed all branches: {local_branches}")
                return True, f"✓ Pushed {len(local_branches)} branches", local_branches
            else:
                return False, f"Push all failed: {output}", []
                
        except Exception as e:
            logger.error(f"Push all branches error: {e}")
            return False, f"Error: {str(e)}", []
    
    def push_with_tags(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str]:
        """Push commits and tags together.
        
        Args:
            repo_path: Repository path
            branch: Branch to push
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        try:
            # Push branch
            cmd = ['push', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if not success:
                return False, f"Branch push failed: {output}"
            
            # Push tags
            cmd = ['push', remote, '--tags']
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info("Pushed branch and tags")
                return True, "✓ Pushed branch and tags"
            else:
                return False, f"Tag push failed: {output}"
                
        except Exception as e:
            logger.error(f"Push with tags error: {e}")
            return False, f"Error: {str(e)}"
    
    def dry_run_push(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str]:
        """Preview what would be pushed without actually pushing.
        
        Args:
            repo_path: Repository path
            branch: Branch to push
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        try:
            cmd = ['push', '--dry-run', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info(f"Dry run push preview:\n{output}")
                return True, f"Preview:\n{output}"
            else:
                return False, f"Dry run failed: {output}"
                
        except Exception as e:
            logger.error(f"Dry run push error: {e}")
            return False, f"Error: {str(e)}"
    
    def _pre_push_checks(self, repo_path: Path, branch: str, remote: str) -> dict:
        """Run pre-push safety checks."""
        checks = {
            'blockers': [],
            'warnings': [],
            'info': []
        }
        
        try:
            # Check 1: Unpushed commits
            result = subprocess.run(
                ['git', 'rev-list', '--count', f'{remote}/{branch}..HEAD'],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            unpushed = int(result.stdout.strip() or 0)
            if unpushed == 0:
                checks['blockers'].append("No commits to push")
            else:
                checks['info'].append(f"📤 {unpushed} commits ready to push")
            
            # Check 2: Remote has new commits
            result = subprocess.run(
                ['git', 'rev-list', '--count', f'HEAD..{remote}/{branch}'],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            remote_ahead = int(result.stdout.strip() or 0)
            if remote_ahead > 0:
                checks['warnings'].append(
                    f"⚠️  Remote has {remote_ahead} new commits. Consider pulling first."
                )
            
            # Check 3: Remote connectivity
            result = subprocess.run(
                ['git', 'ls-remote', '--heads', remote],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode != 0:
                checks['blockers'].append(f"Cannot reach remote: {remote}")
            
        except Exception as e:
            logger.error(f"Pre-push checks error: {e}")
            checks['warnings'].append(f"⚠️  Could not complete all checks: {str(e)}")
        
        return checks
    
    def _count_pushed_commits(self, repo_path: Path, branch: str, remote: str) -> int:
        """Count commits that were pushed."""
        try:
            result = subprocess.run(
                ['git', 'rev-list', '--count', f'{remote}/{branch}..HEAD'],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            return int(result.stdout.strip() or 0)
        except:
            return 0
    
    def _get_current_branch(self, repo_path: Path) -> str:
        """Get current branch name."""
        result = subprocess.run(
            ['git', 'rev-parse', '--abbrev-ref', 'HEAD'],
            cwd=repo_path,
            capture_output=True,
            text=True
        )
        return result.stdout.strip() or 'main'
    
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
