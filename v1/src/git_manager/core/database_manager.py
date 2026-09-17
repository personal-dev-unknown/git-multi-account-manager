"""SQLite Database Manager for Git Manager."""

import sqlite3
import json
from pathlib import Path
from typing import Optional, List, Dict, Any
from datetime import datetime, timedelta
from contextlib import contextmanager

from ..utils.logger import get_logger
from ..utils.config_paths import get_config_dir


logger = get_logger(__name__)


class DatabaseManager:
    """Manages SQLite database for Git Manager."""
    
    def __init__(self, db_path: Optional[Path] = None):
        """Initialize Database Manager.
        
        Args:
            db_path: Path to SQLite database file
        """
        if db_path is None:
            config_dir = get_config_dir()
            db_path = config_dir / "gitmanager.db"
        
        self.db_path = Path(db_path)
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        
        logger.info(f"Database initialized at: {self.db_path}")
        self._initialize_database()
    
    @contextmanager
    def get_connection(self):
        """Get database connection context manager."""
        conn = sqlite3.connect(str(self.db_path))
        conn.row_factory = sqlite3.Row
        try:
            yield conn
            conn.commit()
        except Exception as e:
            conn.rollback()
            logger.error(f"Database error: {str(e)}", exc_info=True)
            raise
        finally:
            conn.close()
    
    def _initialize_database(self):
        """Initialize database schema."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            
            # Platforms table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS platforms (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    platform_id TEXT UNIQUE NOT NULL,
                    name TEXT NOT NULL,
                    api_base_url TEXT,
                    ssh_host TEXT,
                    supports_password BOOLEAN DEFAULT FALSE,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            ''')
            
            # Accounts table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS accounts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    platform_id TEXT NOT NULL,
                    username TEXT NOT NULL,
                    email TEXT,
                    display_name TEXT,
                    ssh_key_path TEXT,
                    ssh_key_name TEXT,
                    ssh_host_alias TEXT,
                    pat_token TEXT,
                    pat_expires_at TIMESTAMP,
                    password_saved BOOLEAN DEFAULT FALSE,
                    avatar_url TEXT,
                    profile_url TEXT,
                    api_rate_limit INTEGER,
                    is_active BOOLEAN DEFAULT TRUE,
                    last_used TIMESTAMP,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (platform_id) REFERENCES platforms(platform_id),
                    UNIQUE(platform_id, username)
                )
            ''')
            
            # Repositories table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS repositories (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT UNIQUE NOT NULL,
                    platform_id TEXT NOT NULL,
                    account_id INTEGER,
                    owner TEXT NOT NULL,
                    name TEXT NOT NULL,
                    full_name TEXT NOT NULL,
                    clone_url TEXT NOT NULL,
                    ssh_url TEXT,
                    https_url TEXT,
                    web_url TEXT,
                    repository_type TEXT NOT NULL,
                    visibility TEXT NOT NULL,
                    access_level TEXT,
                    is_fork BOOLEAN DEFAULT FALSE,
                    clone_method TEXT NOT NULL,
                    clone_depth INTEGER,
                    cloned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    default_branch TEXT DEFAULT 'main',
                    description TEXT,
                    language TEXT,
                    size_kb INTEGER,
                    stars INTEGER DEFAULT 0,
                    last_commit_at TIMESTAMP,
                    clone_intent TEXT,
                    tags TEXT,
                    notes TEXT,
                    is_active BOOLEAN DEFAULT TRUE,
                    last_synced TIMESTAMP,
                    FOREIGN KEY (platform_id) REFERENCES platforms(platform_id),
                    FOREIGN KEY (account_id) REFERENCES accounts(id)
                )
            ''')
            
            # Remotes table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS remotes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    repository_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    url TEXT NOT NULL,
                    is_default BOOLEAN DEFAULT FALSE,
                    FOREIGN KEY (repository_id) REFERENCES repositories(id),
                    UNIQUE(repository_id, name)
                )
            ''')
            
            # Clone operations log table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS clone_operations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    repository_id INTEGER,
                    account_id INTEGER,
                    clone_url TEXT NOT NULL,
                    destination TEXT NOT NULL,
                    method TEXT NOT NULL,
                    platform_id TEXT NOT NULL,
                    options_json TEXT,
                    status TEXT NOT NULL,
                    error_message TEXT,
                    error_type TEXT,
                    duration_seconds INTEGER,
                    bytes_transferred INTEGER,
                    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    completed_at TIMESTAMP,
                    FOREIGN KEY (repository_id) REFERENCES repositories(id),
                    FOREIGN KEY (account_id) REFERENCES accounts(id),
                    FOREIGN KEY (platform_id) REFERENCES platforms(platform_id)
                )
            ''')
            
            # SSH Keys table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS ssh_keys (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    account_id INTEGER,
                    key_name TEXT NOT NULL,
                    key_type TEXT NOT NULL,
                    private_key_path TEXT NOT NULL,
                    public_key_path TEXT,
                    public_key TEXT,
                    fingerprint TEXT,
                    email TEXT,
                    comment TEXT,
                    is_active BOOLEAN DEFAULT TRUE,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (account_id) REFERENCES accounts(id)
                )
            ''')
            
            # Repository cache table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS repository_cache (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    account_id INTEGER NOT NULL,
                    platform_id TEXT NOT NULL,
                    repositories_json TEXT NOT NULL,
                    total_count INTEGER,
                    cached_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    expires_at TIMESTAMP,
                    FOREIGN KEY (account_id) REFERENCES accounts(id),
                    FOREIGN KEY (platform_id) REFERENCES platforms(platform_id),
                    UNIQUE(account_id, platform_id)
                )
            ''')
            
            # Insert default platforms
            self._insert_default_platforms(cursor)
            
            conn.commit()
            logger.info("Database schema initialized successfully")
    
    def _insert_default_platforms(self, cursor):
        """Insert default platforms."""
        platforms = [
            ('github', 'GitHub', 'https://api.github.com', 'github.com', False),
            ('gitlab', 'GitLab', 'https://gitlab.com/api/v4', 'gitlab.com', True),
            ('bitbucket', 'Bitbucket', 'https://api.bitbucket.org/2.0', 'bitbucket.org', False),
            ('custom', 'Custom', None, None, False),
        ]
        
        for platform_id, name, api_url, ssh_host, supports_pwd in platforms:
            cursor.execute('''
                INSERT OR IGNORE INTO platforms 
                (platform_id, name, api_base_url, ssh_host, supports_password)
                VALUES (?, ?, ?, ?, ?)
            ''', (platform_id, name, api_url, ssh_host, supports_pwd))
    
    # Account operations
    
    def add_account(self, platform_id: str, username: str, email: Optional[str] = None,
                   display_name: Optional[str] = None, ssh_key_path: Optional[str] = None,
                   ssh_key_name: Optional[str] = None, pat_token: Optional[str] = None) -> int:
        """Add account to database."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('''
                INSERT INTO accounts 
                (platform_id, username, email, display_name, ssh_key_path, ssh_key_name, pat_token)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            ''', (platform_id, username, email, display_name, ssh_key_path, ssh_key_name, pat_token))
            
            logger.info(f"Account added: {platform_id}/{username}")
            return cursor.lastrowid
    
    def get_account(self, account_id: int) -> Optional[Dict]:
        """Get account by ID."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('SELECT * FROM accounts WHERE id = ?', (account_id,))
            row = cursor.fetchone()
            return dict(row) if row else None
    
    def get_accounts_by_platform(self, platform_id: str) -> List[Dict]:
        """Get all accounts for a platform."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('SELECT * FROM accounts WHERE platform_id = ? AND is_active = TRUE', (platform_id,))
            return [dict(row) for row in cursor.fetchall()]
    
    def list_all_accounts(self) -> List[Dict]:
        """List all accounts."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('SELECT * FROM accounts WHERE is_active = TRUE ORDER BY platform_id, username')
            return [dict(row) for row in cursor.fetchall()]
    
    def update_account(self, account_id: int, **kwargs) -> bool:
        """Update account."""
        allowed_fields = ['email', 'display_name', 'ssh_key_path', 'pat_token', 'is_active', 'last_used']
        fields = {k: v for k, v in kwargs.items() if k in allowed_fields}
        
        if not fields:
            return False
        
        fields['last_used'] = datetime.now()
        
        with self.get_connection() as conn:
            cursor = conn.cursor()
            set_clause = ', '.join(f'{k} = ?' for k in fields.keys())
            values = list(fields.values()) + [account_id]
            
            cursor.execute(f'UPDATE accounts SET {set_clause} WHERE id = ?', values)
            logger.info(f"Account {account_id} updated")
            return cursor.rowcount > 0
    
    # Repository operations
    
    def add_repository(self, path: str, platform_id: str, account_id: Optional[int],
                      owner: str, name: str, full_name: str, clone_url: str,
                      repository_type: str, visibility: str, clone_method: str,
                      ssh_url: Optional[str] = None, https_url: Optional[str] = None,
                      web_url: Optional[str] = None, **kwargs) -> int:
        """Add repository to database."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('''
                INSERT INTO repositories 
                (path, platform_id, account_id, owner, name, full_name, clone_url,
                 ssh_url, https_url, web_url, repository_type, visibility, clone_method)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (path, platform_id, account_id, owner, name, full_name, clone_url,
                  ssh_url, https_url, web_url, repository_type, visibility, clone_method))
            
            repo_id = cursor.lastrowid
            logger.info(f"Repository added: {full_name} at {path}")
            return repo_id
    
    def get_repository(self, repo_id: int) -> Optional[Dict]:
        """Get repository by ID."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('SELECT * FROM repositories WHERE id = ?', (repo_id,))
            row = cursor.fetchone()
            return dict(row) if row else None
    
    def get_repository_by_path(self, path: str) -> Optional[Dict]:
        """Get repository by path."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('SELECT * FROM repositories WHERE path = ?', (path,))
            row = cursor.fetchone()
            return dict(row) if row else None
    
    def list_repositories(self, account_id: Optional[int] = None,
                         platform_id: Optional[str] = None) -> List[Dict]:
        """List repositories with optional filters."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            
            query = 'SELECT * FROM repositories WHERE is_active = TRUE'
            params = []
            
            if account_id:
                query += ' AND account_id = ?'
                params.append(account_id)
            
            if platform_id:
                query += ' AND platform_id = ?'
                params.append(platform_id)
            
            query += ' ORDER BY cloned_at DESC'
            
            cursor.execute(query, params)
            return [dict(row) for row in cursor.fetchall()]
    
    # Clone operations log
    
    def log_clone_operation(self, clone_url: str, destination: str, method: str,
                           platform_id: str, account_id: Optional[int] = None,
                           repository_id: Optional[int] = None, status: str = 'started',
                           options_json: Optional[str] = None) -> int:
        """Log clone operation."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('''
                INSERT INTO clone_operations 
                (clone_url, destination, method, platform_id, account_id, repository_id, status, options_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (clone_url, destination, method, platform_id, account_id, repository_id, status, options_json))
            
            return cursor.lastrowid
    
    def update_clone_operation(self, operation_id: int, status: str,
                              error_message: Optional[str] = None,
                              error_type: Optional[str] = None,
                              duration_seconds: Optional[int] = None,
                              bytes_transferred: Optional[int] = None) -> bool:
        """Update clone operation status."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('''
                UPDATE clone_operations 
                SET status = ?, error_message = ?, error_type = ?, 
                    duration_seconds = ?, bytes_transferred = ?, completed_at = CURRENT_TIMESTAMP
                WHERE id = ?
            ''', (status, error_message, error_type, duration_seconds, bytes_transferred, operation_id))
            
            return cursor.rowcount > 0
    
    def get_clone_operations(self, limit: int = 100) -> List[Dict]:
        """Get recent clone operations."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('''
                SELECT * FROM clone_operations 
                ORDER BY started_at DESC 
                LIMIT ?
            ''', (limit,))
            return [dict(row) for row in cursor.fetchall()]
    
    # Repository cache operations
    
    def get_repository_cache(self, account_id: int, platform_id: str) -> Optional[Dict]:
        """Get cached repositories."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('''
                SELECT * FROM repository_cache 
                WHERE account_id = ? AND platform_id = ? AND expires_at > CURRENT_TIMESTAMP
            ''', (account_id, platform_id))
            
            row = cursor.fetchone()
            if row:
                result = dict(row)
                result['repositories_json'] = json.loads(result['repositories_json'])
                return result
            return None
    
    def set_repository_cache(self, account_id: int, platform_id: str,
                            repositories: List[Dict], ttl_seconds: int = 300) -> bool:
        """Cache repositories."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            
            expires_at = datetime.now() + timedelta(seconds=ttl_seconds)
            repos_json = json.dumps(repositories)
            
            cursor.execute('''
                INSERT OR REPLACE INTO repository_cache 
                (account_id, platform_id, repositories_json, total_count, expires_at)
                VALUES (?, ?, ?, ?, ?)
            ''', (account_id, platform_id, repos_json, len(repositories), expires_at))
            
            return cursor.rowcount > 0
    
    def clear_repository_cache(self, account_id: Optional[int] = None,
                              platform_id: Optional[str] = None) -> bool:
        """Clear repository cache."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            
            if account_id and platform_id:
                cursor.execute('''
                    DELETE FROM repository_cache 
                    WHERE account_id = ? AND platform_id = ?
                ''', (account_id, platform_id))
            elif account_id:
                cursor.execute('DELETE FROM repository_cache WHERE account_id = ?', (account_id,))
            elif platform_id:
                cursor.execute('DELETE FROM repository_cache WHERE platform_id = ?', (platform_id,))
            else:
                cursor.execute('DELETE FROM repository_cache')
            
            return cursor.rowcount > 0
    
    # Remotes operations
    
    def add_remote(self, repository_id: int, name: str, url: str, is_default: bool = False) -> int:
        """Add remote to repository."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('''
                INSERT INTO remotes (repository_id, name, url, is_default)
                VALUES (?, ?, ?, ?)
            ''', (repository_id, name, url, is_default))
            
            return cursor.lastrowid
    
    def get_remotes(self, repository_id: int) -> List[Dict]:
        """Get remotes for repository."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('SELECT * FROM remotes WHERE repository_id = ?', (repository_id,))
            return [dict(row) for row in cursor.fetchall()]
    
    def get_default_remote(self, repository_id: int) -> Optional[Dict]:
        """Get default remote for repository."""
        with self.get_connection() as conn:
            cursor = conn.cursor()
            cursor.execute('''
                SELECT * FROM remotes WHERE repository_id = ? AND is_default = TRUE
            ''', (repository_id,))
            row = cursor.fetchone()
            return dict(row) if row else None
