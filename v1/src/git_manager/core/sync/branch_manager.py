"""Branch Management - Handle branch operations and selection."""

import subprocess
from pathlib import Path
from typing import Optional, List, Dict, Tuple
from dataclasses import dataclass

from ...utils.logger import get_logger


logger = get_logger(__name__)


@dataclass
class Branch:
    """Branch information."""
    name: str
    is_current: bool = False
    is_remote: bool = False
    is_tracking: bool = False
    tracking_branch: Optional[str] = None
    commit_count_ahead: int = 0
    commit_count_behind: int = 0


class BranchManager:
    """Manage Git branches with intelligent selection."""
    
    def __init__(self):
        """Initialize Branch Manager."""
        logger.info("BranchManager initialized")
    
    def get_current_branch(self, repo_path: Optional[Path] = None) -> str:
        """Get current branch name.
        
        Args:
            repo_path: Repository path
            
        Returns:
            Current branch name
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            result = subprocess.run(
                ['git', 'rev-parse', '--abbrev-ref', 'HEAD'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.stdout.strip() or 'main'
        except Exception as e:
            logger.error(f"Error getting current branch: {e}")
            return 'main'
    
    def get_default_branch(self, repo_path: Optional[Path] = None) -> str:
        """Get repository default branch (main/master).
        
        Args:
            repo_path: Repository path
            
        Returns:
            Default branch name
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            # Try to get from remote
            result = subprocess.run(
                ['git', 'symbolic-ref', 'refs/remotes/origin/HEAD'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                # Output format: refs/remotes/origin/main
                ref = result.stdout.strip()
                return ref.split('/')[-1]
            
            # Fallback to checking local branches
            for branch_name in ['main', 'master', 'develop']:
                if self._branch_exists(repo_path, branch_name):
                    return branch_name
            
            # Last resort: get current branch
            return self.get_current_branch(repo_path)
            
        except Exception as e:
            logger.error(f"Error getting default branch: {e}")
            return 'main'
    
    def list_local_branches(self, repo_path: Optional[Path] = None) -> List[Branch]:
        """List all local branches.
        
        Args:
            repo_path: Repository path
            
        Returns:
            List of Branch objects
        """
        repo_path = repo_path or Path.cwd()
        branches = []
        
        try:
            result = subprocess.run(
                ['git', 'branch', '-v'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            current = self.get_current_branch(repo_path)
            
            for line in result.stdout.split('\n'):
                if not line.strip():
                    continue
                
                is_current = line.startswith('*')
                branch_name = line.replace('*', '').strip().split()[0]
                
                branch = Branch(
                    name=branch_name,
                    is_current=is_current,
                    is_remote=False
                )
                branches.append(branch)
            
            return branches
            
        except Exception as e:
            logger.error(f"Error listing local branches: {e}")
            return []
    
    def list_remote_branches(self, repo_path: Optional[Path] = None, remote: str = 'origin') -> List[Branch]:
        """List all remote branches.
        
        Args:
            repo_path: Repository path
            remote: Remote name
            
        Returns:
            List of Branch objects
        """
        repo_path = repo_path or Path.cwd()
        branches = []
        
        try:
            result = subprocess.run(
                ['git', 'branch', '-r'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            for line in result.stdout.split('\n'):
                if not line.strip() or 'HEAD' in line:
                    continue
                
                branch_name = line.strip().replace(f'{remote}/', '')
                
                branch = Branch(
                    name=branch_name,
                    is_remote=True
                )
                branches.append(branch)
            
            return branches
            
        except Exception as e:
            logger.error(f"Error listing remote branches: {e}")
            return []
    
    def create_branch(
        self,
        repo_path: Optional[Path] = None,
        branch_name: str = None,
        from_branch: Optional[str] = None
    ) -> Tuple[bool, str]:
        """Create a new branch.
        
        Args:
            repo_path: Repository path
            branch_name: New branch name
            from_branch: Create from this branch (default: current)
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not branch_name:
            return False, "Branch name required"
        
        try:
            if from_branch:
                cmd = ['git', 'checkout', '-b', branch_name, from_branch]
            else:
                cmd = ['git', 'checkout', '-b', branch_name]
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Created branch: {branch_name}")
                return True, f"✓ Created branch: {branch_name}"
            else:
                return False, result.stderr.strip()
                
        except Exception as e:
            logger.error(f"Error creating branch: {e}")
            return False, str(e)
    
    def switch_branch(
        self,
        repo_path: Optional[Path] = None,
        branch_name: str = None
    ) -> Tuple[bool, str]:
        """Switch to a different branch.
        
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
                return True, f"✓ Switched to: {branch_name}"
            else:
                return False, result.stderr.strip()
                
        except Exception as e:
            logger.error(f"Error switching branch: {e}")
            return False, str(e)
    
    def delete_branch(
        self,
        repo_path: Optional[Path] = None,
        branch_name: str = None,
        force: bool = False
    ) -> Tuple[bool, str]:
        """Delete a branch.
        
        Args:
            repo_path: Repository path
            branch_name: Branch to delete
            force: Force delete
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not branch_name:
            return False, "Branch name required"
        
        try:
            cmd = ['git', 'branch', '-D' if force else '-d', branch_name]
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Deleted branch: {branch_name}")
                return True, f"✓ Deleted branch: {branch_name}"
            else:
                return False, result.stderr.strip()
                
        except Exception as e:
            logger.error(f"Error deleting branch: {e}")
            return False, str(e)
    
    def get_branch_info(
        self,
        repo_path: Optional[Path] = None,
        branch_name: Optional[str] = None
    ) -> Dict:
        """Get detailed branch information.
        
        Args:
            repo_path: Repository path
            branch_name: Branch name (default: current)
            
        Returns:
            Dictionary with branch info
        """
        repo_path = repo_path or Path.cwd()
        branch_name = branch_name or self.get_current_branch(repo_path)
        
        try:
            # Get tracking branch
            result = subprocess.run(
                ['git', 'rev-parse', '--abbrev-ref', f'{branch_name}@{{u}}'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            tracking_branch = result.stdout.strip() if result.returncode == 0 else None
            
            # Get commit counts
            ahead, behind = 0, 0
            if tracking_branch:
                result = subprocess.run(
                    ['git', 'rev-list', '--left-right', '--count', f'{branch_name}...{tracking_branch}'],
                    cwd=repo_path,
                    capture_output=True,
                    text=True,
                    timeout=10
                )
                if result.returncode == 0:
                    ahead, behind = map(int, result.stdout.split())
            
            return {
                'name': branch_name,
                'tracking_branch': tracking_branch,
                'commits_ahead': ahead,
                'commits_behind': behind,
                'is_current': branch_name == self.get_current_branch(repo_path)
            }
            
        except Exception as e:
            logger.error(f"Error getting branch info: {e}")
            return {'name': branch_name, 'error': str(e)}
    
    def _branch_exists(self, repo_path: Path, branch_name: str) -> bool:
        """Check if branch exists."""
        try:
            result = subprocess.run(
                ['git', 'rev-parse', '--verify', branch_name],
                cwd=repo_path,
                capture_output=True,
                timeout=10
            )
            return result.returncode == 0
        except:
            return False
