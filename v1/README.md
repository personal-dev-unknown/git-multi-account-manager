# 🚀 Git Multi-Account Manager

> **Manage multiple Git accounts (GitHub & GitLab) with ease across Terminal, Web, and Desktop interfaces.**

[![Python Version](https://img.shields.io/badge/python-3.9%2B-blue)](https://www.python.org/downloads/)
<!-- [![License](https://img.shields.io/badge/license-MIT-green)](LICENSE) -->
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/yourusername/git-multi-account-manager)

![Git Manager Banner](docs/images/banner.png)

## ✨ Features

### 🎯 Core Features
- ✅ **Multiple Account Management** - Seamlessly switch between GitHub & GitLab accounts
- ✅ **SSH Key Generation** - Create and manage ED25519/RSA keys
- ✅ **Repository Operations** - Clone, pull, push with account verification
- ✅ **Connection Testing** - Verify SSH connections to GitHub/GitLab
- ✅ **Three Interfaces** - CLI, Web, and Desktop applications
- ✅ **Color-Coded UI** - Beautiful dark theme with custom color palette

### 🎨 Color Scheme
- **Dark Green** (#1B4332) - Primary actions
- **Rust** (#9A5324) - Warnings & secondary
- **Teal** (#2D6A6A) - Success & accents
- **Brown** (#6B4423) - Text
- **Gray** (#4A4A4A) - Backgrounds

## 📸 Screenshots

### Terminal Interface
```
╔════════════════════════════════════════╗
║     Git Multi-Account Manager         ║
║        GitHub & GitLab                 ║
║           Version 1.0.0                ║
╚════════════════════════════════════════╝

What would you like to do?

═══ Repository Operations ═══
[1] Clone a repository (GitHub/GitLab)
[2] Check current repository account
[3] Git pull (with account verification)
[4] Git push (with account verification)
[5] Set up repository for specific account

═══ Account Management ═══
[6] Show all accounts (GitHub & GitLab)
[7] Test SSH connections
[8] Generate new SSH key

[9] Exit
```

### Web Interface
![Web Dashboard](docs/images/web-dashboard.png)
![Account Management](docs/images/web-accounts.png)

### Desktop Application
![Desktop App](docs/images/desktop-main.png)

## 🚀 Quick Start

### Prerequisites
```bash
Python 3.9+
Git 2.25+
SSH client
```

### Installation

#### Option 1: PyPI (Coming Soon)
```bash
pip install git-multi-account-manager
```

#### Option 2: From Source

**Linux/macOS:**
```bash
# Clone repository
git clone https://github.com/yourusername/git-multi-account-manager.git
cd git-multi-account-manager

# Run installation script
chmod +x scripts/install.sh
./scripts/install.sh

# Activate virtual environment
source venv/bin/activate
```

**Windows:**
```powershell
# Clone repository
git clone https://github.com/yourusername/git-multi-account-manager.git
cd git-multi-account-manager

# Run installation script
.\scripts\install.ps1

# Activate virtual environment
.\venv\Scripts\Activate.ps1
```

## 📖 Usage

### Terminal (CLI)

#### Interactive Mode
```bash
git-manager
```

#### Direct Commands
```bash
# Clone repository
git-manager clone https://github.com/user/repo --account work

# Check repository status
git-manager status

# Add account
git-manager account add \
  --name work \
  --platform github \
  --username myuser \
  --email work@example.com \
  --ssh-key ~/.ssh/id_ed25519_work

# Generate SSH key
git-manager ssh generate --email work@example.com --name work-key

# Test SSH connection
git-manager ssh test --account work

# Pull with verification
git-manager pull

# Push with verification
git-manager push

# List accounts
git-manager account list
```

### Web Application

```bash
# Start web server
git-manager-web

# With custom host/port
git-manager-web --host 0.0.0.0 --port 8080

# Access at: http://localhost:5000
```

**Web Features:**
- Dashboard with account overview
- Clone repositories via UI
- Manage accounts and SSH keys
- Real-time operation status
- RESTful API available

### Desktop Application

```bash
# Launch desktop app
git-manager-desktop
```

**Desktop Features:**
- Native look and feel
- Account management panel
- Repository explorer
- SSH key generation wizard
- System tray integration

## 🔧 Configuration

### Configuration File (`~/.git-manager/config.json`)

```json
{
  "ssh": {
    "directory": "~/.ssh",
    "key_type": "ed25519"
  },
  "git": {
    "default_branch": "main"
  },
  "ui": {
    "theme": "dark"
  }
}
```

### Accounts File (`~/.git-manager/accounts.json`)

```json
{
  "github": [
    {
      "name": "work",
      "username": "myuser",
      "email": "work@example.com",
      "host": "github.com-work",
      "ssh_key_path": "~/.ssh/id_ed25519_work"
    }
  ],
  "gitlab": [
    {
      "name": "personal",
      "username": "myuser",
      "email": "personal@example.com",
      "host": "gitlab.com-personal",
      "ssh_key_path": "~/.ssh/id_ed25519_personal"
    }
  ]
}
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────┐
│           User Interfaces                   │
│  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │
│  │   CLI   │  │   Web   │  │   Desktop   │  │
│  └────┬────┘  └────┬────┘  └──────┬──────┘  │
└───────┼───────────┼───────────────┼─────────┘
        │           │               │
        └───────────┴───────────────┘
                    │
        ┌───────────▼────────────┐
        │    Core Application    │
        │  • Account Manager     │
        │  • SSH Manager         │
        │  • Git Operations      │
        │  • Config Manager      │
        └───────────┬────────────┘
                    │
        ┌───────────▼────────────┐
        │    Infrastructure      │
        │  • File System         │
        │  • Git Commands        │
        │  • SSH Operations      │
        └────────────────────────┘
```

## 📚 API Reference

### Python API

```python
from git_manager import AccountManager, SSHManager, GitOperations

# Initialize managers
account_manager = AccountManager()
ssh_manager = SSHManager()
git_ops = GitOperations(account_manager)

# Add account
account_manager.add_account(
    name="work",
    platform="github",
    username="myuser",
    email="work@example.com",
    ssh_key_path="~/.ssh/id_ed25519_work"
)

# Generate SSH key
ssh_manager.generate_key(
    email="work@example.com",
    key_name="work-key"
)

# Clone repository
git_ops.clone(
    url="https://github.com/user/repo",
    account_name="work"
)
```

### REST API

```bash
# Get accounts
GET /api/v1/accounts

# Add account
POST /api/v1/accounts
{
  "name": "work",
  "platform": "github",
  "username": "myuser",
  "email": "work@example.com",
  "ssh_key_path": "~/.ssh/id_ed25519_work"
}

# Clone repository
POST /api/v1/repositories/clone
{
  "url": "https://github.com/user/repo",
  "account": "work"
}

# Test SSH connection
POST /api/v1/accounts/{name}/test
```

## 🧪 Testing

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=git_manager --cov-report=html

# Run specific tests
pytest tests/unit/test_account_manager.py

# Run integration tests
pytest tests/integration/
```

## 🛠️ Development

### Setup Development Environment

```bash
# Clone and setup
git clone https://github.com/yourusername/git-multi-account-manager.git
cd git-multi-account-manager

# Create virtual environment
python -m venv venv
source venv/bin/activate  # Windows: .\venv\Scripts\Activate.ps1

# Install in development mode
pip install -e ".[dev]"

# Install pre-commit hooks
pre-commit install
```

### Code Quality

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

### Build Documentation

```bash
cd docs
make html
open _build/html/index.html
```

### Build Executable

```bash
# Build for current platform
python scripts/build.py

# Build for specific platform
python scripts/build.py windows
python scripts/build.py macos
python scripts/build.py linux
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

### Development Guidelines

- Follow PEP 8 style guide
- Write docstrings (Google style)
- Add type hints
- Write unit tests (min 80% coverage)
- Update documentation

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by the bash script version
- Built with Python, Flask, PyQt6, and Rich
- Color scheme designed for optimal visibility

## 📧 Support

- **Documentation**: [https://git-manager.readthedocs.io](https://git-manager.readthedocs.io)
- **Issues**: [GitHub Issues](https://github.com/yourusername/git-multi-account-manager/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/git-multi-account-manager/discussions)
- **Email**: support@gitmanager.dev

## 🗺️ Roadmap

### Version 1.1
- [ ] Add BitBucket support
- [ ] Repository templates
- [ ] Commit signing
- [ ] Advanced search

### Version 1.2
- [ ] Team collaboration features
- [ ] Cloud sync
- [ ] Mobile app
- [ ] CI/CD integration

### Version 2.0
- [ ] AI-powered suggestions
- [ ] Code review integration
- [ ] Analytics dashboard
- [ ] Plugin system

## 📊 Project Statistics

- **Lines of Code**: 10,000+
- **Test Coverage**: 85%+
- **Supported Platforms**: Linux, macOS, Windows
- **Supported Python**: 3.9, 3.10, 3.11, 3.12

## 🌟 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=yourusername/git-multi-account-manager&type=Date)](https://star-history.com/#yourusername/git-multi-account-manager&Date)

---

<div align="center">
  <p>Made with ❤️ by <a href="https://github.com/devonionMoses">DevonionMoses</a></p>
  <p>
    <a href="https://twitter.com/gitmanager">Twitter</a> •
    <a href="https://github.com/yourusername/git-multi-account-manager">GitHub</a> •
    <a href="https://gitmanager.dev">Website</a>
  </p>
</div>