"""Authentication methods for clone operations."""

from .ssh import SSHAuth
from .https_pat import HTTPSPATAuth
from .https_password import HTTPSPasswordAuth
from .anonymous import AnonymousAuth

__all__ = [
    'SSHAuth',
    'HTTPSPATAuth',
    'HTTPSPasswordAuth',
    'AnonymousAuth',
]
