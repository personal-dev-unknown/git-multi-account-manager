# src/git_manager/core/ssh/exceptions.py
"""
SSH Module Exception Handling

Custom exceptions for SSH operations with detailed error information.
"""


class SSHException(Exception):
    """Base exception for SSH operations."""
    
    def __init__(self, message: str, error_code: str = None, details: dict = None):
        """
        Initialize SSH exception.
        
        Args:
            message: Error message
            error_code: Error code for categorization
            details: Additional error details
        """
        self.message = message
        self.error_code = error_code or "SSH_ERROR"
        self.details = details or {}
        super().__init__(self.message)
    
    def __str__(self):
        """Return formatted error message."""
        if self.details:
            details_str = ", ".join(f"{k}={v}" for k, v in self.details.items())
            return f"[{self.error_code}] {self.message} ({details_str})"
        return f"[{self.error_code}] {self.message}"


class SSHKeyGenerationError(SSHException):
    """Raised when SSH key generation fails."""
    
    def __init__(self, message: str, details: dict = None):
        super().__init__(message, "SSH_KEY_GENERATION_ERROR", details)


class SSHKeyNotFoundError(SSHException):
    """Raised when SSH key is not found."""
    
    def __init__(self, key_name: str, key_path: str = None):
        details = {"key_name": key_name}
        if key_path:
            details["key_path"] = key_path
        super().__init__(f"SSH key not found: {key_name}", "SSH_KEY_NOT_FOUND", details)


class SSHKeyAlreadyExistsError(SSHException):
    """Raised when SSH key already exists."""
    
    def __init__(self, key_name: str, key_path: str = None):
        details = {"key_name": key_name}
        if key_path:
            details["key_path"] = key_path
        super().__init__(f"SSH key already exists: {key_name}", "SSH_KEY_EXISTS", details)


class SSHKeyPermissionError(SSHException):
    """Raised when SSH key has incorrect permissions."""
    
    def __init__(self, key_path: str, current_mode: str, expected_mode: str = "600"):
        details = {
            "key_path": key_path,
            "current_mode": current_mode,
            "expected_mode": expected_mode
        }
        super().__init__(
            f"SSH key has incorrect permissions: {current_mode} (expected {expected_mode})",
            "SSH_KEY_PERMISSION_ERROR",
            details
        )


class SSHKeyValidationError(SSHException):
    """Raised when SSH key validation fails."""
    
    def __init__(self, key_path: str, reason: str):
        details = {"key_path": key_path, "reason": reason}
        super().__init__(
            f"SSH key validation failed: {reason}",
            "SSH_KEY_VALIDATION_ERROR",
            details
        )


class SSHAgentError(SSHException):
    """Raised when SSH agent operations fail."""
    
    def __init__(self, message: str, operation: str = None):
        details = {}
        if operation:
            details["operation"] = operation
        super().__init__(message, "SSH_AGENT_ERROR", details)


class SSHAgentNotRunningError(SSHException):
    """Raised when SSH agent is not running."""
    
    def __init__(self):
        super().__init__(
            "SSH agent is not running",
            "SSH_AGENT_NOT_RUNNING"
        )


class SSHConfigError(SSHException):
    """Raised when SSH config operations fail."""
    
    def __init__(self, message: str, config_file: str = None):
        details = {}
        if config_file:
            details["config_file"] = config_file
        super().__init__(message, "SSH_CONFIG_ERROR", details)


class SSHConfigParseError(SSHException):
    """Raised when SSH config parsing fails."""
    
    def __init__(self, config_file: str, line_number: int = None, reason: str = None):
        details = {"config_file": config_file}
        if line_number:
            details["line_number"] = line_number
        if reason:
            details["reason"] = reason
        super().__init__(
            f"Failed to parse SSH config: {config_file}",
            "SSH_CONFIG_PARSE_ERROR",
            details
        )


class SSHConnectionTestError(SSHException):
    """Raised when SSH connection test fails."""
    
    def __init__(self, host: str, error_output: str = None):
        details = {"host": host}
        if error_output:
            details["error_output"] = error_output[:100]  # Truncate for safety
        super().__init__(
            f"SSH connection test failed for host: {host}",
            "SSH_CONNECTION_TEST_ERROR",
            details
        )


class SSHURLConversionError(SSHException):
    """Raised when URL conversion fails."""
    
    def __init__(self, url: str, reason: str = None):
        details = {"url": url}
        if reason:
            details["reason"] = reason
        super().__init__(
            f"Failed to convert URL: {url}",
            "SSH_URL_CONVERSION_ERROR",
            details
        )


class SSHMetadataError(SSHException):
    """Raised when SSH metadata operations fail."""
    
    def __init__(self, message: str, operation: str = None):
        details = {}
        if operation:
            details["operation"] = operation
        super().__init__(message, "SSH_METADATA_ERROR", details)


class SSHBackupError(SSHException):
    """Raised when SSH key backup fails."""
    
    def __init__(self, key_path: str, reason: str = None):
        details = {"key_path": key_path}
        if reason:
            details["reason"] = reason
        super().__init__(
            f"Failed to backup SSH key: {key_path}",
            "SSH_BACKUP_ERROR",
            details
        )


class SSHIntegrationError(SSHException):
    """Raised when SSH integration with other systems fails."""
    
    def __init__(self, message: str, component: str = None):
        details = {}
        if component:
            details["component"] = component
        super().__init__(message, "SSH_INTEGRATION_ERROR", details)


class SSHDatabaseError(SSHException):
    """Raised when SSH database operations fail."""
    
    def __init__(self, message: str, operation: str = None):
        details = {}
        if operation:
            details["operation"] = operation
        super().__init__(message, "SSH_DATABASE_ERROR", details)


# Exception handler utility
class SSHExceptionHandler:
    """Utility for handling SSH exceptions."""
    
    @staticmethod
    def handle(exception: SSHException, logger=None) -> dict:
        """
        Handle SSH exception and return structured error response.
        
        Args:
            exception: SSH exception to handle
            logger: Optional logger instance
            
        Returns:
            Dict with error information
        """
        error_response = {
            "success": False,
            "error_code": exception.error_code,
            "message": exception.message,
            "details": exception.details
        }
        
        if logger:
            logger.error(str(exception), exc_info=True)
        
        return error_response
    
    @staticmethod
    def wrap_operation(func, logger=None):
        """
        Decorator to wrap SSH operations with exception handling.
        
        Args:
            func: Function to wrap
            logger: Optional logger instance
            
        Returns:
            Wrapped function
        """
        def wrapper(*args, **kwargs):
            try:
                return func(*args, **kwargs)
            except SSHException as e:
                return SSHExceptionHandler.handle(e, logger)
            except Exception as e:
                generic_error = SSHException(
                    str(e),
                    "SSH_UNKNOWN_ERROR",
                    {"original_exception": type(e).__name__}
                )
                return SSHExceptionHandler.handle(generic_error, logger)
        
        return wrapper
