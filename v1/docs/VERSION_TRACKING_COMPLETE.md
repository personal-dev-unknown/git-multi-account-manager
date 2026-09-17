# Complete Version Tracking System

## Overview

The version management system now tracks and automatically updates **10 files** across the entire project, ensuring complete version consistency.

## Tracked Files

### 1. Core Version File
**File**: `VERSION` (root)
- **Type**: Single source of truth
- **Format**: Plain text (e.g., `1.0.0`)
- **Purpose**: Primary version reference

### 2. Package Configuration Files

#### `setup.py`
- **Pattern**: `version='X.Y.Z'`
- **Purpose**: Python package metadata for PyPI
- **Updated by**: FileUpdater._update_setup_py()

#### `pyproject.toml`
- **Pattern**: `version = "X.Y.Z"`
- **Purpose**: PEP 517 build system configuration
- **Updated by**: FileUpdater._update_pyproject_toml()

### 3. Application Code

#### `src/git_manager/utils/constants.py`
- **Pattern**: `APP_VERSION = "X.Y.Z"`
- **Purpose**: Application version constant used throughout the app
- **Updated by**: FileUpdater._update_constants_py()

#### `src/git_manager/__init__.py`
- **Pattern**: Imports from constants (no direct update)
- **Purpose**: Package initialization
- **Updated by**: FileUpdater._update_init_py() (skipped, inherits from constants)

#### `src/git_manager/web/routes/api.py`
- **Pattern**: `'version': 'X.Y.Z'`
- **Purpose**: API health endpoint version response
- **Updated by**: FileUpdater._update_api_py()

### 4. Documentation

#### `README.md`
- **Pattern**: `vX.Y.Z` (version references)
- **Purpose**: Documentation version references
- **Updated by**: FileUpdater._update_readme()

### 5. Distribution & Packaging

#### `snap/snapcraft.yaml`
- **Pattern**: `version: 'X.Y.Z'`
- **Purpose**: Snap package version configuration
- **Updated by**: FileUpdater._update_snapcraft_yaml()

### 6. Build & Release Scripts

#### `scripts/build.py`
- **Pattern**: `BUILD_SCRIPT_VERSION = "X.Y.Z"`
- **Purpose**: Build script version tracking
- **Updated by**: FileUpdater._update_build_py()
- **Usage**: Can be imported to get build script version
  ```python
  from scripts.build import BUILD_SCRIPT_VERSION
  print(f"Build script v{BUILD_SCRIPT_VERSION}")
  ```

#### `scripts/release.py`
- **Pattern**: `RELEASE_SCRIPT_VERSION = "X.Y.Z"`
- **Purpose**: Release script version tracking
- **Updated by**: FileUpdater._update_release_py()
- **Usage**: Can be imported to get release script version
  ```python
  from scripts.release import RELEASE_SCRIPT_VERSION
  print(f"Release script v{RELEASE_SCRIPT_VERSION}")
  ```

## Version Update Flow

```
VERSION file (source of truth)
    ↓
./scripts/version-manager.sh set X.Y.Z
    ↓
FileUpdater.update_all_files(X.Y.Z)
    ├── Update VERSION file
    ├── Update setup.py
    ├── Update pyproject.toml
    ├── Update constants.py
    ├── Update __init__.py (skip)
    ├── Update api.py
    ├── Update README.md
    ├── Update snap/snapcraft.yaml
    ├── Update scripts/build.py
    └── Update scripts/release.py
```

## Usage Examples

### Update All Versions

```bash
./scripts/version-manager.sh set 1.1.0
```

**Output**:
```
INFO: Updating version to 1.1.0...
INFO: Updated VERSION file: 1.1.0
INFO: Updated: setup.py
INFO: Updated: pyproject.toml
INFO: Updated: src/git_manager/utils/constants.py
DEBUG: Skipping src/git_manager/__init__.py (imports from constants)
INFO: Updated: src/git_manager/web/routes/api.py
INFO: Updated: README.md
INFO: Updated: snap/snapcraft.yaml
INFO: Updated: scripts/build.py
INFO: Updated: scripts/release.py
```

### Verify All Versions

```bash
./scripts/version-manager.sh list
```

### Programmatic Access

```python
from git_manager.core.version import VersionReader, FileUpdater

# Read current version
reader = VersionReader()
current = reader.read_version_file()
print(f"Current version: {current}")

# Get all versions
all_versions = reader.get_all_versions()
for source, version in all_versions.items():
    print(f"{source}: {version}")

# Update all files
updater = FileUpdater()
results = updater.update_all_files("1.1.0")
summary = updater.get_updated_files_summary(results)
print(f"Updated {summary['updated']}/{summary['total']} files")
```

### Access Script Versions

```python
# Get build script version
from scripts.build import BUILD_SCRIPT_VERSION
print(f"Build script: {BUILD_SCRIPT_VERSION}")

# Get release script version
from scripts.release import RELEASE_SCRIPT_VERSION
print(f"Release script: {RELEASE_SCRIPT_VERSION}")
```

## File Updater Methods

### Core Methods

- **`update_all_files(new_version)`** - Updates all tracked files
- **`get_updated_files_summary(results)`** - Generates summary of updates
- **`update_version_file(new_version)`** - Updates VERSION file

### Specific Update Methods

- **`_update_setup_py(new_version)`** - Updates setup.py
- **`_update_pyproject_toml(new_version)`** - Updates pyproject.toml
- **`_update_constants_py(new_version)`** - Updates constants.py
- **`_update_init_py(new_version)`** - Skips __init__.py (inherits from constants)
- **`_update_api_py(new_version)`** - Updates api.py
- **`_update_readme(new_version)`** - Updates README.md
- **`_update_snapcraft_yaml(new_version)`** - Updates snap/snapcraft.yaml
- **`_update_build_py(new_version)`** - Updates scripts/build.py
- **`_update_release_py(new_version)`** - Updates scripts/release.py

## Version Consistency Verification

The `VersionReader` class provides consistency checking:

```python
from git_manager.core.version import VersionReader

reader = VersionReader()
is_consistent, versions = reader.verify_version_consistency()

if is_consistent:
    print("✓ All versions are consistent")
else:
    print("✗ Version mismatch detected:")
    for source, version in versions.items():
        print(f"  {source}: {version}")
```

## Release Workflow

1. **Update versions**:
   ```bash
   ./scripts/version-manager.sh set 1.1.0
   ```

2. **Verify consistency**:
   ```bash
   ./scripts/version-manager.sh list
   ```

3. **Commit changes**:
   ```bash
   git add VERSION setup.py pyproject.toml src/git_manager/utils/constants.py \
           src/git_manager/web/routes/api.py README.md snap/snapcraft.yaml \
           scripts/build.py scripts/release.py
   git commit -m "chore: bump version to 1.1.0"
   ```

4. **Create tag**:
   ```bash
   git tag v1.1.0
   git push origin v1.1.0
   ```

5. **Build and publish**:
   ```bash
   python3 scripts/build.py
   python -m build
   twine upload dist/*
   ```

Or use the automated release script:
```bash
python3 scripts/release.py 1.1.0
```

## Benefits

✅ **Single Source of Truth**: VERSION file is the primary reference  
✅ **Automatic Synchronization**: All files updated together  
✅ **Complete Coverage**: All version references tracked  
✅ **Script Versioning**: Build and release scripts have version constants  
✅ **Snap Support**: Snap package configuration updated  
✅ **Consistency Verification**: Check all versions match  
✅ **Error Handling**: Comprehensive error reporting  
✅ **Logging**: Full audit trail of updates  

## Troubleshooting

### Version not updated in a file

1. Check if file exists:
   ```bash
   ls -la <file_path>
   ```

2. Verify version pattern in file:
   ```bash
   grep -n "version\|VERSION" <file_path>
   ```

3. Check logs for errors:
   ```bash
   ./scripts/version-manager.sh set X.Y.Z 2>&1 | grep ERROR
   ```

### Inconsistent versions

Run consistency check:
```bash
./scripts/version-manager.sh list
```

Then update all files:
```bash
./scripts/version-manager.sh set <correct_version>
```

## Future Enhancements

- [ ] Add version constants to more scripts
- [ ] Support for pre-release versions (1.0.0-alpha)
- [ ] Changelog generation from version history
- [ ] Automated version bumping from git commits
- [ ] Version validation in CI/CD pipeline

## Summary

The version management system now provides **complete, automatic version tracking** across all 10 project files, ensuring consistency and reducing manual errors during releases.
