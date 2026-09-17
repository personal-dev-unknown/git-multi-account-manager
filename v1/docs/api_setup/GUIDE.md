# GitHub & GitLab API Endpoints Guide

## 📋 Complete API Reference

### GitHub API Base URL
```
https://api.github.com
```

### GitLab API Base URL
```
https://gitlab.com/api/v4
```

---

## 1. 🗂️ List Repositories

### GitHub - List User Repositories
```bash
# List authenticated user's repos
GET /user/repos

# List specific user's public repos
GET /users/{username}/repos

# List organization repos
GET /orgs/{org}/repos
```

**Examples:**
```bash
# Your repos (requires authentication)
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.github.com/user/repos?per_page=100

# Public repos of a user
curl https://api.github.com/users/devonionMoses/repos

# With parameters
curl -H "Authorization: Bearer YOUR_TOKEN" \
  "https://api.github.com/user/repos?type=all&sort=updated&per_page=100&page=1"
```

**Query Parameters:**
- `type`: all, owner, public, private, member (default: owner)
- `sort`: created, updated, pushed, full_name (default: full_name)
- `direction`: asc, desc
- `per_page`: 1-100 (default: 30)
- `page`: page number

### GitLab - List Projects
```bash
# List all projects (user must be authenticated)
GET /projects

# List user's projects
GET /users/{user_id}/projects

# List group projects
GET /groups/{group_id}/projects
```

**Examples:**
```bash
# Your projects
curl --header "PRIVATE-TOKEN: YOUR_TOKEN" \
  "https://gitlab.com/api/v4/projects?per_page=100"

# With visibility filter
curl --header "PRIVATE-TOKEN: YOUR_TOKEN" \
  "https://gitlab.com/api/v4/projects?visibility=private&order_by=last_activity_at"

# Specific user's projects
curl "https://gitlab.com/api/v4/users/devchiwhale/projects"
```

**Query Parameters:**
- `visibility`: public, internal, private
- `order_by`: id, name, path, created_at, updated_at, last_activity_at
- `sort`: asc, desc
- `search`: search term
- `simple`: true (returns minimal fields)
- `owned`: true (only owned projects)
- `starred`: true (only starred projects)

---

## 2. 👤 User Information & Settings

### GitHub - Get User Info
```bash
# Get authenticated user
GET /user

# Get specific user
GET /users/{username}

# Get user emails
GET /user/emails
```

**Examples:**
```bash
# Your profile
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.github.com/user

# User's email addresses
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.github.com/user/emails

# Specific user
curl https://api.github.com/users/devonionMoses
```

**Response includes:**
- `login`, `id`, `name`, `email`
- `public_repos`, `total_private_repos`
- `created_at`, `updated_at`
- `bio`, `location`, `blog`

### GitLab - Get User Info
```bash
# Get current user
GET /user

# Get specific user
GET /users/{id}

# Get user's events
GET /users/{id}/events
```

**Examples:**
```bash
# Your profile
curl --header "PRIVATE-TOKEN: YOUR_TOKEN" \
  https://gitlab.com/api/v4/user

# Get user emails
curl --header "PRIVATE-TOKEN: YOUR_TOKEN" \
  https://gitlab.com/api/v4/user/emails

# Specific user by username
curl "https://gitlab.com/api/v4/users?username=devchiwhale"
```

---

## 3. 🔑 SSH Keys & Personal Access Tokens

### GitHub - SSH Keys

#### List SSH Keys
```bash
GET /user/keys
```

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/user/keys
```

#### Add SSH Key
```bash
POST /user/keys
```

```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/user/keys \
  -d '{
    "title": "My Dev Machine",
    "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl [email protected]"
  }'
```

#### Delete SSH Key
```bash
DELETE /user/keys/{key_id}
```

```bash
curl -X DELETE \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/user/keys/12345
```

### GitHub - Personal Access Tokens

**⚠️ Important:** GitHub **DEPRECATED** the API for creating Personal Access Tokens programmatically. You **must** create them via the web UI.

**Create Token via Web UI:**
1. Go to: https://github.com/settings/tokens
2. Click "Generate new token" → "Generate new token (classic)"
3. Select scopes:
   - `repo` - Full control of private repositories
   - `write:public_key` - Write access to SSH keys
   - `read:org` - Read organization data
   - `user` - Update user data
4. Click "Generate token"
5. **Copy immediately** - You can't see it again!

**Required Scopes for Common Operations:**
- List/Create repos: `repo` or `public_repo`
- Manage SSH keys: `admin:public_key` or `write:public_key`
- User info: `read:user` or `user`
- Organizations: `read:org`

### GitLab - SSH Keys

#### List SSH Keys
```bash
GET /user/keys
```

```bash
curl --header "PRIVATE-TOKEN: YOUR_TOKEN" \
  https://gitlab.com/api/v4/user/keys
```

#### Add SSH Key
```bash
POST /user/keys
```

```bash
curl -X POST \
  --header "PRIVATE-TOKEN: YOUR_TOKEN" \
  --header "Content-Type: application/json" \
  --data '{
    "title": "My Dev Machine",
    "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl",
    "expires_at": "2025-12-31"
  }' \
  https://gitlab.com/api/v4/user/keys
```

#### Delete SSH Key
```bash
DELETE /user/keys/{key_id}
```

```bash
curl -X DELETE \
  --header "PRIVATE-TOKEN: YOUR_TOKEN" \
  https://gitlab.com/api/v4/user/keys/25
```

### GitLab - Personal Access Tokens

**⚠️ Important:** GitLab also requires web UI for creating Personal Access Tokens for security reasons.

**Create Token via Web UI:**
1. Go to: https://gitlab.com/-/profile/personal_access_tokens
2. Fill in:
   - Name: "My Dev Token"
   - Expiration date: (required)
   - Scopes: api, read_api, read_repository, write_repository
3. Click "Create personal access token"
4. **Copy immediately**!

**Required Scopes:**
- `api` - Full API access (includes all below)
- `read_api` - Read-only API access
- `read_repository` - Read repositories
- `write_repository` - Write to repositories
- `read_user` - Read user profile
- `sudo` - Perform API actions as any user (admin only)

---

## 4. 🆕 Create Repositories

### GitHub - Create Repository

#### Create for authenticated user
```bash
POST /user/repos
```

```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/user/repos \
  -d '{
    "name": "my-new-repo",
    "description": "This is my new repository",
    "homepage": "https://github.com",
    "private": false,
    "has_issues": true,
    "has_projects": true,
    "has_wiki": true,
    "auto_init": true,
    "gitignore_template": "Python",
    "license_template": "mit"
  }'
```

#### Create in organization
```bash
POST /orgs/{org}/repos
```

```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Accept: application/vnd.github+json" \
  https://api.github.com/orgs/my-org/repos \
  -d '{
    "name": "team-project",
    "private": true,
    "auto_init": true
  }'
```

**Parameters:**
- `name` (required): Repository name
- `description`: Description
- `homepage`: Homepage URL
- `private`: true/false (default: false)
- `visibility`: public, private, internal
- `auto_init`: Create initial commit with README
- `gitignore_template`: .gitignore template (Python, Node, etc.)
- `license_template`: mit, apache-2.0, gpl-3.0, etc.
- `has_issues`, `has_projects`, `has_wiki`: Enable features

### GitLab - Create Project

```bash
POST /projects
```

```bash
curl -X POST \
  --header "PRIVATE-TOKEN: YOUR_TOKEN" \
  --header "Content-Type: application/json" \
  --data '{
    "name": "my-new-project",
    "description": "My new GitLab project",
    "path": "my-new-project",
    "visibility": "private",
    "initialize_with_readme": true,
    "default_branch": "main",
    "namespace_id": 42
  }' \
  https://gitlab.com/api/v4/projects
```

#### Create in group
```bash
curl -X POST \
  --header "PRIVATE-TOKEN: YOUR_TOKEN" \
  --header "Content-Type: application/json" \
  --data '{
    "name": "team-project",
    "namespace_id": 123,
    "visibility": "private"
  }' \
  https://gitlab.com/api/v4/projects
```

**Parameters:**
- `name` (required): Project name
- `path`: URL path (defaults to name)
- `namespace_id`: Group/user ID to create under
- `description`: Description
- `visibility`: public, internal, private
- `initialize_with_readme`: Create README
- `default_branch`: Default branch name
- `issues_enabled`, `wiki_enabled`, `merge_requests_enabled`

---

## 5. 📊 Repository Details

### GitHub - Get Repository
```bash
GET /repos/{owner}/{repo}
```

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.github.com/repos/devonionMoses/my-repo
```

### GitHub - List Branches
```bash
GET /repos/{owner}/{repo}/branches
```

### GitHub - List Commits
```bash
GET /repos/{owner}/{repo}/commits
```

### GitLab - Get Project
```bash
GET /projects/{id}
```

```bash
curl --header "PRIVATE-TOKEN: YOUR_TOKEN" \
  https://gitlab.com/api/v4/projects/123
```

### GitLab - List Branches
```bash
GET /projects/{id}/repository/branches
```

### GitLab - List Commits
```bash
GET /projects/{id}/repository/commits
```

---

## 📝 Complete Implementation Examples

See the Python implementation artifacts for:
1. GitHub API Client
2. GitLab API Client  
3. Unified API Manager
4. CLI Integration

---

## 🔒 Security Best Practices

### Token Storage
```python
# ❌ NEVER hardcode tokens
token = "ghp_xxxxxxxxxxxx"

# ✅ Use environment variables
import os
token = os.getenv('GITHUB_TOKEN')

# ✅ Use config files (with proper permissions)
from pathlib import Path
token = (Path.home() / '.config/git-manager/token').read_text().strip()
```

### Token Permissions
- **Principle of Least Privilege**: Only grant necessary scopes
- **Separate tokens** for different purposes
- **Set expiration dates** when possible
- **Rotate tokens** regularly

### Rate Limiting
- GitHub: 5,000 requests/hour (authenticated), 60/hour (unauthenticated)
- GitLab: 300 requests/minute per IP address
- Implement exponential backoff
- Cache responses when appropriate

---

## 🚀 Quick Start URLs

### GitHub
- **Create Token**: https://github.com/settings/tokens
- **API Documentation**: https://docs.github.com/en/rest
- **SSH Keys**: https://github.com/settings/keys
- **Developer Settings**: https://github.com/settings/developers

### GitLab
- **Create Token**: https://gitlab.com/-/profile/personal_access_tokens
- **API Documentation**: https://docs.gitlab.com/ee/api/
- **SSH Keys**: https://gitlab.com/-/profile/keys
- **User Settings**: https://gitlab.com/-/profile

---

## 📚 Additional Resources

### API Libraries
- **GitHub**: PyGithub, Octokit, github3.py
- **GitLab**: python-gitlab, gitlab-api

### Testing APIs
- **GitHub**: https://docs.github.com/en/rest/overview/testing-the-rest-api
- **GitLab**: Use your personal tokens with curl
- **Postman Collections**: Available for both platforms

### Webhooks
- Configure webhooks for CI/CD integration
- Real-time notifications on repo events
- Trigger custom workflows