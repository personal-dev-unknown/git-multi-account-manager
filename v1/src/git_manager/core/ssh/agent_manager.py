# src/git_manager/core/ssh/agent_manager.py
"""
SSH Agent Management Module - Modular & Creative

Handles:
✅ SSH agent lifecycle management
✅ Key loading and unloading
✅ Agent status monitoring
✅ Key listing with details
✅ Automatic agent restart
✅ Environment variable management
"""

import subprocess
import os
from pathlib import Path
from typing import Tuple, List, Dict, Optional
import re


class SSHAgentManager:
    """
    Manages SSH agent for key management.
    
    Features:
    - Start/stop SSH agent
    - Add/remove keys
    - List loaded keys
    - Status monitoring
    - Environment management
    """
    
    def __init__(self):
        """Initialize SSH agent manager."""
        self.agent_pid = None
        self.agent_sock = None
        self._detect_running_agent()
    
    def _detect_running_agent(self):
        """Detect if SSH agent is already running."""
        try:
            # Check SSH_AGENT_PID environment variable
            if "SSH_AGENT_PID" in os.environ:
                self.agent_pid = os.environ["SSH_AGENT_PID"]
            
            # Check SSH_AUTH_SOCK environment variable
            if "SSH_AUTH_SOCK" in os.environ:
                self.agent_sock = os.environ["SSH_AUTH_SOCK"]
        
        except Exception:
            pass
    
    def is_running(self) -> bool:
        """Check if SSH agent is running."""
        try:
            result = subprocess.run(
                ["ssh-add", "-l"],
                capture_output=True,
                text=True,
                timeout=5
            )
            # ssh-add returns 0 if agent is running
            return result.returncode in [0, 1]  # 1 = no keys loaded, 0 = keys loaded
        
        except Exception:
            return False
    
    def start(self) -> Tuple[bool, str]:
        """
        Start SSH agent if not already running.
        
        Returns:
            Tuple of (success, message)
        """
        try:
            # Check if already running
            if self.is_running():
                return True, "SSH agent already running"
            
            # Start new agent
            result = subprocess.run(
                ["ssh-agent", "-s"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode != 0:
                return False, f"Failed to start SSH agent: {result.stderr}"
            
            # Parse output to get PID and socket
            output = result.stdout
            
            # Extract SSH_AGENT_PID
            pid_match = re.search(r"SSH_AGENT_PID=(\d+)", output)
            if pid_match:
                self.agent_pid = pid_match.group(1)
            
            # Extract SSH_AUTH_SOCK
            sock_match = re.search(r"SSH_AUTH_SOCK=([^;]+)", output)
            if sock_match:
                self.agent_sock = sock_match.group(1)
            
            # Set environment variables
            if self.agent_pid:
                os.environ["SSH_AGENT_PID"] = self.agent_pid
            if self.agent_sock:
                os.environ["SSH_AUTH_SOCK"] = self.agent_sock
            
            return True, f"SSH agent started (PID: {self.agent_pid})"
        
        except subprocess.TimeoutExpired:
            return False, "SSH agent startup timed out"
        except Exception as e:
            return False, f"Error starting SSH agent: {str(e)}"
    
    def stop(self) -> Tuple[bool, str]:
        """
        Stop SSH agent.
        
        Returns:
            Tuple of (success, message)
        """
        try:
            if not self.agent_pid:
                return False, "SSH agent PID not found"
            
            result = subprocess.run(
                ["kill", self.agent_pid],
                capture_output=True,
                text=True,
                timeout=5
            )
            
            if result.returncode == 0:
                self.agent_pid = None
                self.agent_sock = None
                return True, "SSH agent stopped"
            else:
                return False, f"Failed to stop SSH agent: {result.stderr}"
        
        except Exception as e:
            return False, f"Error stopping SSH agent: {str(e)}"
    
    def add_key(
        self,
        key_path: Path,
        passphrase: Optional[str] = None
    ) -> Tuple[bool, str]:
        """
        Add SSH key to agent.
        
        Args:
            key_path: Path to private key
            passphrase: Optional passphrase for the key
            
        Returns:
            Tuple of (success, message)
        """
        try:
            # Ensure agent is running
            if not self.is_running():
                success, msg = self.start()
                if not success:
                    return False, msg
            
            # Build command
            cmd = ["ssh-add", str(key_path)]
            
            # Add key
            result = subprocess.run(
                cmd,
                input=passphrase if passphrase else "",
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode == 0:
                return True, f"Key added to agent: {key_path.name}"
            else:
                error = result.stderr or result.stdout
                return False, f"Failed to add key: {error}"
        
        except subprocess.TimeoutExpired:
            return False, "Key addition timed out"
        except Exception as e:
            return False, f"Error adding key: {str(e)}"
    
    def remove_key(self, key_path: Path) -> Tuple[bool, str]:
        """
        Remove SSH key from agent.
        
        Args:
            key_path: Path to private key
            
        Returns:
            Tuple of (success, message)
        """
        try:
            result = subprocess.run(
                ["ssh-add", "-d", str(key_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                return True, f"Key removed from agent: {key_path.name}"
            else:
                return False, f"Failed to remove key: {result.stderr}"
        
        except Exception as e:
            return False, f"Error removing key: {str(e)}"
    
    def list_keys(self) -> Tuple[bool, List[Dict[str, str]]]:
        """
        List all keys loaded in agent.
        
        Returns:
            Tuple of (success, list of key info dicts)
        """
        try:
            result = subprocess.run(
                ["ssh-add", "-l"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            keys = []
            
            if result.returncode == 0:
                # Parse output
                for line in result.stdout.strip().split('\n'):
                    if line and not line.startswith("The agent"):
                        # Format: "256 SHA256:xxxxx comment (ED25519)"
                        parts = line.split()
                        if len(parts) >= 3:
                            keys.append({
                                "bits": parts[0],
                                "fingerprint": parts[1],
                                "comment": " ".join(parts[2:])
                            })
            
            elif result.returncode == 1:
                # No keys loaded (agent is running but empty)
                return True, []
            
            else:
                return False, []
            
            return True, keys
        
        except Exception as e:
            return False, []
    
    def clear_keys(self) -> Tuple[bool, str]:
        """
        Remove all keys from agent.
        
        Returns:
            Tuple of (success, message)
        """
        try:
            result = subprocess.run(
                ["ssh-add", "-D"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                return True, "All keys removed from agent"
            else:
                return False, f"Failed to clear keys: {result.stderr}"
        
        except Exception as e:
            return False, f"Error clearing keys: {str(e)}"
    
    def get_status(self) -> Dict[str, any]:
        """
        Get detailed SSH agent status.
        
        Returns:
            Dict with agent status information
        """
        status = {
            "running": self.is_running(),
            "pid": self.agent_pid,
            "socket": self.agent_sock,
            "keys_loaded": 0,
            "keys": []
        }
        
        try:
            success, keys = self.list_keys()
            if success:
                status["keys_loaded"] = len(keys)
                status["keys"] = keys
        
        except Exception:
            pass
        
        return status
    
    def restart(self) -> Tuple[bool, str]:
        """
        Restart SSH agent.
        
        Returns:
            Tuple of (success, message)
        """
        try:
            # Stop current agent
            if self.is_running():
                self.stop()
            
            # Start new agent
            return self.start()
        
        except Exception as e:
            return False, f"Error restarting SSH agent: {str(e)}"


# Example usage
if __name__ == "__main__":
    manager = SSHAgentManager()
    
    # Check status
    status = manager.get_status()
    print(f"Agent running: {status['running']}")
    print(f"Keys loaded: {status['keys_loaded']}")
    
    # Start if not running
    if not status['running']:
        success, msg = manager.start()
        print(f"Start: {msg}")
    
    # List keys
    success, keys = manager.list_keys()
    if success:
        for key in keys:
            print(f"  {key['fingerprint']} - {key['comment']}")
