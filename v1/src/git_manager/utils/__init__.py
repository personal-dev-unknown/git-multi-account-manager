# src/git_manager/utils/__init__.py
"""Git Manager Utils Module.

This module provides utility functions and classes for the Git Manager application.
"""

# Import from constants
from .constants import APP_NAME, APP_VERSION

# Import from file_operations
from .file_operations import (
    read_json,
    write_json,
    read_yaml,
    write_yaml,
    ensure_directory,
    copy_file,
    delete_path,
)

# Import from git_helpers
from .git_helpers import (
    run_git_command,
    get_current_branch,
    get_remote_url,
    is_git_repository,
    get_uncommitted_changes,
    get_commit_info,
)

# Import from logger
from .logger import get_logger

# Import from ssh_helpers
from .ssh_helpers import (
    is_ssh_agent_running,
    start_ssh_agent,
    get_ssh_key_fingerprint,
    test_ssh_connection,
    get_public_key,
)

# Import from validators
from .validators import Validators

# Export validator functions
validate_username = Validators.validate_username
validate_email = Validators.validate_email
validate_ssh_key = Validators.validate_ssh_key
validate_url = Validators.validate_url

__all__ = [
    # Constants
    'APP_NAME',
    'APP_VERSION',
    
    # File operations
    'read_json',
    'write_json',
    'read_yaml',
    'write_yaml',
    'ensure_directory',
    'copy_file',
    'delete_path',
    
    # Git helpers
    'run_git_command',
    'get_current_branch',
    'get_remote_url',
    'is_git_repository',
    'get_uncommitted_changes',
    'get_commit_info',
    
    # Logger
    'setup_logging',
    'get_logger',
    
    # SSH helpers
    'is_ssh_agent_running',
    'start_ssh_agent',
    'get_ssh_key_fingerprint',
    'test_ssh_connection',
    'get_public_key',
    
    # Validators
    'validate_username',
    'validate_email',
    'validate_ssh_key',
    'validate_url',
]