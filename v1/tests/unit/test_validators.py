"""Unit tests for validators."""

import pytest
from git_manager.utils.validators import validate_username, validate_email, validate_ssh_key_path

class TestValidators:
    """Test cases for validation functions."""
    
    @pytest.mark.parametrize("username,expected", [
        ("testuser", True),
        ("test-user", True),
        ("test.user", True),
        ("test", True),
        ("", False),
        ("a" * 40, False),  # Too long
        ("test@user", False),  # Invalid character
    ])
    def test_validate_username(self, username, expected):
        """Test username validation."""
        assert validate_username(username) == expected
    
    @pytest.mark.parametrize("email,expected", [
        ("test@example.com", True),
        ("test.user@example.com", True),
        ("test+user@example.co.uk", True),
        ("test", False),
        ("test@", False),
        ("@example.com", False),
        ("test@.com", False),
    ])
    def test_validate_email(self, email, expected):
        """Test email validation."""
        assert validate_email(email) == expected
    
    def test_validate_ssh_key_path(self, tmp_path):
        """Test SSH key path validation."""
        test_file = tmp_path / "test_key"
        test_file.write_text("test key content")
        
        assert validate_ssh_key_path(str(test_file)) is True
        assert validate_ssh_key_path("/nonexistent/path") is False
