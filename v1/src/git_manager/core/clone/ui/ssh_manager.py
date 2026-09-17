# src/git_manager/core/ssh_manager.py
"""SSH Manager - Handles SSH key generation and management."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple
import re
import stat

from ..utils.logger import get_logger
from ..models.ssh_key import SSHKey, SSHKeyType
from ..utils.config_paths import get_ssh_keys_dir, get_ssh_config_file
from .exceptions import SSHError


logger = get_logger(__name__)


class SSHManager:
    """Manages SSH keys and connections."""
    
    def __init__(self, ssh_dir: Optional[Path] = None):
        """Initialize SSH Manager.
        
        Args:
            ssh_dir: SSH directory path (defaults to ~/.ssh/gitmanager)
        """
        # Use gitmanager SSH directory by default
        self.ssh_dir = ssh_dir or get_ssh_keys_dir()
        self.ssh_config = get_ssh_config_file()
        self._ensure_ssh_dir()
        logger.info(f"SSHManager initialized with directory: {self.ssh_dir}")
    
    def generate_key(
        self,
        email: str,
        key_name: str,
        key_type: SSHKeyType = SSHKeyType.ED25519,
        passphrase: Optional[str] = None,
        bits: Optional[int] = None
    ) -> SSHKey:
        """Generate a new SSH key pair.
        
        Args:
            email: Email address for key comment
            key_name: Name for the key file
            key_type: Type of SSH key to generate
            passphrase: Optional passphrase for key
            bits: Key size in bits (for RSA)
            
        Returns:
            SSHKey object
            
        Raises:
            SSHError: If key generation fails
        """
        key_path = self.ssh_dir / f"id_{key_type.value}_{key_name}"
        
        if key_path.exists():
            raise SSHError(f"Key already exists: {key_path}")
        
        # Build ssh-keygen command
        cmd = [
            'ssh-keygen',
            '-t', key_type.value,
            '-C', email,
            '-f', str(key_path),
            '-N', passphrase or ''
        ]
        
        if key_type == SSHKeyType.RSA and bits:
            cmd.extend(['-b', str(bits)])
        
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                check=True
            )
            logger.info(f"Generated SSH key: {key_path}")
            
            # Read public key
            pub_key_path = Path(f"{key_path}.pub")
            with open(pub_key_path, 'r') as f:
                public_key = f.read().strip()
            
            ssh_key = SSHKey(
                name=key_name,
                key_type=key_type,
                private_key_path=key_path,
                public_key_path=pub_key_path,
                public_key=public_key,
                email=email
            )
            
            # Add to SSH agent
            self.add_to_agent(key_path, passphrase)
            
            return ssh_key
            
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to generate SSH key: {e.stderr}")
            raise SSHError(f"Failed to generate SSH key: {e.stderr}")
    
    def add_to_agent(self, key_path: Path, passphrase: Optional[str] = None) -> None:
        """Add SSH key to SSH agent.
        
        Args:
            key_path: Path to private key
            passphrase: Optional passphrase
            
        Raises:
            SSHError: If adding to agent fails
        """
        try:
            # Start SSH agent if not running
            subprocess.run(['ssh-agent'], check=False, capture_output=True)
            
            # Convert Path to string and ensure it's properly quoted for the shell
            key_path_str = str(key_path)
            
            # Add key
            if passphrase:
                # Use pexpect for passphrase input
                import pexpect
                cmd = ['ssh-add', key_path_str]
                child = pexpect.spawn(' '.join(f'"{part}"' if ' ' in part else part for part in cmd))
                try:
                    child.expect('Enter passphrase.*:', timeout=5)
                    child.sendline(passphrase)
                    child.expect(pexpect.EOF, timeout=5)
                    output = child.before.decode('utf-8')
                    exitstatus = child.exitstatus
                    if exitstatus != 0:
                        raise SSHError(f"Failed to add key to agent. Exit status: {exitstatus}. Output: {output}")
                except pexpect.EOF:
                    output = child.before.decode('utf-8')
                    if 'Bad passphrase' in output:
                        raise SSHError("Incorrect passphrase")
                    raise SSHError(f"Failed to add key to agent. Output: {output}")
                except pexpect.TIMEOUT:
                    child.terminate()
                    raise SSHError("Timed out waiting for passphrase prompt")
            else:
                # No passphrase, use subprocess directly
                result = subprocess.run(
                    ['ssh-add', key_path_str],
                    capture_output=True,
                    text=True
                )
                if result.returncode != 0:
                    error_msg = result.stderr.strip()
                    if 'No such file or directory' in error_msg:
                        raise SSHError(f"SSH key not found: {key_path}")
                    raise SSHError(f"Failed to add key to agent: {error_msg}")
            
            logger.info(f"Successfully added key to SSH agent: {key_path}")
            
        except FileNotFoundError as e:
            error_msg = f"SSH key not found: {key_path}"
            logger.error(error_msg)
            raise SSHError(error_msg) from e
        except Exception as e:
            error_msg = f"Failed to add key to agent: {str(e)}"
            logger.error(error_msg)
            raise SSHError(error_msg) from e
    
    def test_connection(
        self,
        host: str,
        key_path: Path,
        timeout: int = 30
    ) -> Tuple[bool, str]:
        """Test SSH connection to host.
        
        Args:
            host: SSH host to test
            key_path: Path to SSH key
            timeout: Connection timeout in seconds
            
        Returns:
            Tuple of (success, message)
        """
        cmd = [
            'ssh',
            '-T',
            '-i', str(key_path),
            '-o', 'ConnectTimeout=' + str(timeout),
            '-o', 'StrictHostKeyChecking=no',
            f'git@{host}'
        ]
        
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            
            # GitHub/GitLab return exit code 1 with success message
            output = result.stdout + result.stderr
            
            # Check for success indicators from GitHub and GitLab
            success_indicators = [
                'successfully authenticated',  # GitHub
                'welcome to gitlab',            # GitLab
                'hi ',                          # GitHub (Hi username!)
            ]
            
            if any(indicator in output.lower() for indicator in success_indicators):
                logger.info(f"SSH connection successful: {host}")
                return True, output
            else:
                logger.warning(f"SSH connection failed: {host}")
                return False, output
                
        except subprocess.TimeoutExpired:
            msg = f"Connection timed out after {timeout}s"
            logger.error(msg)
            return False, msg
        except Exception as e:
            msg = f"Connection test failed: {e}"
            logger.error(msg)
            return False, msg
    
    def add_to_config(
        self,
        host: str,
        hostname: str,
        identity_file: Path,
        user: str = 'git'
    ) -> None:
        """Add entry to SSH config file.
        
        Args:
            host: SSH host alias
            hostname: Actual hostname
            identity_file: Path to SSH key
            user: SSH user (default: git)
        """
        config_entry = f"""
        # Git account: {host}
        Host {host}
        HostName {hostname}
        User {user}
        IdentityFile {identity_file}
        IdentitiesOnly yes

        """
        
        # Check if entry already exists
        if self.ssh_config.exists():
            with open(self.ssh_config, 'r') as f:
                content = f.read()
                if f"Host {host}" in content:
                    logger.info(f"SSH config entry already exists: {host}")
                    return
        
        # Append to config
        with open(self.ssh_config, 'a') as f:
            f.write(config_entry)
        
        logger.info(f"Added SSH config entry: {host}")
    
    def _ensure_ssh_dir(self) -> None:
        """Ensure SSH directory exists with correct permissions."""
        self.ssh_dir.mkdir(parents=True, exist_ok=True)
        self.ssh_dir.chmod(0o700)