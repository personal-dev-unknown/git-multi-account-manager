# Version Module Architecture

## Overview

The version management system is now organized as a modular package under `src/git_manager/core/version/` following best software engineering practices and separation of concerns.

## Directory Structure

```
src/git_manager/core/version/
├── __init__.py              # Module exports
├── version_reader.py        # Read version information
├── version_comparator.py    # Compare semantic versions
├── file_updater.py          # Update version strings in files
└── update_checker.py        # Check for available updates
```

## Module Components

### 1. VersionReader (`version_reader.py`)

**Responsibility**: Read and parse version information from various sources.

**Key Classes**:
- `VersionReader`: Reads version from VERSION file and project files

**Key Methods**:
```python
read_version_file()              # Read from VERSION file
read_setup_py_version()          # Read from setup.py
read_pyproject_toml_version()    # Read from pyproject.toml
read_constants_py_version()      # Read from constants.py
read_api_version()               # Read from API routes
get_all_versions()               # Get all version references
verify_version_consistency()     # Check if all versions match
```

**Example Usage**:
```python
from git_manager.core.version import VersionReader

reader = VersionReader()
current_version = reader.read_version_file()
print(f"Current version: {current_version}")

# Check consistency
is_consistent, versions = reader.verify_version_consistency()
```

### 2. VersionComparator (`version_comparator.py`)

**Responsibility**: Compare semantic versions and determine update availability.

**Key Classes**:
- `VersionComparator`: Compares MAJOR.MINOR.PATCH versions

**Key Methods**:
```python
parse_version(version)           # Parse version string
compare(current, latest)         # Compare two versions (-1, 0, 1)
is_update_available(current, latest)  # Check if update available
get_version_difference(current, latest)  # Get version differences
format_version_change(current, latest)   # Human-readable change message
```

**Example Usage**:
```python
from git_manager.core.version import VersionComparator

# Check if update is available
if VersionComparator.is_update_available("1.0.0", "1.1.0"):
    print("Update available!")

# Get version difference
diff = VersionComparator.get_version_difference("1.0.0", "2.0.0")
print(f"Major update: {diff['is_major_update']}")
```

### 3. FileUpdater (`file_updater.py`)

**Responsibility**: Update version strings in project files.

**Key Classes**:
- `FileUpdater`: Updates version in setup.py, pyproject.toml, constants.py, etc.

**Key Methods**:
```python
update_version_file(new_version)  # Update VERSION file
update_all_files(new_version)     # Update all tracked files
get_updated_files_summary(results)  # Get summary of updates
```

**Tracked Files**:
- `VERSION` - Core version file
- `setup.py` - Python package setup
- `pyproject.toml` - PEP 517 build config
- `src/git_manager/utils/constants.py` - App constants
- `src/git_manager/__init__.py` - Package init
- `src/git_manager/web/routes/api.py` - API version
- `README.md` - Documentation

**Example Usage**:
```python
from git_manager.core.version import FileUpdater

updater = FileUpdater()
results = updater.update_all_files("1.1.0")
summary = updater.get_updated_files_summary(results)

print(f"Updated: {summary['updated']}/{summary['total']}")
```

### 4. UpdateChecker (`update_checker.py`)

**Responsibility**: Check for available updates from GitHub and PyPI.

**Key Classes**:
- `UpdateChecker`: Checks for updates from GitHub/PyPI
- `UpdateNotifier`: Generates user notifications

**Key Methods**:
```python
check_github_updates()       # Check GitHub releases
check_pypi_updates()         # Check PyPI
check_for_updates(source)    # Check for updates
get_update_info()            # Get update information
install_update_pip()         # Install via pip
```

**Example Usage**:
```python
from git_manager.core.version import UpdateChecker, UpdateNotifier

checker = UpdateChecker()
if checker.check_for_updates(source='github'):
    notifier = UpdateNotifier(checker)
    print(notifier.notify_update_available())
```

## Integration Points

### CLI Integration

The version manager script (`scripts/version-manager.sh`) uses the version module:

```bash
./scripts/version-manager.sh get           # Get current version
./scripts/version-manager.sh set 1.1.0     # Set version
./scripts/version-manager.sh bump-minor    # Bump minor version
./scripts/version-manager.sh list          # List all versions
```

### Application Integration

To check for updates on startup:

```python
from git_manager.core.version import check_for_updates_on_startup

# In your main application
check_for_updates_on_startup(source='github', notify=True)
```

### Programmatic Usage

```python
from git_manager.core.version import (
    VersionReader,
    VersionComparator,
    FileUpdater,
    UpdateChecker,
)

# Read current version
reader = VersionReader()
current = reader.read_version_file()

# Check for updates
checker = UpdateChecker()
if checker.check_for_updates():
    # Update available
    latest = checker.latest_version
    
    # Show what changed
    diff = VersionComparator.get_version_difference(current, latest)
    print(f"Major update: {diff['is_major_update']}")
```

## Design Principles

### 1. Separation of Concerns
- Each module has a single, well-defined responsibility
- No circular dependencies
- Clear interfaces between modules

### 2. Reusability
- Modules can be used independently
- No tight coupling to specific file formats
- Extensible for future version sources

### 3. Testability
- Pure functions where possible
- Dependency injection for file paths
- Logging for debugging

### 4. Maintainability
- Clear naming conventions
- Comprehensive docstrings
- Type hints throughout

## Error Handling

All modules include proper error handling:

```python
try:
    reader = VersionReader()
    version = reader.read_version_file()
except FileNotFoundError:
    print("VERSION file not found")
except Exception as e:
    print(f"Error reading version: {e}")
```

## Logging

The module uses Python's logging system:

```python
import logging

logger = logging.getLogger(__name__)
logger.info("Version updated successfully")
logger.warning("Version file not found")
logger.error("Failed to update version")
```

## Version File Format

The `VERSION` file contains a single line with the version:

```
1.0.0
```

## Semantic Versioning

The project follows semantic versioning:

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

Example progression:
```
1.0.0  (initial release)
1.0.1  (patch fix)
1.1.0  (new feature)
2.0.0  (breaking change)
```

## GitHub Integration

The update checker uses the GitHub API:

```
Repository: https://github.com/Zanabuni/git-multi-account-manager
API Endpoint: /repos/Zanabuni/git-multi-account-manager/releases/latest
```

## PyPI Integration

The update checker also supports PyPI:

```
Package: git-multi-account-manager
API Endpoint: https://pypi.org/pypi/git-multi-account-manager/json
```

## Future Extensions

The modular design allows for easy extensions:

1. **New version sources**: Add new checker classes
2. **Custom file formats**: Add new updater methods
3. **Version validation**: Extend VersionComparator
4. **Notification channels**: Add new notifier classes

## See Also

- [VERSION_MANAGEMENT.md](./VERSION_MANAGEMENT.md) - User guide
- [Semantic Versioning](https://semver.org/)
- [Python Packaging](https://packaging.python.org/)
