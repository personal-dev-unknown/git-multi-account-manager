"""
Version Reader Module

Reads and parses version information from various sources.
"""

import logging
import re
from pathlib import Path
from typing import Optional, Tuple

logger = logging.getLogger(__name__)


class VersionReader:
    """Reads version information from project files."""

    def __init__(self, project_root: Optional[Path] = None):
        """Initialize the version reader.
        
        Args:
            project_root: Root directory of the project. If None, auto-detects.
        """
        if project_root is None:
            # Auto-detect project root (go up from this file to src, then to root)
            project_root = Path(__file__).parent.parent.parent.parent.parent
        
        self.project_root = project_root
        self.version_file = project_root / 'VERSION'

    def read_version_file(self) -> str:
        """Read version from VERSION file.
        
        Returns:
            Version string from VERSION file.
            
        Raises:
            FileNotFoundError: If VERSION file not found.
        """
        if not self.version_file.exists():
            raise FileNotFoundError(
                f"VERSION file not found at {self.version_file}"
            )
        
        version = self.version_file.read_text().strip()
        logger.debug(f"Read version from file: {version}")
        return version

    def read_setup_py_version(self) -> Optional[str]:
        """Read version from setup.py.
        
        Returns:
            Version string from setup.py, or None if not found.
        """
        setup_file = self.project_root / 'setup.py'
        if not setup_file.exists():
            return None
        
        content = setup_file.read_text()
        match = re.search(r"version='([^']+)'", content)
        
        if match:
            version = match.group(1)
            logger.debug(f"Read version from setup.py: {version}")
            return version
        
        return None

    def read_pyproject_toml_version(self) -> Optional[str]:
        """Read version from pyproject.toml.
        
        Returns:
            Version string from pyproject.toml, or None if not found.
        """
        pyproject_file = self.project_root / 'pyproject.toml'
        if not pyproject_file.exists():
            return None
        
        content = pyproject_file.read_text()
        match = re.search(r'version = "([^"]+)"', content)
        
        if match:
            version = match.group(1)
            logger.debug(f"Read version from pyproject.toml: {version}")
            return version
        
        return None

    def read_constants_py_version(self) -> Optional[str]:
        """Read version from constants.py.
        
        Returns:
            Version string from constants.py, or None if not found.
        """
        constants_file = self.project_root / 'src/git_manager/utils/constants.py'
        if not constants_file.exists():
            return None
        
        content = constants_file.read_text()
        match = re.search(r'APP_VERSION = "([^"]+)"', content)
        
        if match:
            version = match.group(1)
            logger.debug(f"Read version from constants.py: {version}")
            return version
        
        return None

    def read_api_version(self) -> Optional[str]:
        """Read API version from API routes.
        
        Returns:
            API version string, or None if not found.
        """
        api_file = self.project_root / 'src/git_manager/web/routes/api.py'
        if not api_file.exists():
            return None
        
        content = api_file.read_text()
        match = re.search(r"'version': '([^']+)'", content)
        
        if match:
            version = match.group(1)
            logger.debug(f"Read API version: {version}")
            return version
        
        return None

    def get_all_versions(self) -> dict:
        """Get all version references in the project.
        
        Returns:
            Dictionary with version information from all sources.
        """
        return {
            'VERSION_file': self.read_version_file(),
            'setup_py': self.read_setup_py_version(),
            'pyproject_toml': self.read_pyproject_toml_version(),
            'constants_py': self.read_constants_py_version(),
            'api_version': self.read_api_version(),
        }

    def verify_version_consistency(self) -> Tuple[bool, dict]:
        """Verify that all version files have consistent versions.
        
        Returns:
            Tuple of (is_consistent, versions_dict)
        """
        versions = self.get_all_versions()
        
        # Get the main version from VERSION file
        main_version = versions.get('VERSION_file')
        
        # Check if all other versions match
        is_consistent = all(
            v == main_version
            for k, v in versions.items()
            if k != 'VERSION_file' and v is not None
        )
        
        if is_consistent:
            logger.info(f"All version files are consistent: {main_version}")
        else:
            logger.warning("Version files are inconsistent!")
            for key, version in versions.items():
                if version != main_version:
                    logger.warning(f"  {key}: {version} (expected: {main_version})")
        
        return is_consistent, versions
