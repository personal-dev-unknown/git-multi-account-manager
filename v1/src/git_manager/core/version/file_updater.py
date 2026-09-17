"""
File Updater Module

Updates version strings in project files.
"""

import logging
import re
from pathlib import Path
from typing import Dict, Callable, Optional

logger = logging.getLogger(__name__)


class FileUpdater:
    """Updates version strings in project files."""

    def __init__(self, project_root: Optional[Path] = None):
        """Initialize the file updater.
        
        Args:
            project_root: Root directory of the project. If None, auto-detects.
        """
        if project_root is None:
            # Auto-detect project root
            project_root = Path(__file__).parent.parent.parent.parent.parent
        
        self.project_root = project_root
        self.version_file = project_root / 'VERSION'
        
        # Define files to update with their update functions
        self.files_to_update: Dict[str, Callable] = {
            'setup.py': self._update_setup_py,
            'pyproject.toml': self._update_pyproject_toml,
            'src/git_manager/utils/constants.py': self._update_constants_py,
            'src/git_manager/__init__.py': self._update_init_py,
            'src/git_manager/web/routes/api.py': self._update_api_py,
            'README.md': self._update_readme,
            'snap/snapcraft.yaml': self._update_snapcraft_yaml,
            'scripts/build.py': self._update_build_py,
            'scripts/release.py': self._update_release_py,
        }

    def _update_file(self, file_path: Path, old_pattern: str, new_text: str) -> bool:
        """Update a file with new content.
        
        Args:
            file_path: Path to the file to update.
            old_pattern: Regex pattern to find.
            new_text: Text to replace with.
            
        Returns:
            True if file was updated, False if file not found.
        """
        if not file_path.exists():
            logger.warning(f"File not found: {file_path}")
            return False
        
        try:
            content = file_path.read_text()
            new_content = re.sub(old_pattern, new_text, content)
            
            if new_content != content:
                file_path.write_text(new_content)
                logger.info(f"Updated: {file_path}")
                return True
            else:
                logger.warning(f"No version string found in: {file_path}")
                return False
        except Exception as e:
            logger.error(f"Error updating {file_path}: {e}")
            return False

    def _update_setup_py(self, new_version: str) -> bool:
        """Update version in setup.py."""
        file_path = self.project_root / 'setup.py'
        return self._update_file(
            file_path,
            r"version='[^']*'",
            f"version='{new_version}'"
        )

    def _update_pyproject_toml(self, new_version: str) -> bool:
        """Update version in pyproject.toml."""
        file_path = self.project_root / 'pyproject.toml'
        return self._update_file(
            file_path,
            r'version = "[^"]*"',
            f'version = "{new_version}"'
        )

    def _update_constants_py(self, new_version: str) -> bool:
        """Update version in constants.py."""
        file_path = self.project_root / 'src/git_manager/utils/constants.py'
        return self._update_file(
            file_path,
            r'APP_VERSION = "[^"]*"',
            f'APP_VERSION = "{new_version}"'
        )

    def _update_init_py(self, new_version: str) -> bool:
        """Update version in __init__.py (if needed)."""
        file_path = self.project_root / 'src/git_manager/__init__.py'
        if not file_path.exists():
            return False
        
        # __init__.py imports from constants, so no direct update needed
        logger.debug(f"Skipping {file_path} (imports from constants)")
        return True

    def _update_api_py(self, new_version: str) -> bool:
        """Update version in API routes."""
        file_path = self.project_root / 'src/git_manager/web/routes/api.py'
        return self._update_file(
            file_path,
            r"'version': '[^']*'",
            f"'version': '{new_version}'"
        )

    def _update_readme(self, new_version: str) -> bool:
        """Update version references in README.md."""
        file_path = self.project_root / 'README.md'
        if not file_path.exists():
            logger.warning(f"File not found: {file_path}")
            return False
        
        try:
            content = file_path.read_text()
            new_content = re.sub(
                r'v[0-9]+\.[0-9]+\.[0-9]+',
                f'v{new_version}',
                content
            )
            
            if new_content != content:
                file_path.write_text(new_content)
                logger.info(f"Updated: {file_path}")
                return True
            else:
                logger.warning(f"No version string found in: {file_path}")
                return False
        except Exception as e:
            logger.error(f"Error updating {file_path}: {e}")
            return False

    def _update_snapcraft_yaml(self, new_version: str) -> bool:
        """Update version in snap/snapcraft.yaml."""
        file_path = self.project_root / 'snap/snapcraft.yaml'
        return self._update_file(
            file_path,
            r"version: '[^']*'",
            f"version: '{new_version}'"
        )

    def _update_build_py(self, new_version: str) -> bool:
        """Update version in scripts/build.py.
        
        Updates the BUILD_SCRIPT_VERSION constant that tracks the build script version.
        """
        file_path = self.project_root / 'scripts/build.py'
        if not file_path.exists():
            logger.warning(f"File not found: {file_path}")
            return False
        
        try:
            content = file_path.read_text()
            # Update version in BuildConfig._read_version() docstring/default
            new_content = re.sub(
                r'BUILD_SCRIPT_VERSION = "[^"]*"',
                f'BUILD_SCRIPT_VERSION = "{new_version}"'
            ) and self._update_file(
                file_path,
                r'return "([0-9]+\.[0-9]+\.[0-9]+)"',
                f'return "{new_version}"'
            )

            # return self._update_file(
            # file_path,

            if new_content != content:
                file_path.write_text(new_content)
                logger.info(f"Updated: {file_path}")
                return True
            else:
                logger.warning(f"No version string found in: {file_path}")
                return False
        except Exception as e:
            logger.error(f"Error updating {file_path}: {e}")
            return False

    def _update_release_py(self, new_version: str) -> bool:
        """Update version in scripts/release.py.
        
        Updates the RELEASE_SCRIPT_VERSION constant that tracks the release script version.
        """
        file_path = self.project_root / 'scripts/release.py'
        return self._update_file(
            file_path,
            r'RELEASE_SCRIPT_VERSION = "[^"]*"',
            f'RELEASE_SCRIPT_VERSION = "{new_version}"'
        )

    def update_version_file(self, new_version: str) -> bool:
        """Update the VERSION file.
        
        Args:
            new_version: New version string.
            
        Returns:
            True if updated successfully, False otherwise.
        """
        try:
            self.version_file.write_text(f"{new_version}\n")
            logger.info(f"Updated VERSION file: {new_version}")
            return True
        except Exception as e:
            logger.error(f"Error updating VERSION file: {e}")
            return False

    def update_all_files(self, new_version: str) -> dict:
        """Update all version files.
        
        Args:
            new_version: New version string.
            
        Returns:
            Dictionary with update results for each file.
        """
        results = {}
        
        # Update VERSION file first
        results['VERSION'] = self.update_version_file(new_version)
        
        # Update all tracked files
        for file_name, update_func in self.files_to_update.items():
            try:
                results[file_name] = update_func(new_version)
            except Exception as e:
                logger.error(f"Error updating {file_name}: {e}")
                results[file_name] = False
        
        return results

    def get_updated_files_summary(self, results: dict) -> dict:
        """Get a summary of updated files.
        
        Args:
            results: Dictionary of update results.
            
        Returns:
            Dictionary with summary information.
        """
        updated = [f for f, success in results.items() if success]
        failed = [f for f, success in results.items() if not success]
        
        return {
            'total': len(results),
            'updated': len(updated),
            'failed': len(failed),
            'updated_files': updated,
            'failed_files': failed,
        }
