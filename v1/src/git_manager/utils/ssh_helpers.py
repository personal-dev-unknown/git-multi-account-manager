# src/git_manager/utils/ssh_helpers.py
"""SSH helper functions."""

import subprocess
from pathlib import Path
from typing import Optional, Tuple


def is_ssh_agent_running() -> bool:
    """Check if SSH agent is running.
    
    Returns:
        True if agent is running
    """
    try:
        result = subprocess.run(
            ['ssh-add', '-l'],
            capture_output=True,
            check=False
        )
        # Exit code 0 or 1 means agent is running
        return result.returncode in (0, 1)
    except FileNotFoundError:
        return False


def start_ssh_agent() -> bool:
    """Start SSH agent.
    
    Returns:
        True if successful
    """
    try:
        subprocess.run(['ssh-agent'], check=True, capture_output=True)
        return True
    except (FileNotFoundError, subprocess.CalledProcessError):
        return False


def get_ssh_key_fingerprint(key_path: Path) -> Optional[str]:
    """Get SSH key fingerprint.
    
    Args:
        key_path: Path to SSH key
        
    Returns:
        Key fingerprint or None
    """
    try:
        result = subprocess.run(
            ['ssh-keygen', '-lf', str(key_path)],
            capture_output=True,
            text=True,
            check=True
        )
        # Output format: "2048 SHA256:... user@host (RSA)"
        parts = result.stdout.split()
        if len(parts) >= 2:
            return parts[1]
        return None
    except (FileNotFoundError, subprocess.CalledProcessError):
        return None


def test_ssh_connection(
    host: str,
    key_path: Optional[Path] = None,
    timeout: int = 30
) -> Tuple[bool, str]:
    """Test SSH connection.
    
    Args:
        host: SSH host
        key_path: Optional SSH key path
        timeout: Timeout in seconds
        
    Returns:
        Tuple of (success, message)
    """
    cmd = ['ssh', '-T', '-o', 'ConnectTimeout=' + str(timeout)]
    if key_path:
        cmd.extend(['-i', str(key_path)])
    cmd.append(f'git@{host}')
    
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout
        )
        output = result.stdout + result.stderr
        success = 'successfully authenticated' in output.lower()
        return success, output
    except subprocess.TimeoutExpired:
        return False, f"Connection timeout after {timeout}s"
    except Exception as e:
        return False, str(e)


def get_public_key(private_key_path: Path) -> Optional[str]:
    """Get public key from private key.
    
    Args:
        private_key_path: Path to private key
        
    Returns:
        Public key or None
    """
    public_key_path = Path(f"{private_key_path}.pub")
    if public_key_path.exists():
        with open(public_key_path, 'r') as f:
            return f.read().strip()
    return None