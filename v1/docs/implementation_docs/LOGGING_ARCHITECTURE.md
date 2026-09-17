# Logging System Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Git Multi-Account Manager                             │
│                      Unified Logging System                              │
└─────────────────────────────────────────────────────────────────────────┘

                              ┌──────────────────┐
                              │  Applications    │
                              └──────────────────┘
                                      │
                ┌───────────────────────┼───────────────────────┐
                │                       │                       │
        ┌───────▼────────┐     ┌───────▼────────┐     ┌───────▼────────┐
        │  CLI Platform  │     │ Desktop (PyQt) │     │  Web (Flask)   │
        │  (Click)       │     │  Application   │     │  Application   │
        └───────┬────────┘     └───────┬────────┘     └───────┬────────┘
                │                       │                       │
                │ initialize_logging()  │ initialize_logging()  │
                │ LogLevel.INFO         │ LogLevel.INFO         │
                │ use_json=False        │ use_json=True         │
                │ enable_console=True   │ enable_console=False  │
                │                       │                       │
                └───────────────────────┼───────────────────────┘
                                        │
                        ┌───────────────▼───────────────┐
                        │    LoggerFactory             │
                        │  (Centralized Logger Mgmt)   │
                        └───────────────┬───────────────┘
                                        │
                    ┌───────────────────┼───────────────────┐
                    │                   │                   │
        ┌───────────▼──────────┐  ┌────▼─────────┐  ┌─────▼──────────┐
        │ StructuredFormatter  │  │ RotatingFile │  │ ConsoleHandler │
        │ (JSON/Text)          │  │ Handler      │  │ (CLI only)     │
        └───────────┬──────────┘  └────┬─────────┘  └─────┬──────────┘
                    │                   │                   │
                    └───────────────────┼───────────────────┘
                                        │
                    ┌───────────────────▼───────────────────┐
                    │    Log Storage Location               │
                    │  (LogStorageManager)                  │
                    └───────────────────┬───────────────────┘
                                        │
                ┌───────────────────────┼───────────────────────┐
                │                       │                       │
        ┌───────▼────────┐     ┌───────▼────────┐     ┌───────▼────────┐
        │ System-Wide    │     │ User Home      │     │ Fallback       │
        │ /var/log/      │     │ ~/.git-manager │     │ ~/.git-manager │
        │ git-manager/   │     │ /logs/         │     │ /logs/         │
        └────────────────┘     └────────────────┘     └────────────────┘
                                        │
                ┌───────────────────────┼───────────────────────┐
                │                       │                       │
        ┌───────▼────────┐     ┌───────▼────────┐     ┌───────▼────────┐
        │ activity.log   │     │ error.log      │     │ security.log   │
        │ git_op.log     │     │ performance.log│     │ ssh_op.log     │
        │ audit.log      │     │                │     │                │
        └────────────────┘     └────────────────┘     └────────────────┘
```

## Data Flow

### 1. Application Startup

```
Application Start
        │
        ├─► initialize_logging()
        │   ├─► Detect installation type
        │   ├─► Determine log directory
        │   └─► Configure log level & format
        │
        └─► get_logger(name, category)
            └─► Create logger with handlers
```

### 2. Logging Operation

```
User Action
    │
    ├─► @log_operation decorator
    │   └─► get_logger() → log start
    │
    ├─► Perform operation
    │   └─► May call log_git_operation() or log_ssh_operation()
    │
    └─► Operation complete
        └─► get_logger() → log completion/error
```

### 3. Log Writing

```
Logger.info/error/warning()
    │
    ├─► StructuredFormatter
    │   ├─► Text format (CLI)
    │   └─► JSON format (Desktop/Web)
    │
    ├─► RotatingFileHandler
    │   ├─► Write to log file
    │   └─► Check size & rotate if needed
    │
    └─► ConsoleHandler (CLI only)
        └─► Print to stdout
```

## Component Architecture

### LoggerFactory

```python
┌─────────────────────────────────┐
│      LoggerFactory              │
├─────────────────────────────────┤
│ + initialize()                  │
│ + get_logger()                  │
│ + get_all_loggers()             │
│ + get_log_directory()           │
├─────────────────────────────────┤
│ - _loggers: Dict                │
│ - _log_dir: Path                │
│ - _log_level: LogLevel          │
│ - _use_json: bool               │
└─────────────────────────────────┘
```

### LogStorageManager

```python
┌─────────────────────────────────┐
│   LogStorageManager             │
├─────────────────────────────────┤
│ + get_log_directory()           │
│ + get_storage_info()            │
├─────────────────────────────────┤
│ - _get_available_space()        │
│ - Detect system vs user install │
└─────────────────────────────────┘
```

### StructuredFormatter

```python
┌─────────────────────────────────┐
│   StructuredFormatter           │
├─────────────────────────────────┤
│ + format()                      │
├─────────────────────────────────┤
│ - _format_json()                │
│ - _format_text()                │
│ - use_json: bool                │
│ - category: str                 │
└─────────────────────────────────┘
```

### Specialized Loggers

```
┌────────────────────────────────────────────────────────┐
│              Specialized Loggers                       │
├────────────────────────────────────────────────────────┤
│                                                        │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │ AuditLogger  │  │PerfLogger    │  │ContextLog │  │
│  ├──────────────┤  ├──────────────┤  ├────────────┤  │
│  │log_operation│  │log_op_time() │  │__enter__() │  │
│  │             │  │              │  │__exit__()  │  │
│  └──────────────┘  └──────────────┘  └────────────┘  │
│                                                        │
└────────────────────────────────────────────────────────┘
```

## Log File Rotation Strategy

```
Initial State:
    activity.log (0 bytes)

After 10 MB:
    activity.log.1 (10 MB)
    activity.log (0 bytes)

After 20 MB total:
    activity.log.2 (10 MB)
    activity.log.1 (10 MB)
    activity.log (0 bytes)

After 50 MB total (5 backups):
    activity.log.5 (10 MB) ← Oldest, kept
    activity.log.4 (10 MB)
    activity.log.3 (10 MB)
    activity.log.2 (10 MB)
    activity.log.1 (10 MB)
    activity.log (0 bytes)

After 60 MB total (exceeds 5 backups):
    activity.log.5 (10 MB) ← Deleted
    activity.log.4 (10 MB) ← Becomes .5
    activity.log.3 (10 MB) ← Becomes .4
    activity.log.2 (10 MB) ← Becomes .3
    activity.log.1 (10 MB) ← Becomes .2
    activity.log (0 bytes) ← Becomes .1
```

## Log Categories and Handlers

```
┌─────────────────────────────────────────────────────────┐
│              Log Categories                             │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ACTIVITY ──────────► activity.log                      │
│  ERROR ─────────────► error.log                         │
│  SECURITY ──────────► security.log                      │
│  PERFORMANCE ───────► performance.log                   │
│  GIT_OPERATION ─────► git_operation.log                 │
│  SSH_OPERATION ─────► ssh_operation.log                 │
│  AUDIT ─────────────► audit.log                         │
│                                                         │
└─────────────────────────────────────────────────────────┘

Each category has:
    ├─ RotatingFileHandler (10 MB max, 5 backups)
    ├─ StructuredFormatter (JSON or Text)
    └─ ConsoleHandler (CLI only)
```

## Installation Type Detection

```
LogStorageManager.get_log_directory()
    │
    ├─ Check if root user (euid == 0)
    │   └─ Yes ──► Try /var/log/git-manager/
    │       ├─ Success ──► Return /var/log/git-manager/
    │       └─ Fail ──────► Continue to next
    │
    ├─ Check if /usr/bin/git-manager exists
    │   └─ Yes ──► Try /var/log/git-manager/
    │       ├─ Success ──► Return /var/log/git-manager/
    │       └─ Fail ──────► Continue to next
    │
    └─ Fallback to user home
        └─ Return ~/.git-manager/logs/
```

## Decorator Architecture

```
┌────────────────────────────────────────────────────────┐
│              Logging Decorators                        │
├────────────────────────────────────────────────────────┤
│                                                        │
│  @log_operation()                                      │
│  ├─ Log function start                                │
│  ├─ Execute function                                  │
│  └─ Log completion or error                           │
│                                                        │
│  @log_with_timing()                                   │
│  ├─ Record start time                                 │
│  ├─ Execute function                                  │
│  └─ Log duration with threshold check                 │
│                                                        │
│  @log_errors()                                        │
│  ├─ Execute function                                  │
│  └─ Catch and log exceptions                          │
│                                                        │
└────────────────────────────────────────────────────────┘
```

## Context Manager Flow

```
with ContextLogger("operation_name", category):
    │
    ├─► __enter__()
    │   ├─ Record start time
    │   └─ Log "Starting: operation_name"
    │
    ├─► Execute code block
    │   └─ May raise exception
    │
    └─► __exit__()
        ├─ Calculate duration
        ├─ If exception:
        │   └─ Log error with traceback
        └─ If success:
            └─ Log completion with duration
```

## Cross-Platform Log Flow

```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│     CLI      │  │   Desktop    │  │     Web      │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       │ Text format     │ JSON format     │ JSON format
       │ Console output  │ No console      │ No console
       │                 │                 │
       └─────────────────┼─────────────────┘
                         │
                    ┌────▼─────┐
                    │ Formatter │
                    └────┬─────┘
                         │
                    ┌────▼──────────┐
                    │ File Handler  │
                    │ (Rotating)    │
                    └────┬──────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
    ┌───▼───┐        ┌───▼───┐       ┌───▼───┐
    │activity│        │ error │       │security│
    │.log    │        │.log   │       │.log    │
    └────────┘        └───────┘       └────────┘
```

## Performance Characteristics

```
┌─────────────────────────────────────────────────────┐
│           Performance Metrics                       │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Log Operation Overhead:        < 1ms               │
│ File I/O (async):              Non-blocking        │
│ Memory per logger:             ~50KB                │
│ Max log file size:             10 MB                │
│ Rotation time:                 < 100ms              │
│ Disk space per category:       ~60 MB (5 backups)  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Error Handling

```
Logger Operation
    │
    ├─ Success
    │   └─ Write to file
    │
    └─ Failure
        ├─ File permission error
        │   └─ Fallback to user home
        │
        ├─ Disk full
        │   └─ Rotate existing logs
        │
        └─ Other error
            └─ Log to stderr (silent fail)
```

## Integration Points

```
┌────────────────────────────────────────────────────────┐
│         Integration Points                            │
├────────────────────────────────────────────────────────┤
│                                                        │
│ Core Modules:                                          │
│  ├─ account_manager.py                                │
│  ├─ ssh_manager.py                                    │
│  └─ git_operations.py                                 │
│      └─ Can use @log_operation() decorator            │
│                                                        │
│ CLI Commands:                                          │
│  ├─ clone.py                                          │
│  ├─ account.py                                        │
│  ├─ repository.py                                     │
│  ├─ ssh.py                                            │
│  └─ config.py                                         │
│      └─ Can use log_git_operation(), etc.             │
│                                                        │
│ Web Routes:                                            │
│  ├─ api.py                                            │
│  ├─ accounts.py                                       │
│  ├─ repositories.py                                   │
│  └─ ssh.py                                            │
│      └─ Can use get_logger() for each route           │
│                                                        │
└────────────────────────────────────────────────────────┘
```

## Summary

The logging architecture provides:

✓ **Unified storage** - All platforms log to same location
✓ **Modular design** - Easy to extend and customize
✓ **Automatic rotation** - Prevents disk space issues
✓ **Cross-platform** - Works on CLI, Desktop, Web
✓ **Structured logging** - JSON format for analysis
✓ **Performance** - Minimal overhead
✓ **Audit trail** - Complete operation history
✓ **Error handling** - Graceful fallbacks
