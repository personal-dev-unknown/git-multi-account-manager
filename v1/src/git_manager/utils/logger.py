# src/git_manager/utils/logger.py
"""Logging configuration."""

import logging
import sys
from pathlib import Path
from typing import Optional


class Logger:
    """Centralized logging system."""
    
    _instance: Optional['Logger'] = None
    _initialized = False
    
    def __new__(cls, name: str) -> 'Logger':
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

    def __init__(self, name: str):
        if not self._initialized:
            self.logger = logging.getLogger(name)
            self.logger.setLevel(logging.INFO)
            self._initialized = True
        
    def setup_logging(
        level: int = logging.INFO,
        log_file: Optional[Path] = None,
        format_string: Optional[str] = None
    ) -> None:
        """Setup logging configuration.
        
        Args:
            level: Logging level
            log_file: Optional log file path
            format_string: Optional custom format string
        """
        if format_string is None:
            format_string = '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        
        handlers = [logging.StreamHandler(sys.stdout)]
        
        if log_file:
            log_file.parent.mkdir(parents=True, exist_ok=True)
            handlers.append(logging.FileHandler(log_file))
        
        logging.basicConfig(
            level=level,
            format=format_string,
            handlers=handlers
        )


def get_logger(name: str) -> logging.Logger:
    """Get logger instance.
    
    Args:
        name: Logger name
        
    Returns:
        Logger instance
    """
    return logging.getLogger(name)

    # def log(self, level: int, msg: str) -> None:
    #     """Log a message.
        
    #     Args:
    #         level: Logging level
    #         msg: Message to log
    #     """
    #     self.logger.log(level, msg)

    def debug(self, message: str):
        """Log debug message."""
        self.logger.debug(message)
    
    def info(self, message: str):
        """Log info message."""
        self.logger.info(message)
    
    def warning(self, message: str):
        """Log warning message."""
        self.logger.warning(message)
    
    def error(self, message: str, exc_info: bool = False):
        """Log error message."""
        self.logger.error(message, exc_info=exc_info)
    
    def critical(self, message: str, exc_info: bool = False):
        """Log critical message."""
        self.logger.critical(message, exc_info=exc_info)
