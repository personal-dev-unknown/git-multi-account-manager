"""
Update Checker Module

Checks for available updates from GitHub and PyPI.
"""

import json
import logging
import subprocess
import sys
from typing import Dict, Optional
from urllib.request import urlopen
from urllib.error import URLError

from .version_reader import VersionReader
from .version_comparator import VersionComparator

logger = logging.getLogger(__name__)


class UpdateChecker:
    """Checks for and manages application updates."""

    # GitHub API endpoint for releases
    GITHUB_API_URL = "https://api.github.com/repos/Zanabuni/git-multi-account-manager/releases/latest"
    
    # PyPI API endpoint
    PYPI_API_URL = "https://pypi.org/pypi/git-multi-account-manager/json"

    def __init__(self, timeout: int = 5):
        """Initialize the update checker.
        
        Args:
            timeout: Timeout for API requests in seconds.
        """
        self.timeout = timeout
        self.version_reader = VersionReader()
        
        try:
            self.current_version = self.version_reader.read_version_file()
        except FileNotFoundError:
            self.current_version = "0.0.0"
            logger.warning("Could not read current version")
        
        self.latest_version: Optional[str] = None
        self.release_info: Dict = {}

    def check_github_updates(self) -> Optional[str]:
        """Check for updates from GitHub releases.
        
        Returns:
            Latest version string if available, None otherwise.
        """
        try:
            logger.debug(f"Checking GitHub for updates: {self.GITHUB_API_URL}")
            with urlopen(self.GITHUB_API_URL, timeout=self.timeout) as response:
                data = json.loads(response.read().decode())
                
                # Extract version from tag name (e.g., "v1.0.1" -> "1.0.1")
                tag_name = data.get('tag_name', '')
                version = tag_name.lstrip('v')
                
                if version:
                    self.latest_version = version
                    self.release_info = {
                        'version': version,
                        'tag': tag_name,
                        'url': data.get('html_url'),
                        'body': data.get('body', ''),
                        'published_at': data.get('published_at'),
                        'download_url': data.get('zipball_url'),
                    }
                    logger.info(f"Latest version from GitHub: {version}")
                    return version
        except URLError as e:
            logger.warning(f"Failed to check GitHub for updates: {e}")
        except (json.JSONDecodeError, KeyError) as e:
            logger.warning(f"Failed to parse GitHub response: {e}")
        
        return None

    def check_pypi_updates(self) -> Optional[str]:
        """Check for updates from PyPI.
        
        Returns:
            Latest version string if available, None otherwise.
        """
        try:
            logger.debug(f"Checking PyPI for updates: {self.PYPI_API_URL}")
            with urlopen(self.PYPI_API_URL, timeout=self.timeout) as response:
                data = json.loads(response.read().decode())
                version = data.get('info', {}).get('version')
                
                if version:
                    self.latest_version = version
                    logger.info(f"Latest version from PyPI: {version}")
                    return version
        except URLError as e:
            logger.warning(f"Failed to check PyPI for updates: {e}")
        except (json.JSONDecodeError, KeyError) as e:
            logger.warning(f"Failed to parse PyPI response: {e}")
        
        return None

    def check_for_updates(self, source: str = 'github') -> bool:
        """Check if updates are available.
        
        Args:
            source: Source to check ('github' or 'pypi').
            
        Returns:
            True if update is available, False otherwise.
        """
        if source == 'github':
            latest = self.check_github_updates()
        elif source == 'pypi':
            latest = self.check_pypi_updates()
        else:
            logger.error(f"Unknown update source: {source}")
            return False
        
        if not latest:
            return False
        
        is_available = VersionComparator.is_update_available(
            self.current_version,
            latest
        )
        
        if is_available:
            logger.info(
                f"Update available: {self.current_version} → {latest}"
            )
        else:
            logger.info(f"Already on latest version: {self.current_version}")
        
        return is_available

    def get_update_info(self) -> Dict:
        """Get information about available update.
        
        Returns:
            Dictionary with update information.
        """
        return {
            'current_version': self.current_version,
            'latest_version': self.latest_version,
            'update_available': VersionComparator.is_update_available(
                self.current_version,
                self.latest_version or self.current_version
            ),
            'release_info': self.release_info,
        }

    def install_update_pip(self) -> bool:
        """Install update using pip.
        
        Returns:
            True if installation succeeded, False otherwise.
        """
        if not self.latest_version:
            logger.error("No latest version available")
            return False
        
        try:
            logger.info(f"Installing update to version {self.latest_version}")
            subprocess.run(
                [sys.executable, '-m', 'pip', 'install', '--upgrade',
                 f'git-multi-account-manager=={self.latest_version}'],
                check=True,
                capture_output=True
            )
            logger.info(f"Successfully updated to version {self.latest_version}")
            return True
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to install update: {e}")
            return False


class UpdateNotifier:
    """Notifies users about available updates."""

    def __init__(self, checker: UpdateChecker):
        """Initialize the update notifier.
        
        Args:
            checker: UpdateChecker instance.
        """
        self.checker = checker

    def notify_update_available(self) -> str:
        """Generate notification message for available update.
        
        Returns:
            Notification message string.
        """
        if not self.checker.latest_version:
            return ""
        
        version_change = VersionComparator.format_version_change(
            self.checker.current_version,
            self.checker.latest_version
        )
        
        message = (
            f"\n{'='*60}\n"
            f"🎉 Update Available!\n"
            f"{'='*60}\n"
            f"{version_change}\n"
            f"\nTo update, run:\n"
            f"  pip install --upgrade git-multi-account-manager\n"
            f"\nRelease notes: {self.checker.release_info.get('url', 'N/A')}\n"
            f"{'='*60}\n"
        )
        return message

    def notify_up_to_date(self) -> str:
        """Generate notification message when up to date.
        
        Returns:
            Notification message string.
        """
        return f"✓ You are running the latest version: {self.checker.current_version}\n"


def check_for_updates_on_startup(
    source: str = 'github',
    notify: bool = True,
    timeout: int = 5
) -> Optional[str]:
    """Check for updates on application startup.
    
    This is a convenience function to check for updates without
    blocking the application startup.
    
    Args:
        source: Update source ('github' or 'pypi').
        notify: Whether to print notification messages.
        timeout: Timeout for API requests in seconds.
        
    Returns:
        Latest version string if update is available, None otherwise.
    """
    try:
        checker = UpdateChecker(timeout=timeout)
        
        if checker.check_for_updates(source=source):
            if notify:
                notifier = UpdateNotifier(checker)
                print(notifier.notify_update_available())
            return checker.latest_version
        else:
            if notify:
                notifier = UpdateNotifier(checker)
                print(notifier.notify_up_to_date())
            return None
    except Exception as e:
        logger.debug(f"Update check failed: {e}")
        return None
