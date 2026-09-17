# SSH Integration Architecture

## Overview

The SSH system is composed of multiple integrated modules that work together to provide secure SSH key management and git operations. This document explains how all components work as a unified system.

## Module Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                    Git Operations Layer                      │
│  (push_operations, pull_operations, sync_operations)        │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  GitSSHHelper (NEW)                          │
│  - Executes git commands with SSH authentication            │
│  - Reads SSH key from git config                            │
│  - Integrates with system SSH agent                         │
└────────────────────┬────────────────────────────────────────┘
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────────┐
│ SSHManager   │ │SSHConfigMgr  │ │ ssh_helpers      │
│ - Generate   │ │ - Manage SSH │ │ - Test SSH conn  │
│   keys       │ │   config     │ │ - Get fingerprint│
│ - Add to     │ │ - Add/remove │ │ - SSH agent      │
│   agent      │ │   accounts   │ │   status         │
└──────────────┘ └──────────────┘ └──────────────────┘
        │            │                    │
        └────────────┼────────────────────┘
                     │
                     ▼
        ┌────────────────────────┐
        │  System SSH Agent      │
        │  ~/.ssh/config         │
        │  ~/.ssh/gitmanager/    │
        └────────────────────────┘
```

## Module Responsibilities

### 1. GitSSHHelper (`git_ssh_helper.py`) - NEW
**Location:** `src/git_manager/core/sync/git_ssh_helper.py`

**Purpose:** Bridge between git operations and SSH infrastructure

**Key Methods:**
- `run_git_command()` - Execute git commands with SSH support
- `get_account_ssh_key()` - Extract SSH key from git config

**Integration Points:**
- Uses `is_ssh_agent_running()` from `ssh_helpers`
- Reads `core.sshCommand` from git config
- Sets `GIT_SSH_COMMAND` environment variable

**Responsibilities:**
- ✅ Execute git commands with SSH authentication
- ✅ Read SSH key path from git config
- ✅ Manage environment variables for SSH
- ✅ Handle timeouts and errors
- ✅ Integrate with system SSH agent

### 2. SSHManager (`ssh_manager.py`) - EXISTING
**Location:** `src/git_manager/core/ssh_manager.py`

**Purpose:** Generate and manage SSH keys

**Key Methods:**
- `generate_key()` - Create new SSH key pair
- `add_to_agent()` - Add key to SSH agent
- `remove_from_agent()` - Remove key from agent
- `list_keys()` - List all SSH keys

**Integration Points:**
- Uses `SSHConfigManager` to add config entries
- Stores keys in `~/.ssh/gitmanager/`
- Adds keys to system SSH agent

**Responsibilities:**
- ✅ Generate ED25519 and RSA keys
- ✅ Manage key passphrases
- ✅ Add/remove keys from SSH agent
- ✅ Track key metadata

### 3. SSHConfigManager (`ssh_config_manager.py`) - EXISTING
**Location:** `src/git_manager/core/ssh_config_manager.py`

**Purpose:** Manage SSH config file entries

**Key Methods:**
- `add_account_config()` - Add SSH config entry
- `remove_account_config()` - Remove SSH config entry
- `get_account_config()` - Retrieve SSH config
- `list_account_configs()` - List all configured accounts
- `validate_ssh_config()` - Validate SSH config syntax

**Integration Points:**
- Manages `~/.ssh/config` and `~/.ssh/gitmanager/config`
- Creates Host entries for each account
- Ensures main SSH config includes gitmanager config

**Responsibilities:**
- ✅ Manage SSH config file
- ✅ Create Host entries
- ✅ Map accounts to SSH keys
- ✅ Validate configuration

### 4. SSHConfigParser (`ssh_config_parser.py`) - EXISTING
**Location:** `src/git_manager/core/ssh_config_parser.py`

**Purpose:** Parse SSH config to extract accounts

**Key Methods:**
- `parse_accounts()` - Extract accounts from SSH config
- `_parse_host_blocks()` - Parse Host entries
- `_extract_account()` - Convert Host entry to Account

**Integration Points:**
- Reads SSH config file
- Creates Account objects from Host entries
- Used for account discovery

**Responsibilities:**
- ✅ Parse SSH config syntax
- ✅ Extract account information
- ✅ Convert to Account objects

### 5. ssh_helpers (`ssh_helpers.py`) - EXISTING
**Location:** `src/git_manager/utils/ssh_helpers.py`

**Purpose:** Low-level SSH utilities

**Key Functions:**
- `is_ssh_agent_running()` - Check agent status
- `start_ssh_agent()` - Start SSH agent
- `get_ssh_key_fingerprint()` - Get key fingerprint
- `test_ssh_connection()` - Test SSH connection
- `get_public_key()` - Extract public key

**Integration Points:**
- Used by SSHManager and GitSSHHelper
- Provides SSH agent status
- Tests SSH connections

**Responsibilities:**
- ✅ Check SSH agent status
- ✅ Get key fingerprints
- ✅ Test SSH connections
- ✅ Extract public keys

## Data Flow

### Setup Phase
```
User creates account with SSH key
    ↓
SSHManager.generate_key()
    ├─ Creates key pair
    ├─ Adds to SSH agent
    └─ Returns SSHKey object
    ↓
SSHConfigManager.add_account_config()
    ├─ Creates Host entry in SSH config
    ├─ Maps account to SSH key
    └─ Sets IdentityFile path
    ↓
Repository setup
    ├─ Reads SSH key path from account
    └─ Sets git config: core.sshCommand = "ssh -i /path/to/key"
```

### Operation Phase
```
User runs push/pull/sync
    ↓
PushOperations/PullOperations/SyncOperations
    ├─ Creates GitSSHHelper instance
    └─ Calls _run_git_command()
    ↓
GitSSHHelper.run_git_command()
    ├─ Calls get_account_ssh_key(repo_path)
    ├─ Reads git config: core.sshCommand
    ├─ Extracts SSH key path
    ├─ Sets GIT_SSH_COMMAND environment variable
    └─ Executes git command
    ↓
Git command execution
    ├─ Uses GIT_SSH_COMMAND for SSH
    ├─ SSH agent provides key access
    └─ Authentication succeeds
```

## Configuration Files

### SSH Config Structure
```
~/.ssh/config
├─ Include ~/.ssh/gitmanager/config  (added by SSHConfigManager)

~/.ssh/gitmanager/config
├─ Host github-account1
│  ├─ HostName github.com
│  ├─ User git
│  └─ IdentityFile ~/.ssh/gitmanager/id_ed25519_account1
├─ Host gitlab-account1
│  ├─ HostName gitlab.com
│  ├─ User git
│  └─ IdentityFile ~/.ssh/gitmanager/id_ed25519_account1
└─ ...

~/.ssh/gitmanager/
├─ id_ed25519_account1
├─ id_ed25519_account1.pub
├─ id_rsa_account2
├─ id_rsa_account2.pub
└─ ...
```

### Git Config (per repository)
```
.git/config
[core]
    sshCommand = ssh -i ~/.ssh/gitmanager/id_ed25519_account1 -o StrictHostKeyChecking=accept-new
[user]
    name = username
    email = user@example.com
```

## Integration Points

### 1. Account Creation
```python
# SSHManager creates key
ssh_manager = SSHManager()
ssh_key = ssh_manager.generate_key(email, key_name)

# SSHConfigManager adds config
config_manager = SSHConfigManager()
config_manager.add_account_config(
    host_alias=f"github-{account_name}",
    hostname="github.com",
    identity_file=ssh_key.private_key_path
)
```

### 2. Repository Setup
```python
# Setup configures git with SSH
subprocess.run([
    'git', 'config', 'core.sshCommand',
    f'ssh -i {ssh_key_path} -o StrictHostKeyChecking=accept-new'
])
```

### 3. Git Operations
```python
# Push/Pull/Sync use GitSSHHelper
push_ops = PushOperations()
result = push_ops.safe_push(repo_path)
# Internally:
# - GitSSHHelper reads git config
# - Extracts SSH key path
# - Executes git command with SSH
```

## No Redundancy

### Removed Duplicate Functionality
- ✅ `convert_https_to_ssh()` - Not needed, git config uses SSH directly
- ✅ `set_remote_url()` - Not needed, setup handles this
- ✅ `get_remote_url()` - Not needed, git config handles this
- ✅ `_get_ssh_auth_sock()` - Uses `is_ssh_agent_running()` from ssh_helpers

### Kept Minimal Interface
- ✅ `run_git_command()` - Only method needed for git operations
- ✅ `get_account_ssh_key()` - Only method needed to read config

## Testing Integration

### Test SSH Key Generation
```python
from git_manager.core.ssh_manager import SSHManager

ssh_mgr = SSHManager()
key = ssh_mgr.generate_key("user@example.com", "my-account")
# Key is in ~/.ssh/gitmanager/
# Key is added to SSH agent
```

### Test SSH Config Management
```python
from git_manager.core.ssh_config_manager import SSHConfigManager

config_mgr = SSHConfigManager()
config_mgr.add_account_config(
    "github-myaccount",
    "github.com",
    Path("~/.ssh/gitmanager/id_ed25519_myaccount")
)
# Entry added to ~/.ssh/gitmanager/config
# Main SSH config includes gitmanager config
```

### Test Git Operations with SSH
```python
from git_manager.core.sync import PushOperations
from pathlib import Path

push_ops = PushOperations()
result = push_ops.safe_push(Path("/path/to/repo"))
# Uses SSH key from git config automatically
# No HTTPS prompts
```

## Security

### Key Storage
- ✅ Keys stored in `~/.ssh/gitmanager/` (700 permissions)
- ✅ Each account has separate key
- ✅ Keys never exposed in logs or output

### Authentication
- ✅ SSH agent manages key access
- ✅ Passphrase-protected keys supported
- ✅ No credentials in memory

### Configuration
- ✅ SSH config validated before use
- ✅ StrictHostKeyChecking for new hosts
- ✅ IdentitiesOnly prevents key leakage

## Summary

The SSH integration is a cohesive system where:

1. **SSHManager** creates and manages SSH keys
2. **SSHConfigManager** manages SSH config entries
3. **SSHConfigParser** extracts accounts from config
4. **ssh_helpers** provides low-level utilities
5. **GitSSHHelper** bridges git operations with SSH
6. **Push/Pull/Sync operations** use GitSSHHelper transparently

All components work together without redundancy, providing secure SSH authentication for all git operations.
