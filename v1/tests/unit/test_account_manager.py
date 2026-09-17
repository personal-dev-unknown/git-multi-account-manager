"""Unit tests for account manager."""

import pytest
from pathlib import Path
from git_manager.core.account_manager import AccountManager
from git_manager.models.account import Account, Platform

class TestAccountManager:
    """Test cases for AccountManager class."""
    
    def test_add_account(self, temp_dir):
        """Test adding a new account."""
        manager = AccountManager(temp_dir)
        account = Account(
            name="test",
            platform=Platform.GITHUB,
            username="testuser",
            email="test@example.com",
            ssh_key_path="~/.ssh/test"
        )
        
        manager.add_account(account)
        assert len(manager.list_accounts()) == 1
        assert manager.get_account("test") == account
