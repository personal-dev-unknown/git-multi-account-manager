# Git Push, Pull & Sync Operations Guide

## Overview

GitManager provides a comprehensive, safe, and user-friendly system for pushing, pulling, and syncing Git repositories. This guide covers all available operations, their use cases, and best practices.

## Table of Contents

1. [Push Operations](#push-operations)
2. [Pull Operations](#pull-operations)
3. [Sync Operations](#sync-operations)
4. [Pre-Flight Checks](#pre-flight-checks)
5. [Safety Features](#safety-features)
6. [Common Scenarios](#common-scenarios)
7. [Troubleshooting](#troubleshooting)

---

## Push Operations

### 1. Safe Push (Recommended)

**What it does:** Pushes your local commits to remote with comprehensive safety checks.

**When to use:** Default choice for most situations.

**Safety checks:**
- Verifies remote connectivity
- Checks if remote has new commits
- Warns if branches have diverged
- Prevents pushing if no commits to push

**Command:**
```bash
gitmanager push --safe
```

**Interactive CLI:**
```
Select operation: 2
```

**Example output:**
```
✓ Pushed 3 commits
```

---

### 2. Push with Force-Lease (Safer Force Push)

**What it does:** Force pushes but only if no one else has pushed to remote since your last fetch.

**When to use:**
- You need to rewrite history (amend commits, rebase)
- You're the only one working on this branch
- You want safety against accidental overwrites

**Warning:** ⚠️ Will overwrite remote history if conditions are met.

**Command:**
```bash
gitmanager push --force-lease
```

**Interactive CLI:**
```
Select operation: 3
Proceed? yes
```

**Example output:**
```
✓ Force-with-lease push successful
Backup branch created: backup-before-force-main
```

---

### 3. Force Push (Dangerous)

**What it does:** Unconditionally overwrites remote history with your local commits.

**When to use:**
- You're absolutely certain you want to rewrite history
- You're the only one working on this branch
- You understand the consequences

**⚠️ WARNING:** Can cause data loss if others are working on the same branch.

**Requires confirmation:** Must type `FORCE PUSH` to proceed.

**Command:**
```bash
gitmanager push --force
```

**Interactive CLI:**
```
Select operation: 4
Type 'FORCE PUSH' to confirm: FORCE PUSH
```

**Example output:**
```
✓ Force push successful (backup created)
Backup branch: backup-before-force-push-main
```

---

### 4. Push All Branches

**What it does:** Pushes all local branches to remote.

**When to use:**
- You've created multiple feature branches
- You want to back up all branches
- You're syncing multiple branches at once

**Command:**
```bash
gitmanager push --all
```

**Interactive CLI:**
```
Select operation: 5
```

**Example output:**
```
✓ Pushed 5 branches
  • feature/auth
  • feature/database
  • bugfix/login
  • develop
  • main
```

---

### 5. Push with Tags

**What it does:** Pushes commits and all tags together.

**When to use:**
- You've created release tags
- You want to push both commits and tags
- You're preparing a release

**Command:**
```bash
gitmanager push --tags
```

**Interactive CLI:**
```
Select operation: 6
```

**Example output:**
```
✓ Pushed branch and tags
```

---

### 6. Dry Run Push

**What it does:** Shows what would be pushed without actually pushing.

**When to use:**
- You want to preview before pushing
- You're unsure about what will be pushed
- You want to verify your commits

**Command:**
```bash
gitmanager push --dry-run
```

**Interactive CLI:**
```
Select operation: 7
```

**Example output:**
```
Preview:
To github.com:user/repo.git
   abc1234..def5678  main -> main
```

---

## Pull Operations

### 1. Safe Pull (Recommended)

**What it does:** Pulls remote changes with automatic stash/unstash for uncommitted changes.

**When to use:** Default choice for most situations.

**How it works:**
1. Detects uncommitted changes
2. Stashes them temporarily
3. Fetches from remote
4. Merges remote changes
5. Reapplies stashed changes

**Command:**
```bash
gitmanager pull --safe
```

**Interactive CLI:**
```
Select operation: 8
```

**Example output:**
```
✓ Pulled 2 commits
```

---

### 2. Smart Pull

**What it does:** Automatically selects the best pull strategy based on repository state.

**When to use:**
- You want GitManager to decide the best approach
- You're not sure which strategy to use
- You want intelligent handling

**Strategies:**
- Uses **merge** if branches haven't diverged
- Uses **rebase** if branches have diverged (cleaner history)

**Command:**
```bash
gitmanager pull --smart
```

**Interactive CLI:**
```
Select operation: 9
```

**Example output:**
```
✓ Smart pull successful
```

---

### 3. Pull with Rebase

**What it does:** Pulls changes and rebases your local commits on top.

**When to use:**
- You want a clean, linear history
- You prefer rebase over merge
- You're working on a feature branch

**Benefit:** Avoids merge commits, keeps history clean.

**Command:**
```bash
gitmanager pull --rebase
```

**Interactive CLI:**
```
Select operation: 10
```

**Example output:**
```
✓ Pull with rebase successful (clean history)
```

---

### 4. Fast-Forward Only Pull

**What it does:** Pulls only if a fast-forward merge is possible.

**When to use:**
- You want the safest pull option
- You don't want merge commits
- You want to prevent accidental merges

**Benefit:** 100% safe - fails if merge would be needed.

**Command:**
```bash
gitmanager pull --ff-only
```

**Interactive CLI:**
```
Select operation: 11
```

**Example output:**
```
✓ Fast-forward pull successful (safest option)
```

---

### 5. Pull with Autostash

**What it does:** Automatically stashes/unstashes uncommitted changes during pull.

**When to use:**
- You have uncommitted changes
- You want automatic handling
- You don't want to manually stash

**Command:**
```bash
gitmanager pull --autostash
```

**Interactive CLI:**
```
Select operation: 12
```

**Example output:**
```
✓ Pull with autostash successful
```

---

### 6. Force Pull (Destructive)

**What it does:** Resets local repository to match remote exactly.

**When to use:**
- You want to discard all local changes
- You want to start fresh from remote
- You're sure you don't need local changes

**⚠️ WARNING:** Discards ALL uncommitted changes and unpushed commits.

**Requires confirmation:** Must type `DELETE MY WORK` to proceed.

**Command:**
```bash
gitmanager pull --force
```

**Interactive CLI:**
```
Select operation: 13
Type 'DELETE MY WORK' to confirm: DELETE MY WORK
```

**Example output:**
```
✓ Reset to origin/main (backup: backup-before-force-pull-main)
```

---

### 7. Fetch Only

**What it does:** Downloads changes from remote without integrating them.

**When to use:**
- You want to see what's on remote first
- You want 100% safe operation
- You want to review before merging

**Benefit:** Completely safe - makes no changes to your working directory.

**Command:**
```bash
gitmanager pull --fetch-only
```

**Interactive CLI:**
```
Select operation: 14
```

**Example output:**
```
✓ Fetch successful (100% safe, no changes made)
```

---

## Sync Operations

### Smart Sync (Recommended)

**What it does:** Intelligently synchronizes local and remote repositories.

**How it works:**
1. Checks current state
2. Stashes uncommitted changes
3. Fetches from remote
4. Pulls changes
5. Pushes local commits
6. Reapplies stashed changes

**When to use:** Default choice for bidirectional synchronization.

**Command:**
```bash
gitmanager sync --smart
```

**Interactive CLI:**
```
Select operation: 1
```

**Example output:**
```
✓ Sync completed successfully
```

---

## Pre-Flight Checks

Before any operation, GitManager runs comprehensive checks:

### Checks Performed

1. **Uncommitted Changes**
   - Detects modified files
   - Shows count of affected files
   - Warns if changes will be stashed

2. **Unpushed Commits**
   - Counts commits ahead of remote
   - Warns if commits are at risk

3. **New Remote Commits**
   - Detects commits on remote
   - Recommends pulling first

4. **Branch Divergence**
   - Checks if branches have diverged
   - Recommends appropriate strategy

5. **Remote Connectivity**
   - Verifies remote is reachable
   - Blocks operations if unreachable

### Example Output

```
═══ Git Push, Pull & Sync ═══

Running pre-flight checks...

Pre-Flight Warnings:
  ⚠️  Remote has 2 new commits. Consider pulling first.

Status Table:
┌─────────────────────┬────────────────────┐
│ Status              │ Value              │
├─────────────────────┼────────────────────┤
│ Uncommitted Changes │ 3 files            │
│ Unpushed Commits    │ 1 commit           │
│ New Remote Commits  │ 2 commits          │
│ Branches Diverged   │ No                 │
└─────────────────────┴────────────────────┘
```

---

## Safety Features

### 1. Automatic Backups

Before destructive operations, GitManager creates backup branches:

```
backup-before-force-push-main
backup-before-force-pull-main
backup-before-force-lease-main
```

These allow you to recover if something goes wrong.

### 2. Stash/Unstash Workflow

For operations with uncommitted changes:

1. Uncommitted changes are stashed
2. Operation proceeds
3. Changes are reapplied
4. Conflicts are reported if any

### 3. Confirmation Dialogs

Destructive operations require explicit confirmation:

```
🚨 WARNING: Force push will overwrite remote history!
Type 'FORCE PUSH' to confirm: _
```

### 4. Pre-Flight Checks

Every operation runs safety checks:
- Remote connectivity
- Branch state
- Uncommitted changes
- Conflict detection

### 5. Clear Error Messages

When something fails, you get helpful information:

```
❌ Push Failed: Cannot Reach GitHub

Problem: Unable to connect to GitHub
Possible causes:
  • No internet connection
  • GitHub is down
  • Firewall blocking connection

What you can do:
  1. Check your internet connection
  2. Try again in a moment
  3. Verify remote URL: git remote -v
```

---

## Common Scenarios

### Scenario 1: Simple Push

**Situation:** You've made commits and want to push them.

**Steps:**
1. Run `gitmanager --cli`
2. Select option 4 (Git push)
3. Select operation 2 (Safe Push)
4. Done!

**Result:** Your commits are safely pushed to remote.

---

### Scenario 2: Pull Latest Changes

**Situation:** Your team has pushed changes and you want to get them.

**Steps:**
1. Run `gitmanager --cli`
2. Select option 4 (Git push)
3. Select operation 8 (Safe Pull)
4. Done!

**Result:** Latest changes are pulled and merged.

---

### Scenario 3: Sync Everything

**Situation:** You want to sync both ways - pull new changes and push your commits.

**Steps:**
1. Run `gitmanager --cli`
2. Select option 4 (Git push)
3. Select operation 1 (Smart Sync)
4. Done!

**Result:** Everything is synchronized.

---

### Scenario 4: Rewrite History

**Situation:** You've amended commits and need to force push.

**Steps:**
1. Run `gitmanager --cli`
2. Select option 4 (Git push)
3. Select operation 3 (Push with Force-Lease)
4. Confirm with 'yes'
5. Done!

**Result:** History is rewritten safely (with backup).

---

### Scenario 5: Discard Local Changes

**Situation:** You've made mistakes and want to start fresh from remote.

**Steps:**
1. Run `gitmanager --cli`
2. Select option 4 (Git push)
3. Select operation 13 (Force Pull)
4. Confirm with 'DELETE MY WORK'
5. Done!

**Result:** Local repository matches remote exactly.

---

## Troubleshooting

### Problem: "Cannot reach remote"

**Causes:**
- No internet connection
- GitHub/GitLab is down
- Firewall blocking connection
- Wrong remote URL

**Solutions:**
1. Check internet connection
2. Check GitHub status: https://status.github.com
3. Verify remote URL: `git remote -v`
4. Check firewall settings

---

### Problem: "Merge conflicts"

**Causes:**
- Same lines edited in both local and remote
- Conflicting changes

**Solutions:**
1. Fetch to see what's on remote: `gitmanager pull --fetch-only`
2. Review conflicts in your editor
3. Resolve conflicts manually
4. Commit resolved changes
5. Push again

---

### Problem: "Push rejected"

**Causes:**
- Remote has new commits
- Branch protection rules
- Insufficient permissions

**Solutions:**
1. Pull first: `gitmanager pull --safe`
2. Check branch protection rules
3. Verify you have push permissions
4. Try again

---

### Problem: "Stash conflict"

**Causes:**
- Changes conflict when reapplying stash
- Same lines modified

**Solutions:**
1. Manually resolve conflicts
2. Use `git stash drop` to discard if not needed
3. Try operation again

---

### Problem: "Large files rejected"

**Causes:**
- File size exceeds remote limits
- Large binary files

**Solutions:**
1. Use Git LFS for large files
2. Split into smaller commits
3. Remove large files from history

---

## Best Practices

### 1. Always Use Safe Operations

- ✅ Use Safe Push/Pull by default
- ❌ Avoid Force Push unless necessary

### 2. Check Status First

- Always run pre-flight checks
- Review warnings before proceeding
- Use Fetch Only to preview

### 3. Commit Before Major Operations

- Commit or stash changes before pulling
- Don't leave uncommitted changes
- Use Safe Pull for auto-stash

### 4. Use Appropriate Strategies

- **Merge:** When integrating features
- **Rebase:** For clean linear history
- **Force-Lease:** For safe history rewrites
- **Force:** Only when absolutely necessary

### 5. Backup Important Work

- Create tags for releases
- Use backup branches
- Keep local copies of important commits

### 6. Communicate with Team

- Discuss force pushes with team
- Avoid force pushing shared branches
- Use protected branches for main/develop

---

## Advanced Usage

### Custom Sync Strategies

For complex workflows, combine operations:

```bash
# Pull with rebase, then push
gitmanager pull --rebase
gitmanager push --safe

# Fetch, review, then pull
gitmanager pull --fetch-only
# Review changes...
gitmanager pull --safe

# Backup before force push
gitmanager push --dry-run
gitmanager push --force-lease
```

### Protected Branches

For important branches (main, develop):

1. Enable branch protection on remote
2. Use Force-Lease instead of Force
3. Require pull requests for merges
4. Require status checks before merge

### Multiple Remotes

Work with multiple remotes:

```bash
# Push to specific remote
gitmanager push --remote upstream

# Pull from specific remote
gitmanager pull --remote upstream

# Sync with specific remote
gitmanager sync --remote upstream
```

---

## Configuration

### User Preferences

Store in `~/.config/gitmanager/config.json`:

```json
{
  "preferences": {
    "default_sync_strategy": "smart",
    "auto_stash": true,
    "confirm_destructive_ops": true,
    "show_advanced_options": false
  }
}
```

### Per-Repository Settings

Store in `.git/gitmanager.json`:

```json
{
  "repo_settings": {
    "sync_strategy": "merge",
    "protected": true,
    "default_remote": "origin",
    "default_branch": "main"
  }
}
```

---

## Summary

GitManager provides:

✅ **Safe by default** - Comprehensive checks and confirmations
✅ **Smart operations** - Intelligent strategy selection
✅ **Multiple options** - From simple to advanced
✅ **Clear feedback** - Helpful messages and warnings
✅ **Data protection** - Backups before destructive ops
✅ **Easy to use** - Interactive CLI with clear prompts

Start with Safe Push/Pull/Sync and explore advanced options as you become more comfortable!
