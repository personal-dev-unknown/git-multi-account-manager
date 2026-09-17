# src/git_manager/core/account_manager.py
"""Account Manager - Handles Git account operations."""

from typing import List, Optional, Dict
from pathlib import Path
import json
from dataclasses import dataclass, asdict
from enum import Enum

from ..utils.logger import get_logger
from ..utils import validate_username, validate_email, validate_ssh_key
from ..utils.config_paths import get_accounts_file, ensure_config_dirs
from ..models.account import Account, Platform
from .exceptions import AccountError, AccountNotFoundError, DuplicateAccountError
from .ssh import SSHConfigParser
from .platform_config import PlatformManager


logger = get_logger(__name__)


class AccountManager:
    """Manages multiple Git accounts (GitHub, GitLab, Bitbucket, Azure DevOps, Self-Hosted, Cloud Storage, Local Path, SourceForge)."""
    
    def __init__(self, config_path: Optional[Path] = None):
        """Initialize Account Manager.
        
        Args:
            config_path: Path to accounts configuration file (uses XDG by default)
        """
        ensure_config_dirs()
        self.config_path = config_path or get_accounts_file()
        self.accounts: Dict[str, Account] = {}
        self._load_accounts()
        logger.info(f"AccountManager initialized with {len(self.accounts)} accounts")
    
    def add_account(
        self,
        name: str,
        platform: Platform,
        username: str,
        ssh_key_path: str,
        host: Optional[str] = None,
        email: Optional[str] = None,
        description: Optional[str] = None,
        pat_token: Optional[str] = None
    ) -> Account:
        """Add a new Git account.
        
        Args:
            name: Unique account identifier
            platform: Git platform (github, gitlab, bitbucket, azure_devops, self_hosted, cloud_storage, local_path, sourceforge)
            username: Platform username
            ssh_key_path: Path to SSH private key
            host: Custom SSH host (e.g., github.com-work)
            email: User email (optional, captured during SSH key generation)
            description: Account description
            pat_token: Personal Access Token for API access (optional)
            
        Returns:
            Created Account object
            
        Raises:
            DuplicateAccountError: If account name already exists
            AccountError: If validation fails
        """
        if name in self.accounts:
            raise DuplicateAccountError(f"Account '{name}' already exists")
        
        # Validate inputs
        if not validate_username(username):
            raise AccountError(f"Invalid username: {username}")
        if email and not validate_email(email):
            raise AccountError(f"Invalid email: {email}")
        if not validate_ssh_key(ssh_key_path):
            raise AccountError(f"SSH key not found: {ssh_key_path}")
        
        # Generate default host if not provided
        if not host:
            host = f"{platform.value}.com-{name}"
        
        account = Account(
            name=name,
            platform=platform,
            username=username,
            email=email,
            ssh_key_path=Path(ssh_key_path),
            host=host,
            description=description or f"{platform.value} account for {username}",
            pat_token=pat_token
        )
        
        self.accounts[name] = account
        self._save_accounts()
        logger.info(f"Added account: {name} ({platform.value})")
        
        return account
    
    def get_account(self, name: str) -> Account:
        """Get account by name.
        
        Args:
            name: Account name
            
        Returns:
            Account object
            
        Raises:
            AccountNotFoundError: If account doesn't exist
        """
        if name not in self.accounts:
            raise AccountNotFoundError(f"Account '{name}' not found")
        return self.accounts[name]
    
    def list_accounts(self, platform: Optional[Platform] = None) -> List[Account]:
        """List all accounts or filter by platform.
        
        Args:
            platform: Optional platform filter
            
        Returns:
            List of Account objects
        """
        accounts = list(self.accounts.values())
        if platform:
            accounts = [a for a in accounts if a.platform == platform]
        return sorted(accounts, key=lambda a: a.name)
    
    def remove_account(self, name: str) -> None:
        """Remove an account.
        
        Args:
            name: Account name
            
        Raises:
            AccountNotFoundError: If account doesn't exist
        """
        if name not in self.accounts:
            raise AccountNotFoundError(f"Account '{name}' not found")
        
        del self.accounts[name]
        self._save_accounts()
        logger.info(f"Removed account: {name}")
    
    def update_account(self, name: str, **kwargs) -> Account:
        """Update account properties.
        
        Args:
            name: Account name
            **kwargs: Properties to update
            
        Returns:
            Updated Account object
            
        Raises:
            AccountNotFoundError: If account doesn't exist
        """
        account = self.get_account(name)
        
        for key, value in kwargs.items():
            if hasattr(account, key):
                setattr(account, key, value)
        
        self._save_accounts()
        logger.info(f"Updated account: {name}")
        
        return account
    
    def _load_accounts(self) -> None:
        """Load accounts from configuration file and SSH config."""
        # First, try to load from JSON config
        if self.config_path.exists():
            try:
                with open(self.config_path, 'r') as f:
                    data = json.load(f)
                
                for platform_name, accounts_list in data.items():
                    try:
                        platform = Platform(platform_name)
                        for account_data in accounts_list:
                            account = Account.from_dict(account_data)
                            self.accounts[account.name] = account
                    except ValueError:
                        # Skip invalid platform names
                        logger.warning(f"Skipping invalid platform: {platform_name}")
                        continue
                
                logger.debug(f"Loaded {len(self.accounts)} accounts from {self.config_path}")
            except Exception as e:
                logger.error(f"Failed to load accounts from JSON: {e}")
        
        # Also load from SSH config if no accounts found
        if not self.accounts:
            try:
                ssh_parser = SSHConfigParser()
                ssh_accounts = ssh_parser.parse_accounts()
                for account in ssh_accounts:
                    self.accounts[account.name] = account
                
                if ssh_accounts:
                    logger.debug(f"Loaded {len(ssh_accounts)} accounts from SSH config")
                    # Save to JSON for future use
                    self._save_accounts()
            except Exception as e:
                logger.warning(f"Failed to load accounts from SSH config: {e}")
        
        # Ensure config directory exists
            self.config_path.parent.mkdir(parents=True, exist_ok=True)
    
    def _save_accounts(self) -> None:
        """Save accounts to configuration file."""
        data = {}
        for account in self.accounts.values():
            platform_key = account.platform.value
            if platform_key not in data:
                data[platform_key] = []
            data[platform_key].append(account.to_dict())
        
        try:
            self.config_path.parent.mkdir(parents=True, exist_ok=True)
            with open(self.config_path, 'w') as f:
                json.dump(data, f, indent=2)
            logger.debug(f"Saved {len(self.accounts)} accounts to {self.config_path}")
        except Exception as e:
            logger.error(f"Failed to save accounts: {e}")
            raise AccountError(f"Failed to save accounts: {e}")
