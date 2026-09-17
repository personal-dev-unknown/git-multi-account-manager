# Database Strategy - SQLite + JSON Hybrid Approach

## Current Architecture

Your application uses a **hybrid approach** combining JSON files and SQLite:

### **JSON Files (Current Primary Storage)**
```
~/.config/git-manager/
├── accounts.json          # Account configurations
├── config.yaml            # Application settings
└── ssh_mappings.yaml      # SSH host mappings
```

### **SQLite Database (Recommended for Future)**
```
~/.config/git-manager/gitmanager.db
```

## Why Hybrid Approach?

### **JSON Files - Best For:**
- ✅ Simple configuration storage
- ✅ Human-readable format
- ✅ Easy backup/restore
- ✅ No database setup required
- ✅ Works offline
- ✅ Current implementation

### **SQLite - Best For:**
- ✅ Complex queries
- ✅ Relationships between data (accounts ↔ SSH keys ↔ repositories)
- ✅ Transaction support
- ✅ Large datasets
- ✅ Activity logging
- ✅ Future scalability

## Recommended Migration Path

### **Phase 1 (Current)** - JSON Storage
```
accounts.json
├── Account configurations
├── SSH key metadata
└── Repository mappings
```

### **Phase 2 (Recommended)** - SQLite Database
```
gitmanager.db
├── ssh_keys table
│   ├── id, key_name, email
│   ├── key_path, public_key
│   └── created_at, last_used
├── accounts table
│   ├── id, account_name, platform
│   ├── username, email, ssh_key_id
│   └── host_alias, description
├── repositories table
│   ├── id, repo_name, repo_path
│   ├── remote_url, account_id
│   ├── default_branch, created_at
│   └── last_synced
└── activity_logs table
    ├── id, action, timestamp
    ├── account_id, repository_id
    └── status, details
```

## Database Schema (When Migrating to SQLite)

### **SSH Keys Table**
```sql
CREATE TABLE ssh_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_name TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    key_path TEXT NOT NULL,
    public_key TEXT NOT NULL,
    key_type TEXT DEFAULT 'ed25519',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_used TIMESTAMP,
    is_active BOOLEAN DEFAULT 1
);
```

### **Accounts Table**
```sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_name TEXT UNIQUE NOT NULL,
    platform TEXT NOT NULL,  -- 'github', 'gitlab'
    username TEXT NOT NULL,
    email TEXT,
    ssh_key_id INTEGER NOT NULL,
    host_alias TEXT,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1,
    FOREIGN KEY (ssh_key_id) REFERENCES ssh_keys(id)
);
```

### **Repositories Table**
```sql
CREATE TABLE repositories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_name TEXT NOT NULL,
    repo_path TEXT NOT NULL UNIQUE,
    remote_url TEXT NOT NULL,
    account_id INTEGER NOT NULL,
    default_branch TEXT DEFAULT 'main',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_synced TIMESTAMP,
    is_active BOOLEAN DEFAULT 1,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);
```

### **Activity Logs Table**
```sql
CREATE TABLE activity_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action TEXT NOT NULL,  -- 'clone', 'push', 'pull', 'setup'
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    account_id INTEGER,
    repository_id INTEGER,
    status TEXT,  -- 'success', 'failed', 'pending'
    details TEXT,
    FOREIGN KEY (account_id) REFERENCES accounts(id),
    FOREIGN KEY (repository_id) REFERENCES repositories(id)
);
```

## Current Implementation (JSON)

### **accounts.json Structure**
```json
{
  "SKYREAPER-SPEC": {
    "name": "SKYREAPER-SPEC",
    "platform": "github",
    "username": "devonionrouting4Moses",
    "email": "muranja404@gmail.com",
    "ssh_key_path": "~/.ssh/gitmanager/SKYREAPER-SPEC",
    "host": "github-SKYREAPER-SPEC",
    "description": "GitHub account for SKYREAPER-SPEC"
  }
}
```

## SSH Configuration Integration

### **How SSH Config Linking Works**

1. **Main SSH Config** (`~/.ssh/config`)
```ssh
Include ~/.ssh/gitmanager/config
```

2. **Git Manager SSH Config** (`~/.ssh/gitmanager/config`)
```ssh
# Git account: github-SKYREAPER-SPEC
Host github-SKYREAPER-SPEC
    HostName github.com
    User git
    IdentityFile ~/.ssh/gitmanager/SKYREAPER-SPEC
    IdentitiesOnly yes
```

3. **How It Works**
```
User runs: git clone git@github-SKYREAPER-SPEC:username/repo.git
    ↓
SSH reads ~/.ssh/config
    ↓
Finds "Include ~/.ssh/gitmanager/config"
    ↓
Reads ~/.ssh/gitmanager/config
    ↓
Finds "Host github-SKYREAPER-SPEC"
    ↓
Uses IdentityFile: ~/.ssh/gitmanager/SKYREAPER-SPEC
    ↓
Connects to github.com with correct SSH key
```

## Data Flow

```
┌─────────────────────────────────────────────────────┐
│ User Input (Interactive Mode)                       │
└────────────────┬────────────────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────────────────┐
│ Account Manager                                     │
│ - Validates input                                   │
│ - Creates Account object                            │
└────────────────┬────────────────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────────────────┐
│ SSH Config Manager                                  │
│ - Adds entry to ~/.ssh/gitmanager/config           │
│ - Updates main ~/.ssh/config (Include statement)   │
└────────────────┬────────────────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────────────────┐
│ Storage (JSON or SQLite)                            │
│ - accounts.json (current)                           │
│ - gitmanager.db (future)                            │
└─────────────────────────────────────────────────────┘
```

## Migration Strategy (JSON → SQLite)

### **Step 1: Create SQLite Database**
```python
import sqlite3
from pathlib import Path

db_path = Path.home() / '.config' / 'git-manager' / 'gitmanager.db'
conn = sqlite3.connect(db_path)
cursor = conn.cursor()

# Create tables
cursor.execute('''CREATE TABLE ssh_keys (...)''')
cursor.execute('''CREATE TABLE accounts (...)''')
cursor.execute('''CREATE TABLE repositories (...)''')
cursor.execute('''CREATE TABLE activity_logs (...)''')

conn.commit()
conn.close()
```

### **Step 2: Migrate Data from JSON**
```python
import json
from pathlib import Path

# Read JSON
accounts_file = Path.home() / '.config' / 'git-manager' / 'accounts.json'
with open(accounts_file) as f:
    accounts = json.load(f)

# Write to SQLite
for account_name, account_data in accounts.items():
    cursor.execute('''
        INSERT INTO accounts (account_name, platform, username, email, ...)
        VALUES (?, ?, ?, ?, ...)
    ''', (account_data['name'], account_data['platform'], ...))
```

### **Step 3: Keep JSON as Backup**
- Keep JSON files for backward compatibility
- Use SQLite as primary database
- Sync both on changes

## Recommendation

### **For Now (Short Term)**
✅ Keep using JSON files
- Simple and working
- No additional dependencies
- Easy to understand and debug

### **For Future (Long Term)**
✅ Migrate to SQLite
- Better for complex relationships
- Supports transactions
- Better for logging and analytics
- Scales better with many accounts/repos

## Implementation Priority

1. **Phase 1 (Current)** - JSON storage ✅
2. **Phase 2** - Option 5: Repository Setup (uses JSON)
3. **Phase 3** - SQLite migration (optional)
4. **Phase 4** - Advanced features (activity logs, analytics)

## Summary

| Aspect | JSON | SQLite |
|--------|------|--------|
| **Current** | ✅ Primary | - |
| **Complexity** | Low | Medium |
| **Scalability** | Good | Excellent |
| **Relationships** | Limited | Full support |
| **Transactions** | No | Yes |
| **Logging** | Manual | Built-in |
| **Migration** | Easy | One-time |

Your application currently uses **JSON as primary storage**, which is perfect for the current use case. SQLite can be added later when you need more advanced features like activity logging, complex queries, and transaction support.
