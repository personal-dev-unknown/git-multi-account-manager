# src/git_manager/utils/constants.py
"""Constants and configuration."""

from enum import Enum
from pathlib import Path
from typing import Dict


# Application info
APP_NAME = "Git Multi-Account Manager"
APP_VERSION = "1.0.0"
APP_AUTHOR = "DevonionMoses"

# Paths
HOME_DIR = Path.home()
CONFIG_DIR = HOME_DIR / '.git-manager'
ACCOUNTS_FILE = CONFIG_DIR / 'accounts.json'
CONFIG_FILE = CONFIG_DIR / 'config.json'

# Logging paths (determined at runtime by LogStorageManager)
# LOG_FILE = CONFIG_DIR / 'git-manager.log'  # Deprecated, use LogStorageManager instead

# SSH
SSH_DIR = HOME_DIR / '.ssh'
SSH_CONFIG = SSH_DIR / 'config'
DEFAULT_SSH_KEY_TYPE = 'ed25519'
RSA_KEY_SIZE = 4096
SSH_KEY_TYPES = ["ed25519", "rsa"]

# Git
DEFAULT_BRANCH = 'main'
DEFAULT_REMOTE = 'origin'

class Platform(str, Enum):
    """Supported Git platforms."""
    GITHUB = "github"
    GITLAB = "gitlab"
    BITBUCKET = "bitbucket"
    AZURE_DEVOPS = "azure_devops"
    SELF_HOSTED = "self_hosted"
    CLOUD_STORAGE = "cloud_storage"
    LOCAL_PATH = "local_path"
    SOURCEFORGE = "sourceforge"

class GitOperation(str, Enum):
    """Git operation types."""
    CLONE = "clone"
    PULL = "pull"
    PUSH = "push"
    STATUS = "status"
    COMMIT = "commit"

class RepoStatus(str, Enum):
    """Repository status codes."""
    CLEAN = "clean"
    UNCOMMITTED = "uncommitted"
    AHEAD = "ahead"
    BEHIND = "behind"
    DIVERGED = "diverged"
    NO_UPSTREAM = "no_upstream"

# Configuration paths
# CONFIG_DIR = "~/.gitmulti"
# CONFIG_FILE = "config.yaml"
# ACCOUNTS_FILE = "accounts.yaml"
# LOG_FILE = "/var/log/gitmulti.log"

# Color scheme
COLORS: Dict[str, str] = {
    'primary': '#1B4332',      # Dark Green
    'secondary': '#9A5324',    # Rust
    'accent': '#2D6A6A',       # Teal
    'text': '#6B4423',         # Brown
    'background': '#4A4A4A',   # Gray
    'success': '#2D6A6A',      # Teal
    'warning': '#9A5324',      # Rust
    'error': '#8B0000',        # Dark Red
    'info': '#1B4332'          # Dark Green
}

# Terminal colors (ANSI)
TERMINAL_COLORS = {
    'primary': '\033[38;2;27;67;50m',      # Dark Green
    'secondary': '\033[38;2;154;83;36m',   # Rust
    'accent': '\033[38;2;45;106;106m',     # Teal
    'text': '\033[38;2;107;68;35m',        # Brown
    'background': '\033[48;2;74;74;74m',   # Gray
    'success': '\033[38;2;45;106;106m',    # Teal
    'warning': '\033[38;2;154;83;36m',     # Rust
    'error': '\033[38;2;139;0;0m',         # Dark Red
    'info': '\033[38;2;27;67;50m',         # Dark Green
    'reset': '\033[0m'
}

# API
API_VERSION = 'v1'
API_BASE_PATH = f'/api/{API_VERSION}'

# Platform Configuration
PLATFORM_CONFIG = {
    'github': {
        'name': 'GitHub',
        'api_url': 'https://api.github.com',
        'ssh_host': 'github.com',
        'https_host': 'github.com',
        'supports_ssh': True,
        'supports_https': True,
        'supports_pat': True,
        'description': 'GitHub - Popular open-source and private development platform'
    },
    'gitlab': {
        'name': 'GitLab',
        'api_url': 'https://gitlab.com/api/v4',
        'ssh_host': 'gitlab.com',
        'https_host': 'gitlab.com',
        'supports_ssh': True,
        'supports_https': True,
        'supports_pat': True,
        'description': 'GitLab - Integrated CI/CD pipelines and DevOps platform'
    },
    'bitbucket': {
        'name': 'Bitbucket',
        'api_url': 'https://api.bitbucket.org/2.0',
        'ssh_host': 'bitbucket.org',
        'https_host': 'bitbucket.org',
        'supports_ssh': True,
        'supports_https': True,
        'supports_pat': True,
        'description': 'Bitbucket - Jira and Trello integration, free private repos'
    },
    'azure_devops': {
        'name': 'Azure DevOps',
        'api_url': 'https://dev.azure.com',
        'ssh_host': 'ssh.dev.azure.com',
        'https_host': 'dev.azure.com',
        'supports_ssh': True,
        'supports_https': True,
        'supports_pat': True,
        'description': 'Azure DevOps - Microsoft enterprise Git hosting with CI/CD'
    },
    'self_hosted': {
        'name': 'Self-Hosted Server',
        'api_url': None,
        'ssh_host': None,
        'https_host': None,
        'supports_ssh': True,
        'supports_https': True,
        'supports_pat': False,
        'description': 'Self-hosted Git server on private machine or VPS'
    },
    'cloud_storage': {
        'name': 'Cloud Storage',
        'api_url': None,
        'ssh_host': None,
        'https_host': None,
        'supports_ssh': False,
        'supports_https': True,
        'supports_pat': False,
        'description': 'Cloud storage services (S3, etc.) with Git configurations'
    },
    'local_path': {
        'name': 'Local Path',
        'api_url': None,
        'ssh_host': None,
        'https_host': None,
        'supports_ssh': False,
        'supports_https': False,
        'supports_pat': False,
        'description': 'Local directory or network share as Git remote'
    },
    'sourceforge': {
        'name': 'SourceForge',
        'api_url': 'https://sourceforge.net/api',
        'ssh_host': 'git.code.sf.net',
        'https_host': 'git.code.sf.net',
        'supports_ssh': True,
        'supports_https': True,
        'supports_pat': False,
        'description': 'SourceForge - Open-source project hosting'
    }
}

# API URLs (legacy, kept for backward compatibility)
# GITHUB_API = "https://api.github.com"
# GITLAB_API = "https://gitlab.com/api/v4"