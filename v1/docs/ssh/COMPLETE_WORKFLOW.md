# Complete SSH Workflow Guide

## 🎯 Overview

This system provides **automated SSH key management** for multiple Git accounts across GitHub, GitLab, Bitbucket, Gitea, and custom servers. It handles the entire workflow from key generation to connection testing.

## 🔑 What This System Does

### Automated Workflow
1. **Generate SSH Keys** - Creates unique keys for each account
2. **Start SSH Agent** - Ensures agent is running
3. **Add Keys to Agent** - Loads keys for immediate use
4. **Configure SSH Config** - Sets up host aliases automatically
5. **Upload Keys** - Adds keys to GitHub/GitLab via API (optional)
6. **Test Connections** - Verifies everything works
7. **Convert URLs** - Automatically converts HTTPS to SSH with correct account

### Behind the Scenes
- **Smart URL conversion**: Paste any URL, system picks right SSH key
- **Account isolation**: Each account has its own key and alias
- **Automatic detection**: System knows which account to use
- **Remote fixing**: Automatically fixes incorrect remotes

---

## 🚀 Quick Start

### Setup Your First Account

```bash
# Interactive wizard (recommended)
git-manager ssh setup-account

# Or with options
git-manager ssh setup-account \
  --name devonionMoses \
  --email devonion@school.edu \
  --platform github \
  --account-type school \
  --upload
```

**The wizard will:**
1. ✅ Generate `~/.ssh/gitmanager/id_ed25519_devonionMoses`
2. ✅ Add to SSH agent
3. ✅ Create config entry for `github.com-devonionMoses`
4. ✅ Upload key to GitHub (if token provided)
5. ✅ Test connection
6. ✅ Show you're ready to use it!

---

## 📋 Complete Examples

### Example 1: School Account (GitHub)

```bash
git-manager ssh setup-account \
  --name devonionMoses \
  --email moses@school.edu \
  --platform github \
  --account-type school
```

**Results in:**
- Key: `~/.ssh/gitmanager/id_ed25519_devonionMoses`
- Alias: `github.com-devonionMoses`
- Config entry automatically added
- Ready to use!

**Clone repos with:**
```bash
git clone git@github.com-devonionMoses:SchoolOrg/project.git
```

### Example 2: Work Account (Zanabuni on GitHub)

```bash
git-manager ssh setup-account \
  --name drmuranja \
  --email dr@zanabuni.com \
  --platform github \
  --account-type zanabuni
```

**Results in:**
- Key: `~/.ssh/gitmanager/id_ed25519_drmuranja`
- Alias: `github.com-drmuranja`

**Clone repos with:**
```bash
git clone git@github.com-drmuranja:Zanabuni/react-frontend.git
```

### Example 3: Personal Account (GitLab)

```bash
git-manager ssh setup-account \
  --name devchiwhale \
  --email dev@personal.com \
  --platform gitlab \
  --account-type personal
```

**Results in:**
- Key: `~/.ssh/gitmanager/id_ed25519_devchiwhale`
- Alias: `gitlab.com-devchiwhale`

---

## 🔄 URL Conversion (The Magic Part!)

### Automatic HTTPS to SSH Conversion

The system automatically converts any URL format to the correct SSH URL:

```bash
# User pastes HTTPS URL
Input: https://github.com/Zanabuni/react-frontend.git

# System asks: "Which account?"
User selects: drmuranja (zanabuni work account)

# System converts to:
Output: git@github.com-drmuranja:Zanabuni/react-frontend.git

# Behind the scenes, this uses the correct SSH key automatically!
```

### CLI Command

```bash
# Convert URL
git-manager ssh convert-url \
  "https://github.com/Zanabuni/react-frontend.git" \
  --account drmuranja

# Output:
# SSH URL (with account 'drmuranja'):
#   git@github.com-drmuranja:Zanabuni/react-frontend.git
```

---

## 🔧 Fix Existing Repositories

### Problem: Cloned with HTTPS or Wrong Account

If you already cloned a repo with HTTPS or the wrong account:

```bash
# Navigate to repo
cd ~/projects/zanabuni-react-frontend

# Fix it!
git-manager ssh fix-remote --account drmuranja
```

**What happens:**
1. System detects current remote: `https://github.com/Zanabuni/react-frontend.git`
2. Converts to: `git@github.com-drmuranja:Zanabuni/react-frontend.git`
3. Updates remote
4. Verifies change

**Verify:**
```bash
git remote -v
# origin  git@github.com-drmuranja:Zanabuni/react-frontend.git (fetch)
# origin  git@github.com-drmuranja:Zanabuni/react-frontend.git (push)
```

---

## 👥 Multi-Account Setup

### Complete Setup for 4 Accounts

```bash
# 1. School account (GitHub)
git-manager ssh setup-account \
  --name devonionMoses \
  --email moses@school.edu \
  --platform github \
  --account-type school

# 2. Zanabuni work (GitHub)
git-manager ssh setup-account \
  --name drmuranja \
  --email dr@zanabuni.com \
  --platform github \
  --account-type zanabuni

# 3. Casini work (GitHub)
git-manager ssh setup-account \
  --name cipherscribesultan \
  --email cipher@casini.com \
  --platform github \
  --account-type casini

# 4. Personal (GitHub)
git-manager ssh setup-account \
  --name SKYREAPER-SPEC \
  --email sky@personal.com \
  --platform github \
  --account-type personal

# 5. Personal (GitLab)
git-manager ssh setup-account \
  --name devchiwhale \
  --email dev@personal.com \
  --platform gitlab \
  --account-type personal
```

### Result: Your SSH Config

After setup, your `~/.ssh/config` will contain:

```ini
# School account (devonionMoses)
Host github.com-devonionMoses
  HostName github.com
  User git
  IdentityFile ~/.ssh/gitmanager/id_ed25519_devonionMoses
  IdentitiesOnly yes

# Zanabuni account (drmuranja)
Host github.com-drmuranja
  HostName github.com
  User git
  IdentityFile ~/.ssh/gitmanager/id_ed25519_drmuranja
  IdentitiesOnly yes

# Casini account (cipherscribesultan)
Host github.com-cipherscribesultan
  HostName github.com
  User git
  IdentityFile ~/.ssh/gitmanager/id_ed25519_cipherscribesultan
  IdentitiesOnly yes

# Personal account (SKYREAPER-SPEC)
Host github.com-SKYREAPER-SPEC
  HostName github.com
  User git
  IdentityFile ~/.ssh/gitmanager/id_ed25519_SKYREAPER_SPEC
  IdentitiesOnly yes

# Personal GitLab (devchiwhale)
Host gitlab.com-devchiwhale
  HostName gitlab.com
  User git
  IdentityFile ~/.ssh/gitmanager/id_ed25519_devchiwhale
  IdentitiesOnly yes
```

---

## 🎬 Real-World Usage Scenarios

### Scenario 1: Clone School Project

```bash
# User selects: Clone repository
# System asks: Which account?
# User selects: devonionMoses (school)

# User pastes URL (any format works):
https://github.com/SchoolOrg/cs-project.git

# System converts to:
git clone git@github.com-devonionMoses:SchoolOrg/cs-project.git

# Behind the scenes: Uses id_ed25519_devonionMoses key
# ✓ Cloned successfully with correct account!
```

### Scenario 2: Push to Zanabuni Work Project

```bash
cd ~/projects/zanabuni-react-frontend

# User: "Push changes"
# System: Checks remote URL
# Remote: git@github.com-drmuranja:Zanabuni/react-frontend.git
# System: Uses id_ed25519_drmuranja key automatically

git push origin main
# ✓ Pushed successfully as drmuranja account!
```

### Scenario 3: Fix Wrong Account

```bash
# Oops! Cloned with personal account instead of work
cd ~/projects/casini-project

# Current remote (wrong):
git remote -v
# origin  git@github.com-SKYREAPER-SPEC:CasiniOrg/project.git

# Fix it:
git-manager ssh fix-remote --account cipherscribesultan

# New remote (correct):
# origin  git@github.com-cipherscribesultan:CasiniOrg/project.git

# Now push works with correct account:
git push origin main
# ✓ Pushed as cipherscribesultan!
```

---

## 🔍 Testing & Verification

### Test All Connections

```bash
git-manager ssh test-connection
```

**Output:**
```
╭─────────────────── Connection Tests ───────────────────╮
│ Account                    │ Status      │ Username    │
├────────────────────────────┼─────────────┼─────────────┤
│ github.com-devonionMoses   │ ✓ Connected │ devonionMoses│
│ github.com-drmuranja       │ ✓ Connected │ drmuranja   │
│ github.com-cipherscribe... │ ✓ Connected │ cipherscr...│
│ github.com-SKYREAPER-SPEC  │ ✓ Connected │ SKYREAPER...│
│ gitlab.com-devchiwhale     │ ✓ Connected │ devchiwhale │
╰────────────────────────────┴─────────────┴─────────────╯
```

### List Configured Accounts

```bash
git-manager ssh list-accounts
```

**Output:**
```
╭─────────────── Configured SSH Accounts ───────────────╮
│ Alias                      │ Platform    │ Status     │
├────────────────────────────┼─────────────┼────────────┤
│ github.com-devonionMoses   │ github.com  │ ✓ Active   │
│ github.com-drmuranja       │ github.com  │ ✓ Active   │
│ github.com-cipherscribe... │ github.com  │ ✓ Active   │
│ github.com-SKYREAPER-SPEC  │ github.com  │ ✓ Active   │
│ gitlab.com-devchiwhale     │ gitlab.com  │ ✓ Active   │
╰────────────────────────────┴─────────────┴────────────╯

Total: 5 accounts configured
```

---

## 🔐 API Token Setup (Optional)

### Why Tokens?

Tokens enable automatic key upload to GitHub/GitLab. **Without tokens, you manually copy/paste keys** (which is fine!).

### Setup GitHub Token

```bash
# 1. Create token at: https://github.com/settings/tokens
# 2. Select scopes: admin:public_key (minimum)
# 3. Copy token

# 4. Add to environment
export GITHUB_TOKEN="ghp_xxxxxxxxxxxx"

# Or save to config
mkdir -p ~/.config/git-manager
echo "ghp_xxxxxxxxxxxx" > ~/.config/git-manager/github_token
chmod 600 ~/.config/git-manager/github_token
```

### Setup GitLab Token

```bash
# 1. Create token at: https://gitlab.com/-/profile/personal_access_tokens
# 2. Select scopes: api (full access)
# 3. Copy token

# 4. Add to environment
export GITLAB_TOKEN="glpat-xxxxxxxxxxxx"

# Or save to config
echo "glpat-xxxxxxxxxxxx" > ~/.config/git-manager/gitlab_token
chmod 600 ~/.config/git-manager/gitlab_token
```

### Using Tokens in Setup

```bash
# With token in environment
git-manager ssh setup-account \
  --name myaccount \
  --email my@email.com \
  --platform github \
  --account-type work \
  --upload  # ← Will use GITHUB_TOKEN

# Without token (manual upload)
git-manager ssh setup-account \
  --name myaccount \
  --email my@email.com \
  --platform github \
  --account-type work \
  --no-upload  # ← Will show manual instructions
```

---

## 📱 Integration with Main App

### In Interactive Mode

When user selects "Clone repository":

```python
# System workflow:
1. Ask user for repository URL (any format)
2. Ask which account to use (show list)
3. Convert URL to SSH with correct alias
4. Clone using converted URL
5. Done! ✓
```

### In Option Menu

```
Git Multi-Account Manager
=========================

Repository Operations:
[1] Clone repository          ← Uses SSH workflow
[2] Check current account     ← Shows which SSH key is used
[3] Pull changes             ← Uses configured SSH
[4] Push changes             ← Uses configured SSH
[5] Fix remote URL           ← Uses fix-remote workflow

SSH Management:
[6] Setup new account        ← Complete SSH setup
[7] Test connections         ← Test all SSH keys
[8] List accounts            ← Show configured accounts
[9] Convert URL              ← Preview SSH conversion

[0] Exit
```

---

## 🛡️ Security Features

### Key Organization
- All keys stored in `~/.ssh/gitmanager/`
- Separate directory from system keys
- Each account has unique key
- Proper permissions (600) enforced

### Passphrase Support
```bash
# With passphrase (recommended)
git-manager ssh setup-account \
  --name myaccount \
  --email my@email.com \
  --platform github \
  --passphrase

# System will prompt securely:
# Enter passphrase: ********
# Confirm passphrase: ********
```

### Key Isolation
- `IdentitiesOnly yes` prevents key confusion
- Each host alias uses only its specific key
- No accidental key leakage between accounts

---

## 🐛 Troubleshooting

### Connection Fails

```bash
# Test specific account
git-manager ssh test-connection --account devonionMoses

# Check SSH config
cat ~/.ssh/config | grep -A5 "github.com-devonionMoses"

# Verify key exists
ls -la ~/.ssh/gitmanager/id_ed25519_devonionMoses*

# Test manually
ssh -T git@github.com-devonionMoses
```

### Wrong Account Used

```bash
# Check current remote
cd /path/to/repo
git remote -v

# Fix remote
git-manager ssh fix-remote --account correct-account

# Verify
git remote -v
```

### Key Not Found

```bash
# Check SSH agent
ssh-add -l

# Add key manually
ssh-add ~/.ssh/gitmanager/id_ed25519_accountname

# Or restart workflow
git-manager ssh setup-account --name accountname ...
```

---

## 📊 Directory Structure

```
~/.ssh/
├── config                              # SSH configuration
├── gitmanager/                         # All Git Manager keys
│   ├── id_ed25519_devonionMoses
│   ├── id_ed25519_devonionMoses.pub
│   ├── id_ed25519_drmuranja
│   ├── id_ed25519_drmuranja.pub
│   ├── id_ed25519_cipherscribesultan
│   ├── id_ed25519_cipherscribesultan.pub
│   ├── id_ed25519_SKYREAPER_SPEC
│   ├── id_ed25519_SKYREAPER_SPEC.pub
│   ├── id_ed25519_devchiwhale
│   └── id_ed25519_devchiwhale.pub
└── known_hosts                         # SSH known hosts

~/.config/git-manager/
├── github_token                        # Optional: GitHub token
├── gitlab_token                        # Optional: GitLab token
└── accounts.db                         # Account database
```

---

## ✅ Best Practices

1. **Use descriptive account types**: school, work-zanabuni, work-casini, personal
2. **Test after setup**: Always run test-connection after adding account
3. **Fix remotes immediately**: If you cloned with HTTPS, fix it right away
4. **Use passphrases**: Add extra security with key passphrases
5. **Keep tokens secure**: Never commit tokens to repositories
6. **Regular testing**: Periodically test all connections

---

## 🎓 Summary

### What You Get

✅ **Automated SSH setup** - No manual config editing
✅ **Multi-account support** - Unlimited accounts, any platform
✅ **Smart URL conversion** - Paste any URL, system handles it
✅ **Automatic key selection** - Right key for right account, always
✅ **Remote fixing** - Convert existing repos easily
✅ **Connection testing** - Verify everything works
✅ **API integration** - Optional automatic key upload

### What You Don't Need to Remember

❌ SSH config syntax
❌ Which key for which account
❌ Manual URL conversion
❌ Host alias naming
❌ Permission settings

**The system handles everything!** Just provide account details, system does the rest.