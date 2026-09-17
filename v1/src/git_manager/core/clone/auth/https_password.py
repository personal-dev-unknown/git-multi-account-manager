"""HTTPS password authentication for clone operations."""

import subprocess
from typing import Tuple
from ..parsers import URLParser


class HTTPSPasswordAuth:
    """HTTPS username/password authentication (GitLab only)."""
    
    @staticmethod
    def clone(
        repo_url: str,
        username: str,
        destination: str,
        recursive: bool = False,
        shallow: bool = False
    ) -> Tuple[bool, str]:
        """
        Clone using username/password (GitLab only).
        
        Args:
            repo_url: Repository URL
            username: GitLab username
            destination: Clone destination
            recursive: Clone submodules recursively
            shallow: Shallow clone (--depth=1)
        
        Returns:
            (success, message)
        """
        
        try:
            # Parse URL
            parsed = URLParser.parse(repo_url)
            
            # Format URL with username
            url_with_user = f"https://{username}@{parsed.host}/{parsed.owner}/{parsed.repo}.git"
            
            # Build git command
            cmd = ['git', 'clone']
            
            if shallow:
                cmd.extend(['--depth', '1'])
            
            if recursive:
                cmd.append('--recursive')
            
            cmd.extend([url_with_user, destination])
            
            # Execute clone (git will prompt for password interactively)
            result = subprocess.run(
                cmd,
                timeout=300
            )
            
            if result.returncode == 0:
                return True, "Clone successful"
            else:
                return False, "Clone failed"
        
        except subprocess.TimeoutExpired:
            return False, "Clone operation timed out"
        except Exception as e:
            return False, f"Password clone failed: {str(e)}"
