# Git Multi-Account Manager - Platform Support

## Overview

The Git Multi-Account Manager now supports **8 major Git hosting platforms** and repository types.

---

## Supported Platforms

### 1. GitHub
- **Key:** `github`
- **API URL:** `https://api.github.com`
- **SSH Host:** `github.com`
- **Features:** SSH, HTTPS, PAT, OAuth
- **Description:** Popular open-source and private development platform

### 2. GitLab
- **Key:** `gitlab`
- **API URL:** `https://gitlab.com/api/v4`
- **SSH Host:** `gitlab.com`
- **Features:** SSH, HTTPS, PAT, Integrated CI/CD, Self-hosted support
- **Description:** Integrated CI/CD pipelines and DevOps platform

### 3. Bitbucket
- **Key:** `bitbucket`
- **API URL:** `https://api.bitbucket.org/2.0`
- **SSH Host:** `bitbucket.org`
- **Features:** SSH, HTTPS, PAT, Jira/Trello integration
- **Description:** Robust integration with Jira and Trello

### 4. Azure DevOps
- **Key:** `azure_devops`
- **API URL:** `https://dev.azure.com`
- **SSH Host:** `ssh.dev.azure.com`
- **Features:** SSH, HTTPS, PAT, Azure Pipelines, Enterprise-grade
- **Description:** Microsoft's enterprise Git hosting platform

### 5. Self-Hosted Server
- **Key:** `self_hosted`
- **Features:** SSH, HTTPS, Custom infrastructure
- **Description:** Self-hosted Git server (Gitea, Gitolite, Gogs)

### 6. Cloud Storage
- **Key:** `cloud_storage`
- **Features:** HTTPS only (S3, Google Cloud Storage)
- **Description:** Cloud storage services with Git configurations

### 7. Local Path
- **Key:** `local_path`
- **Features:** Local filesystem or network share
- **Description:** Local directory or network share as Git remote

### 8. SourceForge
- **Key:** `sourceforge`
- **API URL:** `https://sourceforge.net/api`
- **SSH Host:** `git.code.sf.net`
- **Features:** SSH, HTTPS
- **Description:** Open-source project hosting platform

---

## Platform Capabilities Matrix

| Platform | SSH | HTTPS | PAT | API | Self-Hosted |
|----------|-----|-------|-----|-----|-------------|
| GitHub | ✅ | ✅ | ✅ | ✅ | ❌ |
| GitLab | ✅ | ✅ | ✅ | ✅ | ✅ |
| Bitbucket | ✅ | ✅ | ✅ | ✅ | ❌ |
| Azure DevOps | ✅ | ✅ | ✅ | ✅ | ❌ |
| Self-Hosted | ✅ | ✅ | ❌ | ⚠️ | ✅ |
| Cloud Storage | ❌ | ✅ | ❌ | ❌ | ✅ |
| Local Path | ❌ | ❌ | ❌ | ❌ | ✅ |
| SourceForge | ✅ | ✅ | ❌ | ✅ | ❌ |

---

## Usage Examples

### List Available Platforms
```bash
git-manager account platforms
```

### Add GitHub Account
```bash
git-manager account add \
  --name github-personal \
  --platform github \
  --username john-doe \
  --ssh-key ~/.ssh/id_ed25519_github
```

### Add Self-Hosted Account
```bash
git-manager account add \
  --name internal-git \
  --platform self_hosted \
  --username john \
  --ssh-key ~/.ssh/id_ed25519_internal \
  --host git.internal.company.com
```

### Add Local Path Account
```bash
git-manager account add \
  --name local-backup \
  --platform local_path \
  --username local \
  --ssh-key ~/.ssh/id_ed25519_local
```

### Clone from Any Platform
```bash
# GitHub
git-manager clone https://github.com/user/repo --account github-personal

# GitLab
git-manager clone https://gitlab.com/user/repo --account gitlab-work

# Self-Hosted
git-manager clone https://git.company.com/user/repo --account internal-git

# Local Path
git-manager clone /mnt/backup/repo.git --account local-backup
```

---

## Integration Points

### Core Modules Updated
- **models/account.py** - Platform enum expanded to 8 platforms
- **utils/constants.py** - Platform configuration dictionary added
- **core/platform_config.py** - New PlatformManager class for platform management
- **utils/platform_helpers.py** - Platform-specific utility functions
- **utils/validators.py** - URL validation for all platforms
- **core/git_operations.py** - Enhanced URL conversion for all platforms
- **core/account_manager.py** - Multi-platform account management

### CLI Commands Updated
- **account list** - Filter by any platform
- **account add** - Support all 8 platforms
- **account platforms** - New command to list available platforms
- **ssh setup-account** - Support all 8 platforms

### Interactive Mode Updated
- Menu now shows 8 platforms supported
- Platform selection in interactive workflows

---

## Implementation Details

### Platform Configuration
Each platform is defined with:
- **key** - Unique identifier (e.g., 'github', 'self_hosted')
- **name** - Human-readable name
- **api_url** - API endpoint (if available)
- **ssh_host** - SSH hostname (if supported)
- **https_host** - HTTPS hostname (if supported)
- **supports_ssh** - SSH authentication support
- **supports_https** - HTTPS authentication support
- **supports_pat** - Personal Access Token support
- **description** - Platform description

### URL Handling
- Automatic platform detection from URLs
- URL conversion between SSH and HTTPS formats
- Support for short format (owner/repo)
- Local path validation
- Self-hosted URL detection

### Account Management
- Store platform-specific SSH hosts
- Support custom host aliases (e.g., github.com-work)
- PAT token storage for supported platforms
- Platform validation on account creation

---

## Future Enhancements

- API integration for repository listing
- Automatic SSH key upload to platforms
- Repository sync across platforms
- Platform-specific authentication flows
- Custom platform configuration
