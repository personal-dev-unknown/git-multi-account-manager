# Scripts Usage Guide

## Version Management Scripts

The project includes three complementary scripts for version management:

### 1. `scripts/version-manager.sh` (Python Executable)

**Type**: Python script with shebang  
**Shebang**: `#!/usr/bin/env python3`  
**Execution**: Direct Python execution

**Usage**:
```bash
./scripts/version-manager.sh get
./scripts/version-manager.sh set 1.1.0
./scripts/version-manager.sh bump-patch
./scripts/version-manager.sh bump-minor
./scripts/version-manager.sh bump-major
./scripts/version-manager.sh list
./scripts/version-manager.sh validate 1.1.0
```

**How it works**:
- Contains full Python implementation
- Imports from `git_manager.core.version` module
- Uses `VersionReader`, `VersionComparator`, and `FileUpdater` classes
- Provides colored terminal output

### 2. `scripts/version-manager` (Bash Wrapper)

**Type**: Bash shell script  
**Shebang**: `#!/bin/bash`  
**Execution**: Bash wrapper that calls Python script

**Usage**:
```bash
./scripts/version-manager get
./scripts/version-manager set 1.1.0
./scripts/version-manager bump-minor
```

**How it works**:
- Lightweight bash wrapper
- Checks for Python 3 availability
- Delegates to `version_manager.py`
- Useful for shell environments

### 3. `scripts/version_manager.py` (Python Module)

**Type**: Pure Python module  
**Shebang**: `#!/usr/bin/env python3`  
**Execution**: Python module or direct execution

**Usage**:
```bash
# Direct execution
python3 scripts/version_manager.py get
python3 scripts/version_manager.py set 1.1.0

# As a module
python3 -m git_manager.core.version
```

**How it works**:
- Standalone Python module
- Can be imported in other scripts
- Integrates with version module
- Provides VersionManager class

## Release Script

### `scripts/release.py`

**Type**: Python executable script  
**Purpose**: Automate the complete release process

**Usage**:
```bash
python3 scripts/release.py 1.1.0
```

**What it does**:
1. Validates version format
2. Checks version consistency
3. Updates all version files
4. Runs tests (pytest)
5. Builds package
6. Creates git tag
7. Prints next steps

**Integration**:
- Uses `VersionReader` to read current versions
- Uses `VersionComparator` to validate format
- Uses `FileUpdater` to update all files
- Fully integrated with version module

## Recommended Usage

### For Development
```bash
# Check current version
./scripts/version-manager.sh get

# Update version during development
./scripts/version-manager.sh set 1.1.0

# Or bump automatically
./scripts/version-manager.sh bump-minor
```

### For Releases
```bash
# Execute full release process
python3 scripts/release.py 1.1.0

# Then follow the printed instructions
```

### For CI/CD
```bash
# In shell scripts
./scripts/version-manager get

# In Python scripts
from git_manager.core.version import VersionReader
reader = VersionReader()
version = reader.read_version_file()
```

## File Permissions

Make scripts executable:
```bash
chmod +x scripts/version-manager.sh
chmod +x scripts/version-manager
chmod +x scripts/release.py
```

## Troubleshooting

### "Permission denied" error
```bash
chmod +x scripts/version-manager.sh
```

### "Python not found" error
Ensure Python 3 is installed:
```bash
python3 --version
```

### "Module not found" error
Ensure you're running from project root:
```bash
cd /path/to/git-multi-account-manager
./scripts/version-manager.sh get
```

### "No such file or directory" when using bash
Don't use `bash` to execute Python scripts:
```bash
# ❌ Wrong
bash scripts/version-manager.sh get

# ✅ Correct
./scripts/version-manager.sh get
python3 scripts/version-manager.py get
./scripts/version-manager get
```

## Architecture

```
scripts/
├── version-manager          # Bash wrapper
├── version-manager.sh       # Python executable (main)
├── version_manager.py       # Python module
└── release.py              # Release automation

src/git_manager/core/version/
├── __init__.py             # Module exports
├── version_reader.py       # Read versions
├── version_comparator.py   # Compare versions
├── file_updater.py         # Update files
└── update_checker.py       # Check updates
```

## Integration Points

### CLI
```bash
./scripts/version-manager.sh set 1.1.0
```

### Python
```python
from git_manager.core.version import FileUpdater
updater = FileUpdater()
updater.update_all_files("1.1.0")
```

### Release Automation
```bash
python3 scripts/release.py 1.1.0
```

## Best Practices

1. **Always use the version manager** to update versions
2. **Never manually edit** version strings in multiple files
3. **Use release.py** for complete release automation
4. **Check version consistency** before releases
5. **Run tests** before creating releases
6. **Create git tags** for all releases

## See Also

- [VERSION_MANAGEMENT.md](./VERSION_MANAGEMENT.md) - User guide
- [VERSION_MODULE_ARCHITECTURE.md](./VERSION_MODULE_ARCHITECTURE.md) - Technical architecture
- [VERSION_SYSTEM_IMPLEMENTATION.md](./VERSION_SYSTEM_IMPLEMENTATION.md) - Implementation details
- [RELEASE_WORKFLOW.md](./RELEASE_WORKFLOW.md) - Release process
