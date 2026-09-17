# Git Push, Pull & Sync API Reference

## Overview

This document provides comprehensive API reference for the Git Push, Pull, and Sync operations in GitManager.

## Table of Contents

1. [GitPush Class](#gitpush-class)
2. [GitPull Class](#gitpull-class)
3. [GitSync Class](#gitsync-class)
4. [Return Types](#return-types)
5. [Examples](#examples)

---

## GitPush Class

**Module:** `src/git_manager/core/git_push.py`

Handles all Git push operations with safety checks.

### Methods

#### `safe_push(repo_path, branch, remote, set_upstream)`

Safe push with pre-flight checks.

**Parameters:**
- `repo_path` (Path, optional): Repository path (default: current directory)
- `branch` (str, optional): Branch to push (default: current branch)
- `remote` (str): Remote name (default: "origin")
- `set_upstream` (bool): Set upstream tracking (default: False)

**Returns:**
```python
(success: bool, message: str, details: dict)
```

**Details dict:**
```python
{
    "commits_pushed": int,      # Number of commits pushed
    "remote_ahead": bool,       # Remote has new commits
    "large_files": list         # Large files detected
}
```

**Example:**
```python
from git_manager.core.git_push import GitPush
from pathlib import Path

git_push = GitPush()
success, message, details = git_push.safe_push(
    repo_path=Path("/path/to/repo"),
    branch="main",
    remote="origin",
    set_upstream=True
)

if success:
    print(f"✓ {message}")
    print(f"Pushed {details['commits_pushed']} commits")
else:
    print(f"✗ {message}")
```

---

#### `push_with_lease(repo_path, branch, remote)`

Force push with safety (force-with-lease).

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to push
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str)
```

**Example:**
```python
success, message = git_push.push_with_lease(
    repo_path=Path("/path/to/repo"),
    branch="feature/new-feature"
)

if success:
    print(f"✓ {message}")
else:
    print(f"✗ {message}")
```

---

#### `force_push(repo_path, branch, remote)`

Unconditional force push (dangerous).

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to push
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str)
```

**⚠️ WARNING:** Creates backup branch before executing.

**Example:**
```python
success, message = git_push.force_push(
    repo_path=Path("/path/to/repo"),
    branch="main"
)

if success:
    print(f"✓ {message}")
    # Backup created: backup-before-force-push-main
else:
    print(f"✗ {message}")
```

---

#### `push_all_branches(repo_path, remote)`

Push all local branches.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str, branches: list)
```

**Example:**
```python
success, message, branches = git_push.push_all_branches(
    repo_path=Path("/path/to/repo")
)

if success:
    print(f"✓ {message}")
    for branch in branches:
        print(f"  • {branch}")
else:
    print(f"✗ {message}")
```

---

#### `push_with_tags(repo_path, branch, remote)`

Push commits and tags together.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to push
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str)
```

**Example:**
```python
success, message = git_push.push_with_tags(
    repo_path=Path("/path/to/repo"),
    branch="main"
)

if success:
    print(f"✓ {message}")
else:
    print(f"✗ {message}")
```

---

#### `dry_run_push(repo_path, branch, remote)`

Preview what would be pushed.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to push
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str)
```

**Example:**
```python
success, message = git_push.dry_run_push(
    repo_path=Path("/path/to/repo"),
    branch="main"
)

if success:
    print("Preview:")
    print(message)
else:
    print(f"✗ {message}")
```

---

## GitPull Class

**Module:** `src/git_manager/core/git_pull.py`

Handles all Git pull operations with safety checks.

### Methods

#### `safe_pull(repo_path, branch, remote)`

Safe pull with automatic stash/unstash.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to pull
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str, details: dict)
```

**Details dict:**
```python
{
    "stashed": bool,            # Changes were stashed
    "pulled": bool,             # Pull was successful
    "conflicts": bool,          # Conflicts occurred
    "commits_pulled": int       # Number of commits pulled
}
```

**Example:**
```python
from git_manager.core.git_pull import GitPull
from pathlib import Path

git_pull = GitPull()
success, message, details = git_pull.safe_pull(
    repo_path=Path("/path/to/repo"),
    branch="main"
)

if success:
    print(f"✓ {message}")
    if details["stashed"]:
        print("Changes were stashed and reapplied")
else:
    print(f"✗ {message}")
    if details["conflicts"]:
        print("Please resolve conflicts manually")
```

---

#### `smart_pull(repo_path, branch, remote)`

Intelligent pull with automatic strategy selection.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to pull
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str)
```

**Strategy selection:**
- Uses **rebase** if branches have diverged
- Uses **merge** if branches haven't diverged

**Example:**
```python
success, message = git_pull.smart_pull(
    repo_path=Path("/path/to/repo"),
    branch="main"
)

if success:
    print(f"✓ {message}")
else:
    print(f"✗ {message}")
```

---

#### `pull_rebase(repo_path, branch, remote)`

Pull and rebase local commits.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to pull
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str)
```

**Example:**
```python
success, message = git_pull.pull_rebase(
    repo_path=Path("/path/to/repo"),
    branch="feature/new-feature"
)

if success:
    print(f"✓ {message}")
else:
    print(f"✗ {message}")
```

---

#### `pull_ff_only(repo_path, branch, remote)`

Pull only if fast-forward is possible.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to pull
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str)
```

**Example:**
```python
success, message = git_pull.pull_ff_only(
    repo_path=Path("/path/to/repo"),
    branch="main"
)

if success:
    print(f"✓ {message}")
else:
    print(f"✗ Pull failed - merge would be needed")
```

---

#### `pull_autostash(repo_path, branch, remote)`

Pull with automatic stash/unstash.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to pull
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str)
```

**Example:**
```python
success, message = git_pull.pull_autostash(
    repo_path=Path("/path/to/repo"),
    branch="main"
)

if success:
    print(f"✓ {message}")
else:
    print(f"✗ {message}")
```

---

#### `force_pull(repo_path, branch, remote)`

Reset to remote (destructive).

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to reset to
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str)
```

**⚠️ WARNING:** Discards all local changes. Creates backup branch.

**Example:**
```python
success, message = git_pull.force_pull(
    repo_path=Path("/path/to/repo"),
    branch="main"
)

if success:
    print(f"✓ {message}")
    # Backup created: backup-before-force-pull-main
else:
    print(f"✗ {message}")
```

---

#### `fetch_only(repo_path, remote)`

Download changes without integrating.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str)
```

**Example:**
```python
success, message = git_pull.fetch_only(
    repo_path=Path("/path/to/repo")
)

if success:
    print(f"✓ {message}")
    # Now review changes before pulling
else:
    print(f"✗ {message}")
```

---

#### `pull_all_branches(repo_path, remote)`

Update all tracking branches.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str, branches_updated: int)
```

**Example:**
```python
success, message, count = git_pull.pull_all_branches(
    repo_path=Path("/path/to/repo")
)

if success:
    print(f"✓ {message}")
    print(f"Updated {count} branches")
else:
    print(f"✗ {message}")
```

---

## GitSync Class

**Module:** `src/git_manager/core/git_sync.py`

Handles bidirectional synchronization.

### Methods

#### `safe_sync(repo_path, branch, remote)`

Intelligent bidirectional sync.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to sync
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
(success: bool, message: str, details: dict)
```

**Details dict:**
```python
{
    "stashed": bool,            # Changes were stashed
    "pulled": bool,             # Pull was successful
    "pushed": bool,             # Push was successful
    "conflicts": bool,          # Conflicts occurred
    "commits_pulled": int,      # Number of commits pulled
    "commits_pushed": int       # Number of commits pushed
}
```

**Example:**
```python
from git_manager.core.git_sync import GitSync
from pathlib import Path

git_sync = GitSync()
success, message, details = git_sync.safe_sync(
    repo_path=Path("/path/to/repo"),
    branch="main"
)

if success:
    print(f"✓ {message}")
    print(f"Pulled {details['commits_pulled']} commits")
    print(f"Pushed {details['commits_pushed']} commits")
else:
    print(f"✗ {message}")
```

---

#### `get_pre_flight_checks(repo_path, branch, remote)`

Run comprehensive pre-operation checks.

**Parameters:**
- `repo_path` (Path, optional): Repository path
- `branch` (str, optional): Branch to check
- `remote` (str): Remote name (default: "origin")

**Returns:**
```python
{
    "uncommitted_changes": bool,
    "uncommitted_count": int,
    "unpushed_commits": bool,
    "unpushed_count": int,
    "new_remote_commits": bool,
    "new_remote_count": int,
    "branches_diverged": bool,
    "warnings": list,
    "blockers": list
}
```

**Example:**
```python
checks = git_sync.get_pre_flight_checks(
    repo_path=Path("/path/to/repo")
)

if checks["blockers"]:
    print("Cannot proceed:")
    for blocker in checks["blockers"]:
        print(f"  ✗ {blocker}")

if checks["warnings"]:
    print("Warnings:")
    for warning in checks["warnings"]:
        print(f"  ⚠️  {warning}")
```

---

## Return Types

### Success Response

```python
(True, "✓ Operation successful", details_dict)
```

### Failure Response

```python
(False, "✗ Operation failed: reason", details_dict)
```

### Details Dictionary

Varies by operation:

**Push:**
```python
{
    "commits_pushed": 3,
    "remote_ahead": False,
    "large_files": []
}
```

**Pull:**
```python
{
    "stashed": True,
    "pulled": True,
    "conflicts": False,
    "commits_pulled": 2
}
```

**Sync:**
```python
{
    "stashed": True,
    "pulled": True,
    "pushed": True,
    "conflicts": False,
    "commits_pulled": 2,
    "commits_pushed": 3
}
```

---

## Examples

### Example 1: Simple Push

```python
from git_manager.core.git_push import GitPush
from pathlib import Path

git_push = GitPush()
success, message, details = git_push.safe_push(
    repo_path=Path.cwd(),
    set_upstream=True
)

if success:
    print(f"✓ {message}")
else:
    print(f"✗ {message}")
```

### Example 2: Pull with Error Handling

```python
from git_manager.core.git_pull import GitPull
from pathlib import Path

git_pull = GitPull()
success, message, details = git_pull.safe_pull(
    repo_path=Path.cwd()
)

if success:
    print(f"✓ {message}")
    if details["conflicts"]:
        print("⚠️  Please resolve conflicts manually")
else:
    print(f"✗ {message}")
```

### Example 3: Smart Sync

```python
from git_manager.core.git_sync import GitSync
from pathlib import Path

git_sync = GitSync()

# Check status first
checks = git_sync.get_pre_flight_checks(Path.cwd())

if checks["blockers"]:
    print("Cannot sync:")
    for blocker in checks["blockers"]:
        print(f"  ✗ {blocker}")
else:
    # Proceed with sync
    success, message, details = git_sync.safe_sync(Path.cwd())
    
    if success:
        print(f"✓ {message}")
        print(f"Summary:")
        print(f"  Pulled: {details['commits_pulled']} commits")
        print(f"  Pushed: {details['commits_pushed']} commits")
    else:
        print(f"✗ {message}")
```

### Example 4: Force Push with Confirmation

```python
from git_manager.core.git_push import GitPush
from pathlib import Path

git_push = GitPush()

# Show what would be pushed
success, preview = git_push.dry_run_push(Path.cwd())
if success:
    print("Preview:")
    print(preview)

# Ask for confirmation
confirm = input("Proceed with force push? (yes/no): ")

if confirm.lower() == "yes":
    success, message = git_push.force_push(Path.cwd())
    if success:
        print(f"✓ {message}")
    else:
        print(f"✗ {message}")
else:
    print("Cancelled")
```

### Example 5: Batch Operations

```python
from git_manager.core.git_push import GitPush
from git_manager.core.git_pull import GitPull
from pathlib import Path

git_push = GitPush()
git_pull = GitPull()

# Pull all branches
success, message, count = git_pull.pull_all_branches(Path.cwd())
print(f"Pulled {count} branches")

# Push all branches
success, message, branches = git_push.push_all_branches(Path.cwd())
print(f"Pushed {len(branches)} branches")
```

---

## Error Handling

All methods return `(success, message)` or `(success, message, details)` tuples.

Always check the `success` boolean:

```python
success, message, details = git_push.safe_push(repo_path)

if not success:
    # Handle error
    print(f"Error: {message}")
    # Log error, show user, etc.
else:
    # Handle success
    print(f"Success: {message}")
```

---

## Thread Safety

All methods are thread-safe when operating on different repositories. For the same repository, use locks to prevent concurrent operations.

---

## Performance

Typical operation times:

- **Safe Push:** 1-5 seconds
- **Safe Pull:** 1-5 seconds
- **Smart Sync:** 2-10 seconds
- **Pre-flight Checks:** 0.5-2 seconds
- **Fetch Only:** 0.5-2 seconds

Times vary based on repository size and network speed.

---

## Logging

All operations are logged to `~/.config/gitmanager/logs/`:

```
git_operations.log
git_push.log
git_pull.log
git_sync.log
```

Enable debug logging:

```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

---

## Summary

The Git Push, Pull, and Sync API provides:

✅ **Safe operations** with pre-flight checks
✅ **Multiple strategies** for different scenarios
✅ **Clear return values** for easy integration
✅ **Comprehensive logging** for debugging
✅ **Error handling** with helpful messages
✅ **Backup creation** before destructive ops

Start with `safe_push()`, `safe_pull()`, and `safe_sync()` for most use cases!
