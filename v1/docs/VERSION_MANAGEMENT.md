# Version Management System

This document describes the version management system for Git Multi-Account Manager.

## Overview

The project uses a robust semantic versioning system (MAJOR.MINOR.PATCH) with automated version management across multiple files and an automatic update checker.

## Version Files

### Core Version Definition

- **`VERSION`** (root directory)
  - Single source of truth for the application version
  - Plain text file containing version string (e.g., `1.0.0`)
  - Used by version management scripts

### Version References

Version strings are maintained in the following files:

1. **`setup.py`** - Python package setup configuration
   ```python
   version='1.0.0'
   ```

2. **`pyproject.toml`** - PEP 517 build configuration
   ```toml
   version = "1.0.0"
   ```

3. **`src/git_manager/utils/constants.py`** - Application constants
   ```python
   APP_VERSION = "1.0.0"
   ```

4. **`src/git_manager/web/routes/api.py`** - API health check endpoint
   ```python
   'version': '1.0.0'
   ```

5. **`README.md`** - Documentation version references
   ```markdown
   v1.0.0
   ```

## Version Manager Script

### Location

`scripts/version-manager.sh` - Python executable script with shebang

### Usage

```bash
# Make the script executable (one-time)
chmod +x scripts/version-manager.sh

# Get current version
./scripts/version-manager.sh get

# Set a specific version
./scripts/version-manager.sh set 2.1.0

# Bump patch version (1.0.0 → 1.0.1)
./scripts/version-manager.sh bump-patch

# Bump minor version (1.0.0 → 1.1.0)
./scripts/version-manager.sh bump-minor

# Bump major version (1.0.0 → 2.0.0)
./scripts/version-manager.sh bump-major

# List all version files and their current versions
./scripts/version-manager.sh list

# Validate version format
./scripts/version-manager.sh validate 2.1.0

# Show help
./scripts/version-manager.sh help
```

### Features

- **Atomic Updates**: Updates all version files in a single operation
- **Validation**: Ensures version format is MAJOR.MINOR.PATCH
- **Color Output**: Provides colored terminal output for better readability
- **Error Handling**: Gracefully handles missing files and invalid formats
- **Version Bumping**: Automatically increment major, minor, or patch versions

## Update Checker Module

### Location

`src/git_manager/core/update_checker.py`

### Components

#### VersionComparator

Compares semantic versions to determine if updates are available.

```python
from git_manager.core.update_checker import VersionComparator

# Compare versions
result = VersionComparator.compare("1.0.0", "1.1.0")
# Returns: -1 (update available)

# Check if update is available
is_available = VersionComparator.is_update_available("1.0.0", "1.1.0")
# Returns: True
```

#### UpdateChecker

Checks for available updates from GitHub or PyPI.

```python
from git_manager.core.update_checker import UpdateChecker

checker = UpdateChecker(timeout=5)

# Check GitHub for updates
if checker.check_for_updates(source='github'):
    print(f"Update available: {checker.latest_version}")
    
# Get update information
info = checker.get_update_info()
# Returns: {
#     'current_version': '1.0.0',
#     'latest_version': '1.1.0',
#     'update_available': True,
#     'release_info': {...}
# }

# Install update via pip
checker.install_update_pip()
```

#### UpdateNotifier

Generates user-friendly notification messages.

```python
from git_manager.core.update_checker import UpdateChecker, UpdateNotifier

checker = UpdateChecker()
checker.check_for_updates()

notifier = UpdateNotifier(checker)
print(notifier.notify_update_available())
```

### Startup Integration

To check for updates on application startup:

```python
from git_manager.core.update_checker import check_for_updates_on_startup

# Check for updates (non-blocking)
check_for_updates_on_startup(source='github', notify=True, timeout=5)
```

## Workflow

### Releasing a New Version

1. **Update the version**:
   ```bash
   ./scripts/version-manager.sh set 1.1.0
   ```

2. **Verify changes**:
   ```bash
   ./scripts/version-manager.sh list
   ```

3. **Commit changes**:
   ```bash
   git add VERSION setup.py pyproject.toml src/git_manager/utils/constants.py
   git commit -m "chore: bump version to 1.1.0"
   ```

4. **Create git tag**:
   ```bash
   git tag v1.1.0
   git push origin v1.1.0
   ```

5. **Build and publish**:
   ```bash
   python -m build
   twine upload dist/*
   ```

### Checking for Updates

Users can check for updates in several ways:

1. **Via CLI**:
   ```bash
   git-manager --version
   ```

2. **Programmatically**:
   ```python
   from git_manager.core.update_checker import check_for_updates_on_startup
   check_for_updates_on_startup()
   ```

3. **Manual check**:
   ```bash
   pip index versions git-multi-account-manager
   ```

## Version Format

The project uses semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

Example progression:
- `1.0.0` → `1.0.1` (patch fix)
- `1.0.1` → `1.1.0` (new feature)
- `1.1.0` → `2.0.0` (breaking change)

## API Version

The API uses a separate versioning scheme:

- **Location**: `src/git_manager/utils/constants.py`
- **Current**: `API_VERSION = 'v1'`
- **Endpoint**: `/api/v1/...`

The API version is independent of the application version and should only change when the API contract changes significantly.

## Best Practices

1. **Always use the version manager script** to update versions
2. **Never manually edit** version strings in multiple files
3. **Test thoroughly** before releasing a new version
4. **Create git tags** for each release
5. **Update CHANGELOG** when releasing new versions
6. **Notify users** about breaking changes in release notes

## Troubleshooting

### Version manager script not executable

```bash
chmod +x scripts/version-manager.sh
```

### Update check fails

- Ensure internet connectivity
- Check firewall/proxy settings
- Verify GitHub/PyPI is accessible
- Check logs for detailed error messages

### Version mismatch across files

Run the version manager to synchronize:

```bash
./scripts/version-manager.sh set <VERSION>
```

## See Also

- [Semantic Versioning](https://semver.org/)
- [Python Packaging Guide](https://packaging.python.org/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
