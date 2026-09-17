# src/git_manager/__init__.py
"""Git Multi-Account Manager."""

from .core.account_manager import AccountManager
from .core.ssh import SSHWorkflowOrchestrator
from .core.git_operations import GitOperations
from .utils.constants import APP_VERSION

__version__ = APP_VERSION
__all__ = ['AccountManager', 'SSHWorkflowOrchestrator', 'GitOperations']