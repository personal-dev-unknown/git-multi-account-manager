"""Git SSH Helper - SSH-aware git command execution.

Integrates with existing SSH infrastructure:
- SSHManager: Key generation and management
- SSHConfigManager: SSH config file management
- ssh_helpers: Low-level SSH utilities
"""

import subprocess
import os
from pathlib import Path
from typing import Tuple, List, Optional

from ...utils.logger import get_logger
from ...utils.ssh_helpers import is_ssh_agent_running


logger = get_logger(__name__)


class GitSSHHelper:
    """Helper for executing git commands with SSH key support.
    
    This class integrates with the existing SSH infrastructure to provide
    SSH-aware git command execution. It reads SSH configuration from git
    config and uses the system SSH agent for authentication.
    """
    
    def __init__(self):
        """Initialize SSH helper."""
        logger.info("GitSSHHelper initialized")
    
    def run_git_command(
        self,
        args: List[str],
        cwd: Path,
        ssh_key_path: Optional[str] = None,
        timeout: int = 30
    ) -> Tuple[bool, str]:
        """
        Run git command with optional SSH key configuration.
        
        Integrates with system SSH agent and git config to provide
        seamless SSH authentication for git operations.
        
        Args:
            args: Git command arguments
            cwd: Working directory
            ssh_key_path: Optional SSH key path (extracted from git config)
            timeout: Command timeout in seconds
            
        Returns:
            Tuple of (success, output)
        """
        try:
            env = os.environ.copy()
            
            # Configure SSH if key path provided
            if ssh_key_path:
                ssh_key = Path(ssh_key_path).expanduser()
                if ssh_key.exists():
                    # Set GIT_SSH_COMMAND to use specific key
                    # StrictHostKeyChecking=accept-new: Accept new hosts but validate them
                    env['GIT_SSH_COMMAND'] = f'ssh -i {ssh_key} -o StrictHostKeyChecking=accept-new'
                    logger.debug(f"Using SSH key: {ssh_key}")
                else:
                    logger.warning(f"SSH key not found: {ssh_key}")
            
            # Ensure SSH agent is available (uses system SSH agent)
            if 'SSH_AUTH_SOCK' not in env and is_ssh_agent_running():
                logger.debug("SSH agent is running")
            
            result = subprocess.run(
                args,
                cwd=cwd,
                capture_output=True,
                text=True,
                timeout=timeout,
                env=env
            )
            
            output = result.stdout or result.stderr
            success = result.returncode == 0
            
            if not success:
                logger.warning(f"Git command failed: {' '.join(args)}")
                logger.warning(f"Error: {output}")
            
            return success, output.strip()
            
        except subprocess.TimeoutExpired:
            logger.error(f"Git command timed out: {' '.join(args)}")
            return False, "Command timed out"
        except Exception as e:
            logger.error(f"Git command error: {e}")
            return False, str(e)
    
    def get_account_ssh_key(self, repo_path: Path) -> Optional[str]:
        """
        Get SSH key path from git config for repository.
        
        Reads the core.sshCommand from git config and extracts the
        SSH key path. This is set during repository setup.
        
        Args:
            repo_path: Repository path
            
        Returns:
            SSH key path or None
        """
        try:
            result = subprocess.run(
                ['git', 'config', '--get', 'core.sshCommand'],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=5
            )
            
            if result.returncode == 0:
                ssh_command = result.stdout.strip()
                # Extract key path from: ssh -i /path/to/key ...
                if '-i' in ssh_command:
                    parts = ssh_command.split('-i')
                    if len(parts) > 1:
                        key_part = parts[1].strip().split()[0]
                        return key_part
            
            return None
            
        except Exception as e:
            logger.error(f"Error getting SSH key from config: {e}")
            return None
