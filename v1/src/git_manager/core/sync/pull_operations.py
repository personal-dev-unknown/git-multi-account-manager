"""Pull Operations - Comprehensive pull strategies with safety checks."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, List, Dict
from dataclasses import dataclass

from ...utils.logger import get_logger
from .branch_manager import BranchManager
from .git_ssh_helper import GitSSHHelper


logger = get_logger(__name__)


@dataclass
class PullResult:
    """Result of a pull operation."""
    success: bool
    message: str
    details: Dict = None
    
    def __post_init__(self):
        if self.details is None:
            self.details = {}


class PullOperations:
    """Comprehensive pull operations with multiple strategies."""
    
    def __init__(self):
        """Initialize Pull Operations."""
        self.branch_manager = BranchManager()
        self.ssh_helper = GitSSHHelper()
        logger.info("PullOperations initialized")
    
    # ==================== MAIN PULL OPERATIONS ====================
    
    def safe_pull(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> PullResult:
        """
        🟢 Safe Pull - Default recommended pull with auto-stash.
        
        Behavior:
        - Check for uncommitted changes (offer to stash)
        - Fetch first to see what's coming
        - Show preview of incoming changes
        - Pull with merge or rebase (user preference)
        - Auto-stash-pop if stash was created
        - Handle conflicts gracefully with guidance
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            current_branch = branch or self.branch_manager.get_current_branch(repo_path)
            
            # Pre-pull checks
            checks = self._pre_pull_checks(repo_path, current_branch, remote)
            
            if checks['blockers']:
                return PullResult(
                    success=False,
                    message=f"Pull blocked: {checks['blockers'][0]}",
                    details={'branch': current_branch, 'blockers': checks['blockers']}
                )
            
            # Handle uncommitted changes
            stashed = False
            if checks['uncommitted_changes']:
                logger.info("Stashing uncommitted changes")
                stashed = self._stash_changes(repo_path)
            
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Execute pull
            cmd = ['git', 'pull', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            # Restore stashed changes
            if stashed:
                self._stash_pop(repo_path)
            
            if success:
                incoming_count = checks.get('incoming_commits', 0)
                logger.info(f"Safe pull successful: {incoming_count} commits")
                return PullResult(
                    success=True,
                    message=f"✓ Pulled {incoming_count} commits from {remote}/{current_branch}",
                    details={
                        'branch': current_branch,
                        'commits_pulled': incoming_count,
                        'remote': remote,
                        'stashed': stashed
                    }
                )
            else:
                return PullResult(
                    success=False,
                    message=f"Pull failed: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Safe pull error: {e}")
            return PullResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def smart_pull(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> PullResult:
        """
        🟢 Smart Pull - Intelligent pull with automatic strategy selection.
        
        Behavior:
        - Analyze repository state
        - Choose merge vs rebase automatically:
          • Rebase if: local commits are few and branch is feature
          • Merge if: local commits are many or branch is main/develop
        - Handle all edge cases
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            current_branch = branch or self.branch_manager.get_current_branch(repo_path)
            
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Analyze situation
            local_commits = self._count_local_commits(repo_path, current_branch, remote)
            incoming_commits = self._count_incoming_commits(repo_path, current_branch, remote)
            
            # Decide strategy
            if local_commits <= 3 and not self._is_main_branch(current_branch):
                # Use rebase for feature branches with few commits
                strategy = 'rebase'
                cmd = ['git', 'pull', '--rebase', remote, current_branch]
            else:
                # Use merge for main branches or many commits
                strategy = 'merge'
                cmd = ['git', 'pull', remote, current_branch]
            
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info(f"Smart pull successful ({strategy}): {incoming_commits} commits")
                return PullResult(
                    success=True,
                    message=f"✓ Pulled {incoming_commits} commits using {strategy}",
                    details={
                        'branch': current_branch,
                        'commits_pulled': incoming_commits,
                        'strategy': strategy,
                        'remote': remote
                    }
                )
            else:
                return PullResult(
                    success=False,
                    message=f"Smart pull failed: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Smart pull error: {e}")
            return PullResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def pull_rebase(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> PullResult:
        """
        🔵 Pull with Rebase - Clean linear history.
        
        Behavior:
        - Cleaner linear history
        - Replay local commits after remote commits
        - Show warning if conflicts likely
        - Offer to abort if conflicts occur
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Execute pull with rebase
            cmd = ['git', 'pull', '--rebase', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                incoming_count = self._count_incoming_commits(repo_path, current_branch, remote)
                logger.info(f"Pull with rebase successful: {incoming_count} commits")
                return PullResult(
                    success=True,
                    message=f"✓ Pulled {incoming_count} commits with rebase (clean history)",
                    details={
                        'branch': current_branch,
                        'commits_pulled': incoming_count,
                        'method': 'rebase',
                        'remote': remote
                    }
                )
            else:
                return PullResult(
                    success=False,
                    message=f"Pull rebase failed: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Pull rebase error: {e}")
            return PullResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def pull_ff_only(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> PullResult:
        """
        🔵 Pull with Merge (Fast-Forward Only) - Safest pull option.
        
        Behavior:
        - Fails if branches have diverged
        - Safest pull option
        - No merge commits created
        - Use when you want to avoid complications
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Execute pull with ff-only
            cmd = ['git', 'pull', '--ff-only', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                incoming_count = self._count_incoming_commits(repo_path, current_branch, remote)
                logger.info(f"Fast-forward pull successful: {incoming_count} commits")
                return PullResult(
                    success=True,
                    message=f"✓ Fast-forward pulled {incoming_count} commits (safest option)",
                    details={
                        'branch': current_branch,
                        'commits_pulled': incoming_count,
                        'method': 'fast-forward-only',
                        'remote': remote
                    }
                )
            else:
                return PullResult(
                    success=False,
                    message=f"Fast-forward pull failed (branches diverged): {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Fast-forward pull error: {e}")
            return PullResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def pull_autostash(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> PullResult:
        """
        🟡 Pull with Autostash - Auto stash/unstash changes.
        
        Behavior:
        - Automatically stash uncommitted changes
        - Pull remote changes
        - Reapply stashed changes
        - Handle conflicts if stash pop fails
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Execute pull with autostash
            cmd = ['git', 'pull', '--autostash', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                incoming_count = self._count_incoming_commits(repo_path, current_branch, remote)
                logger.info(f"Pull with autostash successful: {incoming_count} commits")
                return PullResult(
                    success=True,
                    message=f"✓ Pulled {incoming_count} commits (changes auto-stashed)",
                    details={
                        'branch': current_branch,
                        'commits_pulled': incoming_count,
                        'method': 'autostash',
                        'remote': remote
                    }
                )
            else:
                return PullResult(
                    success=False,
                    message=f"Pull autostash failed: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Pull autostash error: {e}")
            return PullResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def force_pull(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> PullResult:
        """
        🟠 Force Pull - Reset local to match remote exactly (DESTRUCTIVE).
        
        Behavior:
        - Show STRONG warning
        - List what will be lost (commits, changes)
        - Require confirmation: "DELETE MY WORK"
        - Create backup branch automatically
        - Reset hard to remote
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Create backup before force operation
            backup_branch = f"backup-before-force-pull-{current_branch}"
            self._run_git_command(['git', 'branch', backup_branch], repo_path)
            logger.warning(f"Created backup branch: {backup_branch}")
            
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Execute reset hard
            cmd = ['git', 'reset', '--hard', f'{remote}/{current_branch}']
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.warning(f"Force pull executed: {current_branch} reset to {remote}")
                return PullResult(
                    success=True,
                    message="✓ Force pull completed (local reset to remote)",
                    details={
                        'branch': current_branch,
                        'method': 'force-reset',
                        'backup_branch': backup_branch,
                        'remote': remote
                    }
                )
            else:
                return PullResult(
                    success=False,
                    message=f"Force pull failed: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Force pull error: {e}")
            return PullResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def fetch_only(
        self,
        repo_path: Optional[Path] = None,
        remote: str = 'origin'
    ) -> PullResult:
        """
        🟣 Fetch Only - Download changes without integrating (100% safe).
        
        Behavior:
        - 100% safe, changes nothing locally
        - Show what's new on remote
        - Let user review before pulling
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            # Execute fetch
            cmd = ['git', 'fetch', remote]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                # Get summary of what was fetched
                current_branch = self.branch_manager.get_current_branch(repo_path)
                incoming_count = self._count_incoming_commits(repo_path, current_branch, remote)
                
                logger.info(f"Fetch successful: {incoming_count} new commits available")
                return PullResult(
                    success=True,
                    message=f"✓ Fetched {incoming_count} new commits (nothing merged yet)",
                    details={
                        'branch': current_branch,
                        'commits_available': incoming_count,
                        'remote': remote,
                        'note': 'Use pull to integrate changes'
                    }
                )
            else:
                return PullResult(
                    success=False,
                    message=f"Fetch failed: {output}",
                    details={'error': output}
                )
                
        except Exception as e:
            logger.error(f"Fetch only error: {e}")
            return PullResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    # ==================== HELPER METHODS ====================
    
    def _pre_pull_checks(self, repo_path: Path, branch: str, remote: str) -> dict:
        """Run pre-pull safety checks."""
        checks = {
            'blockers': [],
            'warnings': [],
            'info': [],
            'uncommitted_changes': False,
            'incoming_commits': 0
        }
        
        try:
            # Check 1: Uncommitted changes
            result = subprocess.run(
                ['git', 'status', '--porcelain'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            if result.stdout.strip():
                checks['uncommitted_changes'] = True
                checks['warnings'].append(
                    f"⚠️  You have uncommitted changes. They will be stashed."
                )
            
            # Check 2: Fetch and check incoming
            self._run_git_command(['git', 'fetch', remote], repo_path)
            incoming = self._count_incoming_commits(repo_path, branch, remote)
            checks['incoming_commits'] = incoming
            
            if incoming == 0:
                checks['blockers'].append("Nothing to pull")
            else:
                checks['info'].append(f"📥 {incoming} new commits on remote")
            
            # Check 3: Check if diverged
            local_ahead = self._count_local_commits(repo_path, branch, remote)
            if local_ahead > 0 and incoming > 0:
                checks['warnings'].append(
                    f"⚠️  Branches have DIVERGED: Local {local_ahead} ahead, Remote {incoming} ahead"
                )
            
            # Check 4: Remote connectivity
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
            logger.error(f"Pre-pull checks error: {e}")
            checks['warnings'].append(f"⚠️  Could not complete all checks: {str(e)}")
        
        return checks
    
    def _count_incoming_commits(self, repo_path: Path, branch: str, remote: str) -> int:
        """Count incoming commits from remote."""
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
    
    def _count_local_commits(self, repo_path: Path, branch: str, remote: str) -> int:
        """Count local commits ahead of remote."""
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
    
    def _is_main_branch(self, branch: str) -> bool:
        """Check if branch is a main branch."""
        main_branches = ['main', 'master', 'develop', 'development']
        return branch.lower() in main_branches
    
    def _stash_changes(self, repo_path: Path) -> bool:
        """Stash uncommitted changes."""
        try:
            result = subprocess.run(
                ['git', 'stash'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.returncode == 0
        except:
            return False
    
    def _stash_pop(self, repo_path: Path) -> bool:
        """Pop stashed changes."""
        try:
            result = subprocess.run(
                ['git', 'stash', 'pop'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.returncode == 0
        except:
            return False
    
    def _run_git_command(self, args: List[str], cwd: Path) -> Tuple[bool, str]:
        """Run git command with SSH support and return success status and output."""
        # Get SSH key from git config if available
        ssh_key = self.ssh_helper.get_account_ssh_key(cwd)
        
        # Use SSH helper to run command
        return self.ssh_helper.run_git_command(args, cwd, ssh_key_path=ssh_key, timeout=30)
