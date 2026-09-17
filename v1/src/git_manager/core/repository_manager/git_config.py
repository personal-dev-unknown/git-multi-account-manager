"""Git Configuration - Manages Git settings for repositories."""

import subprocess
from pathlib import Path
from typing import Optional, Dict, Tuple

from ...utils.logger import get_logger
from ..exceptions import RepositoryConfigError


logger = get_logger(__name__)


class GitConfig:
    """Manages Git configuration for repositories."""
    
    @staticmethod
    def set_user_config(
        repo_path: Path,
        username: str,
        email: str
    ) -> Tuple[bool, str]:
        """Set user configuration for repository.
        
        Args:
            repo_path: Repository path
            username: Git username
            email: Git email
            
        Returns:
            Tuple of (success, message)
        """
        try:
            subprocess.run(
                ['git', 'config', 'user.name', username],
                cwd=repo_path,
                capture_output=True,
                check=True
            )
            
            subprocess.run(
                ['git', 'config', 'user.email', email],
                cwd=repo_path,
                capture_output=True,
                check=True
            )
            
            logger.info(f"Configured git user: {username} <{email}>")
            return True, "User configuration set successfully"
        except subprocess.CalledProcessError as e:
            error_msg = f"Failed to set user config: {e.stderr}"
            logger.error(error_msg)
            raise RepositoryConfigError(error_msg)
    
    @staticmethod
    def set_ssh_key(
        repo_path: Path,
        ssh_key_path: str
    ) -> Tuple[bool, str]:
        """Set SSH key for repository.
        
        Args:
            repo_path: Repository path
            ssh_key_path: Path to SSH key
            
        Returns:
            Tuple of (success, message)
        """
        try:
            subprocess.run(
                ['git', 'config', 'core.sshCommand',
                 f'ssh -i {ssh_key_path}'],
                cwd=repo_path,
                capture_output=True,
                check=True
            )
            
            logger.info(f"Configured SSH key: {ssh_key_path}")
            return True, "SSH key configured successfully"
        except subprocess.CalledProcessError as e:
            error_msg = f"Failed to set SSH key: {e.stderr}"
            logger.error(error_msg)
            raise RepositoryConfigError(error_msg)
    
    @staticmethod
    def get_config(repo_path: Path, key: str) -> Optional[str]:
        """Get Git configuration value.
        
        Args:
            repo_path: Repository path
            key: Configuration key
            
        Returns:
            Configuration value or None
        """
        try:
            result = subprocess.run(
                ['git', 'config', key],
                cwd=repo_path,
                capture_output=True,
                text=True,
                check=True
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError:
            return None
    
    @staticmethod
    def set_config(
        repo_path: Path,
        key: str,
        value: str
    ) -> Tuple[bool, str]:
        """Set Git configuration value.
        
        Args:
            repo_path: Repository path
            key: Configuration key
            value: Configuration value
            
        Returns:
            Tuple of (success, message)
        """
        try:
            subprocess.run(
                ['git', 'config', key, value],
                cwd=repo_path,
                capture_output=True,
                check=True
            )
            
            logger.info(f"Set git config: {key}={value}")
            return True, f"Configuration {key} set successfully"
        except subprocess.CalledProcessError as e:
            error_msg = f"Failed to set config {key}: {e.stderr}"
            logger.error(error_msg)
            raise RepositoryConfigError(error_msg)
