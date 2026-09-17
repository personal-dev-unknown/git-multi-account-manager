"""Anonymous authentication for clone operations."""

import subprocess
from typing import Tuple
from ..parsers import URLParser


class AnonymousAuth:
    """Anonymous (no authentication) clone."""
    
    @staticmethod
    def clone(
        repo_url: str,
        destination: str,
        recursive: bool = False,
        shallow: bool = False
    ) -> Tuple[bool, str]:
        """
        Clone anonymously (no authentication).
        
        Args:
            repo_url: Repository URL
            destination: Clone destination
            recursive: Clone submodules recursively
            shallow: Shallow clone (--depth=1)
        
        Returns:
            (success, message)
        """
        
        try:
            # Parse and convert to HTTPS
            parsed = URLParser.parse(repo_url)
            https_url = parsed.to_https()
            
            # Build git command
            cmd = ['git', 'clone']
            
            if shallow:
                cmd.extend(['--depth', '1'])
            
            if recursive:
                cmd.append('--recursive')
            
            cmd.extend([https_url, destination])
            
            # Execute clone
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=300
            )
            
            if result.returncode == 0:
                return True, "Clone successful (read-only)"
            else:
                return False, result.stderr or "Clone failed"
        
        except subprocess.TimeoutExpired:
            return False, "Clone operation timed out"
        except Exception as e:
            return False, f"Anonymous clone failed: {str(e)}"
