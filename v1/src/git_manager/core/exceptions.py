# src/git_manager/core/exceptions.py
"""Custom exceptions for Git Manager."""


class GitManagerError(Exception):
    """Base exception for Git Manager."""
    pass


class AccountError(GitManagerError):
    """Account-related errors."""
    pass


class AccountNotFoundError(AccountError):
    """Account not found."""
    pass


class DuplicateAccountError(AccountError):
    """Duplicate account name."""
    pass


class SSHError(GitManagerError):
    """SSH-related errors."""
    pass


class GitError(GitManagerError):
    """Git operation errors."""
    pass


class RepositoryError(GitError):
    """Repository-related errors."""
    pass


class ConfigError(GitManagerError):
    """Configuration errors."""
    pass


class ValidationError(GitManagerError):
    """Validation errors."""
    pass


# ============================================================================
# SSH Exceptions - Extended from base SSHError
# ============================================================================

class SSHKeyGenerationError(SSHError):
    """Raised when SSH key generation fails."""
    
    def __init__(self, message: str, details: dict = None):
        self.message = message
        self.error_code = "SSH_KEY_GENERATION_ERROR"
        self.details = details or {}
        super().__init__(self.message)


class SSHKeyNotFoundError(SSHError):
    """Raised when SSH key is not found."""
    
    def __init__(self, key_name: str, key_path: str = None):
        self.details = {"key_name": key_name}
        if key_path:
            self.details["key_path"] = key_path
        message = f"SSH key not found: {key_name}"
        super().__init__(message)


class SSHKeyAlreadyExistsError(SSHError):
    """Raised when SSH key already exists."""
    
    def __init__(self, key_name: str, key_path: str = None):
        self.details = {"key_name": key_name}
        if key_path:
            self.details["key_path"] = key_path
        message = f"SSH key already exists: {key_name}"
        super().__init__(message)


class SSHKeyPermissionError(SSHError):
    """Raised when SSH key has incorrect permissions."""
    
    def __init__(self, key_path: str, current_mode: str, expected_mode: str = "600"):
        self.details = {
            "key_path": key_path,
            "current_mode": current_mode,
            "expected_mode": expected_mode
        }
        message = f"SSH key has incorrect permissions: {current_mode} (expected {expected_mode})"
        super().__init__(message)


class SSHKeyValidationError(SSHError):
    """Raised when SSH key validation fails."""
    
    def __init__(self, key_path: str, reason: str):
        self.details = {"key_path": key_path, "reason": reason}
        message = f"SSH key validation failed: {reason}"
        super().__init__(message)


class SSHAgentError(SSHError):
    """Raised when SSH agent operations fail."""
    
    def __init__(self, message: str, operation: str = None):
        self.details = {}
        if operation:
            self.details["operation"] = operation
        super().__init__(message)


class SSHAgentNotRunningError(SSHAgentError):
    """Raised when SSH agent is not running."""
    
    def __init__(self, message: str = "SSH agent is not running"):
        super().__init__(message, operation="agent_check")


class SSHConfigError(SSHError):
    """Raised when SSH config operations fail."""
    pass


class SSHConfigParseError(SSHConfigError):
    """Raised when SSH config parsing fails."""
    
    def __init__(self, config_path: str, reason: str):
        self.details = {"config_path": config_path, "reason": reason}
        message = f"Failed to parse SSH config at {config_path}: {reason}"
        super().__init__(message)


class SSHConnectionTestError(SSHError):
    """Raised when SSH connection test fails."""
    
    def __init__(self, host: str, reason: str):
        self.details = {"host": host, "reason": reason}
        message = f"SSH connection test failed for {host}: {reason}"
        super().__init__(message)


class SSHURLConversionError(SSHError):
    """Raised when SSH URL conversion fails."""
    
    def __init__(self, url: str, reason: str):
        self.details = {"url": url, "reason": reason}
        message = f"Failed to convert SSH URL {url}: {reason}"
        super().__init__(message)


class SSHMetadataError(SSHError):
    """Raised when SSH metadata operations fail."""
    pass


class SSHBackupError(SSHError):
    """Raised when SSH backup operations fail."""
    
    def __init__(self, operation: str, reason: str):
        self.details = {"operation": operation, "reason": reason}
        message = f"SSH backup operation failed ({operation}): {reason}"
        super().__init__(message)


class SSHIntegrationError(SSHError):
    """Raised when SSH integration operations fail."""
    pass


class SSHDatabaseError(SSHError):
    """Raised when SSH database operations fail."""
    pass


# ============================================================================
# Repository Exceptions
# ============================================================================

class RepositorySetupError(RepositoryError):
    """Raised when repository setup fails."""
    pass


class RepositoryInitializationError(RepositoryError):
    """Raised when repository initialization fails."""
    pass


class RepositoryConfigError(RepositoryError):
    """Raised when repository configuration fails."""
    pass