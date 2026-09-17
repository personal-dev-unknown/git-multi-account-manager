#!/usr/bin/env python3
"""Release script for Git Multi-Account Manager.

This script automates the release process:
1. Updates version in all files
2. Runs tests
3. Builds the package
4. Creates a git tag
5. Provides next steps for publishing

Usage:
    python release.py <version>
    python release.py 1.1.0
"""

import logging
import subprocess
import sys
from pathlib import Path

# Add src directory to path for imports
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root / 'src'))

from git_manager.core.version import (
    VersionReader,
    VersionComparator,
    FileUpdater,
)

# Release script version - automatically updated by version manager
RELEASE_SCRIPT_VERSION = "1.0.0"

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(levelname)s: %(message)s'
)
logger = logging.getLogger(__name__)


class ReleaseManager:
    """Manages the release process."""

    def __init__(self, version: str):
        """Initialize the release manager.
        
        Args:
            version: Version string (MAJOR.MINOR.PATCH).
        """
        self.version = version
        self.project_root = project_root
        self.version_reader = VersionReader(project_root)
        self.file_updater = FileUpdater(project_root)

    def validate_version(self) -> bool:
        """Validate version format.
        
        Returns:
            True if version is valid, False otherwise.
        """
        try:
            VersionComparator.parse_version(self.version)
            logger.info(f"✓ Version format valid: {self.version}")
            return True
        except ValueError as e:
            logger.error(f"✗ Invalid version format: {e}")
            return False

    def check_version_consistency(self) -> bool:
        """Check if current versions are consistent.
        
        Returns:
            True if all versions match, False otherwise.
        """
        is_consistent, versions = self.version_reader.verify_version_consistency()
        
        if is_consistent:
            logger.info("✓ All version files are consistent")
            return True
        else:
            logger.warning("⚠ Version files are inconsistent")
            for key, version in versions.items():
                logger.warning(f"  {key}: {version}")
            return True  # Continue anyway, we'll fix it

    def update_version(self) -> bool:
        """Update version in all files.
        
        Returns:
            True if all files updated successfully, False otherwise.
        """
        logger.info(f"Updating version to {self.version}...")
        
        results = self.file_updater.update_all_files(self.version)
        summary = self.file_updater.get_updated_files_summary(results)
        
        logger.info(f"Updated {summary['updated']}/{summary['total']} files")
        
        for file_name in summary['updated_files']:
            logger.info(f"  ✓ {file_name}")
        
        if summary['failed_files']:
            logger.error(f"Failed to update {summary['failed']} files:")
            for file_name in summary['failed_files']:
                logger.error(f"  ✗ {file_name}")
            return False
        
        return True

    def run_tests(self) -> bool:
        """Run pytest.
        
        Returns:
            True if tests pass, False otherwise.
        """
        logger.info("Running tests...")
        try:
            subprocess.run(['pytest'], check=True, cwd=self.project_root)
            logger.info("✓ Tests passed")
            return True
        except subprocess.CalledProcessError as e:
            logger.error(f"✗ Tests failed: {e}")
            return False
        except FileNotFoundError:
            logger.warning("⚠ pytest not found, skipping tests")
            return True

    def build_package(self) -> bool:
        """Build the package.
        
        Returns:
            True if build succeeds, False otherwise.
        """
        logger.info("Building package...")
        try:
            subprocess.run(
                [sys.executable, '-m', 'build'],
                check=True,
                cwd=self.project_root
            )
            logger.info("✓ Package built successfully")
            return True
        except subprocess.CalledProcessError as e:
            logger.error(f"✗ Build failed: {e}")
            return False
        except FileNotFoundError:
            logger.error("✗ build module not found. Install with: pip install build")
            return False

    def create_git_tag(self) -> bool:
        """Create a git tag for the release.
        
        Returns:
            True if tag created successfully, False otherwise.
        """
        tag_name = f"v{self.version}"
        logger.info(f"Creating git tag: {tag_name}")
        
        try:
            subprocess.run(
                ['git', 'tag', tag_name],
                check=True,
                cwd=self.project_root
            )
            logger.info(f"✓ Git tag created: {tag_name}")
            return True
        except subprocess.CalledProcessError as e:
            logger.error(f"✗ Failed to create git tag: {e}")
            return False
        except FileNotFoundError:
            logger.error("✗ git not found")
            return False

    def print_next_steps(self) -> None:
        """Print next steps for publishing the release."""
        print("\n" + "="*60)
        print(f"✓ Release v{self.version} prepared!")
        print("="*60)
        print("\nNext steps:")
        print(f"  1. Review changes:")
        print(f"     git log --oneline -5")
        print(f"\n  2. Push tag to GitHub:")
        print(f"     git push origin v{self.version}")
        print(f"\n  3. Upload to PyPI:")
        print(f"     twine upload dist/*")
        print(f"\n  4. Create GitHub release:")
        print(f"     https://github.com/Zanabuni/git-multi-account-manager/releases/new?tag=v{self.version}")
        print("="*60 + "\n")

    def create_release(self) -> bool:
        """Execute the full release process.
        
        Returns:
            True if release succeeds, False otherwise.
        """
        logger.info(f"Starting release process for v{self.version}...")
        
        # Step 1: Validate version
        if not self.validate_version():
            return False
        
        # Step 2: Check consistency
        self.check_version_consistency()
        
        # Step 3: Update version
        if not self.update_version():
            logger.error("Failed to update version")
            return False
        
        # Step 4: Run tests
        if not self.run_tests():
            logger.error("Tests failed, aborting release")
            return False
        
        # Step 5: Build package
        if not self.build_package():
            logger.error("Build failed, aborting release")
            return False
        
        # Step 6: Create git tag
        if not self.create_git_tag():
            logger.error("Failed to create git tag")
            return False
        
        # Step 7: Print next steps
        self.print_next_steps()
        
        return True


def main():
    """Main entry point."""
    if len(sys.argv) < 2:
        print("Usage: python release.py <version>")
        print("Example: python release.py 1.1.0")
        sys.exit(1)
    
    version = sys.argv[1]
    
    try:
        manager = ReleaseManager(version)
        success = manager.create_release()
        sys.exit(0 if success else 1)
    except Exception as e:
        logger.error(f"Unexpected error: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()