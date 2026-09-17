# Repository Setup Guide - Option 5

## Overview

Option 5 in the interactive CLI provides a guided wizard to set up new repositories with automatic git initialization, account configuration, and branch setup.

## What It Does

The Repository Setup Wizard guides you through:

1. **Select Account** - Choose which git account to use
2. **Repository Name** - Name for your repository
3. **Repository Path** - Where to create the repository
4. **Default Branch** - Branch name (default: `main`)
5. **Description** - Optional repository description
6. **Confirmation** - Review and confirm setup
7. **Initialization** - Automatic git setup
8. **Next Steps** - Instructions for pushing to remote

## Step-by-Step Walkthrough

### Step 1: Select Account

```
═══ Repository Setup Wizard ═══

Step 1: Select Account

┏━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Account Name   ┃ Platform   ┃ Username                           ┃
┡━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ SKYREAPER-SPEC │ github     │ devonionrouting4Moses              │
└────────────────┴────────────┴────────────────────────────────────┘

Select account name: SKYREAPER-SPEC
✓ Selected account: SKYREAPER-SPEC
```

### Step 2: Repository Name

```
Step 2: Repository Name
Enter repository name: my-awesome-project
✓ Repository name: my-awesome-project
```

### Step 3: Repository Path

```
Step 3: Repository Path
Enter repository path [/home/user/my-awesome-project]: /home/user/projects/my-awesome-project
✓ Repository path: /home/user/projects/my-awesome-project
```

### Step 4: Default Branch

```
Step 4: Default Branch
Enter default branch name [main]: main
✓ Default branch: main
```

### Step 5: Repository Description

```
Step 5: Repository Description
Enter repository description (optional): A cool project for learning
✓ Description: A cool project for learning
```

### Step 6: Setup Summary

```
═══ Setup Summary ═══

┏━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Property             ┃ Value                                                  ┃
┡━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ Repository Name      │ my-awesome-project                                     │
│ Repository Path      │ /home/user/projects/my-awesome-project                 │
│ Account              │ SKYREAPER-SPEC                                         │
│ Account Email        │ dev@example.com                                        │
│ Account Username     │ devonionrouting4Moses                                  │
│ Default Branch       │ main                                                   │
│ Description          │ A cool project for learning                            │
└──────────────────────┴────────────────────────────────────────────────────────┘

Proceed with setup? [yes/no] (yes): yes
```

### Step 7: Initialization

```
Initializing repository...
✓ Repository initialized successfully
```

### Step 8: Next Steps

```
═══ Next Steps ═══

1. Navigate to repository:
   cd /home/user/projects/my-awesome-project

2. Add files and commit:
   git add .
   git commit -m "Your commit message"

3. Create remote repository on GITHUB

4. Add remote and push:
   git remote add origin git@github-SKYREAPER-SPEC:devonionrouting4Moses/my-awesome-project.git
   git push -u origin main

5. Or use interactive mode option 4 (Git push) to push automatically
```

## What Gets Initialized

### Directory Structure

```
/home/user/projects/my-awesome-project/
├── .git/                    # Git repository
├── README.md                # Initial README
└── .gitignore               # Git ignore file (optional)
```

### Git Configuration

The setup automatically configures:

```bash
# User configuration
git config user.name "devonionrouting4Moses"
git config user.email "dev@example.com"

# SSH configuration
git config core.sshCommand "ssh -i ~/.ssh/gitmanager/SKYREAPER-SPEC"

# Branch configuration
git branch -M main
```

### Initial Commit

An initial commit is created with:
- **File:** `README.md`
- **Content:** Repository name and description
- **Message:** "Initial commit"

### Repository Configuration Storage

The setup saves repository metadata to:

```
~/.config/git-manager/repositories.json
```

Example:
```json
{
  "my-awesome-project": {
    "name": "my-awesome-project",
    "path": "/home/user/projects/my-awesome-project",
    "account": "SKYREAPER-SPEC",
    "account_platform": "github",
    "account_username": "devonionrouting4Moses",
    "account_email": "dev@example.com",
    "default_branch": "main",
    "description": "A cool project for learning",
    "created_at": "2025-11-21T09:30:00.123456",
    "ssh_key_path": "~/.ssh/gitmanager/SKYREAPER-SPEC",
    "host_alias": "github-SKYREAPER-SPEC"
  }
}
```

## Workflow After Setup

### 1. Add Your Files

```bash
cd /home/user/projects/my-awesome-project

# Create your project files
echo "# My Project" > README.md
echo "print('Hello World')" > main.py

# Stage files
git add .

# Commit
git commit -m "Add initial project files"
```

### 2. Create Remote Repository

On GitHub/GitLab:
1. Create a new repository
2. Use the same name: `my-awesome-project`
3. **Do NOT initialize with README** (you already have one)

### 3. Add Remote and Push

```bash
# Add remote
git remote add origin git@github-SKYREAPER-SPEC:devonionrouting4Moses/my-awesome-project.git

# Push to remote
git push -u origin main
```

### 4. Verify

```bash
# Check remote
git remote -v

# Check branch tracking
git branch -vv
```

## SSH Configuration Integration

The setup automatically integrates with SSH configuration:

### How It Works

1. **Repository Setup** creates local git config:
   ```bash
   git config core.sshCommand "ssh -i ~/.ssh/gitmanager/SKYREAPER-SPEC"
   ```

2. **SSH Config** provides host alias:
   ```ssh
   Host github-SKYREAPER-SPEC
       HostName github.com
       User git
       IdentityFile ~/.ssh/gitmanager/SKYREAPER-SPEC
   ```

3. **Git Push** uses the configured SSH key:
   ```bash
   git push -u origin main
   # Uses SSH key: ~/.ssh/gitmanager/SKYREAPER-SPEC
   ```

## Multiple Accounts Example

### Setup Repository 1 (Work Account)

```
Select account name: work-github
Enter repository name: company-project
Enter repository path: /home/user/work/company-project
Enter default branch: main
```

Result:
- SSH key: `~/.ssh/gitmanager/work-github`
- Remote: `git@github-work:company/company-project.git`

### Setup Repository 2 (Personal Account)

```
Select account name: personal-github
Enter repository name: personal-project
Enter repository path: /home/user/projects/personal-project
Enter default branch: main
```

Result:
- SSH key: `~/.ssh/gitmanager/personal-github`
- Remote: `git@github-personal:username/personal-project.git`

### Push to Different Remotes

```bash
# Work repository
cd /home/user/work/company-project
git push -u origin main
# Uses: ~/.ssh/gitmanager/work-github

# Personal repository
cd /home/user/projects/personal-project
git push -u origin main
# Uses: ~/.ssh/gitmanager/personal-github
```

## Configuration Files

### Repository Configuration

**File:** `~/.config/git-manager/repositories.json`

Stores:
- Repository name and path
- Associated account
- Default branch
- SSH key path
- Creation timestamp

### Git Configuration (Per Repository)

**File:** `.git/config` (in repository)

Contains:
- User name and email
- SSH command with specific key
- Remote URLs
- Branch tracking

### SSH Configuration

**File:** `~/.ssh/gitmanager/config`

Contains:
- Host aliases for each account
- SSH key paths
- GitHub/GitLab hostnames

## Troubleshooting

### "No accounts configured"

**Problem:** Setup wizard says no accounts exist

**Solution:** Create an account first
```bash
python3 -m git_manager --cli
# Option 8: Generate new SSH key
# Then save as account
```

### "Repository already exists"

**Problem:** Directory already exists at the path

**Solution:** Choose a different path or delete the existing directory

### "Git command not found"

**Problem:** Git is not installed

**Solution:** Install git
```bash
# Ubuntu/Debian
sudo apt-get install git

# macOS
brew install git

# Windows
# Download from https://git-scm.com/download/win
```

### "SSH key permission denied"

**Problem:** SSH key has wrong permissions

**Solution:** Fix permissions
```bash
chmod 600 ~/.ssh/gitmanager/*
chmod 700 ~/.ssh/gitmanager/
```

### "Push fails with authentication error"

**Problem:** SSH key not working with remote

**Solution:** Test SSH connection
```bash
ssh -T git@github-SKYREAPER-SPEC
```

If it fails, check:
1. SSH key is added to GitHub/GitLab account
2. SSH key path is correct
3. SSH key has proper permissions

## Advanced Usage

### Custom SSH Key Path

The setup uses the account's configured SSH key. To use a different key:

```bash
cd /path/to/repository
git config core.sshCommand "ssh -i /path/to/custom/key"
```

### Change Default Branch

After setup, change the default branch:

```bash
cd /path/to/repository
git branch -M new-branch-name
```

### Multiple Remotes

Add additional remotes:

```bash
cd /path/to/repository
git remote add upstream git@github-SKYREAPER-SPEC:original-owner/repo.git
git fetch upstream
```

### Staging Area Management

```bash
# See what will be committed
git status

# Stage specific files
git add file1.py file2.py

# Stage all changes
git add .

# Unstage files
git reset HEAD file.py

# Discard changes
git checkout -- file.py
```

## Summary

The Repository Setup Wizard (Option 5) provides:

✅ **Guided Setup** - Interactive step-by-step process
✅ **Automatic Initialization** - Git repo, config, initial commit
✅ **Account Integration** - Links repository to specific account
✅ **SSH Configuration** - Automatic SSH key setup
✅ **Branch Management** - Custom default branch
✅ **Configuration Storage** - Saves repository metadata
✅ **Next Steps Guide** - Clear instructions for pushing to remote

This makes it easy to set up new repositories with proper account configuration and SSH key management!
