"""Push Manager - Handle push operations with branch selection."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, List, Dict

from ...utils.logger import get_logger
from .branch_manager import BranchManager


logger = get_logger(__name__)


class PushManager:
    """Manage Git push operations with intelligent branch selection."""
    
    def __init__(self):
        """Initialize Push Manager."""
        self.branch_manager = BranchManager()
        logger.info("PushManager initialized")
    
    def push_to_default_branch(
        self,
        repo_path: Optional[Path] = None,
        remote: str = 'origin',
        set_upstream: bool = False
    ) -> Tuple[bool, str, Dict]:
        """Push current branch to default remote branch.
        
        Args:
            repo_path: Repository path
            remote: Remote name
            set_upstream: Set upstream tracking
            
        Returns:
            Tuple of (success, message, details)
        """
        repo_path = repo_path or Path.cwd()
        details = {'branch': None, 'commits_pushed': 0}
        
        try:
            current_branch = self.branch_manager.get_current_branch(repo_path)
            default_branch = self.branch_manager.get_default_branch(repo_path)
            
            details['branch'] = current_branch
            
            # Pre-flight checks
            checks = self._pre_push_checks(repo_path, current_branch, remote)
            
            if checks['blockers']:
                return False, f"Push blocked: {checks['blockers'][0]}", details
            
            if checks['warnings']:
                logger.warning(f"Push warnings: {checks['warnings']}")
            
            # Execute push
            cmd = ['git', 'push', remote, current_branch]
            if set_upstream:
                cmd.insert(2, '-u')
            
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                details['commits_pushed'] = self._count_pushed_commits(repo_path, current_branch, remote)
                msg = f"✓ Pushed '{current_branch}' to '{remote}' ({details['commits_pushed']} commits)"
                logger.info(msg)
                return True, msg, details
            else:
                return False, f"Push failed: {output}", details
                
        except Exception as e:
            logger.error(f"Push to default branch error: {e}")
            return False, f"Error: {str(e)}", details
    
    def push_to_new_branch(
        self,
        repo_path: Optional[Path] = None,
        new_branch_name: str = None,
        remote: str = 'origin',
        set_upstream: bool = True
    ) -> Tuple[bool, str, Dict]:
        """Create and push to a new branch.
        
        Args:
            repo_path: Repository path
            new_branch_name: Name for new branch
            remote: Remote name
            set_upstream: Set upstream tracking
            
        Returns:
            Tuple of (success, message, details)
        """
        repo_path = repo_path or Path.cwd()
        details = {'branch': new_branch_name, 'commits_pushed': 0}
        
        if not new_branch_name:
            return False, "New branch name required", details
        
        try:
            current_branch = self.branch_manager.get_current_branch(repo_path)
            
            # Create new branch
            success, msg = self.branch_manager.create_branch(
                repo_path,
                new_branch_name,
                current_branch
            )
            
            if not success:
                return False, f"Failed to create branch: {msg}", details
            
            # Pre-flight checks
            checks = self._pre_push_checks(repo_path, new_branch_name, remote)
            
            if checks['blockers']:
                # Switch back to original branch
                self.branch_manager.switch_branch(repo_path, current_branch)
                return False, f"Push blocked: {checks['blockers'][0]}", details
            
            # Execute push
            cmd = ['git', 'push', remote, new_branch_name]
            if set_upstream:
                cmd.insert(2, '-u')
            
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                details['commits_pushed'] = self._count_pushed_commits(repo_path, new_branch_name, remote)
                msg = f"✓ Created and pushed '{new_branch_name}' to '{remote}' ({details['commits_pushed']} commits)"
                logger.info(msg)
                return True, msg, details
            else:
                # Switch back to original branch
                self.branch_manager.switch_branch(repo_path, current_branch)
                return False, f"Push failed: {output}", details
                
        except Exception as e:
            logger.error(f"Push to new branch error: {e}")
            return False, f"Error: {str(e)}", details
    
    def safe_push(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin',
        set_upstream: bool = False
    ) -> Tuple[bool, str, Dict]:
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
        details = {'commits_pushed': 0, 'remote_ahead': False}
        
        try:
            current_branch = branch or self.branch_manager.get_current_branch(repo_path)
            
            # Pre-flight checks
            checks = self._pre_push_checks(repo_path, current_branch, remote)
            
            if checks['blockers']:
                return False, f"Push blocked: {checks['blockers'][0]}", details
            
            if checks['warnings']:
                logger.warning(f"Push warnings: {checks['warnings']}")
            
            # Execute push
            cmd = ['git', 'push', remote, current_branch]
            if set_upstream:
                cmd.insert(2, '-u')
            
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                details['commits_pushed'] = self._count_pushed_commits(repo_path, current_branch, remote)
                logger.info(f"Safe push successful: {details['commits_pushed']} commits")
                return True, f"✓ Pushed {details['commits_pushed']} commits", details
            else:
                return False, f"Push failed: {output}", details
                
        except Exception as e:
            logger.error(f"Safe push error: {e}")
            return False, f"Error: {str(e)}", details
    
    def push_with_lease(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
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
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Create backup before force operation
            backup_branch = f"backup-before-force-{current_branch}"
            self._run_git_command(['git', 'branch', backup_branch], repo_path)
            logger.info(f"Created backup branch: {backup_branch}")
            
            # Execute force-with-lease
            cmd = ['git', 'push', '--force-with-lease', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info("Force-with-lease push successful")
                return True, "✓ Force-with-lease push successful"
            else:
                return False, f"Force-with-lease failed: {output}"
                
        except Exception as e:
            logger.error(f"Force-with-lease error: {e}")
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
                text=True,
                timeout=10
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
                text=True,
                timeout=10
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
                text=True,
                timeout=10
            )
            return int(result.stdout.strip() or 0)
        except:
            return 0
    
    def _run_git_command(self, args: List[str], cwd: Path) -> Tuple[bool, str]:
        """Run git command and return success status and output."""
        try:
            result = subprocess.run(
                args,
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
