# Git Manager API Setup & Usage Guide

## 🚀 Quick Start

### 1. Install Dependencies

```bash
pip install requests click rich
```

### 2. Get API Tokens

#### GitHub Personal Access Token
1. Go to: https://github.com/settings/tokens
2. Click "Generate new token" → "Generate new token (classic)"
3. Give it a name: "Git Manager CLI"
4. Select scopes:
   - ✅ `repo` - Full control of repositories
   - ✅ `admin:public_key` - Manage SSH keys
   - ✅ `user` - User profile access
5. Click "Generate token"
6. **COPY IT IMMEDIATELY!** (You can't see it again)

#### GitLab Personal Access Token
1. Go to: https://gitlab.com/-/profile/personal_access_tokens
2. Fill in:
   - Name: "Git Manager CLI"
   - Expiration: Select a date (required)
   - Scopes:
     - ✅ `api` - Full API access
     - ✅ `read_api` - Read API
     - ✅ `read_repository` - Read repos
     - ✅ `write_repository` - Write repos
3. Click "Create personal access token"
4. **COPY IT IMMEDIATELY!**

### 3. Configure Tokens

#### Option A: Environment Variables (Recommended)
```bash
# Add to ~/.bashrc or ~/.zshrc
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"
export GITLAB_TOKEN="glpat-xxxxxxxxxxxxxxxxxxxx"

# Reload shell
source ~/.bashrc
```

#### Option B: Configuration File
```bash
# Create secure config directory
mkdir -p ~/.config/git-manager
chmod 700 ~/.config/git-manager

# Save tokens
echo "ghp_xxxxxxxxxxxxxxxxxxxx" > ~/.config/git-manager/github_token
echo "glpat-xxxxxxxxxxxxxxxxxxxx" > ~/.config/git-manager/gitlab_token

# Secure permissions
chmod 600 ~/.config/git-manager/*_token
```

## 📦 Project Structure

```
git-multi-account-manager/
├── src/
│   └── git_manager/
│       ├── api/
│       │   ├── __init__.py
│       │   ├── github_client.py      # GitHub API client
│       │   ├── gitlab_client.py      # GitLab API client
│       │   └── api_manager.py        # Unified manager
│       └── cli/
│           └── commands/
│               └── api_commands.py   # CLI commands
└── requirements.txt
```

## 💻 Usage Examples

### Python API Usage

#### Basic Repository Listing

```python
from git_manager.api.api_manager import APIManager

# Initialize manager (reads from environment variables)
api = APIManager()

# List your GitHub repos
github_repos = api.list_repos('github', per_page=10)
for repo in github_repos:
    print(f"{repo.name}: {repo.ssh_url}")

# List your GitLab projects
gitlab_projects = api.list_repos('gitlab', per_page=10)
for project in gitlab_projects:
    print(f"{project.name}: {project.ssh_url_to_repo}")
```

#### Create Repository

```python
# Create GitHub repo
new_repo = api.create_repo(
    'github',
    name='my-awesome-project',
    description='My new project',
    private=True,
    auto_init=True,
    gitignore_template='Python',
    license_template='mit'
)
print(f"Created: {new_repo.html_url}")

# Create GitLab project
new_project = api.create_repo(
    'gitlab',
    name='my-awesome-project',
    description='My new project',
    visibility='private',
    initialize_with_readme=True
)
print(f"Created: {new_project.web_url}")
```

#### Manage SSH Keys

```python
# List SSH keys
github_keys = api.list_ssh_keys('github')
for key in github_keys:
    print(f"{key.title} (ID: {key.id})")

# Add new SSH key
from pathlib import Path

key_path = Path.home() / '.ssh' / 'id_ed25519.pub'
new_key = api.add_ssh_key(
    'github',
    title='My Dev Machine',
    key=key_path.read_text().strip()
)
print(f"Added key: {new_key.title}")

# Sync key to multiple platforms
results = api.sync_ssh_key_to_platforms(
    title='Development Machine',
    key_path=key_path,
    platforms=['github', 'gitlab']
)
print(f"Sync results: {results}")
```

#### Get User Information

```python
# GitHub user info
github_user = api.get_user('github')
print(f"GitHub: {github_user['login']} - {github_user.get('name')}")
print(f"Public repos: {github_user['public_repos']}")

# GitLab user info
gitlab_user = api.get_user('gitlab')
print(f"GitLab: {gitlab_user['username']} - {gitlab_user.get('name')}")
```

### CLI Usage

```bash
# List repositories
git-manager api list-repos --platform github
git-manager api list-repos --platform gitlab --username devonionMoses

# Create repository
git-manager api create-repo --platform github --name test-repo --private
git-manager api create-repo --platform gitlab --name test-project -d "My test"

# List SSH keys
git-manager api list-keys --platform github
git-manager api list-keys --platform gitlab

# Add SSH key
git-manager api add-key --platform github \
    --title "My Machine" \
    --key-file ~/.ssh/id_ed25519.pub

# Get user info
git-manager api user-info --platform github
git-manager api user-info --platform gitlab
```

## 🔧 Advanced Usage

### Bulk Repository Clone

```python
from git_manager.api.api_manager import APIManager
import subprocess

api = APIManager()

# Get all repos
repos = api.list_repos('github', type='all', per_page=100)

# Clone all private repos
for repo in repos:
    if repo.private:
        print(f"Cloning {repo.name}...")
        subprocess.run(['git', 'clone', repo.ssh_url])
```

### Automated Backup

```python
from git_manager.api.api_manager import APIManager
from pathlib import Path
import json
from datetime import datetime

api = APIManager()

# Backup GitHub repos metadata
github_repos = api.list_repos('github', type='all', per_page=100)
gitlab_projects = api.list_repos('gitlab', per_page=100)

backup_data = {
    'timestamp': datetime.now().isoformat(),
    'github': [
        {
            'name': repo.name,
            'ssh_url': repo.ssh_url,
            'private': repo.private,
            'description': repo.description
        }
        for repo in github_repos
    ],
    'gitlab': [
        {
            'name': proj.name,
            'ssh_url': proj.ssh_url_to_repo,
            'visibility': proj.visibility,
            'description': proj.description
        }
        for proj in gitlab_projects
    ]
}

# Save backup
backup_file = Path.home() / 'repos_backup.json'
backup_file.write_text(json.dumps(backup_data, indent=2))
print(f"Backup saved to: {backup_file}")
```

### Repository Migration

```python
from git_manager.api.api_manager import APIManager
import subprocess

api = APIManager()

# Get repo from GitHub
github_repo = api.github.get_repo('olduser', 'old-repo')

# Create on GitLab
new_project = api.create_repo(
    'gitlab',
    name=github_repo.name,
    description=github_repo.description,
    visibility='private'
)

# Clone and push
subprocess.run(['git', 'clone', github_repo.ssh_url])
subprocess.run(['git', 'remote', 'add', 'gitlab', new_project.ssh_url_to_repo],
               cwd=github_repo.name)
subprocess.run(['git', 'push', 'gitlab', '--all'], cwd=github_repo.name)
```

## 🛠️ Integration with Existing CLI

### Add to Interactive Mode

```python
# In src/git_manager/cli/ui/interactive.py

def handle_choice(self, choice: str):
    """Handle menu choice."""
    actions = {
        '1': self.clone_repo,
        '2': self.check_status,
        # ... existing actions ...
        '9': self.api_operations,  # NEW
        '10': self.exit
    }
    
    action = actions.get(choice)
    if action:
        try:
            action()
        except Exception as e:
            self.console.print(f"[error]Error: {e}[/error]")

def api_operations(self):
    """API operations submenu."""
    from ...api.api_manager import APIManager
    
    api = APIManager()
    
    self.console.print("\n[cyan]═══ API Operations ═══[/cyan]")
    self.console.print("[1] List GitHub repos")
    self.console.print("[2] List GitLab projects")
    self.console.print("[3] Create repository")
    self.console.print("[4] Manage SSH keys")
    self.console.print("[5] Back to main menu")
    
    choice = Prompt.ask("Select option", choices=['1','2','3','4','5'])
    
    if choice == '1':
        repos = api.list_repos('github', per_page=20)
        from ..tables import display_repos_table
        display_repos_table(repos, 'github', self.console)
    # ... implement other options
```

## 🔒 Security Best Practices

### Token Management

```python
# ✅ GOOD: Load from secure location
import os
token = os.getenv('GITHUB_TOKEN')

# ✅ GOOD: Load from secure config file
from pathlib import Path
token = (Path.home() / '.config/git-manager/github_token').read_text().strip()

# ❌ BAD: Never hardcode tokens
token = "ghp_xxxxxxxxxxxx"  # DON'T DO THIS!

# ❌ BAD: Never commit tokens to git
# Always add token files to .gitignore
```

### Token Rotation Script

```python
# rotate_tokens.py
from pathlib import Path
from datetime import datetime

def rotate_token(platform: str, new_token: str):
    """Rotate API token with backup."""
    config_dir = Path.home() / '.config/git-manager'
    token_file = config_dir / f'{platform}_token'
    backup_dir = config_dir / 'backups'
    backup_dir.mkdir(exist_ok=True)
    
    # Backup old token
    if token_file.exists():
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        backup_file = backup_dir / f'{platform}_token.{timestamp}.bak'
        backup_file.write_text(token_file.read_text())
        print(f"Old token backed up to: {backup_file}")
    
    # Write new token
    token_file.write_text(new_token)
    token_file.chmod(0o600)
    print(f"New {platform} token saved")

# Usage
rotate_token('github', 'ghp_new_token_here')
```

## 📊 Rate Limiting Handling

```python
from git_manager.api.github_client import GitHubAPIClient
import time

client = GitHubAPIClient(token)

# Check rate limit
rate_limit = client.get_rate_limit()
remaining = rate_limit['rate']['remaining']
reset_time = rate_limit['rate']['reset']

print(f"Requests remaining: {remaining}")
print(f"Reset time: {datetime.fromtimestamp(reset_time)}")

# Wait if needed
if remaining < 10:
    wait_time = reset_time - time.time()
    print(f"Waiting {wait_time} seconds for rate limit reset...")
    time.sleep(wait_time)
```

## 🐛 Error Handling

```python
from requests.exceptions import HTTPError

try:
    repos = api.list_repos('github')
except HTTPError as e:
    if e.response.status_code == 401:
        print("Authentication failed. Check your token.")
    elif e.response.status_code == 403:
        print("Permission denied. Check token scopes.")
    elif e.response.status_code == 404:
        print("Resource not found.")
    elif e.response.status_code == 429:
        print("Rate limit exceeded. Wait and retry.")
    else:
        print(f"API error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## 📚 Additional Resources

### Official Documentation
- **GitHub API**: https://docs.github.com/en/rest
- **GitLab API**: https://docs.gitlab.com/ee/api/

### Python Libraries
- **PyGithub**: https://pygithub.readthedocs.io/
- **python-gitlab**: https://python-gitlab.readthedocs.io/

### Testing
```bash
# Test GitHub connection
python -c "from git_manager.api.api_manager import APIManager; \
           api = APIManager(); \
           print(api.get_user('github')['login'])"

# Test GitLab connection
python -c "from git_manager.api.api_manager import APIManager; \
           api = APIManager(); \
           print(api.get_user('gitlab')['username'])"
```

## 🎯 Next Steps

1. **Create tokens** on GitHub and GitLab
2. **Save tokens** securely (environment variables or config file)
3. **Test connection** with the Python examples
4. **Integrate** into your CLI application
5. **Implement** additional features as needed

Need help? Check the artifacts for complete implementation code!