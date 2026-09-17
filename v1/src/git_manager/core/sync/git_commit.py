"""Git Commit - Commit operations."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, List, Dict
from dataclasses import dataclass

from ...utils.logger import get_logger


logger = get_logger(__name__)


@dataclass
class CommitInfo:
    """Commit information."""
    hash: str
    message: str
    author: str
    date: str
    files_changed: int = 0
    insertions: int = 0
    deletions: int = 0


class GitCommit:
    """Git commit operations."""
    
    def __init__(self):
        """Initialize Git Commit."""
        logger.info("GitCommit initialized")
    
    def commit(
        self,
        repo_path: Optional[Path] = None,
        message: str = None,
        allow_empty: bool = False
    ) -> Tuple[bool, str]:
        """
        Create a commit with staged changes.
        
        Args:
            repo_path: Repository path
            message: Commit message
            allow_empty: Allow empty commit
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not message:
            return False, "Commit message required"
        
        try:
            cmd = ['git', 'commit', '-m', message]
            if allow_empty:
                cmd.append('--allow-empty')
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Committed: {message}")
                return True, f"✓ Committed: {message}"
            else:
                error = result.stderr.strip()
                return False, f"Failed to commit: {error}"
                
        except Exception as e:
            logger.error(f"Commit error: {e}")
            return False, f"Error: {str(e)}"
    
    def commit_all(
        self,
        repo_path: Optional[Path] = None,
        message: str = None
    ) -> Tuple[bool, str]:
        """
        Stage all changes and commit (git commit -am).
        
        Args:
            repo_path: Repository path
            message: Commit message
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not message:
            return False, "Commit message required"
        
        try:
            result = subprocess.run(
                ['git', 'commit', '-am', message],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Committed all changes: {message}")
                return True, f"✓ Committed all changes: {message}"
            else:
                error = result.stderr.strip()
                return False, f"Failed to commit: {error}"
                
        except Exception as e:
            logger.error(f"Commit all error: {e}")
            return False, f"Error: {str(e)}"
    
    def amend(
        self,
        repo_path: Optional[Path] = None,
        message: Optional[str] = None,
        no_edit: bool = False
    ) -> Tuple[bool, str]:
        """
        Amend the last commit.
        
        Args:
            repo_path: Repository path
            message: New commit message (optional)
            no_edit: Keep existing message
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            cmd = ['git', 'commit', '--amend']
            if no_edit:
                cmd.append('--no-edit')
            elif message:
                cmd.extend(['-m', message])
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info("Amended last commit")
                return True, "✓ Amended last commit"
            else:
                error = result.stderr.strip()
                return False, f"Failed to amend: {error}"
                
        except Exception as e:
            logger.error(f"Amend error: {e}")
            return False, f"Error: {str(e)}"
    
    def get_log(
        self,
        repo_path: Optional[Path] = None,
        max_count: int = 10,
        oneline: bool = True
    ) -> List[str]:
        """
        Get commit log.
        
        Args:
            repo_path: Repository path
            max_count: Maximum number of commits
            oneline: Show one line per commit
            
        Returns:
            List of commit log lines
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            cmd = ['git', 'log']
            if oneline:
                cmd.append('--oneline')
            cmd.extend(['-n', str(max_count)])
            
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                return result.stdout.strip().split('\n') if result.stdout.strip() else []
            else:
                return []
                
        except Exception as e:
            logger.error(f"Get log error: {e}")
            return []
    
    def get_commit_info(
        self,
        repo_path: Optional[Path] = None,
        commit_hash: str = 'HEAD'
    ) -> Optional[CommitInfo]:
        """
        Get detailed information about a commit.
        
        Args:
            repo_path: Repository path
            commit_hash: Commit hash or reference
            
        Returns:
            CommitInfo object or None
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            # Get commit info
            result = subprocess.run(
                ['git', 'show', '--format=%H%n%s%n%an%n%ai', '-s', commit_hash],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode != 0:
                return None
            
            lines = result.stdout.strip().split('\n')
            if len(lines) < 4:
                return None
            
            commit_hash = lines[0]
            message = lines[1]
            author = lines[2]
            date = lines[3]
            
            # Get file stats
            result = subprocess.run(
                ['git', 'show', '--numstat', '--format=', commit_hash],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            files_changed = 0
            insertions = 0
            deletions = 0
            
            if result.returncode == 0:
                for line in result.stdout.strip().split('\n'):
                    if line:
                        parts = line.split('\t')
                        if len(parts) >= 3:
                            try:
                                insertions += int(parts[0])
                                deletions += int(parts[1])
                                files_changed += 1
                            except:
                                pass
            
            return CommitInfo(
                hash=commit_hash,
                message=message,
                author=author,
                date=date,
                files_changed=files_changed,
                insertions=insertions,
                deletions=deletions
            )
            
        except Exception as e:
            logger.error(f"Get commit info error: {e}")
            return None
    
    def revert(
        self,
        repo_path: Optional[Path] = None,
        commit_hash: str = None
    ) -> Tuple[bool, str]:
        """
        Revert a commit (create new commit that undoes it).
        
        Args:
            repo_path: Repository path
            commit_hash: Commit to revert
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not commit_hash:
            return False, "Commit hash required"
        
        try:
            result = subprocess.run(
                ['git', 'revert', '--no-edit', commit_hash],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Reverted commit: {commit_hash}")
                return True, f"✓ Reverted commit {commit_hash[:7]}"
            else:
                error = result.stderr.strip()
                return False, f"Failed to revert: {error}"
                
        except Exception as e:
            logger.error(f"Revert error: {e}")
            return False, f"Error: {str(e)}"
    
    def reset_to_commit(
        self,
        repo_path: Optional[Path] = None,
        commit_hash: str = None,
        mode: str = 'mixed'
    ) -> Tuple[bool, str]:
        """
        Reset to a specific commit.
        
        Args:
            repo_path: Repository path
            commit_hash: Commit to reset to
            mode: Reset mode ('soft', 'mixed', 'hard')
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not commit_hash:
            return False, "Commit hash required"
        
        if mode not in ['soft', 'mixed', 'hard']:
            return False, "Invalid reset mode"
        
        try:
            cmd = ['git', 'reset', f'--{mode}', commit_hash]
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.warning(f"Reset to commit: {commit_hash} ({mode})")
                return True, f"✓ Reset to {commit_hash[:7]} ({mode})"
            else:
                error = result.stderr.strip()
                return False, f"Failed to reset: {error}"
                
        except Exception as e:
            logger.error(f"Reset error: {e}")
            return False, f"Error: {str(e)}"
    
    def cherry_pick(
        self,
        repo_path: Optional[Path] = None,
        commit_hash: str = None
    ) -> Tuple[bool, str]:
        """
        Cherry-pick a commit.
        
        Args:
            repo_path: Repository path
            commit_hash: Commit to cherry-pick
            
        Returns:
            Tuple of (success, message)
        """
        repo_path = repo_path or Path.cwd()
        
        if not commit_hash:
            return False, "Commit hash required"
        
        try:
            result = subprocess.run(
                ['git', 'cherry-pick', commit_hash],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                logger.info(f"Cherry-picked commit: {commit_hash}")
                return True, f"✓ Cherry-picked {commit_hash[:7]}"
            else:
                error = result.stderr.strip()
                return False, f"Failed to cherry-pick: {error}"
                
        except Exception as e:
            logger.error(f"Cherry-pick error: {e}")
            return False, f"Error: {str(e)}"
    
    def get_uncommitted_count(
        self,
        repo_path: Optional[Path] = None
    ) -> int:
        """
        Get count of uncommitted changes.
        
        Args:
            repo_path: Repository path
            
        Returns:
            Number of uncommitted files
        """
        repo_path = repo_path or Path.cwd()
        
        try:
            result = subprocess.run(
                ['git', 'status', '--porcelain'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                files = result.stdout.strip().split('\n') if result.stdout.strip() else []
                return len(files)
            else:
                return 0
                
        except:
            return 0
