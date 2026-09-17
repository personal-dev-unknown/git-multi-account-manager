# src/git_manager/core/git_pull.py
"""Git Pull Operations - Safe, smart, and powerful pull strategies."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, List, Dict
from enum import Enum

from ..utils.logger import get_logger


logger = get_logger(__name__)


class PullStrategy(str, Enum):
    """Pull strategy options."""
    SAFE_PULL = "safe_pull"                    # Default safe pull
    SMART_PULL = "smart_pull"                  # Intelligent strategy
    PULL_REBASE = "pull_rebase"                # Pull with rebase
    PULL_FF_ONLY = "pull_ff_only"              # Fast-forward only
    PULL_AUTOSTASH = "pull_autostash"          # Auto stash/unstash
    FORCE_PULL = "force_pull"                  # Reset to remote
    FETCH_ONLY = "fetch_only"                  # Download only
    PULL_ALL_BRANCHES = "pull_all_branches"    # Update all branches


class GitPull:
    """Handles Git pull operations with safety checks."""
    
    def __init__(self):
        """Initialize Git Pull."""
        logger.info("GitPull initialized")
    
    def safe_pull(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str, dict]:
        """Safe pull with automatic conflict handling.
        
        Args:
            repo_path: Repository path
            branch: Branch to pull
            remote: Remote name
            
        Returns:
            Tuple of (success, message, details)
        """
        repo_path = repo_path or Path.cwd()
        details = {
            "stashed": False,
            "pulled": False,
            "conflicts": False,
            "commits_pulled": 0
        }
        
        try:
            current_branch = branch or self._get_current_branch(repo_path)
            
            # Pre-flight checks
            checks = self._pre_pull_checks(repo_path, current_branch, remote)
            
            if checks['blockers']:
                return False, f"Pull blocked: {checks['blockers'][0]}", details
            
            # Check for uncommitted changes
            uncommitted = self._get_uncommitted_files(repo_path)
            if uncommitted:
                # Stash changes
                success, msg = self._run_git_command(['stash', 'push', '-m', 'Safe pull stash'], repo_path)
                if not success:
                    return False, f"Failed to stash: {msg}", details
                details["stashed"] = True
                logger.info(f"Stashed {len(uncommitted)} uncommitted changes")
            
            # Fetch first
            success, msg = self._run_git_command(['fetch', remote], repo_path)
            if not success:
                return False, f"Fetch failed: {msg}", details
            
            # Pull with merge
            success, msg = self._run_git_command(['pull', remote, current_branch], repo_path)
            
            if not success:
                # Restore stashed changes if pull failed
                if details["stashed"]:
                    self._run_git_command(['stash', 'pop'], repo_path)
                return False, f"Pull failed: {msg}", details
            
            details["pulled"] = True
            details["commits_pulled"] = self._count_pulled_commits(repo_path, current_branch, remote)
            
            # Reapply stashed changes if any
            if details["stashed"]:
                success, msg = self._run_git_command(['stash', 'pop'], repo_path)
                if not success:
                    details["conflicts"] = True
                    return False, f"Conflicts when reapplying changes: {msg}", details
            
            logger.info(f"Safe pull successful: {details['commits_pulled']} commits")
            return True, f"✓ Pulled {details['commits_pulled']} commits", details
            
        except Exception as e:
            logger.error(f"Safe pull error: {e}")
            return False, f"Pull error: {str(e)}", details
    
    def smart_pull(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str]:
        """Intelligent pull with automatic strategy selection.
        
        Args:
            repo_path: Repository path
            branch: Branch to pull
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        try:
            # Analyze situation
            diverged, local_commits, remote_commits = self._check_divergence(repo_path, current_branch, remote)
            
            if diverged and local_commits > 0:
                # Use rebase for cleaner history
                logger.info("Branches diverged - using rebase strategy")
                success, msg = self._run_git_command(['pull', '--rebase', remote, current_branch], repo_path)
            else:
                # Use merge
                success, msg = self._run_git_command(['pull', remote, current_branch], repo_path)
            
            if success:
                logger.info("Smart pull successful")
                return True, "✓ Smart pull successful"
            else:
                return False, f"Smart pull failed: {msg}"
                
        except Exception as e:
            logger.error(f"Smart pull error: {e}")
            return False, f"Error: {str(e)}"
    
    def pull_rebase(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str]:
        """Pull and rebase local commits on top.
        
        Args:
            repo_path: Repository path
            branch: Branch to pull
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        try:
            cmd = ['pull', '--rebase', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info("Pull with rebase successful")
                return True, "✓ Pull with rebase successful (clean history)"
            else:
                return False, f"Pull with rebase failed: {output}"
                
        except Exception as e:
            logger.error(f"Pull rebase error: {e}")
            return False, f"Error: {str(e)}"
    
    def pull_ff_only(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str]:
        """Pull only if fast-forward is possible.
        
        Args:
            repo_path: Repository path
            branch: Branch to pull
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        try:
            cmd = ['pull', '--ff-only', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info("Fast-forward pull successful")
                return True, "✓ Fast-forward pull successful (safest option)"
            else:
                return False, f"Fast-forward pull failed: {output}"
                
        except Exception as e:
            logger.error(f"Fast-forward pull error: {e}")
            return False, f"Error: {str(e)}"
    
    def pull_autostash(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str]:
        """Pull with automatic stash and pop.
        
        Args:
            repo_path: Repository path
            branch: Branch to pull
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        try:
            cmd = ['pull', '--autostash', remote, current_branch]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info("Pull with autostash successful")
                return True, "✓ Pull with autostash successful"
            else:
                return False, f"Pull with autostash failed: {output}"
                
        except Exception as e:
            logger.error(f"Pull autostash error: {e}")
            return False, f"Error: {str(e)}"
    
    def force_pull(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str]:
        """Reset local to match remote exactly (destructive).
        
        Args:
            repo_path: Repository path
            branch: Branch to reset to
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self._get_current_branch(repo_path)
        
        try:
            # Create backup
            backup_branch = f"backup-before-force-pull-{current_branch}"
            self._run_git_command(['branch', backup_branch], repo_path)
            logger.warning(f"Created backup branch: {backup_branch}")
            
            # Fetch
            success, msg = self._run_git_command(['fetch', remote], repo_path)
            if not success:
                return False, f"Fetch failed: {msg}"
            
            # Reset hard
            success, msg = self._run_git_command(
                ['reset', '--hard', f'{remote}/{current_branch}'],
                repo_path
            )
            if not success:
                return False, f"Reset failed: {msg}"
            
            # Clean
            success, msg = self._run_git_command(['clean', '-fd'], repo_path)
            if not success:
                return False, f"Clean failed: {msg}"
            
            logger.warning("Force pull executed - repository reset to remote")
            return True, f"✓ Reset to {remote}/{current_branch} (backup: {backup_branch})"
            
        except Exception as e:
            logger.error(f"Force pull error: {e}")
            return False, f"Error: {str(e)}"
    
    def fetch_only(
        self,
        repo_path: Optional[Path] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str]:
        """Download changes without integrating.
        
        Args:
            repo_path: Repository path
            remote: Remote name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            cmd = ['fetch', remote]
            success, output = self._run_git_command(cmd, repo_path)
            
            if success:
                logger.info("Fetch successful")
                return True, "✓ Fetch successful (100% safe, no changes made)"
            else:
                return False, f"Fetch failed: {output}"
                
        except Exception as e:
            logger.error(f"Fetch error: {e}")
            return False, f"Error: {str(e)}"
    
    def pull_all_branches(
        self,
        repo_path: Optional[Path] = None,
        remote: str = "origin"
    ) -> Tuple[bool, str, int]:
        """Update all tracking branches.
        
        Args:
            repo_path: Repository path
            remote: Remote name
            
        Returns:
            Tuple of (success, message, branches_updated)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            cmd = ['fetch', remote]
            success, output = self._run_git_command(cmd, repo_path)
            
            if not success:
                return False, f"Fetch failed: {output}", 0
            
            # Count updated branches
            result = subprocess.run(
                ['git', 'branch', '-r'],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            branches = len([b for b in result.stdout.split('\n') if b.strip()])
            
            logger.info(f"Updated {branches} remote branches")
            return True, f"✓ Updated {branches} branches", branches
            
        except Exception as e:
            logger.error(f"Pull all branches error: {e}")
            return False, f"Error: {str(e)}", 0
    
    def _pre_pull_checks(self, repo_path: Path, branch: str, remote: str) -> dict:
        """Run pre-pull safety checks."""
        checks = {
            'blockers': [],
            'warnings': [],
            'info': []
        }
        
        try:
            # Check 1: Remote connectivity
            result = subprocess.run(
                ['git', 'ls-remote', '--heads', remote],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode != 0:
                checks['blockers'].append(f"Cannot reach remote: {remote}")
                return checks
            
            # Check 2: Incoming commits
            self._run_git_command(['fetch', remote], repo_path)
            result = subprocess.run(
                ['git', 'rev-list', '--count', f'HEAD..{remote}/{branch}'],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            incoming = int(result.stdout.strip() or 0)
            if incoming == 0:
                checks['info'].append("✓ Already up to date")
            else:
                checks['info'].append(f"📥 {incoming} new commits on remote")
            
            # Check 3: Uncommitted changes
            uncommitted = self._get_uncommitted_files(repo_path)
            if uncommitted:
                checks['warnings'].append(
                    f"⚠️  {len(uncommitted)} uncommitted changes (will be stashed)"
                )
            
        except Exception as e:
            logger.error(f"Pre-pull checks error: {e}")
            checks['warnings'].append(f"⚠️  Could not complete all checks: {str(e)}")
        
        return checks
    
    def _check_divergence(self, repo_path: Path, branch: str, remote: str) -> Tuple[bool, int, int]:
        """Check if local and remote branches have diverged."""
        try:
            result = subprocess.run(
                ['git', 'rev-list', '--left-right', '--count', f'HEAD...{remote}/{branch}'],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            if result.returncode == 0 and result.stdout.strip():
                local, remote_count = map(int, result.stdout.split())
                diverged = local > 0 and remote_count > 0
                return diverged, local, remote_count
        except:
            pass
        return False, 0, 0
    
    def _count_pulled_commits(self, repo_path: Path, branch: str, remote: str) -> int:
        """Count commits that were pulled."""
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
