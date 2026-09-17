# Core Modules Documentation

This document provides a comprehensive overview of the three main core modules in the git-multi-account-manager: **Clone**, **SSH**, and **Sync**.

---

## Table of Contents

- [Clone Module](#clone-module)
- [SSH Module](#ssh-module)
- [Sync Module](#sync-module)

---

## Clone Module

**Location:** `src/git_manager/core/clone/`

The Clone module handles repository cloning operations with support for multiple platforms (GitHub, GitLab, Bitbucket) and authentication methods (SSH, HTTPS PAT, HTTPS Password, Anonymous).

### Core Files

#### `__init__.py`
- **Purpose:** Module initialization and public API export
- **Exports:** CloneWorkflow, URLParser, ParsedURL, RepositoryCache, platform classes, authentication classes, and custom exceptions
- **Role:** Central entry point for clone functionality

#### `workflow.py`
- **Purpose:** Main orchestration of clone operations
- **Key Class:** `CloneWorkflow`
- **Responsibilities:**
  - Orchestrate clone operations across multiple platforms
  - Manage platform instances (GitHub, GitLab, Bitbucket, Custom)
  - Fetch personal repositories from platforms
  - Handle account-to-platform mapping
  - Manage repository caching
  - Integrate with database and account managers

#### `parsers.py`
- **Purpose:** URL parsing and normalization for Git repositories
- **Key Class:** `URLParser`, `ParsedURL`
- **Responsibilities:**
  - Parse SSH, HTTPS, and short Git URLs
  - Extract owner, repository, and host information
  - Convert between URL formats (SSH ↔ HTTPS)
  - Validate URL structure

#### `cache.py`
- **Purpose:** Caching mechanism for repository listings
- **Key Class:** `RepositoryCache`
- **Responsibilities:**
  - Cache repository listings with TTL (time-to-live)
  - Store and retrieve cached data by account/platform
  - Manage cache expiration
  - Reduce API calls to platforms

#### `errors.py`
- **Purpose:** Custom exception hierarchy for clone operations
- **Exception Classes:**
  - `CloneError` - Base exception
  - `AuthenticationError` - Auth failures
  - `PermissionError` - Access denied
  - `RepositoryNotFoundError` - Repo doesn't exist
  - `NetworkError` - Network connectivity issues
  - `DiskSpaceError` - Insufficient disk space
  - `InvalidURLError` - Malformed URLs
  - `PlatformError` - Unsupported platform
  - `SSHError` - SSH operation failures
  - `APIError` - API call failures

### Sub-Modules

#### `api/` Directory
- **clone_api.py** - Main clone API implementation
- **platform_api.py** - Platform-specific API interactions
- **repository_api.py** - Repository information retrieval
- **__init__.py** - API module exports

#### `auth/` Directory
- **ssh.py** - SSH authentication handler
- **https_pat.py** - HTTPS Personal Access Token authentication
- **https_password.py** - HTTPS username/password authentication
- **anonymous.py** - Anonymous (public) repository access
- **__init__.py** - Auth module exports

#### `platforms/` Directory
- **base.py** - Base platform class with common functionality
- **github.py** - GitHub-specific implementation
- **gitlab.py** - GitLab-specific implementation
- **bitbucket.py** - Bitbucket-specific implementation
- **custom.py** - Custom/self-hosted Git server support
- **__init__.py** - Platform module exports

#### `ui/` Directory
- **clone_workflow.py** - UI workflow for clone operations
- **clone_auth_handler.py** - Authentication UI handler
- **clone_repository_fetcher.py** - Repository fetching UI
- **clone_url_parser.py** - URL parsing UI utilities
- **clone_platform_config.py** - Platform configuration UI
- **git_pull.py** - Pull operation UI
- **git_push.py** - Push operation UI
- **git_sync.py** - Sync operation UI
- **ssh_manager.py** - SSH management UI
- **ssh.py** - SSH configuration UI

---

## SSH Module

**Location:** `src/git_manager/core/ssh/`

The SSH module provides comprehensive SSH key management, SSH agent control, and SSH configuration handling for multi-account Git operations.

### Core Files

#### `__init__.py`
- **Purpose:** Module initialization and public API export
- **Exports:** SSHKeyGenerator, SSHAgentManager, SSHConfigManager, SSHWorkflowOrchestrator, exception classes, and integration layer
- **Role:** Central entry point for SSH functionality

#### `key_generator.py`
- **Purpose:** SSH key generation and management
- **Key Classes:** `SSHKeyGenerator`, `KeyMetadata`
- **Responsibilities:**
  - Generate SSH keys (ED25519, RSA)
  - Support passphrase protection
  - Store key metadata (name, email, platform, account type)
  - Manage key fingerprints
  - Create automatic backups
  - Validate key permissions
  - Track key creation timestamps

#### `agent_manager.py`
- **Purpose:** SSH agent lifecycle and key management
- **Key Class:** `SSHAgentManager`
- **Responsibilities:**
  - Start/stop SSH agent
  - Detect running agent instances
  - Add/remove keys from agent
  - List loaded keys with details
  - Monitor agent status
  - Manage SSH environment variables
  - Handle automatic agent restart

#### `config_manager.py`
- **Purpose:** SSH configuration file management
- **Key Classes:** `SSHConfigManager`, `SSHHostEntry`
- **Responsibilities:**
  - Parse existing SSH config files
  - Create/update SSH host entries
  - Manage multi-account SSH configurations
  - Validate SSH config syntax
  - Create backups before modifications
  - Restore from backups
  - Support custom SSH settings per account

#### `orchestrator.py`
- **Purpose:** Complete SSH workflow orchestration
- **Key Class:** `SSHWorkflowOrchestrator`
- **Responsibilities:**
  - Coordinate key generation, agent management, and config management
  - Setup complete SSH account configuration
  - Test SSH connections
  - Convert URLs between formats
  - Handle error recovery
  - Provide unified SSH workflow API

#### `exceptions.py`
- **Purpose:** Custom exception hierarchy for SSH operations
- **Exception Classes:**
  - `SSHException` - Base exception
  - `SSHKeyGenerationError` - Key generation failures
  - `SSHKeyNotFoundError` - Key not found
  - `SSHKeyAlreadyExistsError` - Duplicate key
  - `SSHKeyPermissionError` - Permission issues
  - `SSHKeyValidationError` - Invalid key
  - `SSHAgentError` - Agent operation failures
  - `SSHAgentNotRunningError` - Agent not available
  - `SSHConfigError` - Config file issues
  - `SSHConfigParseError` - Config parsing failures
  - `SSHConnectionTestError` - Connection test failures
  - `SSHURLConversionError` - URL conversion failures
  - `SSHMetadataError` - Metadata issues
  - `SSHBackupError` - Backup operation failures
  - `SSHIntegrationError` - Integration issues
  - `SSHDatabaseError` - Database operation failures
  - `SSHExceptionHandler` - Exception handling utility

#### `integration.py`
- **Purpose:** System integration layer for SSH operations
- **Key Class:** `SSHIntegrationLayer`
- **Responsibilities:**
  - Integrate SSH operations with system components
  - Handle database persistence
  - Manage SSH metadata storage
  - Coordinate with account manager
  - Provide high-level SSH workflow API

---

## Sync Module

**Location:** `src/git_manager/core/sync/`

The Sync module provides comprehensive Git synchronization operations including push, pull, and bidirectional sync with branch management and safety checks.

### Core Files

#### `__init__.py`
- **Purpose:** Module initialization and public API export
- **Exports:** All managers, operations, and utility classes
- **Role:** Central entry point for sync functionality

#### `sync_workflow.py`
- **Purpose:** High-level orchestration of sync operations
- **Key Class:** `SyncWorkflow`
- **Responsibilities:**
  - Orchestrate push, pull, and sync operations
  - Manage branch selection
  - Coordinate with push/pull managers
  - Handle feature branch workflows
  - Provide unified sync API

#### `sync_operations.py`
- **Purpose:** Comprehensive sync strategies and operations
- **Key Classes:** `SyncOperations`, `SyncResult`
- **Responsibilities:**
  - Implement smart sync algorithm
  - Handle bidirectional synchronization
  - Manage merge strategies
  - Provide multiple sync strategies
  - Track sync results and details

#### `push_manager.py`
- **Purpose:** Push operation management with safety checks
- **Key Class:** `PushManager`
- **Responsibilities:**
  - Push to default branch
  - Push to specific branches
  - Handle upstream tracking
  - Manage force push options
  - Provide detailed push results

#### `push_operations.py`
- **Purpose:** Low-level push operation implementation
- **Key Class:** `PushOperations`
- **Responsibilities:**
  - Execute git push commands
  - Handle push conflicts
  - Manage push options (force, set-upstream)
  - Track pushed commits
  - Provide detailed operation results

#### `pull_manager.py`
- **Purpose:** Pull operation management with safety checks
- **Key Class:** `PullManager`
- **Responsibilities:**
  - Safe pull with conflict handling
  - Automatic stashing of uncommitted changes
  - Branch-aware pulling
  - Handle merge conflicts
  - Provide detailed pull results

#### `pull_operations.py`
- **Purpose:** Low-level pull operation implementation
- **Key Class:** `PullOperations`
- **Responsibilities:**
  - Execute git pull commands
  - Handle merge conflicts
  - Manage rebase options
  - Track pulled commits
  - Provide detailed operation results

#### `branch_manager.py`
- **Purpose:** Branch management and selection
- **Key Classes:** `BranchManager`, `Branch`
- **Responsibilities:**
  - List local and remote branches
  - Get current branch
  - Create new branches
  - Delete branches
  - Track branch relationships
  - Provide branch information (ahead/behind commits)

#### `git_status.py`
- **Purpose:** Repository status checking
- **Key Classes:** `GitStatus`, `StatusInfo`
- **Responsibilities:**
  - Check repository status
  - Detect uncommitted changes
  - Track branch divergence (ahead/behind)
  - Identify untracked files
  - Provide comprehensive status information

#### `git_stage.py`
- **Purpose:** Staging and stashing operations
- **Key Class:** `GitStage`
- **Responsibilities:**
  - Stage all changes (git add .)
  - Stage specific files
  - Stash changes
  - Apply stashed changes
  - List stash entries
  - Manage staging area

#### `git_commit.py`
- **Purpose:** Commit operations
- **Key Class:** `GitCommit`
- **Responsibilities:**
  - Create commits with messages
  - Amend commits
  - View commit history
  - Manage commit metadata
  - Handle commit signing

#### `git_branch.py`
- **Purpose:** Branch-specific operations
- **Key Class:** `GitBranch`
- **Responsibilities:**
  - Create branches
  - Delete branches
  - Switch branches
  - Rename branches
  - Track branch relationships
  - Manage branch metadata

#### `git_ssh_helper.py`
- **Purpose:** SSH integration for Git operations
- **Key Class:** `GitSSHHelper`
- **Responsibilities:**
  - Configure SSH for Git operations
  - Manage SSH key selection
  - Handle SSH authentication
  - Provide SSH environment setup
  - Integrate with SSH module

---

## Module Relationships

```
┌─────────────────────────────────────────────────────────────┐
│                    Git Manager Core                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │    Clone     │  │     SSH      │  │     Sync     │     │
│  │   Module     │  │   Module     │  │   Module     │     │
│  ├──────────────┤  ├──────────────┤  ├──────────────┤     │
│  │ • Workflow   │  │ • Orchestr.  │  │ • Workflow   │     │
│  │ • Parsers    │  │ • Key Gen.   │  │ • Push/Pull  │     │
│  │ • Cache      │  │ • Agent Mgr. │  │ • Sync Ops   │     │
│  │ • Auth       │  │ • Config Mgr.│  │ • Branch Mgr.│     │
│  │ • Platforms  │  │ • Integration│  │ • Status     │     │
│  │ • UI         │  │ • Exceptions │  │ • Stage      │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│         │                  │                  │             │
│         └──────────────────┼──────────────────┘             │
│                            │                                │
│                    ┌───────▼────────┐                       │
│                    │ Account Manager│                       │
│                    │ Database Mgr   │                       │
│                    └────────────────┘                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Usage Flow

### Clone Workflow
1. User initiates clone operation
2. `CloneWorkflow` parses repository URL using `URLParser`
3. Selects appropriate platform (GitHub, GitLab, etc.)
4. Uses correct authentication method (SSH, HTTPS PAT, etc.)
5. Checks cache to avoid redundant API calls
6. Clones repository to specified location

### SSH Setup Workflow
1. User initiates SSH account setup
2. `SSHWorkflowOrchestrator` coordinates the process
3. `SSHKeyGenerator` creates new SSH key pair
4. `SSHConfigManager` updates SSH config file
5. `SSHAgentManager` loads key into agent
6. Connection is tested and metadata is stored

### Sync Workflow
1. User initiates sync operation
2. `SyncWorkflow` checks repository status using `GitStatus`
3. `BranchManager` determines current and target branch
4. `PushManager`/`PullManager` execute push/pull operations
5. `GitStage` and `GitCommit` handle staging and commits
6. `SyncOperations` implements smart sync strategies
7. Results are returned with detailed information

---

## Key Design Patterns

- **Orchestrator Pattern:** Each module has an orchestrator class that coordinates sub-components
- **Manager Pattern:** Specialized managers handle specific operations (PushManager, PullManager, etc.)
- **Strategy Pattern:** Multiple sync strategies and authentication methods
- **Factory Pattern:** Platform and authentication method selection
- **Dataclass Pattern:** Structured data representation (ParsedURL, KeyMetadata, StatusInfo, etc.)
- **Exception Hierarchy:** Custom exceptions for detailed error handling

---

## Integration Points

- **Database Manager:** Stores and retrieves account, SSH key, and operation metadata
- **Account Manager:** Provides account information for platform operations
- **Config Manager:** Manages application-wide configuration
- **Logger:** Comprehensive logging across all modules
- **CLI:** Command-line interface for user interactions
- **Desktop UI:** Graphical interface for operations

