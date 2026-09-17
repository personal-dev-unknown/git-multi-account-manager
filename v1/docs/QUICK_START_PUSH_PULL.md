# Quick Start: Push, Pull & Sync

## TL;DR

```bash
# Start GitManager
python3 -m git_manager --cli

# Select option 4: Git push
# Choose your operation (1-15)
```

---

## The 15 Operations

### Sync (1)
```
1. Smart Sync (Recommended)
   → Pulls, pushes, and syncs everything automatically
```

### Push Operations (2-7)
```
2. Safe Push
   → Push commits with safety checks ✅ RECOMMENDED

3. Push with Force-Lease
   → Safer force push (force-with-lease)

4. Force Push
   → Dangerous! Requires "FORCE PUSH" confirmation

5. Push All Branches
   → Push all local branches

6. Push with Tags
   → Push commits and tags together

7. Dry Run Push
   → Preview what would be pushed (100% safe)
```

### Pull Operations (8-14)
```
8. Safe Pull
   → Pull with auto-stash ✅ RECOMMENDED

9. Smart Pull
   → Intelligent strategy selection

10. Pull with Rebase
    → Clean linear history

11. Fast-Forward Only
    → Safest pull option (fails if merge needed)

12. Pull with Autostash
    → Auto stash/unstash

13. Force Pull
    → Reset to remote (DESTRUCTIVE!)
    → Requires "DELETE MY WORK" confirmation

14. Fetch Only
    → Download only (100% safe)
```

---

## Common Scenarios

### "I want to push my commits"
```
Select operation: 2 (Safe Push)
✓ Done!
```

### "I want to get latest changes"
```
Select operation: 8 (Safe Pull)
✓ Done!
```

### "I want to sync everything"
```
Select operation: 1 (Smart Sync)
✓ Done!
```

### "I want to preview before pushing"
```
Select operation: 7 (Dry Run Push)
# Review output
Select operation: 2 (Safe Push)
✓ Done!
```

### "I have uncommitted changes and want to pull"
```
Select operation: 8 (Safe Pull)
# Changes are auto-stashed and reapplied
✓ Done!
```

### "I need to rewrite history"
```
Select operation: 3 (Push with Force-Lease)
Proceed? yes
✓ Done! (Backup created)
```

### "I want to discard everything and start fresh"
```
Select operation: 13 (Force Pull)
Type 'DELETE MY WORK' to confirm: DELETE MY WORK
✓ Done! (Backup created)
```

---

## Safety Features

✅ **Pre-flight Checks**
- Checks remote connectivity
- Warns about uncommitted changes
- Warns if remote has new commits
- Detects branch divergence

✅ **Automatic Backups**
- Created before force operations
- Named: `backup-before-force-{operation}-{branch}`
- Allows recovery if needed

✅ **Confirmations**
- Destructive ops require explicit confirmation
- Type-based (e.g., "DELETE MY WORK")
- Clear warnings

✅ **Auto-Stash**
- Uncommitted changes are stashed
- Automatically reapplied after operation
- Prevents data loss

---

## What Each Operation Does

| # | Operation | What It Does | Risk | When to Use |
|---|-----------|-------------|------|------------|
| 1 | Smart Sync | Pull + Push | Low | Default sync |
| 2 | Safe Push | Push commits | Low | Default push |
| 3 | Force-Lease | Safer force push | Medium | Rewrite history safely |
| 4 | Force Push | Dangerous force | High | Only if necessary |
| 5 | Push All | All branches | Low | Multiple branches |
| 6 | Push Tags | Commits + tags | Low | Releases |
| 7 | Dry Run | Preview only | None | Before pushing |
| 8 | Safe Pull | Pull changes | Low | Default pull |
| 9 | Smart Pull | Auto strategy | Low | Intelligent pull |
| 10 | Rebase | Clean history | Medium | Feature branches |
| 11 | FF Only | Safest pull | None | Maximum safety |
| 12 | Autostash | Auto stash | Low | With uncommitted |
| 13 | Force Pull | Reset to remote | High | Discard everything |
| 14 | Fetch Only | Download only | None | Preview first |

---

## Color Coding

- 🟢 **Green** - Safe, recommended
- 🟡 **Yellow** - Caution, requires confirmation
- 🟠 **Orange** - Dangerous, requires confirmation
- 🔴 **Red** - Very dangerous, requires confirmation
- 🔵 **Blue** - Specialized use
- 🟣 **Purple** - Preview/utility

---

## Decision Tree

```
Do you want to...

├─ SYNC (both ways)?
│  └─ Select 1: Smart Sync ✅
│
├─ PUSH (upload commits)?
│  ├─ Normal push?
│  │  └─ Select 2: Safe Push ✅
│  ├─ Rewrite history?
│  │  └─ Select 3: Force-Lease ⚠️
│  ├─ Push all branches?
│  │  └─ Select 5: Push All ✅
│  ├─ Push with tags?
│  │  └─ Select 6: Push Tags ✅
│  └─ Preview first?
│     └─ Select 7: Dry Run ✅
│
├─ PULL (download changes)?
│  ├─ Normal pull?
│  │  └─ Select 8: Safe Pull ✅
│  ├─ Clean history?
│  │  └─ Select 10: Rebase ✅
│  ├─ Safest option?
│  │  └─ Select 11: FF Only ✅
│  ├─ Have uncommitted changes?
│  │  └─ Select 12: Autostash ✅
│  ├─ Preview first?
│  │  └─ Select 14: Fetch Only ✅
│  └─ Discard everything?
│     └─ Select 13: Force Pull 🚨
│
└─ CANCEL?
   └─ Select 15: Cancel
```

---

## Keyboard Shortcuts

| Action | Command |
|--------|---------|
| Start GitManager | `python3 -m git_manager --cli` |
| Select option 4 | Type `4` and press Enter |
| Select operation | Type `1-15` and press Enter |
| Confirm | Type `yes` or required text |
| Cancel | Type `no` or press Ctrl+C |

---

## Error Messages

### "Cannot reach remote"
**Problem:** Network issue or wrong URL
**Solution:** Check internet, verify remote URL

### "Merge conflicts"
**Problem:** Same lines edited in both places
**Solution:** Resolve manually, then commit

### "Push rejected"
**Problem:** Remote has new commits
**Solution:** Pull first, then push

### "Stash conflict"
**Problem:** Stashed changes conflict
**Solution:** Resolve manually, use `git stash drop` if needed

---

## Pro Tips

1. **Always use Safe Push/Pull first**
   - They have built-in safety checks
   - Auto-stash handles uncommitted changes

2. **Use Dry Run before Force Push**
   - Preview what will happen
   - Verify before executing

3. **Create Backups**
   - Force operations create backup branches
   - Keep them until you're sure

4. **Use Fetch Only to Preview**
   - Download without integrating
   - Review before pulling

5. **Communicate with Team**
   - Discuss force pushes
   - Avoid force pushing shared branches

6. **Use Fast-Forward Only for Safety**
   - Fails if merge needed
   - Maximum safety option

---

## Troubleshooting

### "I pushed the wrong thing"
```
1. Check backup branch: git branch -a
2. Find: backup-before-force-push-{branch}
3. Reset to it: git reset --hard backup-before-force-push-{branch}
4. Push: Select 2 (Safe Push)
```

### "I lost my changes"
```
1. Check stash: git stash list
2. Restore: git stash pop
3. Or use backup branch if available
```

### "Merge conflicts"
```
1. Open files with conflicts
2. Resolve (keep both, choose one, or combine)
3. Stage resolved files: git add .
4. Commit: git commit -m "Resolve conflicts"
5. Push: Select 2 (Safe Push)
```

### "Remote is ahead"
```
1. Pull first: Select 8 (Safe Pull)
2. Then push: Select 2 (Safe Push)
```

---

## Cheat Sheet

```bash
# Most common workflow:
# 1. Pull latest
Select 8: Safe Pull

# 2. Make changes
# 3. Push your work
Select 2: Safe Push

# Or sync everything:
Select 1: Smart Sync

# To preview before pushing:
Select 7: Dry Run Push

# To see what's on remote:
Select 14: Fetch Only

# To discard everything:
Select 13: Force Pull
Type: DELETE MY WORK
```

---

## Remember

✅ **Safe by default** - All operations have safety checks
✅ **Backups created** - Before destructive operations
✅ **Confirmations required** - For dangerous operations
✅ **Auto-stash** - Uncommitted changes are preserved
✅ **Clear messages** - Know what's happening

**Start with Safe Push/Pull and explore advanced options as you get comfortable!**

---

## Need More Help?

- **User Guide:** `docs/PUSH_PULL_SYNC_GUIDE.md`
- **API Reference:** `docs/PUSH_PULL_SYNC_API.md`
- **Full Spec:** `docs/implementation/impl.md`

Happy pushing! 🚀
