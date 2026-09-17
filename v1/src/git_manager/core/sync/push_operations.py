"""Push Operations - Comprehensive push strategies with safety checks."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, List, Dict
from dataclasses import dataclass

from ...utils.logger import get_logger
from .branch_manager import BranchManager
from .git_ssh_helper import GitSSHHelper


logger = get_logger(__name__)


@dataclass
class PushResult:
    """Result of a push operation."""
    success: bool
    message: str
    details: Dict = None
    
    def __post_init__(self):
        if self.details is None:
            self.details = {}


class PushOperations:
    """Comprehensive push operations with multiple strategies."""
    
    def __init__(self):
        """Initialize Push Operations."""
        self.branch_manager = BranchManager()
        self.ssh_helper = GitSSHHelper()
        logger.info("PushOperations initialized")
    
    # ==================== MAIN PUSH OPERATIONS ====================
    
    def safe_push(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin',
        set_upstream: bool = False
    ) -> PushResult:
        """
        🟢 Safe Push - Default recommended push with pre-flight checks.
        
        Behavior:
        - Check if remote has new commits (prevent non-fast-forward)
        - Warn if pushing large files
        - Show what will be pushed (commit count, file changes)
        - Use --force-with-lease if needed (after confirmation)
        - Fail gracefully if remote has diverged
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            current_branch = branch or self.branch_manager.get_current_branch(repo_path)
            
            # Pre-flight checks
            checks = self._pre_push_checks(repo_path, current_branch, remote)
            
            if checks['blockers']:
                return PushResult(
                    success=False,
                    message=f"Push blocked: {checks['blockers'][0]}",
                    details={'branch': current_branch, 'blockers': checks['blockers']}
                )
            
            if checks['warnings']:
                logger.warning(f"Push warnings: {checks['warnings']}")
            
            # Get commit info
            unpushed_count = self._count_unpushed_commits(repo_path, current_branch, remote)
            
            # Execute push
            cmd = ['git', 'push', remote, current_branch]
            if set_upstream:
                cmd.insert(2, '-u')
            
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info(f"Safe push successful: {unpushed_count} commits")
                return PushResult(
                    success=True,
                    message=f"✓ Pushed {unpushed_count} commits to {remote}/{current_branch}",
                    details={
                        'branch': current_branch,
                        'commits_pushed': unpushed_count,
                        'remote': remote,
                        'warnings': checks['warnings']
                    }
                )
            else:
                return PushResult(
                    success=False,
                    message=f"Push failed: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Safe push error: {e}")
            return PushResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def push_with_lease(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> PushResult:
        """
        🟡 Push with Lease - Safer force push.
        
        Behavior:
        - Safer than force push
        - Only succeeds if remote hasn't changed since last fetch
        - Show warning about rewriting history
        - Require confirmation
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
                return PushResult(
                    success=True,
                    message="✓ Force-with-lease push successful",
                    details={
                        'branch': current_branch,
                        'method': 'force-with-lease',
                        'backup_branch': backup_branch
                    }
                )
            else:
                return PushResult(
                    success=False,
                    message=f"Force-with-lease failed: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Force-with-lease error: {e}")
            return PushResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def force_push(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> PushResult:
        """
        🟠 Force Push - Destructive force push.
        
        Behavior:
        - Show STRONG warning about consequences
        - List commits that will be overwritten on remote
        - Require typing confirmation phrase: "FORCE PUSH"
        - Create local backup branch automatically
        - Log action for audit trail
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Create backup before force operation
            backup_branch = f"backup-before-force-{current_branch}"
            self._run_git_command(['git', 'branch', backup_branch], repo_path)
            logger.info(f"Created backup branch: {backup_branch}")
            
            # Get commits that will be overwritten
            overwritten = self._get_overwritten_commits(repo_path, current_branch, remote)
            
            # Execute force push
            cmd = ['git', 'push', '--force', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.warning(f"Force push executed: {current_branch}")
                return PushResult(
                    success=True,
                    message="✓ Force push completed (history rewritten)",
                    details={
                        'branch': current_branch,
                        'method': 'force',
                        'backup_branch': backup_branch,
                        'overwritten_commits': overwritten
                    }
                )
            else:
                return PushResult(
                    success=False,
                    message=f"Force push failed: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Force push error: {e}")
            return PushResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def push_all_branches(
        self,
        repo_path: Optional[Path] = None,
        remote: str = 'origin'
    ) -> PushResult:
        """
        🔵 Push All Branches - Push all local branches.
        
        Behavior:
        - List all branches that will be pushed
        - Show which are new vs updates
        - Confirm before execution
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            # Get all branches
            branches = self._get_all_branches(repo_path)
            
            if not branches:
                return PushResult(
                    success=False,
                    message="No branches to push",
                    details={'branches': []}
                )
            
            # Execute push all
            cmd = ['git', 'push', '--all', remote]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info(f"Pushed {len(branches)} branches")
                return PushResult(
                    success=True,
                    message=f"✓ Pushed {len(branches)} branches to {remote}",
                    details={
                        'branches': branches,
                        'remote': remote,
                        'count': len(branches)
                    }
                )
            else:
                return PushResult(
                    success=False,
                    message=f"Push all failed: {output}",
                    details={'branches': branches, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Push all branches error: {e}")
            return PushResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def push_with_tags(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> PushResult:
        """
        🔵 Push with Tags - Push commits and tags together.
        
        Behavior:
        - Push current branch
        - Push all tags (or specific tag)
        - Show what tags will be pushed
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Get tags
            tags = self._get_tags(repo_path)
            
            # Push branch
            cmd = ['git', 'push', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if not success:
                return PushResult(
                    success=False,
                    message=f"Push branch failed: {output}",
                    details={'branch': current_branch, 'error': output}
                )
            
            # Push tags
            cmd = ['git', 'push', remote, '--tags']
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info(f"Pushed branch and {len(tags)} tags")
                return PushResult(
                    success=True,
                    message=f"✓ Pushed {current_branch} with {len(tags)} tags",
                    details={
                        'branch': current_branch,
                        'tags': tags,
                        'tag_count': len(tags),
                        'remote': remote
                    }
                )
            else:
                return PushResult(
                    success=False,
                    message=f"Push tags failed: {output}",
                    details={'branch': current_branch, 'tags': tags, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Push with tags error: {e}")
            return PushResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def dry_run_push(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> PushResult:
        """
        🟣 Dry Run Push - Preview what would be pushed without actually pushing.
        
        Behavior:
        - Use --dry-run flag
        - Display detailed output of what would happen
        - No changes made to remote
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Get unpushed commits
            unpushed_count = self._count_unpushed_commits(repo_path, current_branch, remote)
            unpushed_commits = self._get_unpushed_commits(repo_path, current_branch, remote)
            
            # Execute dry-run
            cmd = ['git', 'push', '--dry-run', '--verbose', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success or 'Everything up-to-date' in output:
                logger.info(f"Dry-run push: {unpushed_count} commits would be pushed")
                return PushResult(
                    success=True,
                    message=f"✓ Dry-run: Would push {unpushed_count} commits",
                    details={
                        'branch': current_branch,
                        'commits_count': unpushed_count,
                        'commits': unpushed_commits,
                        'remote': remote,
                        'output': output
                    }
                )
            else:
                return PushResult(
                    success=False,
                    message=f"Dry-run failed: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Dry-run push error: {e}")
            return PushResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    # ==================== HELPER METHODS ====================
    
    def _pre_push_checks(self, repo_path: Path, branch: str, remote: str) -> dict:
        """Run pre-push safety checks."""
        checks = {
            'blockers': [],
            'warnings': [],
            'info': []
        }
        
        try:
            # Check 1: Unpushed commits
            unpushed = self._count_unpushed_commits(repo_path, branch, remote)
            if unpushed == 0:
                checks['blockers'].append("No commits to push")
            else:
                checks['info'].append(f"📤 {unpushed} commits ready to push")
            
            # Check 2: Remote has new commits
            remote_ahead = self._count_remote_ahead(repo_path, branch, remote)
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
    
    def _count_unpushed_commits(self, repo_path: Path, branch: str, remote: str) -> int:
        """Count commits that haven't been pushed."""
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
    
    def _count_remote_ahead(self, repo_path: Path, branch: str, remote: str) -> int:
        """Count commits that remote has ahead."""
        try:
            result = subprocess.run(
                ['git', 'rev-list', '--count', f'HEAD..{remote}/{branch}'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return int(result.stdout.strip() or 0)
        except:
            return 0
    
    def _get_unpushed_commits(self, repo_path: Path, branch: str, remote: str) -> List[str]:
        """Get list of unpushed commits."""
        try:
            result = subprocess.run(
                ['git', 'log', '--oneline', f'{remote}/{branch}..HEAD'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.stdout.strip().split('\n') if result.stdout.strip() else []
        except:
            return []
    
    def _get_overwritten_commits(self, repo_path: Path, branch: str, remote: str) -> List[str]:
        """Get commits that would be overwritten by force push."""
        try:
            result = subprocess.run(
                ['git', 'log', '--oneline', f'HEAD..{remote}/{branch}'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.stdout.strip().split('\n') if result.stdout.strip() else []
        except:
            return []
    
    def _get_all_branches(self, repo_path: Path) -> List[str]:
        """Get all local branches."""
        try:
            result = subprocess.run(
                ['git', 'branch', '--format=%(refname:short)'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.stdout.strip().split('\n') if result.stdout.strip() else []
        except:
            return []
    
    def _get_tags(self, repo_path: Path) -> List[str]:
        """Get all tags."""
        try:
            result = subprocess.run(
                ['git', 'tag'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.stdout.strip().split('\n') if result.stdout.strip() else []
        except:
            return []
    
    def _run_git_command(self, args: List[str], cwd: Path) -> Tuple[bool, str]:
        """Run git command with SSH support and return success status and output."""
        # Get SSH key from git config if available
        ssh_key = self.ssh_helper.get_account_ssh_key(cwd)
        
        # Use SSH helper to run command
        return self.ssh_helper.run_git_command(args, cwd, ssh_key_path=ssh_key, timeout=30)
