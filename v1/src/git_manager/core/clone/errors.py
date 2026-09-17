"""Custom exceptions for clone operations."""


class CloneError(Exception):
    """Base exception for clone operations."""
    pass


class AuthenticationError(CloneError):
    """Raised when authentication fails."""
    pass


class PermissionError(CloneError):
    """Raised when user doesn't have permission to access repository."""
    pass


class RepositoryNotFoundError(CloneError):
    """Raised when repository is not found."""
    pass


class NetworkError(CloneError):
    """Raised when network connection fails."""
    pass


class DiskSpaceError(CloneError):
    """Raised when there's insufficient disk space."""
    pass


class InvalidURLError(CloneError):
    """Raised when URL format is invalid."""
    pass


class PlatformError(CloneError):
    """Raised when platform is not supported."""
    pass


class SSHError(CloneError):
    """Raised when SSH operation fails."""
    pass


class APIError(CloneError):
    """Raised when API call fails."""
    pass
