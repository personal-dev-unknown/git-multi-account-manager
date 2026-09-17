#!/usr/bin/env python3
"""
Version Manager for Git Multi-Account Manager

This script manages version updates across the entire project.
It updates version strings in setup.py, pyproject.toml, constants.py, and other files.

Usage:
    python version_manager.py get                    # Show current version
    python version_manager.py set 2.1.0              # Set version to 2.1.0
    python version_manager.py bump-minor             # Increment minor version
    python version_manager.py validate 2.1.0         # Check if format is valid
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

    def _validate_version(self, version: str) -> bool:
        """Validate version format (MAJOR.MINOR.PATCH).
        
        Args:
            version: Version string to validate.
            
        Returns:
            True if valid, False otherwise.
        """
        pattern = r'^[0-9]+\.[0-9]+\.[0-9]+$'
        if re.match(pattern, version):
            return True
        
        print(f"{Colors.RED}❌ Invalid version format: {version}{Colors.NC}")
        print("Expected format: MAJOR.MINOR.PATCH (e.g., 2.1.0)")
        return False

    def get_version(self) -> str:
        """Get current version from VERSION file.
        
        Returns:
            Current version string.
            
        Raises:
            FileNotFoundError: If VERSION file not found.
        """
        if not self.version_file.exists():
            raise FileNotFoundError(f"VERSION file not found at {self.version_file}")
        
        return self.version_file.read_text().strip()

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
            return False
        
        content = file_path.read_text()
        new_content = re.sub(old_pattern, new_text, content)
        
        if new_content != content:
            file_path.write_text(new_content)
            return True
        
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
        # But we can verify it's correct
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
            return False
        
        content = file_path.read_text()
        new_content = re.sub(
            r'v[0-9]+\.[0-9]+\.[0-9]+',
            f'v{new_version}',
            content
        )
        
        if new_content != content:
            file_path.write_text(new_content)
            return True
        
        return False

    def set_version(self, new_version: str) -> None:
        """Set new version across all files.
        
        Args:
            new_version: New version string (MAJOR.MINOR.PATCH).
            
        Raises:
            ValueError: If version format is invalid.
        """
        if not self._validate_version(new_version):
            raise ValueError(f"Invalid version format: {new_version}")
        
        current_version = self.get_version()
        
        print(f"{Colors.YELLOW}Updating version: {current_version} → {new_version}{Colors.NC}")
        
        # Update VERSION file
        self.version_file.write_text(f"{new_version}\n")
        print(f"{Colors.GREEN}✅ Updated VERSION file{Colors.NC}")
        
        # Update all tracked files
        updated_files = []
        for file_name, update_func in self.files_to_update.items():
            file_path = self.project_root / file_name
            try:
                if update_func(new_version):
                    print(f"{Colors.GREEN}✅ Updated {file_name}{Colors.NC}")
                    updated_files.append(file_name)
                elif file_path.exists():
                    print(f"{Colors.YELLOW}⚠️  {file_name} exists but no version string found{Colors.NC}")
            except Exception as e:
                print(f"{Colors.RED}❌ Error updating {file_name}: {e}{Colors.NC}")
        
        print(f"\n{Colors.GREEN}🎉 Version updated to {new_version}{Colors.NC}")
        print(f"\nFiles updated: {len(updated_files)}")
        for f in updated_files:
            print(f"  - {f}")

    def _parse_version(self, version: str) -> Tuple[int, int, int]:
        """Parse version string into components.
        
        Args:
            version: Version string (MAJOR.MINOR.PATCH).
            
        Returns:
            Tuple of (major, minor, patch).
        """
        parts = version.split('.')
        return int(parts[0]), int(parts[1]), int(parts[2])

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
        
        print(f"{Colors.YELLOW}Core Version:{Colors.NC}")
        try:
            current = self.get_version()
            print(f"  VERSION: {current}\n")
        except FileNotFoundError as e:
            print(f"  {Colors.RED}Error: {e}{Colors.NC}\n")
        
        for file_name in self.files_to_update.keys():
            file_path = self.project_root / file_name
            if file_path.exists():
                print(f"{Colors.YELLOW}{file_name}:{Colors.NC}")
                content = file_path.read_text()
                # Find version-like strings
                version_patterns = [
                    r"version\s*=\s*['\"]([^'\"]+)['\"]",
                    r"APP_VERSION\s*=\s*['\"]([^'\"]+)['\"]",
                ]
                found = False
                for pattern in version_patterns:
                    matches = re.findall(pattern, content)
                    if matches:
                        for match in matches:
                            print(f"  {match}")
                        found = True
                if not found:
                    print(f"  {Colors.YELLOW}(no version string found){Colors.NC}")
                print()


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
