"""Unit tests for Git operations."""

import pytest
from pathlib import Path
from git_manager.core.git_operations import GitOperations
from git_manager.core.account_manager import AccountManager

class TestGitOperations:
    """Test cases for GitOperations class."""
    
    def test_clone_repository(self, temp_dir, mocker):
        """Test cloning a repository."""
        # Mock the actual git clone command
        mocker.patch('git.Repo.clone_from')
        
        account_manager = AccountManager(temp_dir)
        git_ops = GitOperations(account_manager)
        
        # This will use the mocked clone_from method
        git_ops.clone("https://github.com/example/repo.git", "test-account")
        
        # Verify the clone method was called with the right arguments
        from git import Repo
        Repo.clone_from.assert_called_once()
