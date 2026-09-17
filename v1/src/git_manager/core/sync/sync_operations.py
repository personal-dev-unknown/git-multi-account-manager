"""Sync Operations - Comprehensive bidirectional sync strategies."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, List, Dict
from dataclasses import dataclass

from ...utils.logger import get_logger
from .branch_manager import BranchManager
from .git_ssh_helper import GitSSHHelper


logger = get_logger(__name__)


@dataclass
class SyncResult:
    """Result of a sync operation."""
    success: bool
    message: str
    details: Dict = None
    
    def __post_init__(self):
        if self.details is None:
            self.details = {}


class SyncOperations:
    """Comprehensive sync operations with multiple strategies."""
    
    def __init__(self):
        """Initialize Sync Operations."""
        self.branch_manager = BranchManager()
        self.ssh_helper = GitSSHHelper()
        logger.info("SyncOperations initialized")
    
    # ==================== MAIN SYNC OPERATIONS ====================
    
    def smart_sync(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> SyncResult:
        """
        🟢 Smart Sync - Intelligent bidirectional sync (RECOMMENDED).
        
        Algorithm:
        1. Run pre-sync checks
        2. Fetch to see current state
        3. Analyze situation:
           - Only local commits? → Push
           - Only remote commits? → Pull
           - Both (diverged)? → Pull with rebase, then push
           - Uncommitted changes? → Stash, sync, unstash
        4. Execute safest strategy
        5. Report results
        
        This is the "just sync it" option that handles everything
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            current_branch = branch or self.branch_manager.get_current_branch(repo_path)
            
            # Analyze situation
            situation = self._analyze_sync_situation(repo_path, current_branch, remote)
            
            if situation['type'] == 'UP_TO_DATE':
                return SyncResult(
                    success=True,
                    message="✓ Already up to date with remote",
                    details={
                        'branch': current_branch,
                        'status': 'up_to_date',
                        'actions': []
                    }
                )
            
            # Execute action plan
            actions_taken = []
            for action in situation['actions']:
                if action == 'stash':
                    self._stash_changes(repo_path)
                    actions_taken.append('stashed changes')
                elif action == 'pull_rebase':
                    success, _ = self._run_git_command(
                        ['git', 'pull', '--rebase', remote, current_branch],
                        repo_path
                    )
                    if not success:
                        return SyncResult(
                            success=False,
                            message="Sync failed during rebase",
                            details={'branch': current_branch, 'error': 'rebase failed'}
                        )
                    actions_taken.append('rebased')
                elif action == 'pull':
                    success, _ = self._run_git_command(
                        ['git', 'pull', remote, current_branch],
                        repo_path
                    )
                    if not success:
                        return SyncResult(
                            success=False,
                            message="Sync failed during pull",
                            details={'branch': current_branch, 'error': 'pull failed'}
                        )
                    actions_taken.append('pulled')
                elif action == 'push':
                    success, _ = self._run_git_command(
                        ['git', 'push', remote, current_branch],
                        repo_path
                    )
                    if not success:
                        return SyncResult(
                            success=False,
                            message="Sync failed during push",
                            details={'branch': current_branch, 'error': 'push failed'}
                        )
                    actions_taken.append('pushed')
                elif action == 'stash_pop':
                    self._stash_pop(repo_path)
                    actions_taken.append('restored changes')
            
            logger.info(f"Smart sync completed: {', '.join(actions_taken)}")
            return SyncResult(
                success=True,
                message=f"✓ Sync completed: {', '.join(actions_taken)}",
                details={
                    'branch': current_branch,
                    'status': situation['type'],
                    'actions': actions_taken,
                    'remote': remote
                }
            )
            
        except Exception as e:
            logger.error(f"Smart sync error: {e}")
            return SyncResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def conservative_sync(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> SyncResult:
        """
        🟢 Conservative Sync - Extra-safe sync with user confirmation at each step.
        
        Behavior:
        - Ask before stashing
        - Ask before pulling
        - Ask before pushing
        - Show detailed preview at each step
        - Good for beginners or critical branches
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            current_branch = branch or self.branch_manager.get_current_branch(repo_path)
            
            # Analyze situation
            situation = self._analyze_sync_situation(repo_path, current_branch, remote)
            
            if situation['type'] == 'UP_TO_DATE':
                return SyncResult(
                    success=True,
                    message="✓ Already up to date with remote",
                    details={'branch': current_branch, 'status': 'up_to_date'}
                )
            
            logger.info(f"Conservative sync plan: {situation['actions']}")
            return SyncResult(
                success=True,
                message=f"✓ Conservative sync ready (requires confirmation at each step)",
                details={
                    'branch': current_branch,
                    'status': situation['type'],
                    'actions': situation['actions'],
                    'requires_confirmation': True
                }
            )
            
        except Exception as e:
            logger.error(f"Conservative sync error: {e}")
            return SyncResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def rebase_sync(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> SyncResult:
        """
        🔵 Rebase Sync - Sync with rebase for clean history.
        
        Behavior:
        - Pull with rebase
        - Push (may need force-with-lease if history rewritten)
        - Results in linear history
        - For feature branches
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Pull with rebase
            success, output = self._run_git_command(
                ['git', 'pull', '--rebase', remote, current_branch],
                repo_path
            )
            
            if not success:
                return SyncResult(
                    success=False,
                    message=f"Rebase sync failed during pull: {output}",
                    details={'branch': current_branch, 'error': output}
                )
            
            # Push
            success, output = self._run_git_command(
                ['git', 'push', remote, current_branch],
                repo_path
            )
            
            if success:
                logger.info("Rebase sync completed successfully")
                return SyncResult(
                    success=True,
                    message="✓ Sync completed with rebase (clean linear history)",
                    details={
                        'branch': current_branch,
                        'method': 'rebase',
                        'remote': remote
                    }
                )
            else:
                return SyncResult(
                    success=False,
                    message=f"Rebase sync failed during push: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Rebase sync error: {e}")
            return SyncResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def merge_sync(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> SyncResult:
        """
        🔵 Merge Sync - Sync with merge commits.
        
        Behavior:
        - Pull with merge
        - Push
        - Preserves full history
        - For main/develop branches
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Pull with merge (no rebase)
            success, output = self._run_git_command(
                ['git', 'pull', '--no-rebase', remote, current_branch],
                repo_path
            )
            
            if not success:
                return SyncResult(
                    success=False,
                    message=f"Merge sync failed during pull: {output}",
                    details={'branch': current_branch, 'error': output}
                )
            
            # Push
            success, output = self._run_git_command(
                ['git', 'push', remote, current_branch],
                repo_path
            )
            
            if success:
                logger.info("Merge sync completed successfully")
                return SyncResult(
                    success=True,
                    message="✓ Sync completed with merge (full history preserved)",
                    details={
                        'branch': current_branch,
                        'method': 'merge',
                        'remote': remote
                    }
                )
            else:
                return SyncResult(
                    success=False,
                    message=f"Merge sync failed during push: {output}",
                    details={'branch': current_branch, 'error': output}
                )
                
        except Exception as e:
            logger.error(f"Merge sync error: {e}")
            return SyncResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def aggressive_sync(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> SyncResult:
        """
        🟡 Aggressive Sync - Sync with automatic conflict resolution.
        
        Behavior:
        - Auto-resolve conflicts (use 'theirs' or 'ours' strategy)
        - Force push if needed (with --force-with-lease)
        - Fast but potentially risky
        - For solo work or experimental branches
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Create backup
            backup_branch = f"backup-before-aggressive-sync-{current_branch}"
            self._run_git_command(['git', 'branch', backup_branch], repo_path)
            logger.warning(f"Created backup branch: {backup_branch}")
            
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Pull with rebase and auto-resolve
            success, output = self._run_git_command(
                ['git', 'pull', '--rebase', '-X', 'theirs', remote, current_branch],
                repo_path
            )
            
            if not success:
                return SyncResult(
                    success=False,
                    message=f"Aggressive sync failed during pull: {output}",
                    details={'branch': current_branch, 'error': output, 'backup': backup_branch}
                )
            
            # Force push if needed
            success, output = self._run_git_command(
                ['git', 'push', '--force-with-lease', remote, current_branch],
                repo_path
            )
            
            if success:
                logger.warning("Aggressive sync completed successfully")
                return SyncResult(
                    success=True,
                    message="✓ Aggressive sync completed (conflicts auto-resolved)",
                    details={
                        'branch': current_branch,
                        'method': 'aggressive',
                        'backup_branch': backup_branch,
                        'remote': remote
                    }
                )
            else:
                return SyncResult(
                    success=False,
                    message=f"Aggressive sync failed during push: {output}",
                    details={'branch': current_branch, 'error': output, 'backup': backup_branch}
                )
                
        except Exception as e:
            logger.error(f"Aggressive sync error: {e}")
            return SyncResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    def dry_run_sync(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> SyncResult:
        """
        🟣 Dry Run Sync - Preview what sync would do.
        
        Behavior:
        - Analyze current state
        - Show detailed plan of what would happen
        - No actual changes made
        - Perfect for understanding sync before executing
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            current_branch = branch or self.branch_manager.get_current_branch(repo_path)
            
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Analyze situation
            situation = self._analyze_sync_situation(repo_path, current_branch, remote)
            
            logger.info(f"Dry-run sync plan: {situation}")
            return SyncResult(
                success=True,
                message=f"✓ Dry-run: Would {', '.join(situation['actions'])}",
                details={
                    'branch': current_branch,
                    'status': situation['type'],
                    'plan': situation['actions'],
                    'local_ahead': situation.get('local_ahead', 0),
                    'remote_ahead': situation.get('remote_ahead', 0),
                    'uncommitted': situation.get('uncommitted', False)
                }
            )
            
        except Exception as e:
            logger.error(f"Dry-run sync error: {e}")
            return SyncResult(
                success=False,
                message=f"Error: {str(e)}",
                details={'error': str(e)}
            )
    
    # ==================== HELPER METHODS ====================
    
    def _analyze_sync_situation(self, repo_path: Path, branch: str, remote: str) -> dict:
        """
        Determine what sync actions are needed.
        
        Returns:
            dict with type, actions, warnings, strategy
        """
        situation = {
            'type': None,
            'actions': [],
            'warnings': [],
            'strategy': None,
            'local_ahead': 0,
            'remote_ahead': 0,
            'uncommitted': False
        }
        
        try:
            # Fetch current state
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Get commit counts
            local_ahead = self._count_local_commits(repo_path, branch, remote)
            remote_ahead = self._count_remote_commits(repo_path, branch, remote)
            uncommitted = self._has_uncommitted_changes(repo_path)
            
            situation['local_ahead'] = local_ahead
            situation['remote_ahead'] = remote_ahead
            situation['uncommitted'] = uncommitted
            
            # Determine situation type
            if local_ahead == 0 and remote_ahead == 0:
                situation['type'] = 'UP_TO_DATE'
                situation['actions'] = []
                situation['strategy'] = 'none'
                
            elif local_ahead > 0 and remote_ahead == 0:
                situation['type'] = 'AHEAD_OF_REMOTE'
                situation['actions'] = ['push']
                situation['strategy'] = 'push_only'
                
            elif local_ahead == 0 and remote_ahead > 0:
                situation['type'] = 'BEHIND_REMOTE'
                situation['actions'] = ['pull']
                situation['strategy'] = 'pull_only'
                
            elif local_ahead > 0 and remote_ahead > 0:
                situation['type'] = 'DIVERGED'
                situation['actions'] = ['pull_rebase', 'push']
                situation['strategy'] = 'rebase_sync'
                situation['warnings'].append(
                    "⚠️  Branches have diverged. Will rebase local commits."
                )
            
            # Handle uncommitted changes
            if uncommitted:
                situation['actions'].insert(0, 'stash')
                situation['actions'].append('stash_pop')
                situation['warnings'].append(
                    "ℹ️  Uncommitted changes will be stashed temporarily"
                )
            
        except Exception as e:
            logger.error(f"Analyze sync situation error: {e}")
            situation['warnings'].append(f"⚠️  Could not analyze situation: {str(e)}")
        
        return situation
    
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
    
    def _count_remote_commits(self, repo_path: Path, branch: str, remote: str) -> int:
        """Count remote commits ahead of local."""
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
    
    def _has_uncommitted_changes(self, repo_path: Path) -> bool:
        """Check if there are uncommitted changes."""
        try:
            result = subprocess.run(
                ['git', 'status', '--porcelain'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return bool(result.stdout.strip())
        except:
            return False
    
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
