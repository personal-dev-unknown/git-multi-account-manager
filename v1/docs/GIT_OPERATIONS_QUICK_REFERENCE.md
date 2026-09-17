# Git Operations - Quick Reference Guide

## Main Menu (9 Options)

```
1. 🟢 Git Push      - Push commits to remote
2. 🟢 Git Pull      - Pull changes from remote
3. 🟢 Git Sync      - Bidirectional synchronization
4. 📊 Git Status    - Show detailed repository status
5. 🌿 Git Branch    - Manage branches
6. 📝 Git Stage     - Stage changes or stash
7. 💾 Git Commit    - Create commits
8. ❌ Cancel        - Do nothing
9. ⬅️  Go Back      - Return to main menu
```

## 1. Git Push Submenu (6 Options)

| Option | Name | Use Case |
|--------|------|----------|
| 1 | 🟢 Safe Push | Push with pre-flight checks (RECOMMENDED) |
| 2 | 🟡 Force-Lease | Safer force push when needed |
| 3 | 🟠 Force Push | Dangerous - requires "FORCE PUSH" confirmation |
| 4 | 🔵 Push All | Push all local branches |
| 5 | 🔵 Push Tags | Push commits and tags together |
| 6 | 🟣 Dry Run | Preview what would be pushed |

**When to use:**
- **Safe Push:** Default choice for most situations
- **Force-Lease:** When you need to overwrite but want safety
- **Force Push:** Only when absolutely necessary (requires confirmation)
- **Push All:** When working with multiple branches
- **Push Tags:** When releasing versions
- **Dry Run:** To preview before actual push

## 2. Git Pull Submenu (7 Options)

| Option | Name | Use Case |
|--------|------|----------|
| 1 | 🟢 Safe Pull | Pull with auto-stash (RECOMMENDED) |
| 2 | 🟢 Smart Pull | Intelligent strategy selection |
| 3 | 🔵 Rebase | Clean linear history |
| 4 | 🔵 FF-Only | Safest - fails if merge needed |
| 5 | 🟡 Autostash | Auto stash/unstash changes |
| 6 | 🟠 Force Pull | Reset to remote (DESTRUCTIVE) |
| 7 | 🟣 Fetch Only | Download without integrating |

**When to use:**
- **Safe Pull:** Default choice - handles most situations
- **Smart Pull:** Let system decide merge vs rebase
- **Rebase:** For feature branches (clean history)
- **FF-Only:** When you want to avoid complications
- **Autostash:** When you have uncommitted changes
- **Force Pull:** Only when you want to discard local changes
- **Fetch Only:** To review changes before pulling

## 3. Git Sync Submenu (6 Options)

| Option | Name | Use Case |
|--------|------|----------|
| 1 | 🟢 Smart Sync | Intelligent bidirectional sync (RECOMMENDED) |
| 2 | 🟢 Conservative | Extra-safe with confirmations |
| 3 | 🔵 Rebase | Sync with rebase for clean history |
| 4 | 🔵 Merge | Sync with merge commits |
| 5 | 🟡 Aggressive | Auto-resolve conflicts |
| 6 | 🟣 Dry Run | Preview what would happen |

**When to use:**
- **Smart Sync:** Default choice - handles everything
- **Conservative:** When you want confirmations at each step
- **Rebase Sync:** For feature branches
- **Merge Sync:** For main/develop branches
- **Aggressive Sync:** For solo work or experimental branches
- **Dry Run:** To preview before actual sync

## 4. Git Status

Shows:
- Current branch
- Remote branch
- Local commits ahead
- Remote commits ahead
- Uncommitted files count
- Status recommendations

## 5. Git Branch Submenu (6 Options)

| Option | Name | Action |
|--------|------|--------|
| 1 | List | Show all local branches |
| 2 | Create | Create a new branch |
| 3 | Switch | Switch to different branch |
| 4 | Delete | Delete a branch |
| 5 | Rename | Rename a branch |
| 6 | Info | Show branch information |

## 6. Git Stage/Stash Submenu (7 Options)

### Staging
| Option | Name | Action |
|--------|------|--------|
| 1 | Stage All | Stage all changes (git add .) |
| 2 | Stage File | Stage specific file |
| 3 | Reset All | Unstage all changes |
| 4 | Reset File | Unstage specific file |

### Stashing
| Option | Name | Action |
|--------|------|--------|
| 5 | Stash Save | Stash changes with optional message |
| 6 | Stash Pop | Pop stashed changes |
| 7 | Stash List | List all stashes |

## 7. Git Commit Submenu (6 Options)

| Option | Name | Action |
|--------|------|--------|
| 1 | Commit | Create commit from staged changes |
| 2 | Commit All | Stage and commit all changes |
| 3 | Amend | Amend last commit |
| 4 | Log | Show commit log (last 10) |
| 5 | Revert | Revert a commit |
| 6 | Reset | Reset to a specific commit |

## Common Workflows

### Simple Push
```
1. Make changes
2. Select "6" (Stage)
   → Select "1" (Stage All)
3. Select "7" (Commit)
   → Select "2" (Commit All)
   → Enter message
4. Select "1" (Push)
   → Select "1" (Safe Push)
```

### Pull Latest Changes
```
1. Select "2" (Pull)
   → Select "1" (Safe Pull)
```

### Sync Everything
```
1. Select "3" (Sync)
   → Select "1" (Smart Sync)
```

### Create and Push New Branch
```
1. Select "5" (Branch)
   → Select "2" (Create)
   → Enter branch name
2. Make changes
3. Select "6" (Stage)
   → Select "1" (Stage All)
4. Select "7" (Commit)
   → Select "2" (Commit All)
5. Select "1" (Push)
   → Select "1" (Safe Push)
```

## Safety Tips

✅ **Always use Safe Push/Pull by default**
✅ **Use Dry Run to preview before dangerous operations**
✅ **Create backups before force operations**
✅ **Read warnings carefully**
✅ **Confirm dangerous operations**
✅ **Check status before pushing**
✅ **Stash changes before pulling if needed**
✅ **Use Smart Sync for automatic handling**

## Dangerous Operations

⚠️ **Force Push** - Overwrites remote history
⚠️ **Force Pull** - Discards all local changes
⚠️ **Aggressive Sync** - Auto-resolves conflicts

**These require explicit confirmation!**

## File Locations

**Core Modules:**
- `src/git_manager/core/sync/push_operations.py`
- `src/git_manager/core/sync/pull_operations.py`
- `src/git_manager/core/sync/sync_operations.py`
- `src/git_manager/core/sync/git_status.py`
- `src/git_manager/core/sync/git_branch.py`
- `src/git_manager/core/sync/git_stage.py`
- `src/git_manager/core/sync/git_commit.py`

**Menu System:**
- `src/git_manager/cli/ui/git_operations_menu.py`

**Integration:**
- `src/git_manager/cli/ui/interactive.py` (git_push method)

## Keyboard Shortcuts

- `Ctrl+C` - Cancel current operation
- `Enter` - Use default choice
- `yes/no` - Confirm/reject operations

## Getting Help

For detailed information about each operation, see:
- `docs/GIT_OPERATIONS_IMPLEMENTATION.md` - Full implementation details
- Each operation class has docstrings with examples
