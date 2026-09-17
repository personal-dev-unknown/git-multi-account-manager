# Git Multi-Account Manager - Implementation Guide

## Overview
This guide documents the complete implementation of email handling, XDG Base Directory support, and SSH key generation workflow improvements.

## Key Changes

### 1. Email Handling - Best Practice Implementation

#### Problem
Previously, the system was generating fake emails (username@github.com) instead of capturing real emails.

#### Solution
Email is now captured during SSH key generation and stored in the database:

```
SSH Key Generation Flow:
┌─────────────────────────────────────────────────────────────┐
│ 1. User runs: python3 -m git_manager --cli                  │
│    → Select option [8] Generate new SSH key                 │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. System prompts:                                           │
│    Email: user@example.com  (REAL EMAIL CAPTURED)           │
│    Key name: my-github-key                                  │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. SSH key generated with email as comment                  │
│    ✓ Key generated                                          │
│    Public key: ssh-ed25519 AAAA...                          │
│    Key saved to: ~/.ssh/id_ed25519_my_github_key            │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. System offers: "Save this key as a Git account?"         │
│    → yes                                                    │
│    Account name: my-work-account                            │
│    Git username: myusername                                 │
│    Platform: github                                         │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. Account saved with REAL email:                           │
│    {                                                        │
│      "name": "my-work-account",                             │
│      "platform": "github",                                  │
│      "username": "myusername",                              │
│      "email": "user@example.com",  ← REAL EMAIL             │
│      "ssh_key_path": "~/.ssh/id_ed25519_my_github_key",     │
│      "host": "github.com-my-work-account",                  │
│      "description": "github account for myusername"         │
│    }                                                        │
└─────────────────────────────────────────────────────────────┘
```

### 2. XDG Base Directory Configuration

The application now follows platform-specific configuration standards:

#### Linux/Unix
```
~/.config/git-manager/          # Recommended (XDG standard)
  ├── accounts.json
  ├── config.yaml
  └── cache/
      └── repositories.json

OR

~/.git-manager/                 # Fallback
  ├── accounts.json
  └── logs/
```

#### macOS
```
~/Library/Application Support/git-manager/
  ├── accounts.json
  ├── config.yaml
  └── cache/
```

#### Windows
```
%APPDATA%\git-manager\          # User-specific
  ├── accounts.json
  ├── config.yaml
  └── cache\

OR

%LOCALAPPDATA%\git-manager\     # Local machine
```

### 3. Account Model Changes

#### Before
```python
@dataclass
class Account:
    name: str
    platform: Platform
    username: str
    email: str              # REQUIRED - generated fake value
    ssh_key_path: Path
    host: str
    description: Optional[str] = None
```

#### After
```python
@dataclass
class Account:
    name: str
    platform: Platform
    username: str
    ssh_key_path: Path
    host: str
    email: Optional[str] = None        # OPTIONAL - real email from key generation
    description: Optional[str] = None
```

### 4. SSH Config Parser

The SSH config parser no longer generates fake emails:

```python
# Before: email=f"{username}@{platform.value}.com"
# After:  email=None  # To be filled during key generation
```

### 5. CLI Commands

#### Generate SSH Key (Interactive Mode)
```bash
python3 -m git_manager --cli
→ Select option [8] Generate new SSH key
→ Email: your-real@email.com
→ Key name: github-work
→ [Key generated successfully]
→ Save this key as a Git account? yes
→ Account name: work-github
→ Git username: yourname
→ Platform: github
→ [Account saved with real email]
```

#### Add Account (CLI)
```bash
# Email is now optional
python3 -m git_manager account add \
  --name my-account \
  --platform github \
  --username myusername \
  --ssh-key ~/.ssh/id_ed25519_mykey
  # --email is optional, can be added later

# Or with email
python3 -m git_manager account add \
  --name my-account \
  --platform github \
  --username myusername \
  --ssh-key ~/.ssh/id_ed25519_mykey \
  --email my@email.com
```

#### Update Account Email
```bash
python3 -m git_manager account update my-account \
  --email my-real@email.com
```

#### List Accounts
```bash
python3 -m git_manager account list
# Shows all accounts with their emails (if available)
```

## SSH Connection Timeout

Increased from 10 seconds to 30 seconds to allow adequate time for user authentication:

```python
# ssh_manager.py
def test_connection(
    self,
    host: str,
    key_path: Path,
    timeout: int = 30  # Changed from 10
) -> Tuple[bool, str]:
```

## Files Modified

1. **src/git_manager/models/account.py**
   - Made email optional: `email: Optional[str] = None`
   - Reordered parameters in dataclass

2. **src/git_manager/core/account_manager.py**
   - Updated to use XDG config paths
   - Made email optional in `add_account()` method
   - Updated `_load_accounts()` to handle optional emails

3. **src/git_manager/core/ssh_config_parser.py**
   - Changed email from generated value to None
   - Email to be captured during SSH key generation

4. **src/git_manager/core/ssh_manager.py**
   - Increased timeout from 10s to 30s

5. **src/git_manager/utils/ssh_helpers.py**
   - Increased timeout from 10s to 30s

6. **src/git_manager/utils/config_paths.py** (NEW)
   - XDG Base Directory support
   - Platform-specific paths for Linux, macOS, Windows

7. **src/git_manager/cli/commands/account.py**
   - Made email optional in `add_account` command
   - Added `update_account` command

8. **src/git_manager/cli/ui/interactive.py**
   - Fixed Rich color names (primary→cyan, accent→green, etc.)
   - Enhanced `generate_key()` to save accounts with real emails
   - Dynamic SSH testing with platform selection

## Best Practices Implemented

### 1. Hybrid Approach
- **Database Priority**: Accounts stored in `~/.config/git-manager/accounts.json`
- **SSH Config Fallback**: If no database, parse `~/.ssh/config`
- **Email Capture**: Real emails captured during SSH key generation

### 2. XDG Compliance
- Follows platform-specific standards
- Respects environment variables (XDG_CONFIG_HOME, XDG_CACHE_HOME)
- Graceful fallbacks for non-standard systems

### 3. User Experience
- Email captured at the right time (during key generation)
- Optional email for backward compatibility
- Automatic account creation after key generation
- Clear prompts and confirmations

## Testing

Verify the implementation:

```bash
# Test 1: Generate SSH key and save account
python3 -m git_manager --cli
→ Option 8: Generate new SSH key
→ Provide real email
→ Save as account
→ Verify email in accounts.json

# Test 2: List accounts
python3 -m git_manager account list
→ Should show real emails (not fake ones)

# Test 3: Update email
python3 -m git_manager account update ACCOUNT_NAME --email new@email.com
→ Verify email updated in accounts.json

# Test 4: Test SSH connection
python3 -m git_manager --cli
→ Option 7: Test SSH connections
→ Should have 30 second timeout
```

## Migration from Old System

If you have accounts with fake emails:

### Option 1: Update via CLI
```bash
python3 -m git_manager account update devonionMoses --email real@gmail.com
python3 -m git_manager account update SKYREAPER-SPEC --email real@gmail.com
# ... repeat for all accounts
```

### Option 2: Manual JSON Edit
Edit `~/.config/git-manager/accounts.json` and replace fake emails

### Option 3: Regenerate Accounts
```bash
# Delete old accounts.json
rm ~/.config/git-manager/accounts.json

# Re-add accounts using new workflow
python3 -m git_manager --cli
→ Option 8: Generate SSH key (or add existing key)
→ Provide real email
→ Save as account
```

## Summary

✅ Email is now OPTIONAL - not generated as fake value
✅ Real emails captured during SSH key generation
✅ XDG Base Directory support for all platforms
✅ Hybrid approach: database priority, SSH config fallback
✅ SSH connection timeout increased to 30 seconds
✅ Interactive workflow for saving accounts with real emails
✅ CLI commands updated to support optional emails
