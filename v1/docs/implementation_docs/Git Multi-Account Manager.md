# Git Multi-Account Manager - Complete Documentation

## Table of Contents
1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [Folder Structure](#folder-structure)
4. [Features](#features)
5. [Installation](#installation)
6. [Usage](#usage)
7. [API Reference](#api-reference)
8. [Development Guide](#development-guide)

---

## Project Overview

**Git Multi-Account Manager** is a comprehensive cross-platform application for managing multiple Git accounts (GitHub & GitLab) with SSH authentication. Built with Python, it provides three interfaces:
- Terminal CLI
- Web Application
- Desktop GUI (Cross-platform)

### Technology Stack
- **Core**: Python 3.9+
- **CLI**: Rich, Click
- **Web**: Flask, Flask-SocketIO, Jinja2
- **Desktop**: PyQt6 or Tkinter
- **Database**: SQLite (optional for config storage)
- **Testing**: Pytest
- **Documentation**: Sphinx

### Color Scheme
- **Dark Green**: `#1B4332` (Primary)
- **Rust**: `#9A5324` (Secondary)
- **Teal**: `#2D6A6A` (Accent)
- **Brown**: `#6B4423` (Text/Borders)
- **Gray**: `#4A4A4A` (Background/Neutral)

---

## Architecture

### Design Patterns
1. **MVC Pattern**: Separation of concerns
2. **Repository Pattern**: Data access abstraction
3. **Factory Pattern**: Interface creation
4. **Strategy Pattern**: Platform-specific operations
5. **Observer Pattern**: Event handling

### Core Components

```
┌─────────────────────────────────────────────┐
│           User Interfaces                   │
│  ┌─────────┐  ┌─────────┐  ┌─────────────┐ │
│  │   CLI   │  │   Web   │  │   Desktop   │ │
│  └────┬────┘  └────┬────┘  └──────┬──────┘ │
└───────┼───────────┼───────────────┼─────────┘
        │           │               │
        └───────────┴───────────────┘
                    │
        ┌───────────▼────────────┐
        │    Core Application    │
        │  ┌──────────────────┐  │
        │  │  Account Manager │  │
        │  │  SSH Manager     │  │
        │  │  Git Operations  │  │
        │  │  Config Manager  │  │
        │  └──────────────────┘  │
        └───────────┬────────────┘
                    │
        ┌───────────▼────────────┐
        │    Infrastructure      │
        │  ┌──────────────────┐  │
        │  │  File System     │  │
        │  │  Git Commands    │  │
        │  │  SSH Operations  │  │
        │  │  Logger          │  │
        │  └──────────────────┘  │
        └────────────────────────┘
```

---

## Folder Structure

```
git-multi-account-manager/
│
├── README.md
├── LICENSE
├── setup.py
├── requirements.txt
├── requirements-dev.txt
├── .gitignore
├── .env.example
├── pyproject.toml
├── pytest.ini
│
├── docs/
│   ├── conf.py
│   ├── index.rst
│   ├── installation.rst
│   ├── usage.rst
│   ├── api.rst
│   └── contributing.rst
│
├── src/
│   └── git_manager/
│       ├── __init__.py
│       ├── main.py
│       │
│       ├── core/
│       │   ├── __init__.py
│       │   ├── account_manager.py
│       │   ├── ssh_manager.py
│       │   ├── git_operations.py
│       │   ├── config_manager.py
│       │   ├── repository_manager.py
│       │   └── exceptions.py
│       │
│       ├── models/
│       │   ├── __init__.py
│       │   ├── account.py
│       │   ├── repository.py
│       │   ├── ssh_key.py
│       │   └── config.py
│       │
│       ├── utils/
│       │   ├── __init__.py
│       │   ├── logger.py
│       │   ├── validators.py
│       │   ├── file_operations.py
│       │   ├── git_helpers.py
│       │   ├── ssh_helpers.py
│       │   └── constants.py
│       │
│       ├── cli/
│       │   ├── __init__.py
│       │   ├── app.py
│       │   ├── commands/
│       │   │   ├── __init__.py
│       │   │   ├── clone.py
│       │   │   ├── account.py
│       │   │   ├── repository.py
│       │   │   ├── ssh.py
│       │   │   └── config.py
│       │   └── ui/
│       │       ├── __init__.py
│       │       ├── colors.py
│       │       ├── tables.py
│       │       ├── prompts.py
│       │       └── progress.py
│       │
│       ├── web/
│       │   ├── __init__.py
│       │   ├── app.py
│       │   ├── routes/
│       │   │   ├── __init__.py
│       │   │   ├── main.py
│       │   │   ├── accounts.py
│       │   │   ├── repositories.py
│       │   │   ├── ssh.py
│       │   │   └── api.py
│       │   ├── templates/
│       │   │   ├── base.html
│       │   │   ├── index.html
│       │   │   ├── accounts/
│       │   │   │   ├── list.html
│       │   │   │   ├── add.html
│       │   │   │   └── detail.html
│       │   │   ├── repositories/
│       │   │   │   ├── list.html
│       │   │   │   ├── clone.html
│       │   │   │   └── detail.html
│       │   │   └── ssh/
│       │   │       ├── keys.html
│       │   │       ├── generate.html
│       │   │       └── test.html
│       │   └── static/
│       │       ├── css/
│       │       │   ├── main.css
│       │       │   ├── components.css
│       │       │   └── themes.css
│       │       ├── js/
│       │       │   ├── main.js
│       │       │   ├── api.js
│       │       │   └── components.js
│       │       └── img/
│       │           └── logo.png
│       │
│       └── desktop/
│           ├── __init__.py
│           ├── app.py
│           ├── windows/
│           │   ├── __init__.py
│           │   ├── main_window.py
│           │   ├── account_window.py
│           │   ├── repository_window.py
│           │   └── ssh_window.py
│           ├── widgets/
│           │   ├── __init__.py
│           │   ├── account_list.py
│           │   ├── repository_list.py
│           │   ├── ssh_key_list.py
│           │   └── terminal_widget.py
│           └── resources/
│               ├── icons/
│               │   ├── app.ico
│               │   ├── github.png
│               │   └── gitlab.png
│               └── styles/
│                   └── main.qss
│
├── tests/
│   ├── __init__.py
│   ├── conftest.py
│   ├── unit/
│   │   ├── test_account_manager.py
│   │   ├── test_ssh_manager.py
│   │   ├── test_git_operations.py
│   │   └── test_validators.py
│   ├── integration/
│   │   ├── test_clone_flow.py
│   │   ├── test_ssh_flow.py
│   │   └── test_repository_setup.py
│   └── e2e/
│       ├── test_cli.py
│       ├── test_web.py
│       └── test_desktop.py
│
├── scripts/
│   ├── install.sh
│   ├── install.ps1
│   ├── build.py
│   └── release.py
│
└── config/
    ├── accounts.example.json
    ├── config.example.json
    └── logging.yaml
```

---

## Features

### 1. Account Management
- Add/Remove/Edit GitHub and GitLab accounts
- Store account credentials securely
- Validate account configurations
- Test account connectivity

### 2. SSH Key Management
- Generate ED25519 SSH keys
- Store and manage multiple keys
- Test SSH connections
- Auto-configure SSH config file
- Passphrase management

### 3. Repository Operations
- Clone repositories (personal/external)
- Check current repository account
- Git pull with verification
- Git push with verification
- Setup repository for specific account
- Repository status checking

### 4. Multi-Platform Support
- GitHub
- GitLab
- Custom Git servers (future)

### 5. User Interfaces

#### Terminal (CLI)
- Rich terminal output with colors
- Interactive prompts
- Progress indicators
- Command history
- Tab completion

#### Web Application
- Responsive design
- Real-time updates with WebSockets
- RESTful API
- Dark/Light theme toggle
- Mobile-friendly

#### Desktop Application
- Native look and feel
- System tray integration
- Keyboard shortcuts
- Drag-and-drop support
- Cross-platform (Windows, macOS, Linux)

---

## Installation

### Prerequisites
```bash
Python 3.9+
Git 2.25+
SSH client
```

### From PyPI (Recommended)
```bash
pip install git-multi-account-manager
```

### From Source
```bash
# Clone repository
git clone https://github.com/yourusername/git-multi-account-manager.git
cd git-multi-account-manager

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -e ".[dev]"
```

### Platform-Specific Installation

#### Windows
```powershell
# Run installer script
.\scripts\install.ps1
```

#### macOS/Linux
```bash
# Run installer script
chmod +x scripts/install.sh
./scripts/install.sh
```

---

## Usage

### Terminal (CLI)

#### Basic Commands
```bash
# Start interactive mode
git-manager

# Clone repository
git-manager clone <url> --account <account-name>

# Check current repository
git-manager status

# Add account
git-manager account add --name "work" --platform github --username "myuser"

# Generate SSH key
git-manager ssh generate --email "user@example.com" --name "work-key"

# Test SSH connection
git-manager ssh test --account work

# Pull with verification
git-manager pull

# Push with verification
git-manager push
```

#### Advanced Usage
```bash
# Clone from personal repositories
git-manager clone --personal --account work

# Setup repository for specific account
git-manager repo setup --account work

# List all accounts
git-manager account list

# Export configuration
git-manager config export --output backup.json

# Import configuration
git-manager config import --input backup.json
```

### Web Application

```bash
# Start web server
git-manager web --host 0.0.0.0 --port 5000

# Start with debug mode
git-manager web --debug

# Access at: http://localhost:5000
```

#### Web Features
- Dashboard with account overview
- Repository browser
- SSH key manager
- Real-time operation status
- API documentation at `/api/docs`

### Desktop Application

```bash
# Launch desktop app
git-manager desktop

# Launch with specific account
git-manager desktop --account work
```

#### Desktop Features
- Account management panel
- Repository explorer
- Integrated terminal
- SSH key generation wizard
- Settings panel

---

## API Reference

### Core Classes

#### AccountManager
```python
from git_manager.core import AccountManager

# Initialize
manager = AccountManager()

# Add account
manager.add_account(
    name="work",
    platform="github",
    username="myuser",
    ssh_key_path="~/.ssh/id_ed25519_work"
)

# Get account
account = manager.get_account("work")

# List accounts
accounts = manager.list_accounts(platform="github")

# Remove account
manager.remove_account("work")
```

#### SSHManager
```python
from git_manager.core import SSHManager

# Initialize
ssh_manager = SSHManager()

# Generate key
ssh_manager.generate_key(
    email="user@example.com",
    key_name="work-key",
    passphrase="optional"
)

# Test connection
result = ssh_manager.test_connection(account_name="work")

# Add to SSH config
ssh_manager.add_to_config(
    host="github.com-work",
    hostname="github.com",
    identity_file="~/.ssh/id_ed25519_work"
)
```

#### GitOperations
```python
from git_manager.core import GitOperations

# Initialize
git_ops = GitOperations()

# Clone repository
git_ops.clone(
    url="https://github.com/user/repo",
    account_name="work",
    destination="/path/to/clone"
)

# Check repository status
status = git_ops.check_status()

# Pull changes
git_ops.pull(rebase=True)

# Push changes
git_ops.push(set_upstream=True)
```

### REST API Endpoints

#### Accounts
```
GET    /api/accounts           - List all accounts
POST   /api/accounts           - Create account
GET    /api/accounts/:id       - Get account details
PUT    /api/accounts/:id       - Update account
DELETE /api/accounts/:id       - Delete account
POST   /api/accounts/:id/test  - Test account connection
```

#### Repositories
```
GET    /api/repositories       - List repositories
POST   /api/repositories/clone - Clone repository
GET    /api/repositories/:id   - Get repository details
POST   /api/repositories/:id/pull - Pull changes
POST   /api/repositories/:id/push - Push changes
GET    /api/repositories/:id/status - Get status
```

#### SSH
```
GET    /api/ssh/keys           - List SSH keys
POST   /api/ssh/keys/generate  - Generate new key
POST   /api/ssh/keys/test      - Test SSH connection
GET    /api/ssh/config         - Get SSH config
```

---

## Development Guide

### Setting Up Development Environment

```bash
# Clone and setup
git clone https://github.com/yourusername/git-multi-account-manager.git
cd git-multi-account-manager
python -m venv venv
source venv/bin/activate
pip install -e ".[dev]"

# Install pre-commit hooks
pre-commit install
```

### Running Tests

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=git_manager --cov-report=html

# Run specific test
pytest tests/unit/test_account_manager.py

# Run integration tests
pytest tests/integration/
```

### Code Style

```bash
# Format code
black src/ tests/

# Sort imports
isort src/ tests/

# Lint
flake8 src/ tests/
pylint src/

# Type check
mypy src/
```

### Building Documentation

```bash
# Build docs
cd docs
make html

# View docs
open _build/html/index.html
```

### Building Distribution

```bash
# Build package
python -m build

# Build desktop executable
python scripts/build.py --platform windows
python scripts/build.py --platform macos
python scripts/build.py --platform linux
```

### Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

### Code Standards

- Follow PEP 8
- Write docstrings (Google style)
- Add type hints
- Write unit tests (min 80% coverage)
- Update documentation

---

## Configuration

### Configuration File (`config.json`)

```json
{
  "version": "1.0.0",
  "logging": {
    "level": "INFO",
    "file": "/var/log/git-manager.log"
  },
  "ssh": {
    "directory": "~/.ssh",
    "config_file": "~/.ssh/config",
    "key_type": "ed25519"
  },
  "git": {
    "default_branch": "main",
    "auto_fetch": true,
    "auto_pull": false
  },
  "ui": {
    "theme": "dark",
    "color_scheme": {
      "primary": "#1B4332",
      "secondary": "#9A5324",
      "accent": "#2D6A6A",
      "text": "#6B4423",
      "background": "#4A4A4A"
    }
  }
}
```

### Accounts File (`accounts.json`)

```json
{
  "github": [
    {
      "name": "work",
      "username": "myuser",
      "email": "work@example.com",
      "host": "github.com-work",
      "ssh_key": "~/.ssh/id_ed25519_work",
      "description": "Work account"
    }
  ],
  "gitlab": [
    {
      "name": "personal",
      "username": "myuser",
      "email": "personal@example.com",
      "host": "gitlab.com-personal",
      "ssh_key": "~/.ssh/id_ed25519_personal",
      "description": "Personal projects"
    }
  ]
}
```

---

## Troubleshooting

### Common Issues

1. **SSH Connection Failed**
   - Check SSH key permissions (`chmod 600 ~/.ssh/id_*`)
   - Verify SSH agent is running
   - Test connection manually: `ssh -T git@github.com`

2. **Git Operations Fail**
   - Ensure correct account is selected
   - Check remote URL matches account
   - Verify network connectivity

3. **Permission Denied**
   - Run with appropriate permissions
   - Check file ownership
   - Verify SSH key is added to platform

### Debug Mode

```bash
# CLI debug mode
git-manager --debug <command>

# Web debug mode
git-manager web --debug

# Enable verbose logging
export GIT_MANAGER_LOG_LEVEL=DEBUG
```

---

## License

MIT License - see LICENSE file

## Support

- Documentation: https://git-manager.readthedocs.io
- Issues: https://github.com/yourusername/git-multi-account-manager/issues
- Discussions: https://github.com/yourusername/git-multi-account-manager/discussions

## Changelog

See CHANGELOG.md for version history.

---

*Last Updated: November 2025*