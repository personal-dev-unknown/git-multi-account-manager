"""Integration tests for repository setup and configuration."""

import pytest
from pathlib import Path
from git_manager.core.account_manager import AccountManager
from git_manager.core.git_operations import GitOperations

class TestRepositorySetup:
    """Test repository setup and configuration."""
    
    @pytest.mark.integration
    def test_repository_configuration(self, temp_dir):
        """Test repository configuration with specific account."""
        # Setup
        account_manager = AccountManager(temp_dir)
        
        # Add test account
        from git_manager.models.account import Account, Platform
        account = Account(
            name="test-account",
            platform=Platform.GITHUB,
            username="testuser",
            email="test@example.com",
            ssh_key_path=str(temp_dir / "test_key")
        )
        account_manager.add_account(account)
        
        # Initialize Git operations
        git_ops = GitOperations(account_manager)
        
        # Create a test repository
        repo_path = temp_dir / "test-repo"
        repo_path.mkdir()
        
        # Initialize repository
        repo = git_ops.init_repository(repo_path, "test-account")
        
        # Verify repository configuration
        assert repo_path.exists()
        assert (repo_path / ".git").exists()
        
        # Verify Git config
        config = repo.config_reader()
        assert config.get_value("user", "name") == "testuser"
        assert config.get_value("user", "email") == "test@example.com"
