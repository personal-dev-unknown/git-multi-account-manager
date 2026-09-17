# src/git_manager/core/ssh/integration.py
"""
SSH System Integration Layer

Integrates SSH operations with:
- Database Manager (for persistence)
- Account Manager (for account data)
- Config Manager (for application settings)
"""

from pathlib import Path
from typing import Dict, Optional, Tuple, List, TYPE_CHECKING
from datetime import datetime

from ..database_manager import DatabaseManager
from ..config_manager import ConfigManager
from ...utils.logger import get_logger
from .exceptions import (
    SSHIntegrationError,
    SSHDatabaseError,
    SSHException
)

if TYPE_CHECKING:
    from ..account_manager import AccountManager

logger = get_logger(__name__)


class SSHIntegrationLayer:
    """
    Integrates SSH operations with Git Manager systems.
    
    Handles:
    - Saving SSH key metadata to database
    - Updating account information
    - Syncing with configuration
    - Error handling and recovery
    """
    
    def __init__(
        self,
        database_manager: DatabaseManager,
        account_manager: "AccountManager",
        config_manager: ConfigManager
    ):
        """
        Initialize integration layer.
        
        Args:
            database_manager: Database manager instance
            account_manager: Account manager instance
            config_manager: Config manager instance
        """
        self.db = database_manager
        self.account_manager = account_manager
        self.config = config_manager
        logger.info("SSH Integration Layer initialized")
    
    def save_ssh_key_metadata(
        self,
        account_id: int,
        key_info: Dict
    ) -> Tuple[bool, str]:
        """
        Save SSH key metadata to database.
        
        Args:
            account_id: Account ID
            key_info: Key information dict from key generator
            
        Returns:
            Tuple of (success, message)
        """
        try:
            with self.db.get_connection() as conn:
                cursor = conn.cursor()
                
                # Update account with SSH key information
                cursor.execute('''
                    UPDATE accounts
                    SET ssh_key_path = ?,
                        ssh_key_name = ?,
                        updated_at = ?
                    WHERE id = ?
                ''', (
                    key_info.get('key_path'),
                    key_info.get('name'),
                    datetime.now().isoformat(),
                    account_id
                ))
                
                # Insert SSH key metadata
                cursor.execute('''
                    INSERT INTO ssh_keys (
                        account_id,
                        key_name,
                        private_key_path,
                        public_key_path,
                        public_key,
                        fingerprint,
                        key_type,
                        has_passphrase,
                        email,
                        created_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ''', (
                    account_id,
                    key_info.get('name'),
                    key_info.get('key_path'),
                    key_info.get('public_key_path'),
                    key_info.get('public_key'),
                    key_info.get('fingerprint'),
                    key_info.get('metadata', {}).get('key_type', 'ed25519'),
                    key_info.get('metadata', {}).get('has_passphrase', False),
                    key_info.get('email'),
                    datetime.now().isoformat()
                ))
                
                conn.commit()
                logger.info(f"SSH key metadata saved for account {account_id}")
                return True, "SSH key metadata saved"
        
        except Exception as e:
            logger.error(f"Failed to save SSH key metadata: {str(e)}")
            return False, f"Database error: {str(e)}"
    
    def update_account_with_ssh_config(
        self,
        account_id: int,
        ssh_host_alias: str,
        platform: str
    ) -> Tuple[bool, str]:
        """
        Update account with SSH configuration details.
        
        Args:
            account_id: Account ID
            ssh_host_alias: SSH host alias (e.g., github.com-devonionMoses)
            platform: Platform name (github, gitlab, etc.)
            
        Returns:
            Tuple of (success, message)
        """
        try:
            with self.db.get_connection() as conn:
                cursor = conn.cursor()
                
                # Update account with SSH config
                cursor.execute('''
                    UPDATE accounts
                    SET ssh_host_alias = ?,
                        updated_at = ?
                    WHERE id = ?
                ''', (
                    ssh_host_alias,
                    datetime.now().isoformat(),
                    account_id
                ))
                
                conn.commit()
                logger.info(f"Account {account_id} updated with SSH config")
                return True, "Account SSH configuration updated"
        
        except Exception as e:
            logger.error(f"Failed to update account SSH config: {str(e)}")
            return False, f"Database error: {str(e)}"
    
    def get_account_ssh_info(self, account_id: int) -> Optional[Dict]:
        """
        Get SSH information for an account.
        
        Args:
            account_id: Account ID
            
        Returns:
            Dict with SSH info or None
        """
        try:
            with self.db.get_connection() as conn:
                cursor = conn.cursor()
                
                cursor.execute('''
                    SELECT
                        a.id,
                        a.username,
                        a.email,
                        a.ssh_key_path,
                        a.ssh_key_name,
                        a.ssh_host_alias,
                        k.fingerprint,
                        k.key_type,
                        k.has_passphrase
                    FROM accounts a
                    LEFT JOIN ssh_keys k ON a.id = k.account_id
                    WHERE a.id = ?
                ''', (account_id,))
                
                row = cursor.fetchone()
                if row:
                    return {
                        "account_id": row[0],
                        "username": row[1],
                        "email": row[2],
                        "ssh_key_path": row[3],
                        "ssh_key_name": row[4],
                        "ssh_host_alias": row[5],
                        "fingerprint": row[6],
                        "key_type": row[7],
                        "has_passphrase": row[8]
                    }
                
                return None
        
        except Exception as e:
            logger.error(f"Failed to get account SSH info: {str(e)}")
            return None
    
    def list_accounts_with_ssh(self, platform: str = None) -> List[Dict]:
        """
        List accounts with SSH information.
        
        Args:
            platform: Optional platform filter
            
        Returns:
            List of account dicts with SSH info
        """
        try:
            with self.db.get_connection() as conn:
                cursor = conn.cursor()
                
                if platform:
                    cursor.execute('''
                        SELECT
                            a.id,
                            a.username,
                            a.email,
                            a.platform_id,
                            a.ssh_key_path,
                            a.ssh_host_alias,
                            k.fingerprint
                        FROM accounts a
                        LEFT JOIN ssh_keys k ON a.id = k.account_id
                        WHERE a.platform_id = ?
                        ORDER BY a.username
                    ''', (platform,))
                else:
                    cursor.execute('''
                        SELECT
                            a.id,
                            a.username,
                            a.email,
                            a.platform_id,
                            a.ssh_key_path,
                            a.ssh_host_alias,
                            k.fingerprint
                        FROM accounts a
                        LEFT JOIN ssh_keys k ON a.id = k.account_id
                        ORDER BY a.platform_id, a.username
                    ''')
                
                accounts = []
                for row in cursor.fetchall():
                    accounts.append({
                        "id": row[0],
                        "username": row[1],
                        "email": row[2],
                        "platform": row[3],
                        "ssh_key_path": row[4],
                        "ssh_host_alias": row[5],
                        "fingerprint": row[6]
                    })
                
                return accounts
        
        except Exception as e:
            logger.error(f"Failed to list accounts with SSH: {str(e)}")
            return []
    
    def save_connection_test_result(
        self,
        account_id: int,
        success: bool,
        username: str = None,
        error_message: str = None
    ) -> Tuple[bool, str]:
        """
        Save SSH connection test result.
        
        Args:
            account_id: Account ID
            success: Whether test was successful
            username: Username from successful connection
            error_message: Error message if failed
            
        Returns:
            Tuple of (success, message)
        """
        try:
            with self.db.get_connection() as conn:
                cursor = conn.cursor()
                
                # Insert test result
                cursor.execute('''
                    INSERT INTO ssh_connection_tests (
                        account_id,
                        success,
                        username,
                        error_message,
                        tested_at
                    ) VALUES (?, ?, ?, ?, ?)
                ''', (
                    account_id,
                    success,
                    username,
                    error_message,
                    datetime.now().isoformat()
                ))
                
                # Update account last_tested
                cursor.execute('''
                    UPDATE accounts
                    SET last_ssh_test = ?,
                        last_ssh_test_success = ?
                    WHERE id = ?
                ''', (
                    datetime.now().isoformat(),
                    success,
                    account_id
                ))
                
                conn.commit()
                logger.info(f"Connection test result saved for account {account_id}")
                return True, "Test result saved"
        
        except Exception as e:
            logger.error(f"Failed to save connection test result: {str(e)}")
            return False, f"Database error: {str(e)}"
    
    def get_account_by_ssh_alias(self, ssh_alias: str) -> Optional[Dict]:
        """
        Get account by SSH host alias.
        
        Args:
            ssh_alias: SSH host alias (e.g., github.com-devonionMoses)
            
        Returns:
            Account dict or None
        """
        try:
            with self.db.get_connection() as conn:
                cursor = conn.cursor()
                
                cursor.execute('''
                    SELECT id, username, email, platform_id, ssh_key_path
                    FROM accounts
                    WHERE ssh_host_alias = ?
                ''', (ssh_alias,))
                
                row = cursor.fetchone()
                if row:
                    return {
                        "id": row[0],
                        "username": row[1],
                        "email": row[2],
                        "platform": row[3],
                        "ssh_key_path": row[4]
                    }
                
                return None
        
        except Exception as e:
            logger.error(f"Failed to get account by SSH alias: {str(e)}")
            return None
    
    def sync_ssh_config_to_db(self, ssh_config_entries: List[Dict]) -> Tuple[bool, str]:
        """
        Sync SSH config entries to database.
        
        Args:
            ssh_config_entries: List of SSH config entry dicts
            
        Returns:
            Tuple of (success, message)
        """
        try:
            with self.db.get_connection() as conn:
                cursor = conn.cursor()
                
                for entry in ssh_config_entries:
                    # Find account by username and platform
                    cursor.execute('''
                        SELECT id FROM accounts
                        WHERE username = ? AND platform_id = ?
                    ''', (entry.get('username'), entry.get('platform')))
                    
                    account = cursor.fetchone()
                    if account:
                        # Update with SSH config info
                        cursor.execute('''
                            UPDATE accounts
                            SET ssh_host_alias = ?,
                                ssh_key_path = ?,
                                updated_at = ?
                            WHERE id = ?
                        ''', (
                            entry.get('host_alias'),
                            entry.get('identity_file'),
                            datetime.now().isoformat(),
                            account[0]
                        ))
                
                conn.commit()
                logger.info(f"Synced {len(ssh_config_entries)} SSH config entries to database")
                return True, f"Synced {len(ssh_config_entries)} entries"
        
        except Exception as e:
            logger.error(f"Failed to sync SSH config to database: {str(e)}")
            return False, f"Database error: {str(e)}"


# Example usage
if __name__ == "__main__":
    from pathlib import Path
    
    # Initialize managers
    db = DatabaseManager()
    account_mgr = AccountManager(db)
    config_mgr = ConfigManager()
    
    # Create integration layer
    integration = SSHIntegrationLayer(db, account_mgr, config_mgr)
    
    # Save SSH key metadata
    key_info = {
        "name": "devonionMoses",
        "key_path": str(Path.home() / ".ssh" / "gitmanager" / "id_ed25519_devonionMoses"),
        "public_key": "ssh-ed25519 AAAAC3...",
        "fingerprint": "SHA256:xxxxx",
        "metadata": {
            "key_type": "ed25519",
            "has_passphrase": False
        }
    }
    
    success, msg = integration.save_ssh_key_metadata(1, key_info)
    print(f"Save metadata: {msg}")
    
    # List accounts with SSH info
    accounts = integration.list_accounts_with_ssh("github")
    for acc in accounts:
        print(f"  {acc['username']} - {acc['ssh_host_alias']}")
