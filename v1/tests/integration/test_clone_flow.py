"""Integration tests for repository cloning flow."""

import pytest
from pathlib import Path
from git_manager.core.account_manager import AccountManager
from git_manager.core.ssh_manager import SSHManager
from git_manager.core.git_operations import GitOperations

class TestCloneFlow:
    """Test the complete repository cloning flow."""
    
    @pytest.mark.integration
    def test_complete_clone_flow(self, temp_dir):
        """Test the complete flow of creating an account and cloning a repository."""
        # Setup
        account_manager = AccountManager(temp_dir)
        ssh_manager = SSHManager(temp_dir)
        
        # Generate SSH key
        key = ssh_manager.generate_key("test@example.com", "test-key")
        
        # Add account
        from git_manager.models.account import Account, Platform
        account = Account(
            name="test-account",
            platform=Platform.GITHUB,
            username="testuser",
            email="test@example.com",
            ssh_key_path=str(key.private_key_path)
        )
        account_manager.add_account(account)
        
        # Initialize Git operations
        git_ops = GitOperations(account_manager)
        
        # Test cloning (using a small test repository)
        test_repo = "https://github.com/octocat/Hello-World.git"
        repo_path = temp_dir / "test-repo"
        
        # Clone repository
        repo = git_ops.clone(test_repo, "test-account", destination=repo_path)
        
        # Verify
        assert repo_path.exists()
        assert (repo_path / ".git").exists()
