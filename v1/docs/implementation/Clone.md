# GitManager Clone System: Complete Implementation Specification

## Executive Summary
Implement a comprehensive clone system for GitManager that handles multiple platforms (GitHub, GitLab, Bitbucket, Gitea, custom servers) with four distinct access scenarios, three authentication methods, and intelligent workflow automation.

---

## 1. Main Menu Integration

### Required Menu Flow
```
[1] Clone a repository (GitHub/GitLab)
    ↓
Clone Repository
===============
Choose clone type:
[1] Clone from external repository (any public/private repo)
[2] Clone from your personal repositories
[3] Back to main menu
```

**Implementation Hook:**
```python
# In your main menu handler
if choice == "1":  # Clone repository
    clone_workflow = CloneWorkflow(
        db_manager=your_db_manager,
        account_manager=your_account_manager,
        config_manager=your_config_manager
    )
    clone_workflow.start_clone_menu()
```

---

## 2. Four Core Access Scenarios (MANDATORY)

### Scenario 1: Private Repositories (Owner's Account) 🔒👤

**Requirements:**
- List ALL user's private repositories from their account
- Allow selection from list OR manual URL entry
- Support SSH and HTTPS PAT authentication
- Auto-configure git identity
- Track in GitManager database

**Workflow:**
```
1. User selects "Clone from your personal repositories"
2. Choose platform: [1] GitHub [2] GitLab [3] Bitbucket
3. Select account (if multiple for platform)
4. API fetches user's repositories (with PAT)
5. Display list showing:
   - Repository name
   - 🔒 Private indicator
   - Description (first 60 chars)
   - Last updated date
6. User selects repo from list OR enters URL manually
7. Choose auth method: [1] SSH (default) [2] HTTPS with PAT
8. Choose clone location (default: ~/projects/owner/repo)
9. Additional options (submodules, shallow clone)
10. Execute clone with selected authentication
11. Post-clone setup (git config, SSH setup, DB tracking)
12. Show next steps
```

**Key Features:**
- Pagination for users with 100+ repos
- Search/filter within repository list
- Show both private and public repos (with indicators)
- Sort by: most recent, name, size
- Cache repository list (5 min TTL)

---

### Scenario 2: Public Repositories (Owner's Account) 🌍👤

**Requirements:**
- Can clone with or without authentication
- If authenticated, setup push access automatically
- Faster clone without auth (no credential negotiation)

**Workflow:**
```
1. Included in personal repositories list
2. Show "🌍 Public" indicator
3. Prompt: "Clone with push access?" [Y/n]
   - Yes: Use SSH or PAT
   - No: Clone anonymously via HTTPS
4. If anonymous:
   - Inform: "Read-only clone - you won't be able to push"
   - Option to setup auth later: gitmanager setup-auth
```

---

### Scenario 3: Private Collaborative Repositories 🔒👥

**Requirements:**
- User must be added as collaborator
- Check access level via API (if available)
- Show collaboration details
- Use user's credentials

**Workflow:**
```
1. User enters URL of collaborative repo
2. Platform detected automatically
3. Check access via API:
   - If no access: Show error + "Request Access" option
   - If access: Show role (Read/Write/Admin)
4. Select account with access
5. Show collaboration info:
   - Repository owner
   - Your access level
   - Added by (if available)
6. Authentication methods available
7. Clone with proper credentials
8. Mark as "collaborative" in database
9. Show team workflow tips
```

**Team Workflow Tips:**
```
💡 Best Practices for Team Repositories:
• Use 'Pull with Rebase' to keep clean history
• Create feature branches for changes
• Sync frequently to avoid conflicts
• Follow team's branching strategy
```

---

### Scenario 4: Public Open Source Projects 🌍⭐

**Requirements:**
- Anonymous clone (no auth needed)
- Optional fork workflow for contributions
- Setup upstream remote automatically
- Track as external dependency

**Workflow:**
```
1. User enters URL: github.com/facebook/react
2. Detect as public repository
3. Fetch repo info via API (stars, description, license)
4. Ask intent:
   [1] 📖 Study/Use the code (read-only)
   [2] 🔧 Contribute (fork + clone)
   [3] 🏗️  Build upon it (clone + customize)
   
For STUDY:
- Clone anonymously
- No fork needed
- Read-only
- Track as dependency

For CONTRIBUTE:
- Check if user has account on platform
- Fork to user's account via API
- Clone user's fork with auth
- Add original as 'upstream' remote
- Setup contribution workflow:
  ```
  git remote -v
  origin    git@github.com:you/react.git (your fork)
  upstream  https://github.com/facebook/react.git (original)
  ```
- Show contribution guide

For BUILD UPON:
- Clone with optional auth
- Setup for independent development
```

---

## 3. Three Authentication Methods (MANDATORY)

### Method 1: SSH Key (RECOMMENDED) 🔑

**Requirements:**
- Use user's registered SSH key from `~/.ssh/gitmanager/`
- Set `GIT_SSH_COMMAND` environment variable
- Support multiple keys per platform
- Test SSH connection before clone

**Implementation:**
```python
def clone_with_ssh(repo_url, account):
    """Clone using SSH key"""
    # Get SSH key path
    ssh_key = account['ssh_key_path']  # e.g., ~/.ssh/gitmanager/github-main
    
    # Convert URL to SSH format if needed
    ssh_url = convert_to_ssh_url(repo_url)
    # https://github.com/user/repo → git@github.com:user/repo.git
    
    # Set up SSH command
    env = os.environ.copy()
    env['GIT_SSH_COMMAND'] = f'ssh -i {ssh_key} -o IdentitiesOnly=yes'
    
    # Clone
    subprocess.run(['git', 'clone', ssh_url, destination], env=env)
```

**Advantages:**
- No password needed
- Most secure
- Works with 2FA
- Recommended for all scenarios

---

### Method 2: HTTPS with Personal Access Token (PAT) 🎫

**Requirements:**
- Store PAT encrypted in database
- Inject token into HTTPS URL
- Support token expiration checking
- Work with 2FA-enabled accounts

**Implementation:**
```python
def clone_with_pat(repo_url, account):
    """Clone using Personal Access Token"""
    # Get PAT from account
    pat = account['pat_token']  # Stored encrypted
    
    # Parse URL
    parsed = parse_url(repo_url)
    # github.com/user/repo
    
    # Inject token into URL
    authenticated_url = f"https://{pat}@{parsed['host']}/{parsed['owner']}/{parsed['repo']}.git"
    
    # Clone
    subprocess.run(['git', 'clone', authenticated_url, destination])
    
    # CRITICAL: Remove token from git config after clone
    subprocess.run(['git', '-C', destination, 'remote', 'set-url', 'origin', 
                   f"https://{parsed['host']}/{parsed['owner']}/{parsed['repo']}.git"])
```

**Advantages:**
- Works everywhere
- Good for CI/CD
- Easy to revoke
- Fine-grained permissions

**Token Scopes Required:**
- GitHub: `repo` (full control of private repos)
- GitLab: `read_repository`, `write_repository`
- Bitbucket: Repository read/write

---

### Method 3: HTTPS with Password (GitLab Only) 🔐

**Requirements:**
- Only for GitLab (GitHub deprecated password auth)
- Prompt for password securely
- Don't store password
- Warn about 2FA requirement

**Implementation:**
```python
def clone_with_password(repo_url, account):
    """Clone using username/password (GitLab only)"""
    username = account['username']
    
    # Parse URL
    parsed = parse_url(repo_url)
    
    # Format URL with username
    url_with_user = f"https://{username}@{parsed['host']}/{parsed['owner']}/{parsed['repo']}.git"
    
    # Git will prompt for password interactively
    print("⚠️  GitLab will prompt for your password")
    print("💡 Tip: Use Personal Access Token instead for better security")
    
    subprocess.run(['git', 'clone', url_with_user, destination])
```

**Only Supported On:**
- GitLab (with username + password)
- Self-hosted GitLab instances
- Some custom Git servers

**Not Supported:**
- GitHub (deprecated August 2021)
- Bitbucket (requires App Passwords)

---

## 4. Multi-Platform Support (MANDATORY)

### Supported Platforms

#### GitHub
```python
GITHUB_CONFIG = {
    'api_base': 'https://api.github.com',
    'api_repos_endpoint': '/user/repos',
    'auth_header': 'Authorization: token {pat}',
    'ssh_host': 'github.com',
    'supports_password': False,
    'rate_limit': 5000  # per hour with auth
}
```

**API Integration:**
- List repositories: `GET /user/repos`
- Repository info: `GET /repos/{owner}/{repo}`
- Fork repository: `POST /repos/{owner}/{repo}/forks`
- Check access: `GET /repos/{owner}/{repo}/collaborators/{username}`

#### GitLab
```python
GITLAB_CONFIG = {
    'api_base': 'https://gitlab.com/api/v4',
    'api_projects_endpoint': '/projects',
    'auth_header': 'PRIVATE-TOKEN: {pat}',
    'ssh_host': 'gitlab.com',
    'supports_password': True,
    'rate_limit': 'none'  # No rate limit with auth
}
```

**API Integration:**
- List projects: `GET /projects?membership=true`
- Project info: `GET /projects/{id}`
- Fork project: `POST /projects/{id}/fork`
- Check access: `GET /projects/{id}/members/{user_id}`

#### Bitbucket
```python
BITBUCKET_CONFIG = {
    'api_base': 'https://api.bitbucket.org/2.0',
    'api_repos_endpoint': '/repositories/{username}',
    'auth_header': 'Authorization: Basic {base64(username:app_password)}',
    'ssh_host': 'bitbucket.org',
    'supports_password': False,  # Uses App Passwords
    'rate_limit': 1000  # per hour
}
```

#### Custom/Self-Hosted
```python
CUSTOM_CONFIG = {
    'api_base': None,  # User configures
    'ssh_host': None,  # User configures
    'supports_password': True,  # Usually
    'detection': 'manual'  # User specifies
}
```

### Platform Detection
```python
def detect_platform(url: str) -> str:
    """Auto-detect platform from URL"""
    if 'github.com' in url:
        return 'github'
    elif 'gitlab.com' in url:
        return 'gitlab'
    elif 'bitbucket.org' in url:
        return 'bitbucket'
    else:
        # Check for common self-hosted patterns
        if 'gitlab' in url.lower():
            return 'gitlab_selfhosted'
        elif 'gitea' in url.lower():
            return 'gitea'
        else:
            return 'custom'
```

---

## 5. Personal Repositories List Feature (CRITICAL)

### Requirements
When user selects "Clone from your personal repositories":

1. **Platform Selection**
```
Select platform:
[1] GitHub
[2] GitLab
[3] Bitbucket
[4] All platforms
```

2. **Account Selection** (if multiple)
```
Select account:
[1] @work-account (work@company.com)
[2] @personal-account (personal@gmail.com)
```

3. **Fetch Repositories via API**
```python
def fetch_personal_repositories(platform, account):
    """Fetch user's repositories from platform API"""
    
    if platform == 'github':
        headers = {'Authorization': f"token {account['pat_token']}"}
        response = requests.get(
            'https://api.github.com/user/repos',
            headers=headers,
            params={
                'per_page': 100,
                'sort': 'updated',
                'type': 'all'  # owner, member, all
            }
        )
        repos = response.json()
        
    elif platform == 'gitlab':
        headers = {'PRIVATE-TOKEN': account['pat_token']}
        response = requests.get(
            'https://gitlab.com/api/v4/projects',
            headers=headers,
            params={
                'membership': True,
                'per_page': 100,
                'order_by': 'updated_at'
            }
        )
        repos = response.json()
    
    # Transform to common format
    return normalize_repositories(repos, platform)
```

4. **Display Repository List**
```
📚 Your GitHub repositories (23 found):

[1]  🔒 my-private-app          Updated: 2 hours ago
     A private web application
     
[2]  🌍 open-source-tool        Updated: 1 day ago
     ⭐ 234  An open source utility
     
[3]  🔒 client-project          Updated: 3 days ago
     Client work - confidential
     
[4]  🌍 demo-repository         Updated: 1 week ago
     Demo for presentation

... (showing 1-10 of 23)

[N] Next page    [P] Previous page
[S] Search       [F] Filter
[M] Manual URL   [B] Back
```

5. **Search/Filter Options**
```
Search repositories:
  • By name: [Enter text]
  • By language: [JavaScript/Python/etc]
  • By visibility: [Private/Public/All]
  • By date: [Last week/month/year]
```

6. **Selection and Clone**
```
Select repository [1-10] or action [N/P/S/F/M/B]: 2

✓ Selected: open-source-tool
  Platform: GitHub
  Visibility: 🌍 Public
  SSH: git@github.com:you/open-source-tool.git
  HTTPS: https://github.com/you/open-source-tool.git

Authentication method:
[1] SSH (Recommended)
[2] HTTPS with PAT
[3] Anonymous (read-only)

Continue with clone? [Y/n]:
```

---

## 6. Complete Workflows

### Workflow A: Clone External Repository

```
User Action: Select option 1 from main menu

Step 1: Choose clone type
┌────────────────────────────────────────┐
│ Clone Repository                       │
├────────────────────────────────────────┤
│ [1] External repository                │
│ [2] Personal repositories              │
│ [3] Back                               │
└────────────────────────────────────────┘
User selects: 1

Step 2: Enter URL
📍 Enter repository URL:
   Examples:
   • github.com/user/repo
   • https://gitlab.com/user/project
   • git@bitbucket.org:user/repo.git

Input: github.com/facebook/react

Step 3: Analyze Repository
⏳ Analyzing repository...

✓ Detected platform: GitHub
  Owner: facebook
  Repository: react
  Visibility: 🌍 Public
  Stars: ⭐ 234k
  Description: The library for web and native user interfaces
  License: MIT

Step 4: Determine Intent
🎯 What do you want to do with this repository?

[1] 📖 Study/Use the code
    • Clone read-only
    • No fork needed
    • Fastest option

[2] 🔧 Contribute to the project
    • Fork to your account
    • Clone your fork
    • Setup upstream remote

[3] 🏗️  Build upon it
    • Clone and customize
    • Independent development

[4] ℹ️  Show more info
[5] ❌ Cancel

User selects: 2 (Contribute)

Step 5: Account Selection
👤 Select your GitHub account:

[1] @yourworkaccount (work@company.com)
    • SSH Key: ✓ Available
    • PAT: ✓ Valid until 2026-01-15

[2] @yourpersonal (personal@gmail.com)
    • SSH Key: ✓ Available
    • PAT: ✗ Expired

[3] ➕ Add new account

User selects: 1

Step 6: Fork Repository (for contribute intent)
⏳ Forking facebook/react to your account...
✓ Forked to yourworkaccount/react

Step 7: Authentication Method
🔐 Choose authentication method:

[1] 🔑 SSH (Recommended)
    ✓ Most secure
    ✓ No password needed
    ✓ Works with 2FA
    Using: ~/.ssh/gitmanager/github-work

[2] 🎫 HTTPS with Personal Access Token
    ✓ Works everywhere
    ✓ Easy to revoke
    Token: gh_****abcd1234

[3] ❌ Anonymous (not available - need push access)

User selects: 1 (SSH)

Step 8: Clone Location
📁 Where should we clone the repository?

Default: ~/projects/yourworkaccount/react

[Enter] Use default
[C] Choose custom location
[T] Use template (~/work/projects/)

User presses: Enter

Step 9: Additional Options
⚙️  Clone Options:

☑ Clone submodules recursively
☑ Set up upstream remote (facebook/react)
☐ Shallow clone (--depth=1)
☐ Install dependencies after clone

[A] Accept and continue
[E] Edit options
[C] Cancel

User selects: A

Step 10: Execute Clone
⏳ Cloning repository...

✓ Created directory ~/projects/yourworkaccount/react
⏳ Cloning from git@github.com:yourworkaccount/react.git
  Progress: [████████████████████] 100%
  Received: 56.8 MB
  Objects: 23,100
  Time: 12 seconds

✓ Clone complete

Step 11: Post-Clone Setup
⚙️  Configuring repository...

✓ Set git user.name: Your Name
✓ Set git user.email: work@company.com
✓ Added upstream remote: facebook/react
✓ Configured SSH for this repository
✓ Added to GitManager tracking database

Step 12: Success Summary
╔════════════════════════════════════════╗
║  Clone Successful! ✓                   ║
╚════════════════════════════════════════╝

📁 Location: ~/projects/yourworkaccount/react

🔗 Remotes:
  origin   → yourworkaccount/react (your fork)
  upstream → facebook/react (original)

📊 Repository Info:
  • Default branch: main
  • Size: 56.8 MB
  • Commits: 23,100
  • Last updated: 2 hours ago

💡 Next Steps:
  1. cd ~/projects/yourworkaccount/react
  2. Create a feature branch: git checkout -b my-feature
  3. Make your changes
  4. Push to your fork: git push origin my-feature
  5. Create PR to facebook/react

📝 GitManager Commands:
  • Check status: gitmanager status
  • Push changes: gitmanager push
  • Sync with upstream: gitmanager sync
  • Switch account: gitmanager switch-account

[O] Open in editor    [T] Open terminal    [D] Done
```

---

### Workflow B: Clone Personal Repository

```
User Action: Select option 1 from main menu

Step 1: Choose clone type
User selects: 2 (Personal repositories)

Step 2: Platform Selection
📊 Select platform:

[1] 🐙 GitHub (3 accounts configured)
[2] 🦊 GitLab (1 account configured)
[3] 🗃️  Bitbucket (0 accounts)
[4] 🌐 All platforms
[5] ← Back

User selects: 1 (GitHub)

Step 3: Account Selection (if multiple)
👤 Select GitHub account:

[1] @workaccount (work@company.com)
    • 45 repositories
    • Last used: 2 hours ago

[2] @personalaccount (personal@gmail.com)
    • 23 repositories
    • Last used: 2 days ago

[3] @clientaccount (client@project.com)
    • 12 repositories
    • Last used: 1 week ago

User selects: 1

Step 4: Fetch and Display Repositories
⏳ Fetching your repositories from GitHub...
✓ Found 45 repositories

📚 Your GitHub repositories (@workaccount):

┌──────────────────────────────────────────────────────────┐
│ [1]  🔒 backend-api               Updated: 2 hours ago    │
│      REST API for main application                        │
│      Python • 2.3 MB • main                              │
│                                                           │
│ [2]  🔒 frontend-dashboard        Updated: 5 hours ago    │
│      Admin dashboard with React                           │
│      JavaScript • 15.6 MB • develop                      │
│                                                           │
│ [3]  🌍 company-website           Updated: 1 day ago      │
│      Public company website                               │
│      HTML/CSS • 892 KB • main                            │
│                                                           │
│ [4]  🔒 data-pipeline             Updated: 2 days ago     │
│      ETL pipeline for analytics                           │
│      Python • 5.4 MB • main                              │
│                                                           │
│ [5]  🔒 mobile-app                Updated: 3 days ago     │
│      React Native mobile app                              │
│      TypeScript • 42.1 MB • staging                      │
│                                                           │
│     ... showing 5 of 45 repositories                      │
├──────────────────────────────────────────────────────────┤
│ [N] Next page    [P] Prev page    [1-5] Select           │
│ [S] Search       [F] Filter       [M] Manual URL         │
│ [R] Refresh      [B] Back                                │
└──────────────────────────────────────────────────────────┘

User selects: 1 (backend-api)

Step 5: Repository Selected
✓ Selected: backend-api

📊 Repository Details:
  Owner: workaccount
  Name: backend-api
  Visibility: 🔒 Private
  Language: Python
  Size: 2.3 MB
  Default branch: main
  Last commit: 2 hours ago
  Description: REST API for main application

Step 6: Authentication Method
🔐 Authentication method for private repository:

[1] 🔑 SSH (Recommended)
    Using: ~/.ssh/gitmanager/github-work
    
[2] 🎫 HTTPS with Personal Access Token
    Token: gh_****abcd1234 (valid until 2026-01-15)

User selects: 1 (SSH)

Step 7: Clone Location
📁 Clone location:

Default: ~/projects/workaccount/backend-api

[Enter] Use default
[C] Custom location

User presses: Enter

Step 8: Additional Options
⚙️  Options:

☑ Clone submodules
☑ Setup for immediate work (configure git identity)
☐ Shallow clone
☐ Install dependencies (detected: requirements.txt)

User accepts

Step 9: Execute Clone
⏳ Cloning private repository...

✓ Using SSH key: ~/.ssh/gitmanager/github-work
⏳ Cloning git@github.com:workaccount/backend-api.git
  Progress: [████████████████████] 100%
  Received: 2.3 MB
  Time: 2 seconds

✓ Clone complete
✓ Configured git identity
✓ Added to GitManager tracking

Step 10: Success
╔════════════════════════════════════════╗
║  Clone Successful! ✓                   ║
╚════════════════════════════════════════╝

📁 ~/projects/workaccount/backend-api

🔐 Access: Configured with SSH
📧 Identity: work@company.com

[O] Open in editor    [T] Open terminal    [D] Done
```

---

## 7. Error Handling (MANDATORY)

### Common Errors and Solutions

#### Error 1: Authentication Failed
```
❌ Authentication Failed

Problem: Could not authenticate with GitHub

Possible causes:
  • SSH key not added to GitHub account
  • SSH key has wrong permissions (should be 600)
  • PAT is invalid or expired
  • Wrong username/password

💡 Solutions:
  [1] Test SSH connection: ssh -T git@github.com
  [2] Add SSH key to GitHub: https://github.com/settings/keys
  [3] Generate new PAT: https://github.com/settings/tokens
  [4] Try different authentication method

[R] Retry with different method    [H] Help    [C] Cancel
```

#### Error 2: Permission Denied
```
❌ Permission Denied

Problem: You don't have access to this repository

Repository: company/private-repo
Platform: GitHub

Possible causes:
  • Repository is private and you're not a collaborator
  • You're using wrong account
  • Organization requires SSO authentication

💡 Solutions:
  [1] Request access from repository owner
  [2] Check if using correct account
  [3] If org repo: Enable SSO for your PAT
  [4] Verify repository URL is correct

[A] Request access    [S] Switch account    [C] Cancel
```

#### Error 3: Repository Not Found
```
❌ Repository Not Found

URL: github.com/user/nonexistent-repo

Possible causes:
  • Repository doesn't exist
  • Repository is private (appears as "not found")
  • URL is misspelled
  • Repository was deleted or renamed

💡 Solutions:
  [1] Verify URL on GitHub website
  [2] Check if you have access (may need auth)
  [3] Try with authentication
  [4] Search for repository

[S] Search GitHub    [R] Retry with auth    [C] Cancel
```

#### Error 4: Network Issues
```
❌ Network Error

Problem: Failed to connect to github.com

Possible causes:
  • No internet connection
  • GitHub is down
  • Firewall blocking connection
  • Proxy configuration needed

💡 Solutions:
  [1] Check internet connection
  [2] Check GitHub status: https://githubstatus.com
  [3] Check firewall settings
  [4] Configure proxy if needed
  [5] Try again in a moment

[R] Retry    [C] Check status    [P] Configure proxy    [X] Cancel
```

#### Error 5: Disk Space
```
❌ Insufficient Disk Space

Repository size: 2.5 GB
Available space: 1.2 GB

💡 Solutions:
  [1] Free up disk space
  [2] Use shallow clone (--depth=1)
      Estimated size: ~500 MB
  [3] Clone to different location
  [4] Use sparse checkout (clone specific folders)

[S] Shallow clone    [L] Different location    [F] Free space    [C] Cancel
```

---

## 8. Database Schema (MANDATORY)

```sql
-- Platforms
CREATE TABLE platforms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    platform_id TEXT UNIQUE NOT NULL,  -- 'github', 'gitlab', etc.
    name TEXT NOT NULL,
    api_base_url TEXT,
    ssh_host TEXT,
    supports_password BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Accounts
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    platform_id TEXT NOT NULL,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT,
    
    -- Authentication
    ssh_key_path TEXT,
    ssh_key_name TEXT,
    pat_token TEXT,  -- Encrypted
    pat_expires_at TIMESTAMP,
    password_saved BOOLEAN DEFAULT FALSE,
    
    -- Metadata
    avatar_url TEXT,
    profile_url TEXT,
    api_rate_limit INTEGER,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    last_used TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (platform_id) REFERENCES platforms(platform_id),
    UNIQUE(platform_id, username)
);

-- Repositories
CREATE TABLE repositories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    
    -- Repository info
    platform_id TEXT NOT NULL,
    account_id INTEGER,
    owner TEXT NOT NULL,
    name TEXT NOT NULL,
    full_name TEXT NOT NULL,  -- 'owner/name'
    
    -- URLs
    clone_url TEXT NOT NULL,
    ssh_url TEXT,
    https_url TEXT,
    web_url TEXT,
    
    -- Classification
    repository_type TEXT NOT NULL,  -- 'owned_private', 'owned_public', 'collaborative', 'external'
    visibility TEXT NOT NULL,  -- 'private', 'public'
    access_level TEXT,  -- 'owner', 'write', 'read', 'none'
    is_fork BOOLEAN DEFAULT FALSE,
    
    -- Clone details
    clone_method TEXT NOT NULL,  -- 'ssh', 'https_pat', 'https_password', 'anonymous'
    clone_depth INTEGER,  -- NULL for full clone
    cloned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Repository metadata
    default_branch TEXT DEFAULT 'main',
    description TEXT,
    language TEXT,
    size_kb INTEGER,
    stars INTEGER DEFAULT 0,
    last_commit_at TIMESTAMP,
    
    -- Purpose/Intent
    clone_intent TEXT,  -- 'study', 'contribute', 'work', 'build'
    tags TEXT,  -- JSON array
    notes TEXT,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    last_synced TIMESTAMP,
    
    FOREIGN KEY (platform_id) REFERENCES platforms(platform_id),
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

-- Remotes
CREATE TABLE remotes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repository_id INTEGER NOT NULL,
    name TEXT NOT NULL,  -- 'origin', 'upstream', etc.
    url TEXT NOT NULL,
    is_default BOOLEAN DEFAULT FALSE,
    
    FOREIGN KEY (repository_id) REFERENCES repositories(id),
    UNIQUE(repository_id, name)
);

-- Clone operations log
CREATE TABLE clone_operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repository_id INTEGER,
    account_id INTEGER,
    
    -- Operation details
    clone_url TEXT NOT NULL,
    destination TEXT NOT NULL,
    method TEXT NOT NULL,
    platform_id TEXT NOT NULL,
    
    -- Options
    options_json TEXT,  -- JSON with clone options
    
    -- Result
    status TEXT NOT NULL,  -- 'success', 'failed', 'cancelled'
    error_message TEXT,
    error_type TEXT,  -- 'auth', 'network', 'permission', 'not_found', 'disk', 'other'
    
    -- Performance
    duration_seconds INTEGER,
    bytes_transferred INTEGER,
    
    -- Timestamps
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    
    FOREIGN KEY (repository_id) REFERENCES repositories(id),
    FOREIGN KEY (account_id) REFERENCES accounts(id),
    FOREIGN KEY (platform_id) REFERENCES platforms(platform_id)
);

-- Repository cache (for personal repos list)
CREATE TABLE repository_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    platform_id TEXT NOT NULL,
    
    -- Cached data
    repositories_json TEXT NOT NULL,  -- JSON array of repos
    total_count INTEGER,
    
    -- Cache metadata
    cached_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP,
    
    FOREIGN KEY (account_id) REFERENCES accounts(id),
    FOREIGN KEY (platform_id) REFERENCES platforms(platform_id),
    UNIQUE(account_id, platform_id)
);
```

## 9. Key Implementation Details

### URL Parser
```python
class URLParser:
    """Parse and normalize Git repository URLs"""
    
    @staticmethod
    def parse(url: str) -> Dict[str, str]:
        """
        Parse any Git URL format
        
        Supported formats:
        - github.com/user/repo
        - https://github.com/user/repo
        - https://github.com/user/repo.git
        - git@github.com:user/repo.git
        - user@git.company.com:repo.git
        """
        
        url = url.strip().rstrip('/')
        
        # SSH format: git@host:owner/repo.git
        ssh_pattern = r'(?:git@|([^@]+)@)([^:]+):([^/]+)/(.+?)(?:\.git)?
        ssh_match = re.match(ssh_pattern, url)
        if ssh_match:
            username, host, owner, repo = ssh_match.groups()
            return {
                'format': 'ssh',
                'host': host,
                'owner': owner,
                'repo': repo.replace('.git', ''),
                'username': username or 'git'
            }
        
        # HTTPS format: https://host/owner/repo.git
        https_pattern = r'https?://([^/]+)/([^/]+)/(.+?)(?:\.git)?
        https_match = re.match(https_pattern, url)
        if https_match:
            host, owner, repo = https_match.groups()
            return {
                'format': 'https',
                'host': host,
                'owner': owner,
                'repo': repo.replace('.git', '')
            }
        
        # Shorthand: host/owner/repo
        short_pattern = r'([^/]+\.[^/]+)/([^/]+)/(.+?)(?:\.git)?
        short_match = re.match(short_pattern, url)
        if short_match:
            host, owner, repo = short_match.groups()
            return {
                'format': 'short',
                'host': host,
                'owner': owner,
                'repo': repo.replace('.git', '')
            }
        
        raise ValueError(f"Invalid Git URL: {url}")
    
    @staticmethod
    def to_ssh(parsed: Dict) -> str:
        """Convert to SSH URL"""
        return f"git@{parsed['host']}:{parsed['owner']}/{parsed['repo']}.git"
    
    @staticmethod
    def to_https(parsed: Dict) -> str:
        """Convert to HTTPS URL"""
        return f"https://{parsed['host']}/{parsed['owner']}/{parsed['repo']}.git"
```

### Repository List Fetcher
```python
class RepositoryFetcher:
    """Fetch user's repositories from platform APIs"""
    
    def __init__(self, cache_manager):
        self.cache = cache_manager
    
    def fetch_repositories(self, platform: str, account: dict, 
                          force_refresh: bool = False) -> List[dict]:
        """
        Fetch repositories with caching
        
        Returns normalized list of repositories
        """
        
        # Check cache first
        if not force_refresh:
            cached = self.cache.get(account['id'], platform)
            if cached and not self.cache.is_expired(cached):
                return json.loads(cached['repositories_json'])
        
        # Fetch from API
        if platform == 'github':
            repos = self._fetch_github(account)
        elif platform == 'gitlab':
            repos = self._fetch_gitlab(account)
        elif platform == 'bitbucket':
            repos = self._fetch_bitbucket(account)
        else:
            raise ValueError(f"Unsupported platform: {platform}")
        
        # Normalize format
        normalized = self._normalize_repositories(repos, platform)
        
        # Cache results
        self.cache.set(account['id'], platform, normalized, ttl=300)  # 5 min
        
        return normalized
    
    def _normalize_repositories(self, repos: List[dict], platform: str) -> List[dict]:
        """
        Normalize repository data to common format
        
        Common format:
        {
            'id': int,
            'name': str,
            'full_name': str,
            'owner': str,
            'description': str,
            'visibility': 'private' | 'public',
            'language': str,
            'size_kb': int,
            'stars': int,
            'updated_at': datetime,
            'ssh_url': str,
            'https_url': str,
            'web_url': str,
            'is_fork': bool,
            'default_branch': str
        }
        """
        normalized = []
        
        for repo in repos:
            if platform == 'github':
                normalized.append({
                    'id': repo['id'],
                    'name': repo['name'],
                    'full_name': repo['full_name'],
                    'owner': repo['owner']['login'],
                    'description': repo.get('description', ''),
                    'visibility': 'private' if repo['private'] else 'public',
                    'language': repo.get('language', 'Unknown'),
                    'size_kb': repo['size'],
                    'stars': repo['stargazers_count'],
                    'updated_at': repo['updated_at'],
                    'ssh_url': repo['ssh_url'],
                    'https_url': repo['clone_url'],
                    'web_url': repo['html_url'],
                    'is_fork': repo['fork'],
                    'default_branch': repo['default_branch']
                })
            
            elif platform == 'gitlab':
                normalized.append({
                    'id': repo['id'],
                    'name': repo['name'],
                    'full_name': repo['path_with_namespace'],
                    'owner': repo['namespace']['path'],
                    'description': repo.get('description', ''),
                    'visibility': repo['visibility'],
                    'language': 'Multiple' if not repo.get('language') else repo['language'],
                    'size_kb': repo.get('statistics', {}).get('repository_size', 0) // 1024,
                    'stars': repo['star_count'],
                    'updated_at': repo['last_activity_at'],
                    'ssh_url': repo['ssh_url_to_repo'],
                    'https_url': repo['http_url_to_repo'],
                    'web_url': repo['web_url'],
                    'is_fork': 'forked_from_project' in repo,
                    'default_branch': repo['default_branch']
                })
            
            # ... similar for other platforms
        
        return normalized
```

### Interactive Repository Selector
```python
class RepositorySelector:
    """Interactive repository selection UI"""
    
    def __init__(self):
        self.current_page = 1
        self.items_per_page = 10
        self.search_term = ""
        self.filter_visibility = "all"  # 'all', 'private', 'public'
    
    def display_and_select(self, repositories: List[dict]) -> Optional[dict]:
        """
        Display repository list and get user selection
        
        Returns selected repository or None
        """
        
        # Apply filters
        filtered = self._apply_filters(repositories)
        
        # Paginate
        total_pages = (len(filtered) + self.items_per_page - 1) // self.items_per_page
        start_idx = (self.current_page - 1) * self.items_per_page
        end_idx = start_idx + self.items_per_page
        page_items = filtered[start_idx:end_idx]
        
        while True:
            # Clear screen
            os.system('clear' if os.name != 'nt' else 'cls')
            
            # Header
            print("📚 Your Repositories")
            print("=" * 70)
            
            if self.search_term:
                print(f"🔍 Search: '{self.search_term}'")
            if self.filter_visibility != 'all':
                print(f"🔽 Filter: {self.filter_visibility}")
            
            print(f"\nShowing {start_idx + 1}-{min(end_idx, len(filtered))} of {len(filtered)} repositories")
            print()
            
            # Display repositories
            for idx, repo in enumerate(page_items, 1):
                visibility_icon = "🔒" if repo['visibility'] == 'private' else "🌍"
                
                print(f"[{idx}] {visibility_icon} {repo['name']}")
                
                # Description (truncated)
                if repo['description']:
                    desc = repo['description'][:60]
                    if len(repo['description']) > 60:
                        desc += "..."
                    print(f"    {desc}")
                
                # Metadata
                updated = self._format_time_ago(repo['updated_at'])
                print(f"    {repo['language']} • {self._format_size(repo['size_kb'])} • Updated: {updated}")
                
                if repo['is_fork']:
                    print(f"    🔱 Forked repository")
                
                if repo['stars'] > 0:
                    print(f"    ⭐ {repo['stars']} stars")
                
                print()
            
            # Navigation
            print("=" * 70)
            nav_options = []
            
            if self.current_page > 1:
                nav_options.append("[P] Previous page")
            if self.current_page < total_pages:
                nav_options.append("[N] Next page")
            
            nav_options.extend([
                "[S] Search",
                "[F] Filter",
                "[R] Refresh",
                "[M] Manual URL",
                "[B] Back"
            ])
            
            print("  ".join(nav_options))
            print()
            
            # Get input
            choice = input("Select [1-10] or action: ").strip().upper()
            
            # Handle navigation
            if choice == 'N' and self.current_page < total_pages:
                self.current_page += 1
                continue
            elif choice == 'P' and self.current_page > 1:
                self.current_page -= 1
                continue
            elif choice == 'S':
                self._handle_search()
                self.current_page = 1
                filtered = self._apply_filters(repositories)
                continue
            elif choice == 'F':
                self._handle_filter()
                self.current_page = 1
                filtered = self._apply_filters(repositories)
                continue
            elif choice == 'R':
                return 'REFRESH'
            elif choice == 'M':
                return 'MANUAL'
            elif choice == 'B':
                return None
            
            # Handle selection
            try:
                selection = int(choice)
                if 1 <= selection <= len(page_items):
                    return page_items[selection - 1]
                else:
                    print(f"❌ Invalid selection. Choose 1-{len(page_items)}")
                    input("Press Enter to continue...")
            except ValueError:
                print("❌ Invalid input")
                input("Press Enter to continue...")
    
    def _apply_filters(self, repositories: List[dict]) -> List[dict]:
        """Apply search and filter"""
        filtered = repositories
        
        # Search filter
        if self.search_term:
            filtered = [
                r for r in filtered
                if self.search_term.lower() in r['name'].lower() or
                   self.search_term.lower() in r.get('description', '').lower()
            ]
        
        # Visibility filter
        if self.filter_visibility != 'all':
            filtered = [
                r for r in filtered
                if r['visibility'] == self.filter_visibility
            ]
        
        return filtered
    
    def _handle_search(self):
        """Handle search input"""
        print("\n🔍 Search repositories:")
        print("  • Enter text to search in name and description")
        print("  • Leave empty to clear search")
        self.search_term = input("Search: ").strip()
    
    def _handle_filter(self):
        """Handle filter selection"""
        print("\n🔽 Filter by visibility:")
        print("[1] All repositories")
        print("[2] Private only")
        print("[3] Public only")
        
        choice = input("\nSelect [1-3]: ").strip()
        
        if choice == '1':
            self.filter_visibility = 'all'
        elif choice == '2':
            self.filter_visibility = 'private'
        elif choice == '3':
            self.filter_visibility = 'public'
    
    @staticmethod
    def _format_time_ago(timestamp: str) -> str:
        """Format timestamp as 'X time ago'"""
        from datetime import datetime, timezone
        
        # Parse ISO timestamp
        dt = datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
        now = datetime.now(timezone.utc)
        delta = now - dt
        
        if delta.days > 365:
            years = delta.days // 365
            return f"{years} year{'s' if years > 1 else ''} ago"
        elif delta.days > 30:
            months = delta.days // 30
            return f"{months} month{'s' if months > 1 else ''} ago"
        elif delta.days > 0:
            return f"{delta.days} day{'s' if delta.days > 1 else ''} ago"
        elif delta.seconds > 3600:
            hours = delta.seconds // 3600
            return f"{hours} hour{'s' if hours > 1 else ''} ago"
        elif delta.seconds > 60:
            minutes = delta.seconds // 60
            return f"{minutes} minute{'s' if minutes > 1 else ''} ago"
        else:
            return "just now"
    
    @staticmethod
    def _format_size(size_kb: int) -> str:
        """Format size in human readable format"""
        if size_kb < 1024:
            return f"{size_kb} KB"
        elif size_kb < 1024 * 1024:
            return f"{size_kb / 1024:.1f} MB"
        else:
            return f"{size_kb / (1024 * 1024):.1f} GB"
```

---

## 10. Success Criteria

The implementation is considered complete when:

✅ **Functionality**
- [ ] All four access scenarios work correctly
- [ ] All three authentication methods work
- [ ] Personal repositories can be listed and selected
- [ ] External repositories can be cloned
- [ ] Fork workflow works for contributions
- [ ] Post-clone setup configures everything correctly

✅ **Platform Support**
- [ ] GitHub fully supported (API + clone)
- [ ] GitLab fully supported (API + clone)
- [ ] Bitbucket supported (API + clone)
- [ ] Custom Git servers supported (basic clone)

✅ **User Experience**
- [ ] Clear, intuitive menu flow
- [ ] Helpful error messages with solutions
- [ ] Progress indication during clone
- [ ] Repository list is fast and searchable
- [ ] Success messages show next steps

✅ **Reliability**
- [ ] Error handling for all common scenarios
- [ ] Graceful degradation when APIs unavailable
- [ ] Cache works correctly
- [ ] Database tracking accurate
- [ ] No data loss on failures

✅ **Security**
- [ ] PAT tokens encrypted in database
- [ ] SSH keys have correct permissions
- [ ] Tokens removed from git config after clone
- [ ] No passwords stored
- [ ] Sensitive data not logged

---

## 11. Final Integration Example

```python
# In your main CLI file

from git_manager.clone.workflow import CloneWorkflow

def handle_main_menu():
    """Main menu handler"""
    
    while True:
        print("\nGit Multi-Account Manager v1.0.0")
        print("=" * 60)
        print("\n[1] Clone a repository (GitHub/GitLab)")
        print("[2] Check current repository account")
        print("[3] Git push/pull/sync")
        # ... other options
        
        choice = input("\nSelect option [1-8]: ").strip()
        
        if choice == "1":
            # Initialize clone workflow
            clone_workflow = CloneWorkflow(
                db_manager=db,
                account_manager=accounts,
                config_manager=config
            )
            
            # Start clone menu
            clone_workflow.start_clone_menu()
        
        # ... handle other options

if __name__ == "__main__":
    handle_main_menu()
```

---

## END OF SPECIFICATION

This specification provides everything needed to implement a complete, production-ready clone system for GitManager. All requirements, workflows, error handling, and implementation details are included.