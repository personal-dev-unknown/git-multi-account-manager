"""HTTPS PAT authentication for clone operations."""

import subprocess
from typing import Tuple
from ..parsers import URLParser


class HTTPSPATAuth:
    """HTTPS Personal Access Token authentication."""
    
    @staticmethod
    def clone(
        repo_url: str,
        pat_token: str,
        destination: str,
        platform: str = 'github',
        recursive: bool = False,
        shallow: bool = False
    ) -> Tuple[bool, str]:
        """
        Clone using Personal Access Token (HTTPS).
        
        Args:
            repo_url: Repository URL
            pat_token: Personal Access Token
            destination: Clone destination
            platform: Platform ID ('github', 'gitlab', etc.)
            recursive: Clone submodules recursively
            shallow: Shallow clone (--depth=1)
        
        Returns:
            (success, message)
        """
        
        try:
            # Parse URL
            parsed = URLParser.parse(repo_url)
            
            # Inject token into HTTPS URL
            if platform == 'gitlab':
                # GitLab uses oauth2 token
                authenticated_url = f"https://oauth2:{pat_token}@{parsed.host}/{parsed.owner}/{parsed.repo}.git"
            else:
                # GitHub and others use token as username
                authenticated_url = f"https://{pat_token}@{parsed.host}/{parsed.owner}/{parsed.repo}.git"
            
            # Build git command
            cmd = ['git', 'clone']
            
            if shallow:
                cmd.extend(['--depth', '1'])
            
            if recursive:
                cmd.append('--recursive')
            
            cmd.extend([authenticated_url, destination])
            
            # Execute clone
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=300
            )
            
            if result.returncode == 0:
                # Remove token from git config
                try:
                    subprocess.run(
                        ['git', '-C', destination, 'remote', 'set-url', 'origin',
                         f"https://{parsed.host}/{parsed.owner}/{parsed.repo}.git"],
                        capture_output=True,
                        timeout=10
                    )
                except Exception:
                    pass  # Ignore errors
                
                return True, "Clone successful"
            else:
                return False, result.stderr or "Clone failed"
        
        except subprocess.TimeoutExpired:
            return False, "Clone operation timed out"
        except Exception as e:
            return False, f"PAT clone failed: {str(e)}"
