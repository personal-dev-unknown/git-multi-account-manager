"""Git Status - Comprehensive repository status checking."""

import subprocess
from pathlib import Path
from typing import Optional, Dict, List
from dataclasses import dataclass

from ...utils.logger import get_logger
from .branch_manager import BranchManager


logger = get_logger(__name__)


@dataclass
class StatusInfo:
    """Repository status information."""
    branch: str
    remote: str
    local_ahead: int
    remote_ahead: int
    uncommitted_files: List[str]
    uncommitted_count: int
    is_clean: bool
    details: Dict = None
    
    def __post_init__(self):
        if self.details is None:
            self.details = {}


class GitStatus:
    """Comprehensive git status operations."""
    
    def __init__(self):
        """Initialize Git Status."""
        self.branch_manager = BranchManager()
        logger.info("GitStatus initialized")
    
    def get_status(
        self,
        repo_path: Optional[Path] = None,
        remote: str = 'origin'
    ) -> StatusInfo:
        """
        Get comprehensive repository status.
        
        Returns:
            StatusInfo with all relevant status information
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            # Get current branch
            current_branch = self.branch_manager.get_current_branch(repo_path)
            
            # Fetch to get latest remote info
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Get commit counts
            local_ahead = self._count_local_commits(repo_path, current_branch, remote)
            remote_ahead = self._count_remote_commits(repo_path, current_branch, remote)
            
            # Get uncommitted changes
            uncommitted_files = self._get_uncommitted_files(repo_path)
            uncommitted_count = len(uncommitted_files)
            is_clean = uncommitted_count == 0
            
            logger.info(f"Status: {current_branch} (local +{local_ahead}, remote +{remote_ahead})")
            
            return StatusInfo(
                branch=current_branch,
                remote=remote,
                local_ahead=local_ahead,
                remote_ahead=remote_ahead,
                uncommitted_files=uncommitted_files,
                uncommitted_count=uncommitted_count,
                is_clean=is_clean,
                details={
                    'status_summary': self._get_status_summary(
                        local_ahead, remote_ahead, uncommitted_count
                    )
                }
            )
            
        except Exception as e:
            logger.error(f"Get status error: {e}")
            return StatusInfo(
                branch='unknown',
                remote=remote,
                local_ahead=0,
                remote_ahead=0,
                uncommitted_files=[],
                uncommitted_count=0,
                is_clean=False,
                details={'error': str(e)}
            )
    
    def get_detailed_status(
        self,
        repo_path: Optional[Path] = None,
        remote: str = 'origin'
    ) -> Dict:
        """
        Get detailed status information including commit messages.
        
        Returns:
            Dict with comprehensive status details
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            current_branch = self.branch_manager.get_current_branch(repo_path)
            
            # Fetch first
            self._run_git_command(['git', 'fetch', remote], repo_path)
            
            # Get all information
            local_commits = self._get_local_commits(repo_path, current_branch, remote)
            remote_commits = self._get_remote_commits(repo_path, current_branch, remote)
            uncommitted_files = self._get_uncommitted_files(repo_path)
            
            return {
                'branch': current_branch,
                'remote': remote,
                'local_commits': local_commits,
                'remote_commits': remote_commits,
                'uncommitted_files': uncommitted_files,
                'local_count': len(local_commits),
                'remote_count': len(remote_commits),
                'uncommitted_count': len(uncommitted_files),
                'is_clean': len(uncommitted_files) == 0,
                'status': self._get_status_summary(
                    len(local_commits), len(remote_commits), len(uncommitted_files)
                )
            }
            
        except Exception as e:
            logger.error(f"Get detailed status error: {e}")
            return {
                'error': str(e),
                'branch': 'unknown',
                'remote': remote
            }
    
    def show_status(
        self,
        repo_path: Optional[Path] = None,
        remote: str = 'origin'
    ) -> str:
        """
        Get human-readable status output.
        
        Returns:
            Formatted status string
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            status = self.get_status(repo_path, remote)
            
            output = f"""
╔════════════════════════════════════════╗
║         Repository Status              ║
╚════════════════════════════════════════╝

Branch: {status.branch}
Remote: {status.remote}/{status.branch}

Commits:
  • Local ahead:  {status.local_ahead} commits
  • Remote ahead: {status.remote_ahead} commits

Working Directory:
  • Uncommitted changes: {status.uncommitted_count} files
  • Status: {'✓ Clean' if status.is_clean else '⚠️  Dirty'}

{self._get_status_recommendations(status)}
"""
            return output
            
        except Exception as e:
            logger.error(f"Show status error: {e}")
            return f"Error getting status: {str(e)}"
    
    # ==================== HELPER METHODS ====================
    
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
    
    def _get_local_commits(self, repo_path: Path, branch: str, remote: str) -> List[str]:
        """Get list of local commits ahead of remote."""
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
    
    def _get_remote_commits(self, repo_path: Path, branch: str, remote: str) -> List[str]:
        """Get list of remote commits ahead of local."""
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
    
    def _get_uncommitted_files(self, repo_path: Path) -> List[str]:
        """Get list of uncommitted files."""
        try:
            result = subprocess.run(
                ['git', 'status', '--porcelain'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.stdout.strip().split('\n') if result.stdout.strip() else []
        except:
            return []
    
    def _get_status_summary(self, local_ahead: int, remote_ahead: int, uncommitted: int) -> str:
        """Get human-readable status summary."""
        if local_ahead == 0 and remote_ahead == 0 and uncommitted == 0:
            return "✓ Everything is up to date"
        
        parts = []
        if local_ahead > 0:
            parts.append(f"📤 {local_ahead} commits to push")
        if remote_ahead > 0:
            parts.append(f"📥 {remote_ahead} commits to pull")
        if uncommitted > 0:
            parts.append(f"📝 {uncommitted} uncommitted files")
        
        return " | ".join(parts) if parts else "✓ Up to date"
    
    def _get_status_recommendations(self, status: StatusInfo) -> str:
        """Get recommendations based on status."""
        recommendations = []
        
        if status.local_ahead > 0 and status.remote_ahead == 0:
            recommendations.append("💡 Recommendation: Use 'Git push' to push your commits")
        elif status.remote_ahead > 0 and status.local_ahead == 0:
            recommendations.append("💡 Recommendation: Use 'Git pull' to get latest changes")
        elif status.local_ahead > 0 and status.remote_ahead > 0:
            recommendations.append("💡 Recommendation: Use 'Git sync' to handle diverged branches")
        
        if not status.is_clean:
            recommendations.append("💡 Recommendation: Commit or stash your changes first")
        
        if recommendations:
            return "Recommendations:\n" + "\n".join(f"  {r}" for r in recommendations)
        return ""
    
    def _run_git_command(self, args: List[str], cwd: Path) -> bool:
        """Run git command and return success status."""
        try:
            result = subprocess.run(
                args,
                cwd=cwd,
                capture_output=True,
                text=True,
                timeout=30
            )
            return result.returncode == 0
        except:
            return False
