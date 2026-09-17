# Quick Reference Guide

## Starting the Application

### CLI Mode
```bash
python3 -m git_manager --cli
```

### Desktop Mode
```bash
python3 -m git_manager --desktop
```

### Web Mode
```bash
python3 -m git_manager --web
# Access at http://localhost:5000
```

## Theme Management

### CLI Commands
```bash
# List all themes
python3 -m git_manager theme list

# Set theme
python3 -m git_manager theme set emerald_green

# View current theme
python3 -m git_manager theme current

# Preview theme
python3 -m git_manager theme preview royal_blue
```

### Web Interface
```
http://localhost:5000/theme
```
- Click any theme to apply
- Changes persist automatically

### Desktop Application
- Click **"Theme"** tab
- Select desired theme
- Restart for full effect

## Repository Operations

### Check Current Repository Account (CLI)
```
1. Start: python3 -m git_manager --cli
2. Select option: 2
3. View detailed account and status information
```

**Shows:**
- Account name, platform, username, email
- Current branch
- Uncommitted changes count
- Commits ahead/behind
- List of uncommitted files

### Clone Repository
```
1. Start: python3 -m git_manager --cli
2. Select option: 1
3. Enter repository URL
4. Select account to use
```

### Git Pull
```
1. Start: python3 -m git_manager --cli
2. Select option: 3
```

### Git Push
```
1. Start: python3 -m git_manager --cli
2. Select option: 4
```

## Account Management

### List All Accounts
```bash
python3 -m git_manager account list
```

### Add New Account
```bash
python3 -m git_manager account add \
  --name work-github \
  --platform github \
  --username myusername \
  --ssh-key ~/.ssh/id_rsa_work
```

### Generate SSH Key
```
1. Start: python3 -m git_manager --cli
2. Select option: 8
3. Follow prompts
4. Option to save as account
```

### Test SSH Connections
```
1. Start: python3 -m git_manager --cli
2. Select option: 7
3. Select account to test
```

## Available Themes

### Light Themes (9)
- pure_white
- soft_gray
- silver
- ivory
- warm_beige
- cream
- light_blue
- light_mint
- soft_yellow

### Dark Themes (7)
- jet_black (default)
- graphite
- charcoal
- dark_navy
- deep_purple
- forest_green
- coffee_brown

### Colored Themes (12)
- royal_blue
- electric_blue
- teal
- emerald_green
- leaf_green
- sunset_orange
- amber
- crimson_red
- burgundy
- purple_orchid
- magenta
- rose_pink

## Configuration Files

### Theme Configuration
```
Linux/Unix: ~/.config/git-manager/theme.json
macOS: ~/Library/Application Support/git-manager/theme.json
Windows: %APPDATA%\git-manager\theme.json
```

### Accounts Configuration
```
Linux/Unix: ~/.config/git-manager/accounts.json
macOS: ~/Library/Application Support/git-manager/accounts.json
Windows: %APPDATA%\git-manager\accounts.json
```

### Main Configuration
```
Linux/Unix: ~/.config/git-manager/config.yaml
macOS: ~/Library/Application Support/git-manager/config.yaml
Windows: %APPDATA%\git-manager\config.yaml
```

## Web API Endpoints

### Theme Endpoints
```bash
# Get current theme
GET /api/v1/theme/current

# List all themes
GET /api/v1/theme/list

# Set theme
POST /api/v1/theme/set
Body: {"theme": "theme_name"}
```

### Account Endpoints
```bash
# List accounts
GET /api/v1/accounts

# Get account details
GET /api/v1/accounts/<name>

# Add account
POST /api/v1/accounts
Body: {"name": "...", "platform": "...", ...}
```

### Repository Endpoints
```bash
# Get repository status
GET /api/v1/repositories/status

# Clone repository
POST /api/v1/repositories/clone
Body: {"url": "...", "account": "..."}
```

## Common Tasks

### Change Theme Across All Platforms

**CLI:**
```bash
python3 -m git_manager theme set royal_blue
python3 -m git_manager --cli
```

**Desktop:**
1. Click Theme tab
2. Select royal_blue
3. Restart app

**Web:**
1. Go to http://localhost:5000/theme
2. Click royal_blue theme

### Check Repository Status

**CLI:**
```bash
cd /path/to/repo
python3 -m git_manager --cli
# Select option 2
```

**Desktop:**
1. Click Repositories tab
2. View status information

**Web:**
1. Go to http://localhost:5000/repositories
2. View repository details

### Add New Git Account

**CLI:**
```bash
python3 -m git_manager account add \
  --name personal \
  --platform github \
  --username myname \
  --ssh-key ~/.ssh/id_rsa
```

**Desktop:**
1. Click Accounts tab
2. Click "Add Account"
3. Fill in details

**Web:**
1. Go to http://localhost:5000/accounts
2. Click "Add Account"
3. Fill in form

## Troubleshooting

### Theme Not Changing
```bash
# Check current theme
python3 -m git_manager theme current

# Reset to default
python3 -m git_manager theme set jet_black

# Delete config and restart
rm ~/.config/git-manager/theme.json
```

### Repository Not Found
```bash
# Ensure you're in a git repository
git status

# Check remote
git remote -v
```

### Account Not Recognized
```bash
# List configured accounts
python3 -m git_manager account list

# Add missing account
python3 -m git_manager account add ...
```

### SSH Connection Failed
```bash
# Test SSH connection
python3 -m git_manager --cli
# Select option 7

# Check SSH key
ssh-keygen -l -f ~/.ssh/id_rsa

# Test SSH directly
ssh -T git@github.com
```

## Keyboard Shortcuts (CLI)

- `1-9` - Select menu option
- `Ctrl+C` - Exit
- `Tab` - Navigate fields
- `Enter` - Confirm

## Tips & Tricks

1. **Use Themes for Different Contexts**
   - Light theme for presentations
   - Dark theme for coding
   - Colored themes for branding

2. **Multiple Accounts**
   - Create separate accounts for work/personal
   - Use different SSH keys
   - Easy account switching

3. **Repository Status**
   - Check status before committing
   - Verify account before pushing
   - Monitor uncommitted changes

4. **SSH Keys**
   - Generate unique keys per account
   - Use strong passphrases
   - Keep keys secure

5. **Performance**
   - Close unused tabs
   - Limit repository size
   - Use .gitignore effectively

## Getting Help

### View Logs
```bash
# Check application logs
ls ~/.config/git-manager/logs/

# View recent logs
tail -f ~/.config/git-manager/logs/git_manager.log
```

### Debug Mode
```bash
python3 -m git_manager --cli --debug
```

### Documentation
```
docs/THEME_SYSTEM_CROSSPLATFORM.md
docs/THEME_AND_REPOSITORY_FEATURES.md
docs/COLOR_SCHEMES.md
```

## Version Information
```bash
python3 -m git_manager --version
```

## Environment Variables

```bash
# Set log level
export GIT_MANAGER_LOG_LEVEL=DEBUG

# Set config directory
export GIT_MANAGER_CONFIG_DIR=/custom/path

# Enable JSON logs
export GIT_MANAGER_JSON_LOGS=1
```
