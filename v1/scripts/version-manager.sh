#!/usr/bin/env python3
"""
Version Manager for Git Multi-Account Manager

This script manages version updates across the entire project.
It updates version strings in setup.py, pyproject.toml, constants.py, and other files.

Usage:
    ./version-manager.sh get                    # Show current version
    ./version-manager.sh set 2.1.0              # Set version to 2.1.0
    ./version-manager.sh bump-minor             # Increment minor version
    ./version-manager.sh validate 2.1.0         # Check if format is valid
"""

import argparse
import sys
from pathlib import Path

# Add src directory to path to import git_manager modules
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root / 'src'))

from git_manager.core.version import (
    VersionReader,
    VersionComparator,
    FileUpdater,
)


# ANSI color codes
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color


class VersionManager:
    """Manages version updates across the project."""

    def __init__(self, project_root: Path = None):
        """Initialize the version manager."""
        if project_root is None:
            project_root = Path(__file__).parent.parent
        
        self.project_root = project_root
        self.version_reader = VersionReader(project_root)
        self.file_updater = FileUpdater(project_root)

    def get_version(self) -> str:
        """Get current version from VERSION file."""
        return self.version_reader.read_version_file()

    def set_version(self, new_version: str) -> None:
        """Set new version across all files."""
        if not self._validate_version(new_version):
            raise ValueError(f"Invalid version format: {new_version}")
        
        current_version = self.get_version()
        
        print(f"{Colors.YELLOW}Updating version: {current_version} → {new_version}{Colors.NC}")
        
        # Update all files
        results = self.file_updater.update_all_files(new_version)
        summary = self.file_updater.get_updated_files_summary(results)
        
        print(f"\n{Colors.GREEN}🎉 Version updated to {new_version}{Colors.NC}")
        print(f"\nFiles updated: {summary['updated']}/{summary['total']}")
        for f in summary['updated_files']:
            print(f"  {Colors.GREEN}✅{Colors.NC} {f}")
        
        if summary['failed_files']:
            print(f"\nFailed to update: {summary['failed']}")
            for f in summary['failed_files']:
                print(f"  {Colors.RED}❌{Colors.NC} {f}")

    @staticmethod
    def _validate_version(version: str) -> bool:
        """Validate version format (MAJOR.MINOR.PATCH)."""
        try:
            VersionComparator.parse_version(version)
            return True
        except ValueError:
            print(f"{Colors.RED}❌ Invalid version format: {version}{Colors.NC}")
            print("Expected format: MAJOR.MINOR.PATCH (e.g., 2.1.0)")
            return False

    @staticmethod
    def _parse_version(version: str) -> tuple:
        """Parse version string into components."""
        return VersionComparator.parse_version(version)

    def bump_patch(self) -> None:
        """Bump patch version (e.g., 1.0.0 → 1.0.1)."""
        current = self.get_version()
        major, minor, patch = self._parse_version(current)
        patch += 1
        new_version = f"{major}.{minor}.{patch}"
        self.set_version(new_version)

    def bump_minor(self) -> None:
        """Bump minor version (e.g., 1.0.0 → 1.1.0)."""
        current = self.get_version()
        major, minor, patch = self._parse_version(current)
        minor += 1
        new_version = f"{major}.{minor}.0"
        self.set_version(new_version)

    def bump_major(self) -> None:
        """Bump major version (e.g., 1.0.0 → 2.0.0)."""
        current = self.get_version()
        major, minor, patch = self._parse_version(current)
        major += 1
        new_version = f"{major}.0.0"
        self.set_version(new_version)

    def list_versions(self) -> None:
        """List all version references in the project."""
        print(f"{Colors.BLUE}Version files in the project:{Colors.NC}\n")
        
        versions = self.version_reader.get_all_versions()
        for source, version in versions.items():
            status = f"{Colors.GREEN}✓{Colors.NC}" if version else f"{Colors.YELLOW}✗{Colors.NC}"
            print(f"{status} {source}: {version or 'Not found'}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='Git Multi-Account Manager Version Manager',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s get                    # Show current version
  %(prog)s set 2.1.0              # Set version to 2.1.0
  %(prog)s bump-minor             # Increment minor version
  %(prog)s validate 2.1.0         # Check if format is valid
        """
    )
    
    subparsers = parser.add_subparsers(dest='command', help='Command to execute')
    
    # Get command
    subparsers.add_parser('get', help='Get current version')
    
    # Set command
    set_parser = subparsers.add_parser('set', help='Set new version')
    set_parser.add_argument('version', help='New version (MAJOR.MINOR.PATCH)')
    
    # Bump commands
    subparsers.add_parser('bump-patch', help='Bump patch version')
    subparsers.add_parser('bump-minor', help='Bump minor version')
    subparsers.add_parser('bump-major', help='Bump major version')
    
    # List command
    subparsers.add_parser('list', help='List all version files')
    
    # Validate command
    validate_parser = subparsers.add_parser('validate', help='Validate version format')
    validate_parser.add_argument('version', help='Version to validate')
    
    args = parser.parse_args()
    
    try:
        manager = VersionManager()
        
        if args.command == 'get':
            print(manager.get_version())
        elif args.command == 'set':
            manager.set_version(args.version)
        elif args.command == 'bump-patch':
            manager.bump_patch()
        elif args.command == 'bump-minor':
            manager.bump_minor()
        elif args.command == 'bump-major':
            manager.bump_major()
        elif args.command == 'list':
            manager.list_versions()
        elif args.command == 'validate':
            if manager._validate_version(args.version):
                print(f"{Colors.GREEN}✅ Version format is valid: {args.version}{Colors.NC}")
        else:
            parser.print_help()
            sys.exit(1)
    
    except Exception as e:
        print(f"{Colors.RED}❌ Error: {e}{Colors.NC}")
        sys.exit(1)


if __name__ == '__main__':
    main()
