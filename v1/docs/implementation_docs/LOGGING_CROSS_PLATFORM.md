# Cross-Platform Logging System

## Overview

The Git Multi-Account Manager uses a **unified logging system** that works seamlessly across all three platforms:

- **Command Line Interface (CLI)**
- **Desktop Application (GUI)**
- **Web Application**

All logs are stored in the **same location** and can be viewed/managed through a single interface.

## Unified Log Storage

Regardless of which platform you use, all logs are stored in:

### System-Wide Installation
```
/var/log/git-manager/
├── activity.log
├── error.log
├── security.log
├── performance.log
├── git_operation.log
├── ssh_operation.log
└── audit.log
```

### User Installation
```
~/.git-manager/logs/
├── activity.log
├── error.log
├── security.log
├── performance.log
├── git_operation.log
├── ssh_operation.log
└── audit.log
```

## Platform-Specific Configuration

### CLI Platform
```python
# Text format (human-readable)
initialize_logging(
    log_level=LogLevel.INFO,
    use_json=False,
    enable_console=True,  # Show logs in terminal
)
```

**Features:**
- Console output for immediate feedback
- Text format for easy reading
- Optional JSON format with `--json-logs` flag

**Example:**
```bash
git-manager --cli logs view
git-manager --cli logs info
git-manager --cli logs export
```

### Desktop Platform
```python
# JSON format (structured)
initialize_logging(
    log_level=LogLevel.INFO,
    use_json=False,
    enable_console=False,  # No console for GUI
)
```

**Features:**
- No console output (GUI-only)
- Logs written to files silently
- Can be viewed via CLI commands
- Useful for debugging GUI issues

**Example:**
```bash
# After running desktop app, view logs with:
git-manager --cli logs view --category activity
```

### Web Platform
```python
# JSON format (structured for analysis)
initialize_logging(
    log_level=LogLevel.INFO,
    use_json=True,  # Structured logging
    enable_console=False,  # No console for web server
)
```

**Features:**
- JSON structured logging for machine parsing
- No console output
- Perfect for log aggregation services
- Can be viewed via CLI commands

**Example:**
```bash
# After running web server, view logs with:
git-manager --cli logs view --category activity
```

## Cross-Platform Usage Examples

### Scenario 1: Using CLI, then checking Desktop logs

```bash
# 1. Run CLI operations
git-manager --cli
# Select options and perform operations

# 2. View logs from CLI
git-manager --cli logs view

# 3. Run Desktop app
git-manager --desktop

# 4. Perform operations in Desktop

# 5. View all logs (CLI + Desktop combined)
git-manager --cli logs view
```

### Scenario 2: Web server logging

```bash
# 1. Start web server
git-manager --web

# 2. In another terminal, view web logs
git-manager --cli logs view --category activity

# 3. Export all logs for analysis
git-manager --cli logs export --output web-logs.txt
```

### Scenario 3: Monitoring all platforms

```bash
# Terminal 1: Run CLI
git-manager --cli

# Terminal 2: Run Desktop
git-manager --desktop

# Terminal 3: Run Web
git-manager --web

# Terminal 4: Monitor logs
watch -n 1 'git-manager --cli logs view --lines 20'
```

## Log Categories Across Platforms

Each platform logs to the same categories:

| Category | CLI | Desktop | Web | Example |
|----------|-----|---------|-----|---------|
| **activity** | ✓ | ✓ | ✓ | App startup, user actions |
| **error** | ✓ | ✓ | ✓ | Exceptions, failures |
| **security** | ✓ | ✓ | ✓ | SSH operations, auth |
| **performance** | ✓ | ✓ | ✓ | Operation timing |
| **git_operation** | ✓ | ✓ | ✓ | Clone, push, pull |
| **ssh_operation** | ✓ | ✓ | ✓ | SSH connections |
| **audit** | ✓ | ✓ | ✓ | Important actions |

## Real-Time Log Monitoring

### Monitor all platforms simultaneously

```bash
# Terminal 1: CLI
git-manager --cli

# Terminal 2: Desktop
git-manager --desktop

# Terminal 3: Web
git-manager --web

# Terminal 4: Watch logs in real-time
tail -f ~/.git-manager/logs/activity.log
```

### Watch specific category

```bash
# Watch git operations
tail -f ~/.git-manager/logs/git_operation.log

# Watch errors
tail -f ~/.git-manager/logs/error.log

# Watch security events
tail -f ~/.git-manager/logs/security.log
```

## Log Format Differences

### CLI (Text Format)
```
[2025-11-21 04:09:06] [INFO    ] [activity] Starting Git Multi-Account Manager v1.0.0
[2025-11-21 04:09:35] [ERROR   ] [error] Failed: git_push - Permission denied
```

### Desktop/Web (JSON Format)
```json
{
  "timestamp": "2025-11-21T04:09:06.123456",
  "level": "INFO",
  "category": "activity",
  "logger": "git_manager.cli.app",
  "message": "Starting Git Multi-Account Manager v1.0.0",
  "module": "app",
  "function": "cli",
  "line": 52
}
```

## Analyzing Logs Across Platforms

### Find all errors from all platforms
```bash
grep "ERROR" ~/.git-manager/logs/*.log
```

### Find operations from specific platform
```bash
# CLI operations
grep "git_manager.cli" ~/.git-manager/logs/activity.log

# Desktop operations
grep "git_manager.desktop" ~/.git-manager/logs/activity.log

# Web operations
grep "git_manager.web" ~/.git-manager/logs/activity.log
```

### Export logs for analysis
```bash
# Export all logs
git-manager --cli logs export

# Export specific category
git-manager --cli logs export --output git-operations.txt
```

## Performance Monitoring Across Platforms

### View performance metrics from all platforms
```bash
git-manager --cli logs view --category performance
```

### Find slow operations
```bash
grep "WARNING" ~/.git-manager/logs/performance.log
```

### Compare platform performance
```bash
# CLI performance
grep "git_manager.cli" ~/.git-manager/logs/performance.log

# Desktop performance
grep "git_manager.desktop" ~/.git-manager/logs/performance.log

# Web performance
grep "git_manager.web" ~/.git-manager/logs/performance.log
```

## Security Audit Trail

All platforms log security events to the same audit log:

```bash
# View all security events
git-manager --cli logs view --category security

# View audit trail
git-manager --cli logs view --category audit

# Export security logs
git-manager --cli logs export --output security-audit.txt
```

## Troubleshooting Across Platforms

### Check if logging is working

```bash
# 1. Run any platform
git-manager --cli  # or --desktop or --web

# 2. Check logs exist
ls -la ~/.git-manager/logs/

# 3. View logs
git-manager --cli logs list

# 4. Check specific category
git-manager --cli logs view --category activity
```

### Debug platform-specific issues

```bash
# CLI debug
git-manager --cli --debug logs view

# Desktop debug (check logs after running)
git-manager --desktop
# Then check:
git-manager --cli logs view --category error

# Web debug (check logs after running)
git-manager --web
# Then check:
git-manager --cli logs view --category error
```

### Clear logs if needed

```bash
# Clear all logs
git-manager --cli logs clear

# Clear specific category
git-manager --cli logs clear --category error
```

## Integration with System Logging

### Forward logs to syslog (Linux)

```bash
# Create symlink
sudo ln -s ~/.git-manager/logs/activity.log /var/log/git-manager-activity.log

# Or use rsyslog to forward
echo "*.* @@localhost:514" | sudo tee -a /etc/rsyslog.d/git-manager.conf
sudo systemctl restart rsyslog
```

### Monitor with journalctl (systemd)

```bash
# If integrated with systemd
journalctl -u git-manager -f

# Or tail logs directly
tail -f ~/.git-manager/logs/activity.log
```

## Best Practices for Cross-Platform Logging

### 1. Use Consistent Categories
Always use the same category across all platforms for similar operations.

### 2. Include Context
Log enough context to understand what happened:
```python
# Good
logger.info(f"Git push to {repo_name} on {branch}")

# Bad
logger.info("Pushing")
```

### 3. Monitor All Platforms
Regularly check logs from all platforms:
```bash
# Weekly log review
git-manager --cli logs export --output weekly-logs-$(date +%Y%m%d).txt
```

### 4. Archive Old Logs
Keep logs for compliance:
```bash
# Archive logs
tar -czf git-manager-logs-$(date +%Y%m%d).tar.gz ~/.git-manager/logs/
```

### 5. Set Up Alerts
Monitor for errors:
```bash
# Alert on errors
watch -n 60 'tail -n 5 ~/.git-manager/logs/error.log'
```

## Log Retention Policy

- **Max file size**: 10 MB per log file
- **Backup count**: 5 backup files per category
- **Total storage**: ~60 MB per category
- **Automatic rotation**: When file reaches 10 MB

### Manual cleanup

```bash
# Clear old logs
git-manager --cli logs clear

# Export before clearing
git-manager --cli logs export --output backup-$(date +%Y%m%d).txt
git-manager --cli logs clear
```

## Summary

The unified logging system ensures:
- ✓ All platforms log to the same location
- ✓ Consistent log format and categories
- ✓ Easy cross-platform debugging
- ✓ Centralized log management
- ✓ Compliance and audit trail
- ✓ Performance monitoring

Use `git-manager --cli logs` commands to manage logs from any platform.
