# SSH Key Architecture - ~/.ssh/gitmanager/

## Overview

Git Multi-Account Manager uses a professional, scalable SSH key storage architecture that keeps application-specific keys separate from system SSH keys.

## Directory Structure

```
~/.ssh/
├── id_rsa                    # User's personal system keys
├── id_ed25519                # System keys
├── known_hosts               # Global known hosts
├── config                    # Main SSH config (includes gitmanager config)
└── gitmanager/               # ✨ Git Manager's isolated space
    ├── SKYREAPER-SPEC        # Private key
    ├── SKYREAPER-SPEC.pub    # Public key
    ├── work-account          # Private key
    ├── work-account.pub      # Public key
    ├── personal-account      # Private key
    ├── personal-account.pub  # Public key
    └── config                # Git Manager SSH config
```

## Why This Architecture?

### 1. **Clear Separation of Concerns**
- System SSH keys stay in `~/.ssh/`
- Application keys stay in `~/.ssh/gitmanager/`
- No interference with user's existing SSH setup
- Easy to backup/restore all gitmanager keys at once

### 2. **Safe and Non-Intrusive**
- Won't modify user's existing SSH keys
- Clean uninstall - just remove the gitmanager directory
- User maintains full control over their SSH configuration
- Easy to audit what the app is doing

### 3. **Scalable Multi-Account Management**
- Store unlimited number of keys for different accounts
- Organize by account name or purpose
- Easy to add/remove accounts without affecting system SSH

### 4. **Professional Best Practices**
- Matches architecture of GitHub CLI, AWS CLI, Google Cloud CLI
- Follows SSH security standards
- Proper file permissions (700 for directories, 600 for keys)

## File Permissions

Git Manager automatically sets proper permissions:

```bash
~/.ssh/gitmanager/           # 700 (rwx------)
~/.ssh/gitmanager/*          # 600 (rw-------)  Private keys
~/.ssh/gitmanager/*.pub      # 644 (rw-r--r--) Public keys
~/.ssh/gitmanager/config     # 600 (rw-------)
```

## SSH Configuration

### Main SSH Config (~/.ssh/config)

Git Manager automatically adds an include statement:

```ssh
Include ~/.ssh/gitmanager/config

# Your existing SSH config...
```

### Git Manager SSH Config (~/.ssh/gitmanager/config)

Automatically generated entries for each account:

```ssh
# Git account: github-SKYREAPER-SPEC
Host github-SKYREAPER-SPEC
    HostName github.com
    User git
    IdentityFile ~/.ssh/gitmanager/SKYREAPER-SPEC
    IdentitiesOnly yes

# Git account: github-work
Host github-work
    HostName github.com
    User git
    IdentityFile ~/.ssh/gitmanager/work-account
    IdentitiesOnly yes

# Git account: gitlab-personal
Host gitlab-personal
    HostName gitlab.com
    User git
    IdentityFile ~/.ssh/gitmanager/personal-account
    IdentitiesOnly yes
```

## Usage Examples

### Example 1: Generate SSH Key for GitHub Account

```bash
python3 -m git_manager --cli
# Select option 8: Generate new SSH key

Email: dev@example.com
Key name: SKYREAPER-SPEC

# Key is automatically saved to:
# ~/.ssh/gitmanager/SKYREAPER-SPEC
# ~/.ssh/gitmanager/SKYREAPER-SPEC.pub

# SSH config automatically updated:
# Host github-SKYREAPER-SPEC
#     HostName github.com
#     User git
#     IdentityFile ~/.ssh/gitmanager/SKYREAPER-SPEC
```

### Example 2: Clone Repository with Specific Account

```bash
# Using the SSH host alias from config
git clone git@github-SKYREAPER-SPEC:username/repo.git

# Or configure in repository
cd repo
git config user.email "dev@example.com"
git config core.sshCommand "ssh -i ~/.ssh/gitmanager/SKYREAPER-SPEC"
```

### Example 3: Multiple GitHub Accounts

```bash
# Generate keys for different accounts
python3 -m git_manager --cli
# Option 8: Generate key for work account
# Option 8: Generate key for personal account

# SSH config will have:
Host github-work
    IdentityFile ~/.ssh/gitmanager/work-account

Host github-personal
    IdentityFile ~/.ssh/gitmanager/personal-account

# Clone with specific account
git clone git@github-work:company/repo.git
git clone git@github-personal:username/repo.git
```

## Database Storage

While SSH keys are stored in `~/.ssh/gitmanager/`, metadata is stored in SQLite:

```
~/.config/git-manager/gitmanager.db
```

### SSH Keys Table

```sql
CREATE TABLE ssh_keys (
    id INTEGER PRIMARY KEY,
    key_name TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    key_path TEXT NOT NULL,
    public_key TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_used TIMESTAMP
);
```

### Accounts Table

```sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    account_name TEXT UNIQUE NOT NULL,
    platform TEXT NOT NULL,
    username TEXT NOT NULL,
    email TEXT,
    ssh_key_id INTEGER,
    host_alias TEXT,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (ssh_key_id) REFERENCES ssh_keys(id)
);
```

## Security Considerations

### 1. **File Permissions**
- Git Manager enforces 700 permissions on `~/.ssh/gitmanager/`
- Private keys have 600 permissions
- Public keys have 644 permissions

### 2. **Key Storage**
- Keys are stored on disk with proper permissions
- No keys are stored in memory longer than necessary
- Database stores key paths, not actual key contents

### 3. **SSH Config**
- SSH config is readable only by owner (600)
- Includes statement is added to main SSH config
- No modification of user's existing SSH keys

### 4. **Best Practices**
- Use strong passphrases for keys
- Regularly rotate keys
- Monitor SSH key usage
- Keep backups in secure location

## Implementation Details

### Files Modified

1. **`src/git_manager/utils/config_paths.py`**
   - Added `get_ssh_keys_dir()` - Returns `~/.ssh/gitmanager/`
   - Added `get_ssh_config_file()` - Returns `~/.ssh/gitmanager/config`
   - Updated `ensure_config_dirs()` - Creates SSH directory with proper permissions

2. **`src/git_manager/core/ssh_manager.py`**
   - Updated to use `~/.ssh/gitmanager/` by default
   - Automatically sets proper permissions
   - Integrates with SSH config manager

3. **`src/git_manager/core/ssh_config_manager.py`** (NEW)
   - Manages SSH configuration
   - Adds/removes account entries
   - Ensures main SSH config includes gitmanager config
   - Validates SSH configuration

### Classes

#### SSHConfigManager

```python
from git_manager.core.ssh_config_manager import SSHConfigManager

manager = SSHConfigManager()

# Add account config
manager.add_account_config(
    host_alias="github-work",
    hostname="github.com",
    identity_file=Path("~/.ssh/gitmanager/work-account"),
    user="git"
)

# List accounts
accounts = manager.list_account_configs()
# Returns: ["github-work", "github-personal", "gitlab-dev"]

# Get account config
config = manager.get_account_config("github-work")
# Returns: {
#     "host": "github-work",
#     "hostname": "github.com",
#     "user": "git",
#     "identityfile": "~/.ssh/gitmanager/work-account",
#     "identitiesonly": "yes"
# }

# Remove account
manager.remove_account_config("github-work")
```

## Troubleshooting

### SSH Key Not Found

```bash
# Check if keys exist
ls -la ~/.ssh/gitmanager/

# Check SSH config
cat ~/.ssh/gitmanager/config

# Test SSH connection
ssh -T git@github-SKYREAPER-SPEC
```

### SSH Config Not Included

```bash
# Check main SSH config
cat ~/.ssh/config

# Should contain:
# Include ~/.ssh/gitmanager/config

# If missing, add manually:
echo "Include ~/.ssh/gitmanager/config" >> ~/.ssh/config
```

### Permission Denied

```bash
# Fix permissions
chmod 700 ~/.ssh/gitmanager/
chmod 600 ~/.ssh/gitmanager/*
chmod 644 ~/.ssh/gitmanager/*.pub
```

### Git Clone Not Using Correct Key

```bash
# Explicitly specify key
git clone -c core.sshCommand="ssh -i ~/.ssh/gitmanager/work-account" \
    git@github.com:company/repo.git

# Or use SSH host alias
git clone git@github-work:company/repo.git
```

## Migration from Default SSH

If you have existing SSH keys in `~/.ssh/`:

```bash
# Backup existing keys
cp ~/.ssh/id_* ~/.ssh/gitmanager/

# Update permissions
chmod 600 ~/.ssh/gitmanager/id_*
chmod 644 ~/.ssh/gitmanager/id_*.pub

# Update SSH config
# Manually add entries to ~/.ssh/gitmanager/config
```

## Future Enhancements

- [ ] SSH key rotation scheduling
- [ ] Automatic backup of keys
- [ ] Key usage analytics
- [ ] Integration with SSH agent
- [ ] Hardware security key support
- [ ] Key encryption at rest

## References

- [SSH Config Man Page](https://man.openbsd.org/ssh_config)
- [SSH Best Practices](https://wiki.archlinux.org/title/SSH_keys)
- [GitHub SSH Documentation](https://docs.github.com/en/authentication/connecting-to-github-with-ssh)
- [GitLab SSH Documentation](https://docs.gitlab.com/ee/ssh/)
