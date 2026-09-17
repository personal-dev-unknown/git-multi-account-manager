"""Integration tests for SSH key generation and management."""

import pytest
from pathlib import Path
from git_manager.core.ssh_manager import SSHManager

class TestSSHFlow:
    """Test the complete SSH key generation and management flow."""
    
    @pytest.mark.integration
    def test_ssh_key_generation(self, temp_dir):
        """Test generating and verifying SSH keys."""
        # Setup
        ssh_manager = SSHManager(temp_dir)
        
        # Generate key pair
        key = ssh_manager.generate_key("test@example.com", "test-key")
        
        # Verify files were created
        assert key.private_key_path.exists()
        assert key.public_key_path.exists()
        
        # Verify key content
        assert key.public_key.startswith("ssh-")
        assert "test@example.com" in key.public_key
        
        # Test key verification
        assert ssh_manager.verify_key(key.private_key_path) is True
