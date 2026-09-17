"""Authentication handler for clone operations."""

import os
import subprocess
from typing import Dict, Optional, Tuple
from pathlib import Path
from .clone_url_parser import URLParser, ParsedURL


class AuthenticationHandler:
    """Handle authentication for clone operations."""
    
    def __init__(self):
        self.ssh_key_dir = Path.home() / '.ssh' / 'gitmanager'
    
    def clone_with_ssh(
        self,
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
    
    def clone_with_pat(
        self,
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
    
    def clone_with_password(
        self,
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
    
    def clone_anonymous(
        self,
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
    
    def test_ssh_connection(self, host: str, ssh_key_path: str) -> Tuple[bool, str]:
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
    
    def setup_post_clone(
        self,
        repo_path: str,
        account: Dict,
        upstream_url: Optional[str] = None
    ) -> Tuple[bool, str]:
        """
        Configure repository after clone.
        
        Args:
            repo_path: Path to cloned repository
            account: Account dictionary with email, username
            upstream_url: Optional upstream remote URL
        
        Returns:
            (success, message)
        """
        
        try:
            # Set git user config
            if account.get('email'):
                subprocess.run(
                    ['git', '-C', repo_path, 'config', 'user.email', account['email']],
                    capture_output=True,
                    timeout=10
                )
            
            if account.get('username'):
                subprocess.run(
                    ['git', '-C', repo_path, 'config', 'user.name', account['username']],
                    capture_output=True,
                    timeout=10
                )
            
            # Add upstream remote if provided
            if upstream_url:
                subprocess.run(
                    ['git', '-C', repo_path, 'remote', 'add', 'upstream', upstream_url],
                    capture_output=True,
                    timeout=10
                )
            
            return True, "Post-clone setup complete"
        
        except Exception as e:
            return False, f"Post-clone setup failed: {str(e)}"
