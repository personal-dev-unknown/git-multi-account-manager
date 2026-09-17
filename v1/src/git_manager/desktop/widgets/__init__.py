# src/git_manager/desktop/widgets/__init__.py
"""Desktop widgets package."""

from .account_list import AccountListWidget
from .repository_list import RepositoryListWidget
from .ssh_key_list import SSHKeyListWidget
from .terminal_widget import TerminalWidget

__all__ = [
    'AccountListWidget',
    'RepositoryListWidget', 
    'SSHKeyListWidget',
    'TerminalWidget'
]
