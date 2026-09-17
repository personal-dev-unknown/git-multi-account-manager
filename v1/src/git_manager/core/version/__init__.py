"""
Version Management Module

This module handles version management, update checking, and version comparisons
for the Git Multi-Account Manager application.

Submodules:
    - version_reader: Read and parse version information
    - version_comparator: Compare semantic versions
    - update_checker: Check for available updates
    - file_updater: Update version strings in project files
"""

from .version_reader import VersionReader
from .version_comparator import VersionComparator
from .update_checker import UpdateChecker, UpdateNotifier, check_for_updates_on_startup
from .file_updater import FileUpdater

__all__ = [
    'VersionReader',
    'VersionComparator',
    'UpdateChecker',
    'UpdateNotifier',
    'FileUpdater',
    'check_for_updates_on_startup',
]
