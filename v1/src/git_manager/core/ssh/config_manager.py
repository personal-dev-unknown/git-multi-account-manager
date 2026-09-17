# src/git_manager/core/ssh/config_manager.py
"""
SSH Config Management Module - Modular & Creative

Handles:
✅ SSH config file parsing and writing
✅ Host entry management
✅ Automatic config updates
✅ Config validation
✅ Backup and restore
✅ Multi-account support
"""

import re
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
import shutil
from datetime import datetime

from ...models.account import Account, Platform
from ...utils.logger import get_logger


logger = get_logger(__name__)


@dataclass
class SSHHostEntry:
    """SSH config host entry."""
    host: str
    hostname: str
    user: str
    identity_file: str
    identities_only: bool = True
    port: int = 22
    add_keys_to_agent: str = "yes"
    comment: Optional[str] = None


class SSHConfigManager:
    """
    Manages SSH configuration file.
    
    Features:
    - Parse existing SSH config
    - Add/update/remove host entries
    - Validate configuration
    - Backup and restore
    - Multi-account support
    """
    
    def __init__(self, ssh_dir: Optional[Path] = None):
        """
        Initialize SSH config manager.
        
        Args:
            ssh_dir: SSH directory (defaults to ~/.ssh)
        """
        self.ssh_dir = ssh_dir or Path.home() / ".ssh"
        self.main_config_file = self.ssh_dir / "config"
        
        # App's own config file (avoids permission issues)
        self.app_config_dir = self.ssh_dir / "gitmanager"
        self.config_file = self.app_config_dir / "config"
        self.backup_dir = self.app_config_dir / "backups"
        
        # Ensure directories exist
        self.ssh_dir.mkdir(parents=True, exist_ok=True)
        self.ssh_dir.chmod(0o700)
        self.app_config_dir.mkdir(parents=True, exist_ok=True)
        self.app_config_dir.chmod(0o700)
        self.backup_dir.mkdir(parents=True, exist_ok=True)
        
        # Create app config if it doesn't exist
        if not self.config_file.exists():
            self.config_file.touch()
            self.config_file.chmod(0o600)
        
        # Ensure main SSH config includes our app config
        self._ensure_include_in_main_config()
    
    def add_host_entry(
        self,
        host: str,
        hostname: str,
        identity_file: str,
        user: str = "git",
        comment: Optional[str] = None,
        backup: bool = True
    ) -> Tuple[bool, str]:
        """
        Add or update SSH host entry.
        
        Args:
            host: Host alias (e.g., github.com-devonionMoses)
            hostname: Actual hostname (e.g., github.com)
            identity_file: Path to SSH key
            user: SSH user (default: git)
            comment: Optional comment
            backup: Whether to backup config before modifying
            
        Returns:
            Tuple of (success, message)
        """
        try:
            # Backup if requested
            if backup:
                self._backup_config()
            
            # Read existing config
            config_text = self.config_file.read_text() if self.config_file.exists() else ""
            
            # Check if host already exists
            if self._host_exists(config_text, host):
                # Update existing entry
                return self._update_host_entry(host, hostname, identity_file, user, comment)
            else:
                # Add new entry
                return self._add_new_host_entry(host, hostname, identity_file, user, comment)
        
        except Exception as e:
            return False, f"Error adding host entry: {str(e)}"
    
    def _host_exists(self, config_text: str, host: str) -> bool:
        """Check if host entry exists in config."""
        pattern = rf"^Host\s+{re.escape(host)}\s*$"
        return bool(re.search(pattern, config_text, re.MULTILINE))
    
    def _add_new_host_entry(
        self,
        host: str,
        hostname: str,
        identity_file: str,
        user: str,
        comment: Optional[str]
    ) -> Tuple[bool, str]:
        """Add new host entry to config."""
        try:
            entry = self._build_host_entry(host, hostname, identity_file, user, comment)
            
            # Append to config
            with open(self.config_file, 'a') as f:
                f.write(entry)
            
            self.config_file.chmod(0o600)
            return True, f"Host entry added: {host}"
        
        except Exception as e:
            return False, f"Failed to add host entry: {str(e)}"
    
    def _update_host_entry(
        self,
        host: str,
        hostname: str,
        identity_file: str,
        user: str,
        comment: Optional[str]
    ) -> Tuple[bool, str]:
        """Update existing host entry in config."""
        try:
            config_text = self.config_file.read_text()
            
            # Pattern to match the host block
            pattern = rf"(Host\s+{re.escape(host)}\s*\n)((?:(?!^Host\s).*\n)*)"
            
            new_entry = self._build_host_entry(host, hostname, identity_file, user, comment)
            
            # Replace the host block
            updated_config = re.sub(
                pattern,
                new_entry,
                config_text,
                flags=re.MULTILINE
            )
            
            self.config_file.write_text(updated_config)
            self.config_file.chmod(0o600)
            
            return True, f"Host entry updated: {host}"
        
        except Exception as e:
            return False, f"Failed to update host entry: {str(e)}"
    
    def _build_host_entry(
        self,
        host: str,
        hostname: str,
        identity_file: str,
        user: str,
        comment: Optional[str]
    ) -> str:
        """Build SSH config host entry string."""
        lines = []
        
        if comment:
            lines.append(f"# {comment}")
        
        lines.append(f"Host {host}")
        lines.append(f"  HostName {hostname}")
        lines.append(f"  User {user}")
        lines.append(f"  IdentityFile {identity_file}")
        lines.append(f"  IdentitiesOnly yes")
        lines.append(f"  AddKeysToAgent yes")
        lines.append("")
        
        return "\n".join(lines)
    
    def remove_host_entry(self, host: str, backup: bool = True) -> Tuple[bool, str]:
        """
        Remove SSH host entry.
        
        Args:
            host: Host alias to remove
            backup: Whether to backup config before modifying
            
        Returns:
            Tuple of (success, message)
        """
        try:
            if backup:
                self._backup_config()
            
            config_text = self.config_file.read_text()
            
            # Pattern to match the host block
            pattern = rf"#.*\n?Host\s+{re.escape(host)}\s*\n(?:(?!^Host\s).*\n)*"
            
            updated_config = re.sub(pattern, "", config_text, flags=re.MULTILINE)
            
            self.config_file.write_text(updated_config)
            self.config_file.chmod(0o600)
            
            return True, f"Host entry removed: {host}"
        
        except Exception as e:
            return False, f"Failed to remove host entry: {str(e)}"
    
    def get_host_entry(self, host: str) -> Optional[SSHHostEntry]:
        """
        Get SSH host entry details.
        
        Args:
            host: Host alias
            
        Returns:
            SSHHostEntry or None if not found
        """
        try:
            config_text = self.config_file.read_text()
            
            # Pattern to match the host block
            pattern = rf"Host\s+{re.escape(host)}\s*\n((?:(?!^Host\s).*\n)*)"
            
            match = re.search(pattern, config_text, re.MULTILINE)
            if not match:
                return None
            
            # Parse the block
            block = match.group(1)
            entry_data = {
                "host": host,
                "hostname": None,
                "user": "git",
                "identity_file": None,
                "identities_only": True,
                "port": 22,
                "add_keys_to_agent": "yes"
            }
            
            for line in block.split('\n'):
                line = line.strip()
                if line.startswith("HostName"):
                    entry_data["hostname"] = line.split(None, 1)[1]
                elif line.startswith("User"):
                    entry_data["user"] = line.split(None, 1)[1]
                elif line.startswith("IdentityFile"):
                    entry_data["identity_file"] = line.split(None, 1)[1]
                elif line.startswith("Port"):
                    entry_data["port"] = int(line.split(None, 1)[1])
            
            return SSHHostEntry(**entry_data)
        
        except Exception:
            return None
    
    def list_host_entries(self) -> List[SSHHostEntry]:
        """
        List all SSH host entries.
        
        Returns:
            List of SSHHostEntry objects
        """
        try:
            config_text = self.config_file.read_text()
            
            # Find all Host entries
            pattern = r"Host\s+([^\s]+)\s*\n((?:(?!^Host\s).*\n)*)"
            
            entries = []
            for match in re.finditer(pattern, config_text, re.MULTILINE):
                host = match.group(1)
                entry = self.get_host_entry(host)
                if entry:
                    entries.append(entry)
            
            return entries
        
        except Exception:
            return []
    
    def validate_config(self) -> Tuple[bool, List[str]]:
        """
        Validate SSH config file.
        
        Returns:
            Tuple of (is_valid, list of errors)
        """
        errors = []
        
        try:
            # Check file exists
            if not self.config_file.exists():
                errors.append("SSH config file not found")
                return False, errors
            
            # Check permissions
            mode = self.config_file.stat().st_mode & 0o777
            if mode != 0o600:
                errors.append(f"SSH config has incorrect permissions: {oct(mode)} (should be 0o600)")
            
            # Try to parse config
            config_text = self.config_file.read_text()
            
            # Check for syntax errors (basic validation)
            for i, line in enumerate(config_text.split('\n'), 1):
                line = line.strip()
                if not line or line.startswith('#'):
                    continue
                
                # Check for valid keywords
                parts = line.split()
                if parts:
                    keyword = parts[0]
                    valid_keywords = [
                        'Host', 'HostName', 'User', 'IdentityFile',
                        'IdentitiesOnly', 'Port', 'AddKeysToAgent'
                    ]
                    if keyword not in valid_keywords:
                        errors.append(f"Line {i}: Unknown keyword '{keyword}'")
            
            return len(errors) == 0, errors
        
        except Exception as e:
            errors.append(f"Validation error: {str(e)}")
            return False, errors
    
    def _backup_config(self) -> Tuple[bool, str]:
        """Backup SSH config file."""
        try:
            if not self.config_file.exists():
                return True, "No config to backup"
            
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            backup_file = self.backup_dir / f"config.{timestamp}.backup"
            
            shutil.copy2(self.config_file, backup_file)
            backup_file.chmod(0o600)
            
            return True, f"Config backed up to {backup_file}"
        
        except Exception as e:
            return False, f"Backup failed: {str(e)}"
    
    def restore_config(self, backup_file: Path) -> Tuple[bool, str]:
        """
        Restore SSH config from backup.
        
        Args:
            backup_file: Path to backup file
            
        Returns:
            Tuple of (success, message)
        """
        try:
            if not backup_file.exists():
                return False, f"Backup file not found: {backup_file}"
            
            shutil.copy2(backup_file, self.config_file)
            self.config_file.chmod(0o600)
            
            return True, f"Config restored from {backup_file}"
        
        except Exception as e:
            return False, f"Restore failed: {str(e)}"
    
    def _ensure_include_in_main_config(self) -> None:
        """
        Ensure main SSH config includes our app config file.
        
        This avoids permission issues by:
        1. Writing to our own config file (~/.ssh/gitmanager/config)
        2. Adding an Include statement to main config (~/.ssh/config)
        3. User owns both files, no sudo needed
        """
        try:
            # Create main config if it doesn't exist
            if not self.main_config_file.exists():
                self.main_config_file.touch()
                self.main_config_file.chmod(0o600)
            
            include_line = f"Include ~/.ssh/gitmanager/config"
            
            # Read main config
            main_config_text = self.main_config_file.read_text() if self.main_config_file.exists() else ""
            
            # Check if include already exists
            if include_line in main_config_text:
                return  # Already included
            
            # Add include at the beginning if not present
            if main_config_text.strip():
                # Prepend to existing content
                updated_config = f"{include_line}\n\n{main_config_text}"
            else:
                # File is empty
                updated_config = f"{include_line}\n"
            
            # Write back to main config
            self.main_config_file.write_text(updated_config)
            self.main_config_file.chmod(0o600)
        
        except Exception as e:
            # Log but don't fail - include can be added manually
            pass


class SSHConfigParser:
    """Parses SSH config file to extract Git accounts.
    
    This class consolidates SSH config parsing functionality,
    replacing the separate ssh_config_parser.py module.
    """
    
    def __init__(self, ssh_config_path: Optional[Path] = None):
        """Initialize SSH Config Parser.
        
        Args:
            ssh_config_path: Path to SSH config file (default: ~/.ssh/config)
        """
        self.ssh_config_path = ssh_config_path or Path.home() / '.ssh' / 'config'
    
    def parse_accounts(self) -> List[Account]:
        """Parse SSH config and extract Git accounts.
        
        Returns:
            List of Account objects parsed from SSH config
        """
        accounts = []
        
        if not self.ssh_config_path.exists():
            logger.warning(f"SSH config not found at {self.ssh_config_path}")
            return accounts
        
        try:
            with open(self.ssh_config_path, 'r') as f:
                content = f.read()
            
            # Parse Host entries
            host_blocks = self._parse_host_blocks(content)
            
            for host_name, host_config in host_blocks.items():
                account = self._extract_account(host_name, host_config)
                if account:
                    accounts.append(account)
            
            logger.debug(f"Parsed {len(accounts)} accounts from SSH config")
            return accounts
        
        except Exception as e:
            logger.error(f"Failed to parse SSH config: {e}")
            return accounts
    
    def _parse_host_blocks(self, content: str) -> Dict[str, Dict[str, str]]:
        """Parse SSH config into Host blocks.
        
        Args:
            content: SSH config file content
            
        Returns:
            Dictionary mapping host names to their configurations
        """
        blocks = {}
        current_host = None
        current_config = {}
        
        for line in content.split('\n'):
            line = line.strip()
            
            # Skip empty lines and comments
            if not line or line.startswith('#'):
                continue
            
            # Check for Host directive
            if line.lower().startswith('host '):
                # Save previous host if exists
                if current_host and current_config:
                    blocks[current_host] = current_config
                
                # Start new host
                current_host = line.split(None, 1)[1] if len(line.split()) > 1 else None
                current_config = {}
            elif current_host and ' ' in line:
                # Parse config key-value pairs
                parts = line.split(None, 1)
                if len(parts) == 2:
                    key, value = parts
                    current_config[key.lower()] = value
        
        # Don't forget the last host
        if current_host and current_config:
            blocks[current_host] = current_config
        
        return blocks
    
    def _extract_account(self, host_name: str, host_config: Dict[str, str]) -> Optional[Account]:
        """Extract Account from Host block.
        
        Args:
            host_name: SSH Host name
            host_config: Host configuration dictionary
            
        Returns:
            Account object if valid Git account, None otherwise
        """
        # Only process GitHub and GitLab hosts
        hostname = host_config.get('hostname', '')
        
        if 'github.com' not in hostname and 'gitlab.com' not in hostname:
            return None
        
        # Determine platform
        if 'github.com' in hostname:
            platform = Platform.GITHUB
        else:
            platform = Platform.GITLAB
        
        # Extract username from host name (e.g., github.com-username -> username)
        username = self._extract_username(host_name)
        if not username:
            return None
        
        # Get SSH key path
        identity_file = host_config.get('identityfile', '')
        if not identity_file:
            return None
        
        # Expand ~ to home directory
        ssh_key_path = Path(identity_file.replace('~', str(Path.home())))
        
        # Verify SSH key exists
        if not ssh_key_path.exists():
            logger.warning(f"SSH key not found: {ssh_key_path}")
            return None
        
        # Create account
        account = Account(
            name=username,
            platform=platform,
            username=username,
            ssh_key_path=ssh_key_path,
            host=host_name,
            email=None,  # Email to be captured during SSH key generation
            description=f"{platform.value} account for {username}"
        )
        
        return account
    
    def _extract_username(self, host_name: str) -> Optional[str]:
        """Extract username from host name.
        
        Args:
            host_name: SSH Host name (e.g., github.com-username)
            
        Returns:
            Extracted username or None
        """
        # Pattern: github.com-username or gitlab.com-username
        match = re.search(r'(?:github|gitlab)\.com-(.+)', host_name)
        if match:
            return match.group(1)
        return None
