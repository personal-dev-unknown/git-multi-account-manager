# src/git_manager/desktop/windows/__init__.py
"""Desktop windows package."""

from ..app import MainWindow
from .account_window import AccountWidget
from .repository_window import RepositoryWidget
from .ssh_window import SSHWidget

__all__ = ['MainWindow', 'AccountWidget', 'RepositoryWidget', 'SSHWidget']
