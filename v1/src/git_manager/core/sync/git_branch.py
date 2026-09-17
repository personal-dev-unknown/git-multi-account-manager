"""Git Branch - Branch management operations."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, List, Dict
from dataclasses import dataclass

from ...utils.logger import get_logger
from .branch_manager import BranchManager


logger = get_logger(__name__)


@dataclass
class BranchInfo:
    """Branch information."""
    name: str
    is_current: bool
    tracking: Optional[str] = None
    ahead: int = 0
    behind: int = 0


class GitBranch:
    """Git branch management operations."""
    
    def __init__(self):
        """Initialize Git Branch."""
        self.branch_manager = BranchManager()
        logger.info("GitBranch initialized")
    
    def list_branches(
        self,
        repo_path: Optional[Path] = None,
        remote: bool = False
    ) -> List[BranchInfo]:
        """
        List all branches (local or remote).
        
        Args:
            repo_path: Repository path
            remote: If True, list remote branches
            
        Returns:
            List of BranchInfo objects
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            if remote:
                result = subprocess.run(
                    ['git', 'branch', '-r', '--format=%(refname:short)'],
                    cwd=repo_path,
                    capture_output=True,
                    text=True,
                    timeout=10
                )
            else:
                result = subprocess.run(
                    ['git', 'branch', '--format=%(refname:short)'],
                    cwd=repo_path,
                    capture_output=True,
                    text=True,
                    timeout=10
                )
            
            branches = result.stdout.strip().split('\n') if result.stdout.strip() else []
            current = self.branch_manager.get_current_branch(repo_path)
            
            branch_infos = []
            for branch in branches:
                if branch:
                    branch_infos.append(BranchInfo(
                        name=branch,
                        is_current=(branch == current)
                    ))
            
            logger.info(f"Listed {len(branch_infos)} branches")
            return branch_infos
            
        except Exception as e:
            logger.error(f"List branches error: {e}")
            return []
    
    def create_branch(
        self,
        repo_path: Optional[Path] = None,
        branch_name: str = None,
        from_branch: Optional[str] = None
    ) -> Tuple[bool, str]:
        """
        Create a new branch.
        
        Args:
            repo_path: Repository path
            branch_name: Name for new branch
            from_branch: Create from this branch (default: current)
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not branch_name:
            return False, "Branch name required"
        
        try:
            if from_branch:
                cmd = ['git', 'branch', branch_name, from_branch]
            else:
                cmd = ['git', 'branch', branch_name]
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Created branch: {branch_name}")
                return True, f"✓ Created branch '{branch_name}'"
            else:
                error = result.stderr.strip()
                return False, f"Failed to create branch: {error}"
                
        except Exception as e:
            logger.error(f"Create branch error: {e}")
            return False, f"Error: {str(e)}"
    
    def switch_branch(
        self,
        repo_path: Optional[Path] = None,
        branch_name: str = None
    ) -> Tuple[bool, str]:
        """
        Switch to a different branch.
        
        Args:
            repo_path: Repository path
            branch_name: Branch to switch to
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not branch_name:
            return False, "Branch name required"
        
        try:
            result = subprocess.run(
                ['git', 'checkout', branch_name],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Switched to branch: {branch_name}")
                return True, f"✓ Switched to '{branch_name}'"
            else:
                error = result.stderr.strip()
                return False, f"Failed to switch branch: {error}"
                
        except Exception as e:
            logger.error(f"Switch branch error: {e}")
            return False, f"Error: {str(e)}"
    
    def delete_branch(
        self,
        repo_path: Optional[Path] = None,
        branch_name: str = None,
        force: bool = False
    ) -> Tuple[bool, str]:
        """
        Delete a branch.
        
        Args:
            repo_path: Repository path
            branch_name: Branch to delete
            force: Force delete even if not merged
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not branch_name:
            return False, "Branch name required"
        
        try:
            cmd = ['git', 'branch']
            if force:
                cmd.append('-D')
            else:
                cmd.append('-d')
            cmd.append(branch_name)
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Deleted branch: {branch_name}")
                return True, f"✓ Deleted branch '{branch_name}'"
            else:
                error = result.stderr.strip()
                return False, f"Failed to delete branch: {error}"
                
        except Exception as e:
            logger.error(f"Delete branch error: {e}")
            return False, f"Error: {str(e)}"
    
    def rename_branch(
        self,
        repo_path: Optional[Path] = None,
        old_name: str = None,
        new_name: str = None
    ) -> Tuple[bool, str]:
        """
        Rename a branch.
        
        Args:
            repo_path: Repository path
            old_name: Current branch name
            new_name: New branch name
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not old_name or not new_name:
            return False, "Both old and new branch names required"
        
        try:
            result = subprocess.run(
                ['git', 'branch', '-m', old_name, new_name],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Renamed branch: {old_name} → {new_name}")
                return True, f"✓ Renamed '{old_name}' to '{new_name}'"
            else:
                error = result.stderr.strip()
                return False, f"Failed to rename branch: {error}"
                
        except Exception as e:
            logger.error(f"Rename branch error: {e}")
            return False, f"Error: {str(e)}"
    
    def get_current_branch(
        self,
        repo_path: Optional[Path] = None
    ) -> str:
        """
        Get the current branch name.
        
        Args:
            repo_path: Repository path
            
        Returns:
            Current branch name
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            return self.branch_manager.get_current_branch(repo_path)
        except Exception as e:
            logger.error(f"Get current branch error: {e}")
            return "unknown"
    
    def get_branch_info(
        self,
        repo_path: Optional[Path] = None,
        branch: Optional[str] = None,
        remote: str = 'origin'
    ) -> Dict:
        """
        Get detailed information about a branch.
        
        Args:
            repo_path: Repository path
            branch: Branch name (default: current)
            remote: Remote name
            
        Returns:
            Dict with branch information
        """
        repo_path = repo_path or Path.cwd()
        current_branch = branch or self.branch_manager.get_current_branch(repo_path)
        
        try:
            # Get tracking info
            result = subprocess.run(
                ['git', 'rev-parse', '--abbrev-ref', f'{current_branch}@{{u}}'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            tracking = result.stdout.strip() if result.returncode == 0 else None
            
            # Get ahead/behind counts
            ahead = self._count_ahead(repo_path, current_branch, remote)
            behind = self._count_behind(repo_path, current_branch, remote)
            
            return {
                'name': current_branch,
                'tracking': tracking,
                'ahead': ahead,
                'behind': behind,
                'remote': remote
            }
            
        except Exception as e:
            logger.error(f"Get branch info error: {e}")
            return {
                'name': current_branch,
                'error': str(e)
            }
    
    # ==================== HELPER METHODS ====================
    
    def _count_ahead(self, repo_path: Path, branch: str, remote: str) -> int:
        """Count commits ahead of remote."""
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
    
    def _count_behind(self, repo_path: Path, branch: str, remote: str) -> int:
        """Count commits behind remote."""
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
