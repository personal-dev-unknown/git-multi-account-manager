# Git Multi-Account Manager - Logging System

## Overview

The Git Multi-Account Manager includes a comprehensive, modular logging system designed for production environments. It automatically detects installation type and stores logs in appropriate locations with intelligent rotation, categorization, and audit trail capabilities.

## Log Storage Locations

### System-Wide Installation
When installed system-wide (e.g., via package manager to `/usr/bin/git-manager`):
```
/var/log/git-manager/
├── activity.log          # General operations
├── error.log             # Errors and exceptions
├── security.log          # Authentication and account operations
├── performance.log       # Performance metrics
├── git_operation.log     # Git-specific operations
├── ssh_operation.log     # SSH-specific operations
└── audit.log             # Audit trail
```

### User Installation
When installed locally (default):
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

## Log Categories

| Category | File | Purpose | Examples |
|----------|------|---------|----------|
| **ACTIVITY** | activity.log | General application operations | App startup, menu selections, user actions |
| **ERROR** | error.log | Errors and exceptions | Failed operations, exceptions with stack traces |
| **SECURITY** | security.log | Authentication and account operations | SSH key generation, account setup, SSH connections |
| **PERFORMANCE** | performance.log | Performance metrics and timing | Operation duration, slow operations (>1s) |
| **GIT_OPERATION** | git_operation.log | Git-specific operations | Clone, push, pull, status checks |
| **SSH_OPERATION** | ssh_operation.log | SSH-specific operations | SSH connections, key tests, key generation |
| **AUDIT** | audit.log | Audit trail for important actions | All important user actions with timestamps |

## Log Rotation

Logs are automatically rotated to prevent disk space issues:
- **Max file size**: 10 MB per log file
- **Backup count**: 5 backup files kept (e.g., `activity.log.1`, `activity.log.2`, etc.)
- **Total max storage**: ~60 MB per category (10 MB × 6 files)

## Log Format

### Text Format (Default)
```
[2024-01-15 14:23:45] [INFO    ] [activity] Starting Git Multi-Account Manager v1.0.0
[2024-01-15 14:23:46] [INFO    ] [git_operation] Git clone: success
[2024-01-15 14:23:47] [ERROR   ] [error] Failed: git_push - Permission denied
```

### JSON Format (with `--json-logs` flag)
```json
{
  "timestamp": "2024-01-15T14:23:45.123456",
  "level": "INFO",
  "category": "activity",
  "logger": "git_manager.cli.app",
  "message": "Starting Git Multi-Account Manager v1.0.0",
  "module": "app",
  "function": "cli",
  "line": 52
}
```

## CLI Commands

### View Log Information
```bash
git-manager logs info
```
Shows log directory, installation type, and available disk space.

### List All Log Files
```bash
git-manager logs list
```
Displays all log files with sizes and modification times.

### View Log Contents
```bash
# View last 50 lines of activity log
git-manager logs view

# View specific category
git-manager logs view --category error

# View custom number of lines
git-manager logs view --lines 100 --category git_operation
```

### Show Available Categories
```bash
git-manager logs categories
```

### Export All Logs
```bash
# Export to auto-generated filename
git-manager logs export

# Export to specific file
git-manager logs export --output /tmp/git-manager-logs.txt
```

### Clear Logs
```bash
# Clear all logs (with confirmation)
git-manager logs clear

# Clear specific category
git-manager logs clear --category error
```

## Programmatic Usage

### Basic Logging

```python
from git_manager.utils.log_config import get_logger, LogCategory

# Get a logger
logger = get_logger(__name__, category=LogCategory.ACTIVITY)

# Log messages
logger.info("Operation started")
logger.warning("Potential issue detected")
logger.error("Operation failed", exc_info=True)
```

### Using Decorators

```python
from git_manager.utils.log_utils import (
    log_operation,
    log_with_timing,
    log_errors,
)
from git_manager.utils.log_config import LogCategory

# Log operation
@log_operation(category=LogCategory.GIT_OPERATION)
def clone_repository(url: str):
    # Operation code
    pass

# Log with timing
@log_with_timing(threshold_ms=500)
def expensive_operation():
    # Operation code
    pass

# Log errors
@log_errors(category=LogCategory.SSH_OPERATION)
def connect_ssh():
    # Operation code
    pass
```

### Context Manager

```python
from git_manager.utils.log_utils import ContextLogger
from git_manager.utils.log_config import LogCategory

with ContextLogger("my_operation", category=LogCategory.GIT_OPERATION):
    # Operation code
    # Automatically logs start, completion, duration, and errors
    pass
```

### Specialized Loggers

```python
from git_manager.utils.log_utils import (
    log_git_operation,
    log_ssh_operation,
    log_security_event,
)

# Log Git operation
log_git_operation("push", repository="my-repo", success=True)

# Log SSH operation
log_ssh_operation("connect", account="github", success=True)

# Log security event
log_security_event(
    "ssh_key_generated",
    {"key_type": "ed25519", "key_name": "github"}
)
```

### Audit Logger

```python
from git_manager.utils.log_config import AuditLogger

audit_logger = AuditLogger()
audit_logger.log_operation(
    operation="account_created",
    status="success",
    user="john_doe",
    details={"platform": "github", "account": "john"}
)
```

### Performance Logger

```python
from git_manager.utils.log_config import PerformanceLogger

perf_logger = PerformanceLogger()
perf_logger.log_operation_time(
    operation="git_clone",
    duration_ms=2500,
    threshold_ms=1000  # Warn if > 1s
)
```

## Initialization

### In CLI Applications

```python
from git_manager.utils.log_config import initialize_logging, LogLevel

# Initialize logging system
initialize_logging(
    log_level=LogLevel.DEBUG,
    use_json=False,
    enable_console=True,
)
```

### In Desktop/Web Applications

```python
from git_manager.utils.log_config import initialize_logging, LogLevel

# Initialize with JSON format for structured logging
initialize_logging(
    log_level=LogLevel.INFO,
    use_json=True,
    enable_console=False,  # Disable console for web apps
)
```

## Best Practices

### 1. Use Appropriate Categories
Choose the right category for your log messages:
- **ACTIVITY**: General user actions
- **ERROR**: Exceptions and failures
- **SECURITY**: Authentication, SSH, account operations
- **PERFORMANCE**: Timing-sensitive operations
- **GIT_OPERATION**: Git-specific operations
- **SSH_OPERATION**: SSH-specific operations
- **AUDIT**: Important user actions

### 2. Include Context
Always include relevant context in log messages:
```python
# Good
logger.info(f"Cloning repository: {url} with account: {account_name}")

# Bad
logger.info("Cloning")
```

### 3. Use Decorators for Consistency
Use decorators to automatically log function execution:
```python
@log_operation(category=LogCategory.GIT_OPERATION)
def clone_repository(url: str):
    # Code is automatically logged
    pass
```

### 4. Log Important Security Events
Always log security-related operations:
```python
log_security_event(
    "ssh_key_generated",
    {"key_type": "ed25519", "account": "github"}
)
```

### 5. Monitor Performance
Use performance logging for potentially slow operations:
```python
@log_with_timing(threshold_ms=1000)
def large_git_operation():
    pass
```

### 6. Regular Log Review
Periodically review logs for:
- Errors and failures
- Security events
- Performance issues
- Unusual patterns

```bash
# View recent errors
git-manager logs view --category error --lines 50

# Export logs for analysis
git-manager logs export --output logs-$(date +%Y%m%d).txt
```

## Troubleshooting

### Logs Not Being Created
1. Check log directory permissions:
   ```bash
   ls -la ~/.git-manager/logs/
   # or
   sudo ls -la /var/log/git-manager/
   ```

2. Verify logging is initialized:
   ```bash
   git-manager --debug logs info
   ```

### Logs Growing Too Large
Logs are automatically rotated at 10 MB. To manually clear:
```bash
git-manager logs clear
```

### Permission Denied Errors
If running as non-root user and system-wide logs can't be written:
- Logs will automatically fall back to `~/.git-manager/logs/`
- No manual action required

### JSON Format Issues
If JSON logs are malformed, check:
1. Ensure `--json-logs` flag is used correctly
2. Verify log file isn't corrupted
3. Try viewing with `git-manager logs view --category activity`

## Storage Information

Get detailed storage information:
```bash
git-manager logs info
```

Output example:
```
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃                          Log Storage Info                                      ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ Log Directory: /var/log/git-manager                                            │
│ Installation Type: System-wide                                                 │
│ Available Space: 1024.50 MB                                                    │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Integration with Existing Code

To integrate logging into existing functions:

```python
# Before
def clone_repository(url: str, account: str):
    repo = GitOperations.clone(url, account)
    return repo

# After
@log_operation(category=LogCategory.GIT_OPERATION)
def clone_repository(url: str, account: str):
    try:
        repo = GitOperations.clone(url, account)
        log_git_operation("clone", repository=url, success=True)
        return repo
    except Exception as e:
        log_git_operation("clone", repository=url, success=False, error_msg=str(e))
        raise
```

## Performance Impact

The logging system is designed to have minimal performance impact:
- **Async file I/O**: Logs are written asynchronously
- **Lazy initialization**: Loggers are created on-demand
- **Efficient rotation**: Automatic rotation prevents large file operations
- **Configurable verbosity**: Adjust log level to reduce I/O

Typical overhead: < 1ms per log operation

## Security Considerations

1. **Log File Permissions**: Logs are created with restricted permissions
2. **Sensitive Data**: Avoid logging passwords or tokens
3. **Audit Trail**: All important operations are logged for compliance
4. **Log Retention**: Implement log retention policies as needed

## Future Enhancements

Planned features:
- Remote log aggregation
- Log filtering and search
- Real-time log streaming
- Log compression
- Integration with system logging (syslog)
