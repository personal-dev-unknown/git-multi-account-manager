"""SSH authentication for clone operations."""

import os
import subprocess
from typing import Tuple
from ..parsers import URLParser


class SSHAuth:
    """SSH key-based authentication."""
    
    @staticmethod
    def clone(
        repo_url: str,
        ssh_key_path: str,
        destination: str,
        recursive: bool = False,
        shallow: bool = False
    ) -> Tuple[bool, str]:
        """
        Clone using SSH key.
        
        Args:
            repo_url: Repository URL (will be converted to SSH format)
            ssh_key_path: Path to SSH private key
            destination: Clone destination
            recursive: Clone submodules recursively
            shallow: Shallow clone (--depth=1)
        
        Returns:
            (success, message)
        """
        
        try:
            # Convert URL to SSH format if needed
            parsed = URLParser.parse(repo_url)
            ssh_url = parsed.to_ssh()
            
            # Verify SSH key exists
            if not os.path.exists(ssh_key_path):
                return False, f"SSH key not found: {ssh_key_path}"
            
            # Verify SSH key permissions
            key_stat = os.stat(ssh_key_path)
            if key_stat.st_mode & 0o077:
                # Fix permissions
                os.chmod(ssh_key_path, 0o600)
            
            # Build git command
            cmd = ['git', 'clone']
            
            if shallow:
                cmd.extend(['--depth', '1'])
            
            if recursive:
                cmd.append('--recursive')
            
            # Set up SSH command
            env = os.environ.copy()
            env['GIT_SSH_COMMAND'] = f'ssh -i {ssh_key_path} -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new'
            
            cmd.extend([ssh_url, destination])
            
            # Execute clone
            result = subprocess.run(
                cmd,
                env=env,
                capture_output=True,
                text=True,
                timeout=300
            )
            
            if result.returncode == 0:
                return True, "Clone successful"
            else:
                return False, result.stderr or "Clone failed"
        
        except subprocess.TimeoutExpired:
            return False, "Clone operation timed out"
        except Exception as e:
            return False, f"SSH clone failed: {str(e)}"
    
    @staticmethod
    def test_connection(host: str, ssh_key_path: str) -> Tuple[bool, str]:
        """
        Test SSH connection to a host.
        
        Args:
            host: SSH host (e.g., github.com)
            ssh_key_path: Path to SSH private key
        
        Returns:
            (success, message)
        """
        
        try:
            cmd = [
                'ssh',
                '-i', ssh_key_path,
                '-o', 'IdentitiesOnly=yes',
                '-o', 'StrictHostKeyChecking=accept-new',
                '-o', 'ConnectTimeout=5',
                f'git@{host}'
            ]
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            # SSH to git@host returns non-zero but with specific message
            if 'successfully authenticated' in result.stderr or 'You\'ve successfully authenticated' in result.stderr:
                return True, "SSH connection successful"
            elif result.returncode == 0:
                return True, "SSH connection successful"
            else:
                return False, result.stderr or "SSH connection failed"
        
        except subprocess.TimeoutExpired:
            return False, "SSH connection timed out"
        except Exception as e:
            return False, f"SSH test failed: {str(e)}"
