"""Unit tests for SSH manager."""

import pytest
from pathlib import Path
from git_manager.core.ssh_manager import SSHManager

class TestSSHManager:
    """Test cases for SSHManager class."""
    
    def test_generate_key(self, temp_dir):
        """Test generating a new SSH key."""
        manager = SSHManager(temp_dir)
        key = manager.generate_key("test@example.com", "test-key")
        
        assert key.private_key_path.exists()
        assert key.public_key_path.exists()
        assert key.public_key.startswith("ssh-")
