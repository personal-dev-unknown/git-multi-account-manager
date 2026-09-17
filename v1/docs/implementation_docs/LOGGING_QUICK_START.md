# Logging System - Quick Start Guide

## What Was Implemented

A **comprehensive, production-ready logging system** that works across all three platforms:
- Command Line Interface (CLI)
- Desktop Application (PyQt6)
- Web Application (Flask)

All logs are stored in a **unified location** and can be managed through CLI commands.

## Log Storage Location

### Automatic Detection

The system automatically detects where to store logs:

```
System-Wide Installation → /var/log/git-manager/
User Installation       → ~/.git-manager/logs/
```

## Log Categories

Seven specialized log files for different types of events:

| File | Purpose |
|------|---------|
| `activity.log` | General application operations |
| `error.log` | Errors and exceptions |
| `security.log` | SSH keys, authentication, accounts |
| `performance.log` | Operation timing and metrics |
| `git_operation.log` | Git clone, push, pull, status |
| `ssh_operation.log` | SSH connections, key generation |
| `audit.log` | Important user actions (audit trail) |

## Quick Commands

### View Logs

```bash
# View last 50 lines of activity log
git-manager --cli logs view

# View specific category
git-manager --cli logs view --category error

# View more lines
git-manager --cli logs view --lines 100 --category git_operation
```

### List Log Files

```bash
# Show all log files with sizes
git-manager --cli logs list
```

### Storage Information

```bash
# Show log directory and available space
git-manager --cli logs info
```

### Show Categories

```bash
# List all available log categories
git-manager --cli logs categories
```

### Export Logs

```bash
# Export all logs to a file
git-manager --cli logs export

# Export to specific location
git-manager --cli logs export --output /tmp/my-logs.txt
```

### Clear Logs

```bash
# Clear all logs (with confirmation)
git-manager --cli logs clear

# Clear specific category
git-manager --cli logs clear --category error
```

## Usage Examples

### Example 1: Monitor CLI Operations

```bash
# Terminal 1: Run CLI
git-manager --cli

# Terminal 2: View logs in real-time
tail -f ~/.git-manager/logs/activity.log
```

### Example 2: Check for Errors

```bash
# View all errors
git-manager --cli logs view --category error

# Or directly
cat ~/.git-manager/logs/error.log
```

### Example 3: Audit Trail

```bash
# View all important operations
git-manager --cli logs view --category audit
```

### Example 4: Performance Analysis

```bash
# Check operation timings
git-manager --cli logs view --category performance

# Find slow operations
grep "WARNING" ~/.git-manager/logs/performance.log
```

### Example 5: Security Events

```bash
# View all security-related events
git-manager --cli logs view --category security
```

## Programmatic Usage

### In Your Code

```python
from git_manager.utils.log_config import get_logger, LogCategory

# Get a logger
logger = get_logger(__name__, category=LogCategory.ACTIVITY)

# Log messages
logger.info("Operation started")
logger.error("Operation failed", exc_info=True)
```

### Using Decorators

```python
from git_manager.utils.log_utils import log_operation
from git_manager.utils.log_config import LogCategory

@log_operation(category=LogCategory.GIT_OPERATION)
def clone_repository(url: str):
    # Automatically logged
    pass
```

### Context Manager

```python
from git_manager.utils.log_utils import ContextLogger
from git_manager.utils.log_config import LogCategory

with ContextLogger("my_operation", category=LogCategory.GIT_OPERATION):
    # Operation code
    # Automatically logs start, end, duration, and errors
    pass
```

## Cross-Platform Logging

### All Platforms Use Same Logs

```bash
# Run CLI
git-manager --cli
# Logs to ~/.git-manager/logs/

# Run Desktop
git-manager --desktop
# Logs to same ~/.git-manager/logs/

# Run Web
git-manager --web
# Logs to same ~/.git-manager/logs/

# View all logs from any platform
git-manager --cli logs view
```

## Log Format

### Text Format (CLI)
```
[2025-11-21 04:09:06] [INFO    ] [activity] Starting Git Multi-Account Manager v1.0.0
[2025-11-21 04:09:35] [ERROR   ] [error] Failed: git_push - Permission denied
```

### JSON Format (Desktop/Web)
```json
{
  "timestamp": "2025-11-21T04:09:06.123456",
  "level": "INFO",
  "category": "activity",
  "logger": "git_manager.cli.app",
  "message": "Starting Git Multi-Account Manager v1.0.0"
}
```

## Log Rotation

Logs automatically rotate to prevent disk space issues:

- **Max file size**: 10 MB per log file
- **Backup count**: 5 backup files (e.g., `activity.log.1`, `activity.log.2`, etc.)
- **Total storage**: ~60 MB per category

## Real-Time Monitoring

### Watch logs in real-time

```bash
# Watch activity log
tail -f ~/.git-manager/logs/activity.log

# Watch errors
tail -f ~/.git-manager/logs/error.log

# Watch git operations
tail -f ~/.git-manager/logs/git_operation.log
```

### Use watch command

```bash
# Update every 1 second
watch -n 1 'tail -n 20 ~/.git-manager/logs/activity.log'
```

## Troubleshooting

### Logs not being created?

```bash
# Check if logging is initialized
git-manager --cli logs info

# Check directory exists
ls -la ~/.git-manager/logs/

# Check permissions
ls -la ~/.git-manager/
```

### Logs growing too large?

```bash
# Clear logs
git-manager --cli logs clear

# Or export first
git-manager --cli logs export --output backup.txt
git-manager --cli logs clear
```

### Find specific information

```bash
# Find all errors
grep "ERROR" ~/.git-manager/logs/*.log

# Find specific operation
grep "git_operation" ~/.git-manager/logs/git_operation.log

# Find by timestamp
grep "2025-11-21 04:09" ~/.git-manager/logs/activity.log
```

## Best Practices

### 1. Regular Review
Check logs weekly for errors and issues:
```bash
git-manager --cli logs view --category error
```

### 2. Archive Logs
Keep backups for compliance:
```bash
tar -czf git-manager-logs-$(date +%Y%m%d).tar.gz ~/.git-manager/logs/
```

### 3. Monitor Performance
Check for slow operations:
```bash
grep "WARNING" ~/.git-manager/logs/performance.log
```

### 4. Security Audit
Review security events:
```bash
git-manager --cli logs view --category security
```

### 5. Export for Analysis
Export logs for detailed analysis:
```bash
git-manager --cli logs export --output analysis-$(date +%Y%m%d).txt
```

## Documentation

For more detailed information, see:

- **LOGGING_SYSTEM.md** - Complete logging system documentation
- **LOGGING_CROSS_PLATFORM.md** - Cross-platform logging guide
- **LOGGING_ARCHITECTURE.md** - Technical architecture details

## Summary

✓ **Unified logging** across CLI, Desktop, and Web
✓ **Automatic log rotation** prevents disk space issues
✓ **Seven log categories** for different event types
✓ **Easy CLI commands** to manage logs
✓ **Structured logging** with JSON format
✓ **Audit trail** for compliance
✓ **Performance monitoring** built-in
✓ **Production-ready** with error handling

Start logging now:
```bash
git-manager --cli logs view
```
