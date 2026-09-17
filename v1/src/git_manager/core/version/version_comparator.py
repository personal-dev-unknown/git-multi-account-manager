"""
Version Comparator Module

Compares semantic versions to determine update availability.
"""

import logging
from typing import Tuple

logger = logging.getLogger(__name__)


class VersionComparator:
    """Compares semantic versions."""

    @staticmethod
    def parse_version(version: str) -> Tuple[int, int, int]:
        """Parse version string into components.
        
        Args:
            version: Version string (MAJOR.MINOR.PATCH).
            
        Returns:
            Tuple of (major, minor, patch).
            
        Raises:
            ValueError: If version format is invalid.
        """
        try:
            parts = version.split('.')
            if len(parts) != 3:
                raise ValueError(f"Invalid version format: {version}")
            return int(parts[0]), int(parts[1]), int(parts[2])
        except (ValueError, AttributeError) as e:
            raise ValueError(f"Cannot parse version '{version}': {e}")

    @staticmethod
    def compare(current: str, latest: str) -> int:
        """Compare two versions.
        
        Args:
            current: Current version string.
            latest: Latest version string.
            
        Returns:
            -1 if current < latest (update available)
             0 if current == latest (up to date)
             1 if current > latest (current is newer)
        """
        try:
            curr_parts = VersionComparator.parse_version(current)
            latest_parts = VersionComparator.parse_version(latest)
            
            if curr_parts < latest_parts:
                return -1
            elif curr_parts > latest_parts:
                return 1
            else:
                return 0
        except ValueError as e:
            logger.error(f"Version comparison failed: {e}")
            return 0

    @staticmethod
    def is_update_available(current: str, latest: str) -> bool:
        """Check if an update is available.
        
        Args:
            current: Current version string.
            latest: Latest version string.
            
        Returns:
            True if update is available, False otherwise.
        """
        return VersionComparator.compare(current, latest) < 0

    @staticmethod
    def get_version_difference(current: str, latest: str) -> dict:
        """Get the difference between two versions.
        
        Args:
            current: Current version string.
            latest: Latest version string.
            
        Returns:
            Dictionary with version component differences.
        """
        try:
            curr_major, curr_minor, curr_patch = VersionComparator.parse_version(current)
            latest_major, latest_minor, latest_patch = VersionComparator.parse_version(latest)
            
            return {
                'major_change': latest_major - curr_major,
                'minor_change': latest_minor - curr_minor,
                'patch_change': latest_patch - curr_patch,
                'is_major_update': latest_major > curr_major,
                'is_minor_update': latest_minor > curr_minor and latest_major == curr_major,
                'is_patch_update': latest_patch > curr_patch and latest_major == curr_major and latest_minor == curr_minor,
            }
        except ValueError as e:
            logger.error(f"Failed to calculate version difference: {e}")
            return {}

    @staticmethod
    def format_version_change(current: str, latest: str) -> str:
        """Format a human-readable version change message.
        
        Args:
            current: Current version string.
            latest: Latest version string.
            
        Returns:
            Formatted version change message.
        """
        diff = VersionComparator.get_version_difference(current, latest)
        
        if diff.get('is_major_update'):
            return f"Major update: {current} → {latest}"
        elif diff.get('is_minor_update'):
            return f"Minor update: {current} → {latest}"
        elif diff.get('is_patch_update'):
            return f"Patch update: {current} → {latest}"
        else:
            return f"Version: {current} → {latest}"
