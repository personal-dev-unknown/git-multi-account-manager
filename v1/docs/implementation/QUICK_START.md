# Quick Start: Interactive Features

## Overview

All interactive features from the CLI are now available in Desktop and Web interfaces.

## Files at a Glance

### Desktop
```
src/git_manager/desktop/
├── interactive_manager.py              # Core functionality
└── widgets/
    ├── interactive_accounts_widget.py  # Account management UI
    ├── interactive_git_widget.py       # Git operations UI
    └── interactive_clone_widget.py     # Clone repository UI
```

### Web
```
src/git_manager/web/
└── routes/
    └── interactive.py                  # REST API endpoints
```

## Usage

### Desktop

```python
from src.git_manager.desktop.interactive_manager import InteractiveDesktopManager

manager = InteractiveDesktopManager()

# List accounts
accounts = manager.list_all_accounts()

# Test SSH
result = manager.test_ssh_connection('account-name')

# Clone repository
result = manager.clone_repository(
    repo_url='https://github.com/user/repo.git',
    account_name='my-account',
    destination='/path/to/repo'
)

# Check status
status = manager.check_repository_status('/path/to/repo')

# Push changes
result = manager.git_push('/path/to/repo')

# Generate SSH key
result = manager.generate_ssh_key(
    account_name='my-account',
    email='user@example.com',
    platform='github'
)
```

### Web API

```bash
# List accounts
curl http://localhost:5000/api/v1/interactive/accounts

# Test SSH
curl -X POST http://localhost:5000/api/v1/interactive/accounts/my-account/test-ssh

# Analyze URL
curl -X POST http://localhost:5000/api/v1/interactive/clone/analyze-url \
  -H "Content-Type: application/json" \
  -d '{"url": "https://github.com/user/repo.git"}'

# Fetch repositories
curl "http://localhost:5000/api/v1/interactive/clone/personal-repositories?platform=github&account=my-account"

# Clone repository
curl -X POST http://localhost:5000/api/v1/interactive/clone/repository \
  -H "Content-Type: application/json" \
  -d '{
    "repo_url": "https://github.com/user/repo.git",
    "account_name": "my-account",
    "destination": "/path/to/repo"
  }'

# Check status
curl "http://localhost:5000/api/v1/interactive/git/status?path=/path/to/repo"

# Push changes
curl -X POST http://localhost:5000/api/v1/interactive/git/push \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/repo"}'

# Pull changes
curl -X POST http://localhost:5000/api/v1/interactive/git/pull \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/repo"}'

# Sync repository
curl -X POST http://localhost:5000/api/v1/interactive/git/sync \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/repo"}'

# Setup new repository
curl -X POST http://localhost:5000/api/v1/interactive/repository/setup \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/path/to/repo",
    "account_name": "my-account",
    "repo_name": "my-repo",
    "description": "My repository",
    "branch": "main"
  }'

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

# List platforms
curl http://localhost:5000/api/v1/interactive/platforms

# Get platform info
curl http://localhost:5000/api/v1/interactive/platforms/github
```

## Features

### Account Management
- ✅ List all accounts
- ✅ Test SSH connections
- ✅ Generate SSH keys
- ✅ Setup PAT tokens
- ✅ Validate tokens

### Clone Operations
- ✅ Analyze URLs
- ✅ Fetch personal repositories
- ✅ Clone with options (recursive, shallow)
- ✅ Multiple auth methods (SSH, HTTPS)
- ✅ Custom destinations

### Git Operations
- ✅ Check status
- ✅ Push changes
- ✅ Pull changes
- ✅ Sync repository
- ✅ Setup new repository

### SSH Management
- ✅ Generate keys (ed25519, rsa)
- ✅ Test connections
- ✅ Validate PAT tokens
- ✅ Support all 8 platforms

## Platforms Supported

1. GitHub
2. GitLab
3. Bitbucket
4. Azure DevOps
5. Self-Hosted
6. Cloud Storage
7. Local Path
8. SourceForge

## Core Modules

All implementations use:
- `AccountManager` - Account management
- `SSHWorkflowOrchestrator` - SSH operations
- `GitOperations` - Git operations
- `CloneWorkflow` - Clone operations
- `RepositoryManager` - Repository setup
- `DatabaseManager` - Data persistence
- `ConfigManager` - Configuration
- `PlatformManager` - Platform information

## Integration

### Desktop Integration

Update `src/git_manager/desktop/app.py`:

```python
from .widgets.interactive_accounts_widget import InteractiveAccountsWidget
from .widgets.interactive_clone_widget import InteractiveCloneWidget
from .widgets.interactive_git_widget import InteractiveGitWidget

# In MainWindow.init_ui():
tabs.addTab(InteractiveCloneWidget(), "Clone Repository")
tabs.addTab(InteractiveGitWidget(), "Git Operations")
tabs.addTab(InteractiveAccountsWidget(), "Accounts")
```

See `DESKTOP_INTEGRATION_GUIDE.md` for complete instructions.

### Web Integration

Already integrated! Just start the web server:

```bash
git-manager --web
```

Then access the API at `http://localhost:5000/api/v1/interactive/`

## Testing

### Desktop
```bash
git-manager --desktop
```

Then use the interactive tabs to test features.

### Web
```bash
git-manager --web
```

Then use curl or Postman to test endpoints.

### CLI
```bash
git-manager --cli
```

Existing interactive mode still available.

## Documentation

- **`INTERACTIVE_IMPLEMENTATION.md`** - Comprehensive guide
- **`IMPLEMENTATION_SUMMARY.md`** - Implementation details
- **`DESKTOP_INTEGRATION_GUIDE.md`** - Desktop integration instructions
- **`QUICK_START.md`** - This file

## Error Handling

All implementations include:
- User-friendly error messages
- Detailed logging
- Retry mechanisms for transient failures
- Proper exception handling

## Performance

- Desktop: Async operations with worker threads
- Web: Concurrent request handling
- Both: Efficient caching and resource management

## Next Steps

1. Integrate widgets into main desktop app
2. Create web UI templates
3. Run comprehensive tests
4. Gather user feedback
5. Optimize based on usage patterns

## Support

For issues or questions:
1. Check the documentation files
2. Review the implementation code
3. Check logs for error details
4. Run tests to verify functionality

## Summary

✅ All interactive CLI features are now available in Desktop and Web
✅ Using the same core modules for consistency
✅ Ready for integration and testing
✅ Comprehensive documentation provided
