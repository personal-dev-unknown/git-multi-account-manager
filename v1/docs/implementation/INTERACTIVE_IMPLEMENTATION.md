# Interactive Features Implementation

This document describes the implementation of interactive CLI features across desktop and web interfaces.

## Overview

All interactive features from `src/git_manager/cli/ui/interactive.py` have been ported to:
- **Desktop**: PyQt6-based GUI with interactive widgets
- **Web**: Flask REST API with comprehensive endpoints

Both implementations use the core modules from `src/git_manager/core/` for consistency.

## Architecture

### Core Modules Used

All implementations leverage these core modules:

1. **AccountManager** - Manages Git accounts across platforms
2. **SSHWorkflowOrchestrator** - Handles SSH key generation and testing
3. **GitOperations** - Performs git operations (push, pull, sync, status)
4. **CloneWorkflow** - Manages repository cloning
5. **RepositoryManager** - Sets up new repositories
6. **DatabaseManager** - Persists data
7. **ConfigManager** - Manages configuration
8. **PlatformManager** - Provides platform information

### Implementation Files

#### Desktop Implementation

**Interactive Manager:**
- `src/git_manager/desktop/interactive_manager.py` - Core manager class that bridges CLI features to desktop

**Interactive Widgets:**
- `src/git_manager/desktop/widgets/interactive_accounts_widget.py` - Account management UI
- `src/git_manager/desktop/widgets/interactive_git_widget.py` - Git operations UI
- `src/git_manager/desktop/widgets/interactive_clone_widget.py` - Clone repository UI

#### Web Implementation

**Interactive Routes:**
- `src/git_manager/web/routes/interactive.py` - REST API endpoints for all interactive features

**Integration:**
- Updated `src/git_manager/web/app.py` to register interactive blueprint

## Features Implemented

### 1. Account Management

#### Desktop
- **InteractiveAccountsWidget**: Displays all accounts with platform info
- **Features**:
  - List all accounts with SSH/PAT status
  - Test SSH connections with retry logic
  - Generate SSH keys for any platform
  - Setup Personal Access Tokens (PAT)
  - Real-time account status updates

#### Web API
- `GET /api/v1/interactive/accounts` - List all accounts
- `POST /api/v1/interactive/accounts/<account_name>/test-ssh` - Test SSH connection
- `POST /api/v1/interactive/ssh/generate-key` - Generate SSH key
- `POST /api/v1/interactive/ssh/test-pat` - Test PAT token validity

### 2. Clone Operations

#### Desktop
- **InteractiveCloneWidget**: Two-tab interface for cloning
  - **External Repository Tab**:
    - Analyze repository URLs
    - Support for all 8 platforms
    - Multiple authentication methods (SSH, HTTPS with PAT)
    - Clone options (recursive, shallow)
    - Custom destination paths
  
  - **Personal Repository Tab**:
    - Platform selection
    - Account selection
    - Repository listing with pagination
    - Visibility indicators (public/private)
    - Same clone options as external

#### Web API
- `POST /api/v1/interactive/clone/analyze-url` - Analyze repository URL
- `GET /api/v1/interactive/clone/personal-repositories` - Fetch personal repositories
- `POST /api/v1/interactive/clone/repository` - Clone a repository

### 3. Git Operations

#### Desktop
- **InteractiveGitWidget**: Three-tab interface
  - **Status Tab**:
    - Check repository status
    - Display branch, remote URL, account info
    - Show uncommitted changes count
    - Display commits ahead/behind
  
  - **Operations Tab**:
    - Push changes
    - Pull changes
    - Sync (pull + push)
    - Real-time operation output
  
  - **Setup Tab**:
    - Setup new local repository
    - Account selection
    - Repository metadata (name, description)
    - Default branch configuration

#### Web API
- `GET /api/v1/interactive/git/status` - Check repository status
- `POST /api/v1/interactive/git/push` - Push changes
- `POST /api/v1/interactive/git/pull` - Pull changes
- `POST /api/v1/interactive/git/sync` - Sync repository
- `POST /api/v1/interactive/repository/setup` - Setup new repository

### 4. SSH Key Management

#### Desktop
- **GenerateSSHKeyDialog**: Interactive SSH key generation
  - Account name and email input
  - Platform selection (all 8 platforms)
  - Account type selection (personal, school, work, organization)
  - Key type selection (ed25519, rsa)
  - Optional passphrase with confirmation
  - Real-time key generation feedback

- **SetupPATDialog**: Personal Access Token setup
  - Platform-specific instructions
  - PAT validation
  - Token testing before saving

#### Web API
- `POST /api/v1/interactive/ssh/generate-key` - Generate SSH key
- `POST /api/v1/interactive/ssh/test-pat` - Test PAT token
- `GET /api/v1/interactive/platforms` - List all platforms
- `GET /api/v1/interactive/platforms/<platform_name>` - Get platform info

## Usage Examples

### Desktop Usage

```python
from src.git_manager.desktop.interactive_manager import InteractiveDesktopManager
from src.git_manager.desktop.widgets.interactive_accounts_widget import InteractiveAccountsWidget

# Create manager
manager = InteractiveDesktopManager()

# List accounts
accounts = manager.list_all_accounts()

# Test SSH
result = manager.test_ssh_connection('my-github-account')

# Clone repository
result = manager.clone_repository(
    repo_url='https://github.com/user/repo.git',
    account_name='my-github-account',
    destination='/home/user/repos/repo',
    auth_method='ssh',
    recursive=False,
    shallow=False
)

# Check status
status = manager.check_repository_status('/home/user/repos/repo')

# Generate SSH key
result = manager.generate_ssh_key(
    account_name='my-account',
    email='user@example.com',
    platform='github',
    account_type='personal',
    key_type='ed25519'
)
```

### Web API Usage

```bash
# List accounts
curl http://localhost:5000/api/v1/interactive/accounts

# Test SSH connection
curl -X POST http://localhost:5000/api/v1/interactive/accounts/my-account/test-ssh

# Analyze URL
curl -X POST http://localhost:5000/api/v1/interactive/clone/analyze-url \
  -H "Content-Type: application/json" \
  -d '{"url": "https://github.com/user/repo.git"}'

# Fetch personal repositories
curl "http://localhost:5000/api/v1/interactive/clone/personal-repositories?platform=github&account=my-account"

# Clone repository
curl -X POST http://localhost:5000/api/v1/interactive/clone/repository \
  -H "Content-Type: application/json" \
  -d '{
    "repo_url": "https://github.com/user/repo.git",
    "account_name": "my-account",
    "destination": "/home/user/repos/repo",
    "auth_method": "ssh",
    "recursive": false,
    "shallow": false
  }'

# Check repository status
curl "http://localhost:5000/api/v1/interactive/git/status?path=/home/user/repos/repo"

# Push changes
curl -X POST http://localhost:5000/api/v1/interactive/git/push \
  -H "Content-Type: application/json" \
  -d '{"path": "/home/user/repos/repo"}'

# Generate SSH key
curl -X POST http://localhost:5000/api/v1/interactive/ssh/generate-key \
  -H "Content-Type: application/json" \
  -d '{
    "account_name": "my-account",
    "email": "user@example.com",
    "platform": "github",
    "account_type": "personal",
    "key_type": "ed25519"
  }'

# Test PAT token
curl -X POST http://localhost:5000/api/v1/interactive/ssh/test-pat \
  -H "Content-Type: application/json" \
  -d '{
    "platform": "github",
    "pat_token": "ghp_xxxxxxxxxxxxxxxxxxxx"
  }'
```

## Platform Support

All implementations support these 8 platforms:

1. **GitHub** - SSH ✓, PAT ✓
2. **GitLab** - SSH ✓, PAT ✓
3. **Bitbucket** - SSH ✓, PAT ✓
4. **Azure DevOps** - SSH ✓, PAT ✓
5. **Self-Hosted** - SSH ✓, PAT ✓
6. **Cloud Storage** - SSH ✗, PAT ✓
7. **Local Path** - SSH ✗, PAT ✗
8. **SourceForge** - SSH ✓, PAT ✓

## Error Handling

### Desktop
- User-friendly error dialogs with actionable messages
- Detailed error logging for debugging
- Retry mechanisms for transient failures (SSH timeouts, network issues)
- Progress indicators for long-running operations

### Web API
- HTTP status codes (200, 400, 404, 500)
- Structured error responses with error messages
- Detailed logging for debugging
- Timeout handling for external API calls

## Thread Safety

### Desktop
- All long-running operations use worker threads
- Progress signals for UI updates
- Thread-safe completion callbacks
- Proper thread cleanup

### Web
- Flask handles concurrent requests
- Thread-safe manager initialization
- Proper exception handling and logging

## Logging

All implementations use the centralized logging system:
- `LogCategory.ACTIVITY` - General activity logs
- `LogCategory.GIT_OPERATION` - Git operation logs
- `LogCategory.SSH_OPERATION` - SSH operation logs

Logs are written to:
- Console (development)
- File (production)
- JSON format (web)

## Testing

To test the implementations:

### Desktop
1. Run the desktop application
2. Navigate to Account Management, Clone, or Git Operations tabs
3. Test each feature with your configured accounts

### Web
1. Start the web server: `git-manager --web`
2. Use curl or Postman to test API endpoints
3. Check logs for detailed operation information

## Integration with Existing Code

The implementations seamlessly integrate with:
- Existing CLI interactive mode
- Desktop application main window
- Web application routes
- Core modules and managers
- Database and configuration systems

All three interfaces (CLI, Desktop, Web) now provide the same comprehensive functionality using the same underlying core modules.

## Future Enhancements

Potential improvements:
1. WebSocket support for real-time updates in web UI
2. Advanced filtering and search in repository lists
3. Batch operations (clone multiple repositories)
4. Repository templates and scaffolding
5. Integration with CI/CD systems
6. Advanced SSH key management (key rotation, expiration)
7. Multi-account synchronization
8. Repository mirroring and backup
