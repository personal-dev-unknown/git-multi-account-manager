# Clone Repository - Quick Start Guide

## Starting the Clone Feature

```bash
python3 -m git_manager --cli
# Select option: 1 (Clone a repository)
```

## Main Menu

```
═══ Clone Repository ═══

Choose clone type:
[1] Clone from external repository (any public/private repo)
[2] Clone from your personal repositories
[3] Back to main menu
```

## Option 1: Clone External Repository

### Use Case: Clone any repository (public or private)

**Steps:**
1. Enter repository URL (any format accepted)
2. System analyzes the repository
3. Choose your intent:
   - 📖 Study/Use the code (read-only)
   - 🔧 Contribute (fork + clone)
   - 🏗️ Build upon it (clone + customize)
4. Select your account
5. Choose authentication method
6. Configure clone options
7. Select clone destination
8. Clone completes with next steps

**Example URLs:**
```
github.com/facebook/react
https://gitlab.com/user/project
git@bitbucket.org:user/repo.git
https://github.com/user/repo.git
```

### Intent: Study/Use the Code

**What happens:**
- Clone read-only
- No fork needed
- Fastest option
- No push access

**Best for:**
- Learning from code
- Using as dependency
- Reference implementation

### Intent: Contribute to Project

**What happens:**
- Fork to your account
- Clone your fork
- Setup upstream remote
- Ready for pull requests

**Best for:**
- Contributing to open source
- Submitting pull requests
- Collaborating on projects

### Intent: Build Upon It

**What happens:**
- Clone with optional auth
- Setup for independent development
- No upstream tracking

**Best for:**
- Creating derivative projects
- Independent customization
- Building on existing code

## Option 2: Clone Personal Repositories

### Use Case: Clone your own repositories

**Steps:**
1. Select platform (GitHub, GitLab, Bitbucket)
2. Select account (if multiple configured)
3. System fetches your repositories
4. Select repository from list
5. Choose authentication method
6. Configure clone options
7. Select clone destination
8. Clone completes with next steps

**Features:**
- Lists all your repositories
- Shows visibility (private/public)
- Shows language and size
- Shows last update time
- Pagination for 100+ repos
- Search and filter options

## Authentication Methods

### 1. SSH (Recommended) 🔑

**Advantages:**
- Most secure
- No password needed
- Works with 2FA
- Best for automation

**Requirements:**
- SSH key configured in account
- SSH key added to platform

**When to use:**
- Always, if available
- Default recommendation

### 2. HTTPS with PAT 🎫

**Advantages:**
- Works everywhere
- Good for CI/CD
- Easy to revoke
- Fine-grained permissions

**Requirements:**
- Personal Access Token configured
- Token with repo access

**When to use:**
- SSH not available
- CI/CD environments
- Temporary access

### 3. HTTPS with Password 🔐

**Advantages:**
- Simple setup
- No token needed

**Disadvantages:**
- Less secure
- Doesn't work with 2FA
- GitLab only

**When to use:**
- GitLab only
- Temporary access
- No other options

### Anonymous (Read-only)

**Advantages:**
- No authentication needed
- Public repositories only
- Fastest clone

**Disadvantages:**
- No push access
- Public repos only

**When to use:**
- Public repositories
- No account needed
- Quick access

## Clone Options

### Submodules
```
Clone submodules recursively? [y/n]
```
- **Yes:** Clone all submodules (slower, larger)
- **No:** Skip submodules (faster, smaller)

### Shallow Clone
```
Shallow clone (--depth=1)? [y/n]
```
- **Yes:** Only latest commit (much faster, smaller)
- **No:** Full history (slower, larger)

## Clone Destination

### Default Location
```
~/projects/account-name/repository-name
```

### Custom Location
```
Enter custom path: /path/to/clone
```

## After Clone

### Next Steps Display
```
1. cd ~/projects/account/repo
2. Start working on the code
3. Use 'gitmanager push' to push changes
```

### For Contributions
```
1. Create feature branch: git checkout -b my-feature
2. Make your changes
3. Push to your fork: git push origin my-feature
4. Create PR to original repository
```

## Common Workflows

### Clone Public Repository to Study

```
Option: 1 (External)
URL: github.com/facebook/react
Intent: 1 (Study)
Auth: 1 (SSH)
Destination: Default
Result: Read-only clone of React
```

### Clone and Contribute to Open Source

```
Option: 1 (External)
URL: github.com/facebook/react
Intent: 2 (Contribute)
Account: Your GitHub account
Auth: 1 (SSH)
Destination: Default
Result: Fork cloned with upstream configured
```

### Clone Your Own Repository

```
Option: 2 (Personal)
Platform: 1 (GitHub)
Account: Your account
Repository: Select from list
Auth: 1 (SSH)
Destination: Default
Result: Your repo cloned locally
```

### Clone Large Repository (Shallow)

```
Option: 1 (External)
URL: github.com/large/repo
Intent: 1 (Study)
Auth: 1 (SSH)
Shallow: y (--depth=1)
Result: Fast clone with latest commit only
```

## Troubleshooting

### Authentication Failed

**Problem:** SSH key not working

**Solution:**
1. Test SSH connection: `ssh -T git@github.com`
2. Add SSH key to platform account
3. Check key permissions: `ls -la ~/.ssh/gitmanager/`
4. Try different authentication method

### Permission Denied

**Problem:** Can't access repository

**Solution:**
1. Check if you're a collaborator
2. Verify using correct account
3. For org repos: Enable SSO for PAT
4. Request access from owner

### Repository Not Found

**Problem:** URL doesn't work

**Solution:**
1. Verify URL on platform website
2. Check if repository is private
3. Try with authentication
4. Check if URL is correct

### Network Error

**Problem:** Can't connect to platform

**Solution:**
1. Check internet connection
2. Check platform status
3. Check firewall settings
4. Try again later

### Disk Space

**Problem:** Not enough space

**Solution:**
1. Free up disk space
2. Use shallow clone (--depth=1)
3. Clone to different location
4. Use sparse checkout

## Tips and Tricks

### Speed Up Clone

```
Use shallow clone: --depth=1
Skip submodules: Don't clone recursively
Use SSH: Faster than HTTPS
```

### Organize Clones

```
Default structure: ~/projects/account/repo
Create custom structure: Use custom location
```

### Multiple Accounts

```
Select different account for each clone
Each clone uses correct SSH key
No credential conflicts
```

### Fork Workflow

```
1. Clone external repo with "Contribute" intent
2. System forks automatically
3. Upstream remote configured
4. Ready for pull requests
```

## Keyboard Shortcuts

```
[1-9]     Select option
[Enter]   Accept default
[y/n]     Yes/No questions
[Ctrl+C]  Cancel operation
```

## Getting Help

```
# Show help for clone command
python3 -m git_manager clone --help

# Check account configuration
python3 -m git_manager account list

# Test SSH connection
python3 -m git_manager ssh test
```

## Summary

The Clone feature provides:
- ✅ Clone from external repositories
- ✅ Clone from personal repositories
- ✅ Multiple authentication methods
- ✅ Fork workflow for contributions
- ✅ Automatic account detection
- ✅ Intelligent error handling
- ✅ Post-clone setup
- ✅ Multiple platform support

**Start cloning:** `python3 -m git_manager --cli` → Option 1
