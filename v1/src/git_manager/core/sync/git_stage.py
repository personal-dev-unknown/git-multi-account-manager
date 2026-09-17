"""Git Stage - Staging and stashing operations."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, List

from ...utils.logger import get_logger


logger = get_logger(__name__)


class GitStage:
    """Git staging and stashing operations."""
    
    def __init__(self):
        """Initialize Git Stage."""
        logger.info("GitStage initialized")
    
    # ==================== STAGING OPERATIONS ====================
    
    def add_all(
        self,
        repo_path: Optional[Path] = None
    ) -> Tuple[bool, str]:
        """
        Stage all changes (git add .).
        
        Args:
            repo_path: Repository path
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            result = subprocess.run(
                ['git', 'add', '.'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                # Get count of staged files
                count = self._count_staged_files(repo_path)
                logger.info(f"Staged {count} files")
                return True, f"✓ Staged {count} files"
            else:
                error = result.stderr.strip()
                return False, f"Failed to stage files: {error}"
                
        except Exception as e:
            logger.error(f"Add all error: {e}")
            return False, f"Error: {str(e)}"
    
    def add_file(
        self,
        repo_path: Optional[Path] = None,
        file_path: str = None
    ) -> Tuple[bool, str]:
        """
        Stage a specific file.
        
        Args:
            repo_path: Repository path
            file_path: File to stage
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not file_path:
            return False, "File path required"
        
        try:
            result = subprocess.run(
                ['git', 'add', file_path],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Staged file: {file_path}")
                return True, f"✓ Staged '{file_path}'"
            else:
                error = result.stderr.strip()
                return False, f"Failed to stage file: {error}"
                
        except Exception as e:
            logger.error(f"Add file error: {e}")
            return False, f"Error: {str(e)}"
    
    def add_interactive(
        self,
        repo_path: Optional[Path] = None
    ) -> Tuple[bool, str]:
        """
        Interactive staging (git add -i).
        
        Args:
            repo_path: Repository path
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            # Note: Interactive mode requires terminal interaction
            # This is a simplified version
            result = subprocess.run(
                ['git', 'add', '-i'],
                cwd=repo_path,
                timeout=60
            )
            
            if result.returncode == 0:
                logger.info("Interactive staging completed")
                return True, "✓ Interactive staging completed"
            else:
                return False, "Interactive staging cancelled"
                
        except Exception as e:
            logger.error(f"Interactive add error: {e}")
            return False, f"Error: {str(e)}"
    
    def reset_all(
        self,
        repo_path: Optional[Path] = None
    ) -> Tuple[bool, str]:
        """
        Unstage all changes (git reset).
        
        Args:
            repo_path: Repository path
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            result = subprocess.run(
                ['git', 'reset'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info("Unstaged all files")
                return True, "✓ Unstaged all files"
            else:
                error = result.stderr.strip()
                return False, f"Failed to unstage files: {error}"
                
        except Exception as e:
            logger.error(f"Reset all error: {e}")
            return False, f"Error: {str(e)}"
    
    def reset_file(
        self,
        repo_path: Optional[Path] = None,
        file_path: str = None
    ) -> Tuple[bool, str]:
        """
        Unstage a specific file.
        
        Args:
            repo_path: Repository path
            file_path: File to unstage
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not file_path:
            return False, "File path required"
        
        try:
            result = subprocess.run(
                ['git', 'reset', file_path],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Unstaged file: {file_path}")
                return True, f"✓ Unstaged '{file_path}'"
            else:
                error = result.stderr.strip()
                return False, f"Failed to unstage file: {error}"
                
        except Exception as e:
            logger.error(f"Reset file error: {e}")
            return False, f"Error: {str(e)}"
    
    # ==================== STASH OPERATIONS ====================
    
    def stash_save(
        self,
        repo_path: Optional[Path] = None,
        message: Optional[str] = None
    ) -> Tuple[bool, str]:
        """
        Stash changes (git stash save).
        
        Args:
            repo_path: Repository path
            message: Optional stash message
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            if message:
                cmd = ['git', 'stash', 'save', message]
            else:
                cmd = ['git', 'stash']
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Stashed changes: {message or 'no message'}")
                return True, f"✓ Stashed changes"
            else:
                error = result.stderr.strip()
                return False, f"Failed to stash: {error}"
                
        except Exception as e:
            logger.error(f"Stash save error: {e}")
            return False, f"Error: {str(e)}"
    
    def stash_pop(
        self,
        repo_path: Optional[Path] = None,
        stash_id: Optional[str] = None
    ) -> Tuple[bool, str]:
        """
        Pop stashed changes (git stash pop).
        
        Args:
            repo_path: Repository path
            stash_id: Specific stash to pop (e.g., 'stash@{0}')
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            if stash_id:
                cmd = ['git', 'stash', 'pop', stash_id]
            else:
                cmd = ['git', 'stash', 'pop']
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info("Popped stashed changes")
                return True, "✓ Popped stashed changes"
            else:
                error = result.stderr.strip()
                return False, f"Failed to pop stash: {error}"
                
        except Exception as e:
            logger.error(f"Stash pop error: {e}")
            return False, f"Error: {str(e)}"
    
    def stash_apply(
        self,
        repo_path: Optional[Path] = None,
        stash_id: Optional[str] = None
    ) -> Tuple[bool, str]:
        """
        Apply stashed changes without removing (git stash apply).
        
        Args:
            repo_path: Repository path
            stash_id: Specific stash to apply (e.g., 'stash@{0}')
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            if stash_id:
                cmd = ['git', 'stash', 'apply', stash_id]
            else:
                cmd = ['git', 'stash', 'apply']
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info("Applied stashed changes")
                return True, "✓ Applied stashed changes"
            else:
                error = result.stderr.strip()
                return False, f"Failed to apply stash: {error}"
                
        except Exception as e:
            logger.error(f"Stash apply error: {e}")
            return False, f"Error: {str(e)}"
    
    def stash_drop(
        self,
        repo_path: Optional[Path] = None,
        stash_id: Optional[str] = None
    ) -> Tuple[bool, str]:
        """
        Delete stashed changes (git stash drop).
        
        Args:
            repo_path: Repository path
            stash_id: Specific stash to drop (e.g., 'stash@{0}')
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            if stash_id:
                cmd = ['git', 'stash', 'drop', stash_id]
            else:
                cmd = ['git', 'stash', 'drop']
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info("Dropped stashed changes")
                return True, "✓ Dropped stashed changes"
            else:
                error = result.stderr.strip()
                return False, f"Failed to drop stash: {error}"
                
        except Exception as e:
            logger.error(f"Stash drop error: {e}")
            return False, f"Error: {str(e)}"
    
    def stash_list(
        self,
        repo_path: Optional[Path] = None
    ) -> Tuple[bool, List[str]]:
        """
        List all stashes (git stash list).
        
        Args:
            repo_path: Repository path
            
        Returns:
            Tuple of (success, list of stashes)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            result = subprocess.run(
                ['git', 'stash', 'list'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                stashes = result.stdout.strip().split('\n') if result.stdout.strip() else []
                logger.info(f"Listed {len(stashes)} stashes")
                return True, stashes
            else:
                return False, []
                
        except Exception as e:
            logger.error(f"Stash list error: {e}")
            return False, []
    
    # ==================== HELPER METHODS ====================
    
    def _count_staged_files(self, repo_path: Path) -> int:
        """Count staged files."""
        try:
            result = subprocess.run(
                ['git', 'diff', '--cached', '--name-only'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            files = result.stdout.strip().split('\n') if result.stdout.strip() else []
            return len(files)
        except:
            return 0
    
    def get_staged_files(
        self,
        repo_path: Optional[Path] = None
    ) -> List[str]:
        """
        Get list of staged files.
        
        Args:
            repo_path: Repository path
            
        Returns:
            List of staged files
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            result = subprocess.run(
                ['git', 'diff', '--cached', '--name-only'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.stdout.strip().split('\n') if result.stdout.strip() else []
        except:
            return []
    
    def get_unstaged_files(
        self,
        repo_path: Optional[Path] = None
    ) -> List[str]:
        """
        Get list of unstaged files.
        
        Args:
            repo_path: Repository path
            
        Returns:
            List of unstaged files
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            result = subprocess.run(
                ['git', 'diff', '--name-only'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.stdout.strip().split('\n') if result.stdout.strip() else []
        except:
            return []
