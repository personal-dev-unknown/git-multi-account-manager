# Clone Feature - Quick Reference Guide

## Main Menu
```
[1] Clone a repository
[2] Check repository status
[3] Setup repository
...
```

## Clone Type Selection
```
Choose clone type:
[1] Clone from external repository (any public/private repo)
[2] Clone from your personal repositories
[3] Back to main menu
```

## Personal Repositories Flow (Option 2)

### Step 1: Platform Selection
```
Select platform:
[1] 🐙 GitHub
[2] 🦊 GitLab
[3] 🗃️  Bitbucket
[4] ← Back
```

### Step 2: Account Selection
```
Select account [1/2/3/...]: 1
```

### Step 3: PAT Validation (if needed)
```
⚠️  Account 'devonionMoses' has no Personal Access Token (PAT)

Enter your Personal Access Token: ••••••••••••••••••••••••••••••••••••••••••••••

⏳ Testing Personal Access Token...
✓ PAT is valid!
```

### Step 4: Repository Selection with Pagination

**Navigation Options:**
- `[1-10]` - Select repository by number
- `[N]` - Next page
- `[P]` - Previous page
- `[S]` - Search repositories
- `[F]` - Filter by visibility
- `[M]` - Enter manual URL
- `[B]` - Back to main menu

**Search Example:**
```
Select repository [1-10] or action [N/P/S/F/M/B]: s

Search repositories by name: lab

📚 Your devonionMoses repositories (3 found)

[Shows matching repositories]
```

**Filter Example:**
```
Select repository [1-10] or action [N/P/S/F/M/B]: f

Filter options:
[1] Private repositories
[2] Public repositories
[3] All repositories

Select filter [1/2/3] (3): 1

[Shows filtered repositories]
```

### Step 5: Authentication Method
```
Authentication method for private repository:

[1] 🔑 SSH (Recommended)
    Using: ~/.ssh/gitmanager/github-work

[2] 🎫 HTTPS with Personal Access Token
    Token: gh_****abcd1234

Select method [1/2] (1): 1
```

### Step 6: Clone Options
```
Clone submodules? [y/n] (n): n
Shallow clone? [y/n] (n): n
```

### Step 7: Clone Destination
```
Default: ~/projects/devonionMoses/backend-api

Use custom location? [y/n] (n): n
```

### Step 8: Clone Execution
```
⏳ Cloning Repository...

✓ Clone successful!
📁 Location: ~/projects/devonionMoses/backend-api

═══ Next Steps ═══
1. cd ~/projects/devonionMoses/backend-api
2. Start working on the code
3. Use 'gitmanager push' to push your changes
4. Use 'gitmanager status' to check repository status
```

## Error Handling

### Authentication Failed
```
❌ Authentication Failed

Problem: Could not authenticate with the Git platform

Possible causes:
  • SSH key not added to your account
  • SSH key has wrong permissions (should be 600)
  • PAT is invalid or expired
  • Wrong username/password

💡 Solutions:
  [1] Test SSH connection: ssh -T git@github.com
  [2] Add SSH key to your account settings
  [3] Generate a new PAT with correct scopes
  [4] Try a different authentication method
```

### Permission Denied
```
❌ Permission Denied

Problem: You don't have access to this repository

Possible causes:
  • Repository is private and you're not a collaborator
  • You're using the wrong account
  • Organization requires SSO authentication

💡 Solutions:
  [1] Request access from repository owner
  [2] Check if you're using the correct account
  [3] Enable SSO for your PAT if required
  [4] Verify the repository URL is correct
```

### Repository Not Found
```
❌ Repository Not Found

Problem: The repository could not be found

Possible causes:
  • Repository doesn't exist
  • Repository is private (appears as 'not found')
  • URL is misspelled
  • Repository was deleted or renamed

💡 Solutions:
  [1] Verify URL on the Git platform website
  [2] Check if you have access (may need authentication)
  [3] Try with authentication enabled
  [4] Search for the repository on the platform
```

### Network Error
```
❌ Network Error

Problem: Failed to connect to the Git platform

Possible causes:
  • No internet connection
  • Git platform is down
  • Firewall blocking connection
  • Proxy configuration needed

💡 Solutions:
  [1] Check your internet connection
  [2] Check platform status page
  [3] Check firewall/proxy settings
  [4] Try again in a moment
```

### Disk Space Error
```
❌ Insufficient Disk Space

Problem: Not enough disk space to clone repository

💡 Solutions:
  [1] Free up disk space
  [2] Use shallow clone (--depth=1) to save space
  [3] Clone to a different location with more space
  [4] Use sparse checkout to clone specific folders
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `1-10` | Select repository |
| `N` | Next page |
| `P` | Previous page |
| `S` | Search |
| `F` | Filter |
| `M` | Manual URL |
| `B` | Back |

## Common Tasks

### Clone a Repository with 47 Repos
1. Select option 2 (Personal repositories)
2. Select platform and account
3. Press `N` to go to next page (shows repos 11-20)
4. Press `N` again to go to page 3 (shows repos 21-30)
5. Press `N` again to go to page 4 (shows repos 31-40)
6. Press `N` again to go to page 5 (shows repos 41-47)
7. Select repository by number

### Search for a Specific Repository
1. Select option 2 (Personal repositories)
2. Select platform and account
3. Press `S` to search
4. Enter search term (e.g., "lab")
5. Select repository from search results

### Clone Only Private Repositories
1. Select option 2 (Personal repositories)
2. Select platform and account
3. Press `F` to filter
4. Select option 1 (Private repositories)
5. Select repository from filtered list

### Clone Using Manual URL
1. Select option 2 (Personal repositories)
2. Select platform and account
3. Press `M` to enter manual URL
4. Enter repository URL (e.g., github.com/user/repo)
5. Proceed with authentication and clone

## Tips & Tricks

✅ **Pagination:** Automatically handles 100+ repositories
✅ **Search:** Case-insensitive, searches by repository name
✅ **Filter:** Quickly find private or public repositories
✅ **Manual URL:** Useful for repositories not in your account
✅ **Error Messages:** Always provide helpful solutions
✅ **SSH Recommended:** More secure than PAT
✅ **Shallow Clone:** Use `--depth=1` for large repositories

## Troubleshooting

**Q: I have 100+ repositories, how do I find the one I want?**
A: Use search [S] or filter [F] to narrow down the list, then navigate pages.

**Q: Clone failed with authentication error, what should I do?**
A: Follow the solutions in the error message:
   1. Test SSH connection: `ssh -T git@github.com`
   2. Add SSH key to your account
   3. Generate a new PAT with correct scopes
   4. Try a different authentication method

**Q: Can I enter a repository URL manually?**
A: Yes! Press [M] to enter a manual URL instead of selecting from the list.

**Q: What's the difference between SSH and PAT?**
A: SSH is more secure and doesn't require password entry. PAT is easier to revoke but requires token management.

**Q: How do I clone a private repository I don't own?**
A: Use option 1 (External repository) and enter the repository URL. You'll need access to clone it.
