# src/git_manager/utils/log_utils.py
"""
Logging utilities and decorators for easy integration.

Provides convenient decorators and utilities for logging operations,
errors, and performance metrics throughout the application.
"""

import functools
import time
import logging
from typing import Any, Callable, Optional
from .log_config import (
    LoggerFactory,
    LogCategory,
    AuditLogger,
    PerformanceLogger,
    get_logger,
)


def log_operation(
    category: LogCategory = LogCategory.ACTIVITY,
    operation_name: Optional[str] = None,
):
    """
    Decorator to log function execution.
    
    Args:
        category: Log category
        operation_name: Custom operation name (defaults to function name)
    
    Example:
        @log_operation(category=LogCategory.GIT_OPERATION)
        def clone_repository(url: str):
            ...
    """
    def decorator(func: Callable) -> Callable:
        @functools.wraps(func)
        def wrapper(*args, **kwargs) -> Any:
            logger = get_logger(func.__module__, category=category)
            op_name = operation_name or func.__name__
            
            try:
                logger.info(f"Starting: {op_name}")
                result = func(*args, **kwargs)
                logger.info(f"Completed: {op_name}")
                return result
            except Exception as e:
                logger.error(f"Failed: {op_name} - {str(e)}", exc_info=True)
                raise
        
        return wrapper
    return decorator


def log_with_timing(
    category: LogCategory = LogCategory.PERFORMANCE,
    threshold_ms: float = 1000,
):
    """
    Decorator to log function execution with timing.
    
    Args:
        category: Log category
        threshold_ms: Warning threshold in milliseconds
    
    Example:
        @log_with_timing(threshold_ms=500)
        def expensive_operation():
            ...
    """
    def decorator(func: Callable) -> Callable:
        @functools.wraps(func)
        def wrapper(*args, **kwargs) -> Any:
            logger = get_logger(func.__module__, category=category)
            perf_logger = PerformanceLogger()
            
            start_time = time.time()
            try:
                logger.info(f"Starting: {func.__name__}")
                result = func(*args, **kwargs)
                return result
            except Exception as e:
                logger.error(f"Failed: {func.__name__} - {str(e)}", exc_info=True)
                raise
            finally:
                duration_ms = (time.time() - start_time) * 1000
                perf_logger.log_operation_time(
                    func.__name__,
                    duration_ms,
                    threshold_ms=threshold_ms,
                )
        
        return wrapper
    return decorator


def log_errors(
    category: LogCategory = LogCategory.ERROR,
    reraise: bool = True,
):
    """
    Decorator to log exceptions.
    
    Args:
        category: Log category
        reraise: Whether to re-raise the exception
    
    Example:
        @log_errors(category=LogCategory.SSH_OPERATION)
        def connect_ssh():
            ...
    """
    def decorator(func: Callable) -> Callable:
        @functools.wraps(func)
        def wrapper(*args, **kwargs) -> Any:
            logger = get_logger(func.__module__, category=category)
            
            try:
                return func(*args, **kwargs)
            except Exception as e:
                logger.error(
                    f"Exception in {func.__name__}: {str(e)}",
                    exc_info=True,
                )
                if reraise:
                    raise
                return None
        
        return wrapper
    return decorator


def log_security_event(
    event_type: str,
    details: Optional[dict] = None,
):
    """
    Log a security-related event.
    
    Args:
        event_type: Type of security event
        details: Additional event details
    
    Example:
        log_security_event(
            "ssh_key_generated",
            {"key_type": "ed25519", "key_name": "github"}
        )
    """
    logger = get_logger(__name__, category=LogCategory.SECURITY)
    audit_logger = AuditLogger()
    
    message = f"Security Event: {event_type}"
    logger.warning(message)
    
    audit_logger.log_operation(
        operation=f"security_{event_type}",
        status="logged",
        details=details or {},
    )


def log_git_operation(
    operation: str,
    repository: Optional[str] = None,
    success: bool = True,
    error_msg: Optional[str] = None,
):
    """
    Log a Git operation.
    
    Args:
        operation: Git operation (clone, push, pull, etc.)
        repository: Repository name/path
        success: Whether operation succeeded
        error_msg: Error message if failed
    
    Example:
        log_git_operation("push", repository="my-repo", success=True)
    """
    logger = get_logger(__name__, category=LogCategory.GIT_OPERATION)
    audit_logger = AuditLogger()
    
    status = "success" if success else "failure"
    message = f"Git {operation}"
    if repository:
        message += f" on {repository}"
    message += f": {status}"
    
    if success:
        logger.info(message)
    else:
        logger.error(f"{message} - {error_msg}")
    
    audit_logger.log_operation(
        operation=f"git_{operation}",
        status=status,
        details={
            "repository": repository,
            "error": error_msg,
        } if error_msg else {"repository": repository},
    )


def log_ssh_operation(
    operation: str,
    account: Optional[str] = None,
    success: bool = True,
    error_msg: Optional[str] = None,
):
    """
    Log an SSH operation.
    
    Args:
        operation: SSH operation (connect, generate_key, etc.)
        account: Account name
        success: Whether operation succeeded
        error_msg: Error message if failed
    
    Example:
        log_ssh_operation("connect", account="github", success=True)
    """
    logger = get_logger(__name__, category=LogCategory.SSH_OPERATION)
    audit_logger = AuditLogger()
    
    status = "success" if success else "failure"
    message = f"SSH {operation}"
    if account:
        message += f" for {account}"
    message += f": {status}"
    
    if success:
        logger.info(message)
    else:
        logger.error(f"{message} - {error_msg}")
    
    audit_logger.log_operation(
        operation=f"ssh_{operation}",
        status=status,
        details={
            "account": account,
            "error": error_msg,
        } if error_msg else {"account": account},
    )


class ContextLogger:
    """Context manager for logging operation blocks."""
    
    def __init__(
        self,
        operation_name: str,
        category: LogCategory = LogCategory.ACTIVITY,
    ):
        self.operation_name = operation_name
        self.category = category
        self.logger = get_logger(__name__, category=category)
        self.start_time = None
    
    def __enter__(self):
        self.start_time = time.time()
        self.logger.info(f"Starting: {self.operation_name}")
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        duration_ms = (time.time() - self.start_time) * 1000
        
        if exc_type is not None:
            self.logger.error(
                f"Failed: {self.operation_name} - {exc_val}",
                exc_info=(exc_type, exc_val, exc_tb),
            )
        else:
            self.logger.info(
                f"Completed: {self.operation_name} ({duration_ms:.2f}ms)"
            )
        
        return False  # Don't suppress exceptions


def get_log_directory_info() -> dict:
    """Get information about log storage."""
    from .log_config import LogStorageManager
    return LogStorageManager.get_storage_info()


def list_log_files() -> list:
    """List all log files."""
    from .log_config import LogStorageManager
    log_dir = LogStorageManager.get_log_directory()
    
    if not log_dir.exists():
        return []
    
    log_files = []
    for log_file in log_dir.glob("*.log*"):
        log_files.append({
            'name': log_file.name,
            'path': str(log_file),
            'size_kb': log_file.stat().st_size / 1024,
            'modified': log_file.stat().st_mtime,
        })
    
    return sorted(log_files, key=lambda x: x['modified'], reverse=True)
