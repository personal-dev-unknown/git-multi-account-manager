# src/git_manager/utils/log_config.py
"""
Advanced logging configuration system for Git Multi-Account Manager.

This module provides a comprehensive, modular logging system with:
- Multiple log streams (activity, errors, security, performance)
- Automatic log rotation
- System-wide and user-level logging
- Structured logging with context
- Audit trail capabilities
"""

import logging
import logging.handlers
import os
import sys
import json
from pathlib import Path
from typing import Optional, Dict, Any
from datetime import datetime
from enum import Enum


class LogLevel(str, Enum):
    """Log level enumeration."""
    DEBUG = "DEBUG"
    INFO = "INFO"
    WARNING = "WARNING"
    ERROR = "ERROR"
    CRITICAL = "CRITICAL"


class LogCategory(str, Enum):
    """Log categories for different types of events."""
    ACTIVITY = "activity"          # General operations
    ERROR = "error"                # Errors and exceptions
    SECURITY = "security"          # Authentication, SSH, account operations
    PERFORMANCE = "performance"    # Performance metrics
    GIT_OPERATION = "git_operation"  # Git-specific operations
    SSH_OPERATION = "ssh_operation"  # SSH-specific operations
    AUDIT = "audit"                # Audit trail for important actions


class LogStorageManager:
    """Manages log storage paths based on installation type."""
    
    @staticmethod
    def get_log_directory() -> Path:
        """
        Determine the appropriate log directory based on installation.
        
        Priority:
        1. System-wide installation: /var/log/git-manager/
        2. User installation: ~/.git-manager/logs/
        3. Fallback: ~/.git-manager/
        
        Returns:
            Path to log directory
        """
        # Check if running as system-wide installation
        if os.geteuid() == 0 or os.path.exists('/usr/bin/git-manager'):
            system_log_dir = Path('/var/log/git-manager')
            try:
                system_log_dir.mkdir(parents=True, exist_ok=True)
                # Test write permission
                test_file = system_log_dir / '.write_test'
                test_file.touch()
                test_file.unlink()
                return system_log_dir
            except (PermissionError, OSError):
                pass
        
        # Fallback to user home directory
        user_log_dir = Path.home() / '.git-manager' / 'logs'
        user_log_dir.mkdir(parents=True, exist_ok=True)
        return user_log_dir
    
    @staticmethod
    def get_storage_info() -> Dict[str, Any]:
        """
        Get information about log storage location.
        
        Returns:
            Dictionary with storage info
        """
        log_dir = LogStorageManager.get_log_directory()
        return {
            'log_directory': str(log_dir),
            'is_system_wide': '/var/log' in str(log_dir),
            'available_space_mb': LogStorageManager._get_available_space(log_dir),
        }
    
    @staticmethod
    def _get_available_space(path: Path) -> float:
        """Get available disk space in MB."""
        try:
            stat = os.statvfs(path)
            available_bytes = stat.f_bavail * stat.f_frsize
            return available_bytes / (1024 * 1024)
        except (OSError, AttributeError):
            return -1


class StructuredFormatter(logging.Formatter):
    """Custom formatter for structured logging with JSON support."""
    
    def __init__(self, use_json: bool = False, category: str = "general"):
        super().__init__()
        self.use_json = use_json
        self.category = category
    
    def format(self, record: logging.LogRecord) -> str:
        """Format log record."""
        if self.use_json:
            return self._format_json(record)
        return self._format_text(record)
    
    def _format_json(self, record: logging.LogRecord) -> str:
        """Format as JSON for structured logging."""
        log_data = {
            'timestamp': datetime.fromtimestamp(record.created).isoformat(),
            'level': record.levelname,
            'category': self.category,
            'logger': record.name,
            'message': record.getMessage(),
            'module': record.module,
            'function': record.funcName,
            'line': record.lineno,
        }
        
        # Add exception info if present
        if record.exc_info:
            log_data['exception'] = self.formatException(record.exc_info)
        
        # Add extra fields if present
        if hasattr(record, 'extra_data'):
            log_data['extra'] = record.extra_data
        
        return json.dumps(log_data)
    
    def _format_text(self, record: logging.LogRecord) -> str:
        """Format as human-readable text."""
        timestamp = datetime.fromtimestamp(record.created).strftime('%Y-%m-%d %H:%M:%S')
        level = record.levelname.ljust(8)
        
        message = f"[{timestamp}] [{level}] [{self.category}] {record.getMessage()}"
        
        if record.exc_info:
            message += f"\n{self.formatException(record.exc_info)}"
        
        return message


class LoggerFactory:
    """Factory for creating configured loggers."""
    
    _loggers: Dict[str, logging.Logger] = {}
    _log_dir: Optional[Path] = None
    _initialized = False
    
    @classmethod
    def initialize(
        cls,
        log_level: LogLevel = LogLevel.INFO,
        use_json: bool = False,
        enable_console: bool = True,
        max_bytes: int = 10 * 1024 * 1024,  # 10MB
        backup_count: int = 5,
    ) -> None:
        """
        Initialize the logging system.
        
        Args:
            log_level: Logging level
            use_json: Use JSON format for logs
            enable_console: Enable console output
            max_bytes: Max size of log file before rotation
            backup_count: Number of backup files to keep
        """
        cls._log_dir = LogStorageManager.get_log_directory()
        cls._log_level = log_level
        cls._use_json = use_json
        cls._enable_console = enable_console
        cls._max_bytes = max_bytes
        cls._backup_count = backup_count
        cls._initialized = True
    
    @classmethod
    def get_logger(
        cls,
        name: str,
        category: LogCategory = LogCategory.ACTIVITY,
    ) -> logging.Logger:
        """
        Get or create a logger.
        
        Args:
            name: Logger name (typically __name__)
            category: Log category
            
        Returns:
            Configured logger instance
        """
        if not cls._initialized:
            cls.initialize()
        
        logger_key = f"{name}_{category.value}"
        
        if logger_key in cls._loggers:
            return cls._loggers[logger_key]
        
        logger = logging.getLogger(name)
        logger.setLevel(cls._log_level.value)
        
        # Remove existing handlers to avoid duplicates
        logger.handlers.clear()
        
        # Add file handler
        file_handler = cls._create_file_handler(category)
        logger.addHandler(file_handler)
        
        # Add console handler if enabled
        if cls._enable_console:
            console_handler = cls._create_console_handler(category)
            logger.addHandler(console_handler)
        
        cls._loggers[logger_key] = logger
        return logger
    
    @classmethod
    def _create_file_handler(cls, category: LogCategory) -> logging.Handler:
        """Create rotating file handler."""
        log_file = cls._log_dir / f"{category.value}.log"
        
        handler = logging.handlers.RotatingFileHandler(
            log_file,
            maxBytes=cls._max_bytes,
            backupCount=cls._backup_count,
        )
        
        formatter = StructuredFormatter(
            use_json=cls._use_json,
            category=category.value,
        )
        handler.setFormatter(formatter)
        handler.setLevel(cls._log_level.value)
        
        return handler
    
    @classmethod
    def _create_console_handler(cls, category: LogCategory) -> logging.Handler:
        """Create console handler with color support."""
        handler = logging.StreamHandler(sys.stdout)
        
        formatter = StructuredFormatter(
            use_json=False,
            category=category.value,
        )
        handler.setFormatter(formatter)
        handler.setLevel(cls._log_level.value)
        
        return handler
    
    @classmethod
    def get_all_loggers(cls) -> Dict[str, logging.Logger]:
        """Get all created loggers."""
        return cls._loggers.copy()
    
    @classmethod
    def get_log_directory(cls) -> Path:
        """Get the log directory."""
        if cls._log_dir is None:
            cls._log_dir = LogStorageManager.get_log_directory()
        return cls._log_dir


class AuditLogger:
    """Specialized logger for audit trail."""
    
    def __init__(self):
        self.logger = LoggerFactory.get_logger(
            __name__,
            category=LogCategory.AUDIT,
        )
    
    def log_operation(
        self,
        operation: str,
        status: str,
        user: Optional[str] = None,
        details: Optional[Dict[str, Any]] = None,
    ) -> None:
        """
        Log an operation for audit trail.
        
        Args:
            operation: Operation name
            status: Operation status (success, failure, started)
            user: User performing operation
            details: Additional details
        """
        message = f"Operation: {operation} | Status: {status}"
        if user:
            message += f" | User: {user}"
        
        record = logging.LogRecord(
            name=self.logger.name,
            level=logging.INFO,
            pathname="",
            lineno=0,
            msg=message,
            args=(),
            exc_info=None,
        )
        
        if details:
            record.extra_data = details
        
        self.logger.handle(record)


class PerformanceLogger:
    """Specialized logger for performance metrics."""
    
    def __init__(self):
        self.logger = LoggerFactory.get_logger(
            __name__,
            category=LogCategory.PERFORMANCE,
        )
    
    def log_operation_time(
        self,
        operation: str,
        duration_ms: float,
        threshold_ms: float = 1000,
    ) -> None:
        """
        Log operation timing.
        
        Args:
            operation: Operation name
            duration_ms: Duration in milliseconds
            threshold_ms: Warning threshold in milliseconds
        """
        level = logging.WARNING if duration_ms > threshold_ms else logging.INFO
        message = f"Operation: {operation} | Duration: {duration_ms:.2f}ms"
        self.logger.log(level, message)


def get_logger(
    name: str,
    category: LogCategory = LogCategory.ACTIVITY,
) -> logging.Logger:
    """
    Convenience function to get a logger.
    
    Args:
        name: Logger name
        category: Log category
        
    Returns:
        Configured logger instance
    """
    return LoggerFactory.get_logger(name, category)


def initialize_logging(
    log_level: LogLevel = LogLevel.INFO,
    use_json: bool = False,
    enable_console: bool = True,
) -> None:
    """
    Initialize the logging system.
    
    Args:
        log_level: Logging level
        use_json: Use JSON format for logs
        enable_console: Enable console output
    """
    LoggerFactory.initialize(
        log_level=log_level,
        use_json=use_json,
        enable_console=enable_console,
    )
