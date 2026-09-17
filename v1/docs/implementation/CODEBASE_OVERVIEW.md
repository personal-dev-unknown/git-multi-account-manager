# Git Multi-Account Manager - Codebase Overview

## Project Summary
A comprehensive Python application for managing multiple Git accounts (GitHub & GitLab) with SSH key management, repository operations, and three user interfaces: CLI, Web, and Desktop.

---

## Directory Structure

```
git-multi-account-manager/
├── src/git_manager/
│   ├── main.py                    # Entry point for all interfaces
│   ├── models/                    # Data models
│   ├── core/                      # Core business logic
│   ├── cli/                       # Command-line interface
│   ├── utils/                     # Utility functions
│   ├── desktop/                   # Desktop GUI (PyQt)
│   └── web/                       # Web application (Flask)
├── tests/                         # Test suites
├── scripts/                       # Build and release scripts
├── docs/                          # Documentation
└── config/                        # Configuration files
```

---

## Core Components

### 1. **Entry Point: `src/git_manager/main.py`**

**Purpose:** Main entry point supporting three application modes

**Key Functions:**
- `show_mode_selection()` - Interactive menu to select CLI, Desktop, or Web mode
- `main()` - Routes to appropriate interface based on arguments

**Features:**
- Monkeypatches `sys.argv` to ensure all elements are strings (Click parser compatibility)
- Detects mode from command-line flags (`--cli`, `--desktop`, `--web`)
- Falls back to interactive selection if no mode specified
- Handles imports and error management for each interface

---

## Models (`src/git_manager/models/`)

### **Account Model** (`account.py`)
Represents a Git account with platform-specific information.

**Classes:**
- `Platform` (Enum) - Git platform types: GITHUB, GITLAB
- `Account` (Dataclass) - Account representation
  - `name` - Unique account identifier
  - `platform` - Git platform (GitHub/GitLab)
  - `username` - Platform username
  - `ssh_key_path` - Path to SSH private key
  - `host` - SSH host alias (e.g., github.com-work)
  - `email` - User email (optional)
  - `description` - Account description
  - `pat_token` - Personal Access Token (optional)

**Methods:**
- `to_dict()` - Convert to dictionary for JSON serialization
- `from_dict()` - Create Account from dictionary

---

### **SSH Key Model** (`ssh_key.py`)
Represents SSH key pairs for authentication.

**Classes:**
- `SSHKeyType` (Enum) - Key types: ED25519, RSA, ECDSA
- `SSHKey` (Dataclass) - SSH key representation
  - `name` - Key identifier
  - `key_type` - Type of SSH key
  - `private_key_path` - Path to private key
  - `public_key_path` - Path to public key
  - `public_key` - Public key content
  - `email` - Associated email
  - `created_at` - Creation timestamp
  - `passphrase_protected` - Whether key has passphrase

---

### **Repository Model** (`repository.py`)
Represents Git repositories and their status.

**Classes:**
- `RepositoryStatus` (Dataclass) - Repository status information
  - `has_uncommitted_changes` - Uncommitted changes flag
  - `current_branch` - Current branch name
  - `commits_ahead/behind` - Sync status with remote
  - `uncommitted_files` - List of modified files
  - Properties: `is_clean`, `is_synced`

- `Repository` (Dataclass) - Repository representation
  - `name` - Repository name
  - `path` - Local repository path
  - `remote_url` - Remote repository URL
  - `account` - Associated Account
  - `branch` - Default branch (main)
  - `description` - Repository description
  - `last_updated` - Last update timestamp

---

## Core Business Logic (`src/git_manager/core/`)

### **Account Manager** (`account_manager.py`)
Manages Git accounts - CRUD operations and persistence.

**Class: `AccountManager`**

**Methods:**
- `__init(config_path)` - Initialize with config file path
- `add_account()` - Add new Git account with validation
- `get_account(name)` - Retrieve account by name
- `list_accounts(platform)` - List accounts with optional platform filter
- `remove_account(name)` - Delete account
- `update_account(name, **kwargs)` - Update account properties
- `_load_accounts()` - Load from JSON config and SSH config
- `_save_accounts()` - Persist accounts to JSON file

**Features:**
- Validates username, email, and SSH key paths
- Generates default SSH host aliases
- Loads from both JSON config and SSH config files
- Automatic config directory creation

---

### **SSH Manager** (`ssh_manager.py`)
Handles SSH key generation and management.

**Class: `SSHManager`**

**Methods:**
- `__init(ssh_dir)` - Initialize with SSH directory (defaults to ~/.ssh/gitmanager)
- `generate_key()` - Generate new SSH key pair
  - Supports ED25519, RSA, ECDSA
  - Optional passphrase protection
  - Configurable key size for RSA
- `add_to_agent()` - Add key to SSH agent
- `test_connection()` - Test SSH connection to platform
- `get_public_key()` - Extract public key from private key
- `list_keys()` - List all managed SSH keys

**Features:**
- Uses `ssh-keygen` for key generation
- Manages SSH config file for host aliases
- Supports SSH agent integration
- Connection testing with GitHub/GitLab

---

### **Git Operations** (`git_operations.py`)
Handles Git repository operations.

**Class: `GitOperations`**

**Methods:**
- `__init(account_manager)` - Initialize with AccountManager
- `clone()` - Clone repository with account-specific SSH
  - Converts HTTPS URLs to SSH
  - Supports branch selection
  - Custom destination paths
- `pull()` - Pull from remote with account verification
- `push()` - Push to remote with account verification
- `check_status()` - Get repository status
  - Uncommitted changes
  - Branch information
  - Sync status with remote
- `_convert_to_ssh_url()` - Convert HTTPS to SSH URL using account host

**Features:**
- Account-aware Git operations
- URL validation and conversion
- Subprocess-based Git command execution
- Comprehensive error handling

---

### **Database Manager** (`database_manager.py`)
SQLite database for persistent data storage.

**Class: `DatabaseManager`**

**Tables:**
- `platforms` - Git platform definitions
- `accounts` - User accounts with metadata
- `repositories` - Repository tracking
- `ssh_keys` - SSH key metadata
- `sync_history` - Repository sync history
- `activity_logs` - User activity tracking

**Methods:**
- `__init(db_path)` - Initialize database
- `get_connection()` - Context manager for DB connections
- `_initialize_database()` - Create schema
- Account CRUD operations
- Repository tracking
- SSH key metadata storage
- Activity logging

**Features:**
- SQLite with automatic schema creation
- Context manager for safe connections
- Foreign key relationships
- Timestamp tracking for all records

---

### **Config Manager** (`config_manager.py`)
Application configuration management.

**Class: `ConfigManager`**

**Methods:**
- `__init(config_path)` - Initialize with config file
- `get(key, default)` - Get config value (supports dot notation)
- `set(key, value)` - Set config value
- `update(updates)` - Update multiple values
- `delete(key)` - Delete config key
- `_load_config()` - Load from JSON file
- `_save_config()` - Persist to JSON file

**Features:**
- Dot notation for nested keys (e.g., "theme.colors.primary")
- Automatic file creation
- Type-safe value handling

---

### **SSH Config Parser** (`ssh_config_parser.py`)
Parses existing SSH configuration.

**Class: `SSHConfigParser`**

**Methods:**
- `parse_accounts()` - Extract accounts from SSH config
- `parse_host_entries()` - Parse Host blocks
- `_parse_identity_file()` - Extract SSH key paths
- `_parse_hostname()` - Extract hostname

**Features:**
- Reads ~/.ssh/config
- Extracts existing account configurations
- Bridges legacy SSH setup to new system

---

### **Repository Setup** (`repository_setup.py`)
Workflow for setting up repositories with specific accounts.

**Key Functions:**
- Repository account configuration
- Git config user.name/user.email setup
- SSH key binding to repository
- Verification of account access

---

## CLI Interface (`src/git_manager/cli/`)

### **Main App** (`app.py`)
Click-based CLI application with command groups.

**Features:**
- Rich console with custom theme
- Debug and JSON logging options
- Manager initialization (Account, SSH, Git, Database, Config)
- Command registration

**Commands:**
- `interactive` - Start interactive mode
- `status` - Check repository status
- `account` - Account management group
- `ssh` - SSH key management group
- `clone` - Clone repositories
- `repository` - Repository operations
- `config` - Configuration management
- `logs` - View application logs
- `theme` - Theme management

---

### **Commands** (`cli/commands/`)

#### **Account Commands** (`account.py`)
- `account list` - List all accounts with optional platform filter
- `account add` - Add new account with validation
- `account remove` - Delete account with confirmation
- `account show` - Display account details
- `account update` - Update account properties

#### **SSH Commands** (`ssh.py`)
- `ssh setup-account` - Complete SSH account setup workflow
- `ssh list` - List SSH keys
- `ssh generate` - Generate new SSH key
- `ssh test` - Test SSH connections
- `ssh add-to-agent` - Add key to SSH agent

#### **Clone Command** (`clone.py`)
- Clone repositories with account selection
- Support for personal repository selection
- Progress indication
- Branch selection

#### **Repository Commands** (`repository.py`)
- `repository status` - Check repository status
- `repository setup` - Setup repository for account
- `repository sync` - Sync with remote

#### **Config Commands** (`config.py`)
- Get/set configuration values
- Theme management
- Application settings

#### **Logs Commands** (`logs.py`)
- View application logs
- Filter by category/level
- Real-time log streaming

#### **Theme Commands** (`theme.py`)
- List available themes
- Set active theme
- Customize colors

---

### **UI Components** (`cli/ui/`)

#### **Interactive Mode** (`interactive.py`)
Main interactive CLI interface with menu system.

**Class: `InteractiveMode`**

**Features:**
- Menu-driven interface
- Account management menu
- SSH key management menu
- Repository operations menu
- Git operations menu
- Settings and configuration menu

**Methods:**
- `run()` - Start interactive mode
- Menu navigation and command execution
- Input validation and error handling

#### **Tables** (`tables.py`)
Rich table formatting for data display.

**Functions:**
- `display_accounts_table()` - Show accounts in table format
- `display_account_details()` - Show single account details
- `display_repository_status()` - Show repository status
- `display_ssh_keys_table()` - Show SSH keys

#### **Color Schemes** (`color_schemes.py`)
Color palette management.

**Features:**
- Multiple color schemes
- Theme customization
- Color constants for UI elements

#### **Theme Manager** (`theme_manager.py`)
Theme persistence and management.

**Class: `ThemeManager`**

**Methods:**
- Load/save theme preferences
- Apply theme to console
- Theme validation

#### **Prompts** (`prompts.py`)
User input prompts and confirmations.

**Functions:**
- `select_personal_repository()` - Select from personal repos
- `confirm_action()` - Yes/No confirmation
- `select_account()` - Account selection prompt

---

## Utilities (`src/git_manager/utils/`)

### **Validators** (`validators.py`)
Input validation functions.

**Functions:**
- `validate_username()` - Validate Git username format
- `validate_email()` - Validate email address
- `validate_ssh_key()` - Check SSH key file exists
- `validate_url()` - Validate repository URL

### **Config Paths** (`config_paths.py`)
XDG-compliant configuration paths.

**Functions:**
- `get_config_dir()` - Get ~/.config/git-manager
- `get_accounts_file()` - Get accounts.json path
- `get_ssh_keys_dir()` - Get SSH keys directory
- `get_ssh_config_file()` - Get SSH config path
- `ensure_config_dirs()` - Create necessary directories

### **Constants** (`constants.py`)
Application constants and enums.

**Constants:**
- `APP_NAME`, `APP_VERSION`
- `COLORS` - Color palette
- `GitOperation` - Git operation types
- `RepoStatus` - Repository status types
- `DEFAULT_BRANCH`, `DEFAULT_REMOTE`

### **Logger** (`logger.py`)
Logging configuration.

**Functions:**
- `get_logger()` - Get configured logger instance
- Log level configuration
- File and console handlers

### **Log Config** (`log_config.py`)
Advanced logging system with categories.

**Classes:**
- `LogLevel` - Log level enum
- `LogCategory` - Log category enum
- `ContextLogger` - Context-aware logging

**Functions:**
- `initialize_logging()` - Setup logging system
- `get_logger()` - Get category-specific logger

### **File Operations** (`file_operations.py`)
File I/O utilities.

**Functions:**
- `read_json()` - Read JSON file
- `write_json()` - Write JSON file
- `read_file()` - Read text file
- `write_file()` - Write text file

### **Git Helpers** (`git_helpers.py`)
Git-specific utility functions.

**Functions:**
- `get_git_config()` - Read git config
- `set_git_config()` - Set git config
- `get_current_branch()` - Get current branch
- `get_remote_url()` - Get remote URL

### **SSH Helpers** (`ssh_helpers.py`)
SSH-specific utility functions.

**Functions:**
- `get_ssh_fingerprint()` - Get key fingerprint
- `parse_ssh_key()` - Parse SSH key file
- `test_ssh_connection()` - Test SSH connection

---

## Exception Handling (`src/git_manager/core/exceptions.py`)

**Custom Exceptions:**
- `GitManagerError` - Base exception
- `AccountError` - Account-related errors
- `AccountNotFoundError` - Account not found
- `DuplicateAccountError` - Duplicate account name
- `SSHError` - SSH operation errors
- `GitError` - Git operation errors
- `RepositoryError` - Repository-related errors
- `ConfigError` - Configuration errors

---

## Testing (`tests/`)

### **Unit Tests**
- `test_account_manager.py` - Account CRUD operations
- `test_ssh_manager.py` - SSH key generation
- `test_git_operations.py` - Git operations
- `test_validators.py` - Input validation
- `test_ssh_parser.py` - SSH config parsing

### **Integration Tests**
- `test_ssh_flow.py` - Complete SSH setup workflow
- `test_clone_flow.py` - Repository cloning workflow
- `test_repository_setup.py` - Repository configuration

### **Test Configuration**
- `conftest.py` - Pytest fixtures and configuration
- `pytest.ini` - Pytest settings

---

## Configuration Files

### **pyproject.toml**
Project metadata and dependencies.

### **setup.py**
Package installation configuration.

### **requirements.txt**
Runtime dependencies:
- Click - CLI framework
- Rich - Terminal UI
- PyQt5 - Desktop GUI
- Flask - Web framework
- Requests - HTTP client

### **requirements-dev.txt**
Development dependencies:
- pytest - Testing framework
- pytest-cov - Coverage reporting

---

## Scripts (`scripts/`)

### **run_cli.py**
Run the CLI application directly.

### **build.py**
Build distribution packages.

### **release.py**
Release automation script.

### **hooks/hook-git_manager.py**
PyInstaller hook for packaging.

---

## Data Flow

### **Account Management Flow**
```
User Input → CLI Command → AccountManager → JSON Config
                                         → SSH Config Parser
                                         → Database
```

### **SSH Key Generation Flow**
```
User Input → SSH Setup Wizard → SSHManager → ssh-keygen
                                          → SSH Config
                                          → Database
                                          → SSH Agent
```

### **Repository Cloning Flow**
```
User Input → Clone Command → GitOperations → Account Lookup
                                          → URL Conversion
                                          → Git Clone
                                          → Repository Tracking
```

---

## Key Design Patterns

1. **Manager Pattern** - AccountManager, SSHManager, GitOperations, ConfigManager
2. **Dataclass Models** - Account, SSHKey, Repository for type safety
3. **Context Manager** - Database connection management
4. **Enum Types** - Platform, SSHKeyType, LogLevel for type safety
5. **Click Command Groups** - Hierarchical CLI command organization
6. **Rich Console** - Beautiful terminal UI with themes
7. **Singleton Pattern** - Logger instances
8. **Factory Pattern** - Account creation from dictionaries

---

## Configuration Locations

- **Accounts:** `~/.config/git-manager/accounts.json`
- **SSH Keys:** `~/.ssh/gitmanager/`
- **SSH Config:** `~/.ssh/config`
- **Database:** `~/.config/git-manager/gitmanager.db`
- **Application Config:** `~/.config/git-manager/config.json`
- **Logs:** `~/.config/git-manager/logs/`

---

## Environment Variables

- `GIT_MANAGER_DEBUG` - Enable debug logging
- `GIT_MANAGER_CONFIG_DIR` - Override config directory
- `GIT_MANAGER_SSH_DIR` - Override SSH directory

---

## Security Considerations

1. **SSH Key Storage** - Private keys stored in `~/.ssh/gitmanager/` with 600 permissions
2. **Passphrase Protection** - Optional passphrase for SSH keys
3. **PAT Token Storage** - Personal Access Tokens stored in database (encrypted recommended)
4. **SSH Agent** - Keys added to SSH agent for authentication
5. **Config Permissions** - Config files with restricted permissions

---

## Future Extensibility

1. **Desktop GUI** - PyQt5-based desktop application
2. **Web Interface** - Flask-based web application
3. **API Integration** - GitHub/GitLab API for repository listing
4. **Sync Workflows** - Automated repository synchronization
5. **Team Management** - Multi-user account sharing
6. **Webhook Support** - Repository event handling
7. **CI/CD Integration** - GitHub Actions/GitLab CI support
