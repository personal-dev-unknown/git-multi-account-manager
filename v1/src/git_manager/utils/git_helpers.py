# src/git_manager/utils/git_helpers.py
"""Git helper functions."""

import subprocess
from pathlib import Path
from typing import Optional, List, Tuple


def run_git_command(
    args: List[str],
    cwd: Optional[Path] = None,
    check: bool = False
) -> subprocess.CompletedProcess:
    """Run git command.
    
    Args:
        args: Git command arguments
        cwd: Working directory
        check: Raise exception on non-zero exit
        
    Returns:
        CompletedProcess object
    """
    cmd = ['git'] + args
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        check=check
    )


def get_current_branch(repo_path: Optional[Path] = None) -> Optional[str]:
    """Get current Git branch.
    
    Args:
        repo_path: Repository path
        
    Returns:
        Branch name or None
    """
    result = run_git_command(['rev-parse', '--abbrev-ref', 'HEAD'], repo_path)
    if result.returncode == 0:
        return result.stdout.strip()
    return None


def get_remote_url(repo_path: Optional[Path] = None) -> Optional[str]:
    """Get remote URL.
    
    Args:
        repo_path: Repository path
        
    Returns:
        Remote URL or None
    """
    result = run_git_command(['remote', 'get-url', 'origin'], repo_path)
    if result.returncode == 0:
        return result.stdout.strip()
    return None


def is_git_repository(path: Path) -> bool:
    """Check if path is a Git repository.
    
    Args:
        path: Path to check
        
    Returns:
        True if Git repository
    """
    result = run_git_command(['rev-parse', '--git-dir'], path)
    return result.returncode == 0


def get_uncommitted_changes(repo_path: Optional[Path] = None) -> List[str]:
    """Get list of uncommitted changes.
    
    Args:
        repo_path: Repository path
        
    Returns:
        List of changed files
    """
    result = run_git_command(['status', '--porcelain'], repo_path)
    if result.returncode == 0 and result.stdout:
        return result.stdout.strip().split('\n')
    return []


def get_commit_info(
    repo_path: Optional[Path] = None
) -> Tuple[Optional[int], Optional[int]]:
    """Get commits ahead/behind remote.
    
    Args:
        repo_path: Repository path
        
    Returns:
        Tuple of (ahead, behind) or (None, None)
    """
    result = run_git_command(
        ['rev-list', '--left-right', '--count', 'HEAD...@{u}'],
        repo_path
    )
    if result.returncode == 0:
        ahead, behind = map(int, result.stdout.split())
        return ahead, behind
    return None, None