"""Enhanced build script for creating executables and distribution packages."""

import os
import sys
import subprocess
import shutil
import platform
import argparse
from pathlib import Path
from typing import Optional, List, Dict
import json

# Add the project root to the Python path
project_root = Path(__file__).parent.parent
if str(project_root) not in sys.path:
    sys.path.insert(0, str(project_root))

# Build script version - automatically updated by version manager
BUILD_SCRIPT_VERSION = "1.0.0"


class Colors:
    """ANSI color codes for terminal output."""
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    CYAN = '\033[96m'
    GRAY = '\033[90m'
    BOLD = '\033[1m'
    RESET = '\033[0m'
    
    @staticmethod
    def supports_color():
        """Check if terminal supports colors."""
        return sys.platform != 'win32' or 'ANSICON' in os.environ


class BuildConfig:
    """Build configuration manager."""
    
    def __init__(self):
        self.project_root = project_root
        self.version = self._read_version()
        self.platform = self._detect_platform()
        self.arch = self._detect_arch()
        
    def _read_version(self) -> str:
        """Read version from VERSION file."""
        version_file = self.project_root / 'VERSION'
        if version_file.exists():
            return version_file.read_text().strip()
        print(f"{Colors.YELLOW}⚠️  VERSION file not found, using default: 2.0.0{Colors.RESET}")
        return "2.0.0"
    
    def _detect_platform(self) -> str:
        """Detect current platform."""
        system = platform.system().lower()
        if system == 'darwin':
            return 'macos'
        return system
    
    def _detect_arch(self) -> str:
        """Detect current architecture."""
        machine = platform.machine().lower()
        if machine in ['x86_64', 'amd64']:
            return 'amd64'
        elif machine in ['aarch64', 'arm64']:
            return 'arm64'
        return machine


class Builder:
    """Main builder class."""
    
    def __init__(self, config: BuildConfig):
        self.config = config
        self.color = Colors.supports_color()
        
    def _print(self, emoji: str, message: str, color: str = Colors.RESET):
        """Print colored output."""
        if self.color:
            print(f"{color}{emoji} {message}{Colors.RESET}")
        else:
            print(f"{emoji} {message}")
    
    def _run_command(self, cmd: List[str], cwd: Optional[Path] = None, 
                     env: Optional[Dict] = None) -> subprocess.CompletedProcess:
        """Run a command with error handling."""
        try:
            result = subprocess.run(
                cmd,
                cwd=cwd or self.config.project_root,
                env={**os.environ, **(env or {})},
                check=True,
                capture_output=True,
                text=True
            )
            return result
        except subprocess.CalledProcessError as e:
            self._print("❌", f"Command failed: {' '.join(cmd)}", Colors.RED)
            self._print("", f"Error: {e.stderr}", Colors.RED)
            raise
    
    def cleanup(self):
        """Clean up build artifacts."""
        self._print("🧹", "Cleaning up build artifacts...", Colors.YELLOW)
        
        dirs_to_clean = [
            self.config.project_root / 'build',
            self.config.project_root / 'dist',
            self.config.project_root / 'deb',
            self.config.project_root / 'deb-arm64',
            self.config.project_root / '__pycache__',
        ]
        
        files_to_clean = [
            self.config.project_root / 'git-manager.spec',
            self.config.project_root / '*.exe',
            self.config.project_root / '*.app',
            self.config.project_root / '*.bin',
        ]
        
        for dir_path in dirs_to_clean:
            if dir_path.exists():
                shutil.rmtree(dir_path)
                
        for pattern in files_to_clean:
            for file_path in self.config.project_root.glob(str(pattern.name)):
                file_path.unlink()
        
        self._print("✅", "Cleanup completed", Colors.GREEN)
    
    def install_pyinstaller(self):
        """Install PyInstaller if not available."""
        try:
            import PyInstaller
            self._print("✓", "PyInstaller already installed", Colors.GRAY)
        except ImportError:
            self._print("📦", "Installing PyInstaller...", Colors.YELLOW)
            self._run_command([sys.executable, '-m', 'pip', 'install', 'pyinstaller'])
            self._print("✅", "PyInstaller installed", Colors.GREEN)
    
    def build_pyinstaller(self, target_platform: str = None, entry_point: str = None):
        """Build executable using PyInstaller."""
        platform_name = target_platform or self.config.platform
        
        self._print("🔨", f"Building for {platform_name}...", Colors.CYAN)
        
        # Determine entry point
        if entry_point:
            entry_file = entry_point
        elif '--cli' in sys.argv:
            entry_file = 'scripts/run_cli.py'
        elif '--desktop' in sys.argv:
            entry_file = 'src/git_manager/desktop/app.py'
        elif '--web' in sys.argv:
            entry_file = 'src/git_manager/web/app.py'
        else:
            entry_file = 'src/git_manager/__main__.py'
        
        # Determine output name
        if platform_name == 'windows':
            output_name = 'git-manager.exe'
            windowed = False  # Use console on Windows
        elif platform_name == 'macos':
            output_name = 'git-manager'
            windowed = False
        else:
            output_name = 'git-manager'
            windowed = False
        
        # Build PyInstaller command
        cmd = [
            'pyinstaller',
            '--name', output_name.replace('.exe', ''),
            '--onefile',
            '--console' if not windowed else '--windowed',
            '--clean',
        ]
        
        # Add icon if available
        icon_path = self.config.project_root / 'src/git_manager/desktop/resources/icons/app.ico'
        if icon_path.exists():
            cmd.extend(['--icon', str(icon_path)])
        
        # Add paths and hidden imports
        cmd.extend([
            '--add-data', f'src/git_manager{os.pathsep}git_manager',
            '--paths', str(self.config.project_root / 'src'),
            '--hidden-import', 'git_manager',
            '--hidden-import', 'git_manager.desktop',
            '--hidden-import', 'git_manager.web',
            '--hidden-import', 'git_manager.cli',
            '--hidden-import', 'git_manager.core',
            '--hidden-import', 'git_manager.models',
            '--hidden-import', 'git_manager.utils',
        ])
        
        # Add hooks directory if it exists
        hooks_dir = self.config.project_root / 'scripts/hooks'
        if hooks_dir.exists():
            cmd.extend(['--additional-hooks-dir', str(hooks_dir)])
        
        # Add platform-specific options
        if platform_name == 'macos':
            cmd.extend(['--osx-bundle-identifier', 'com.gitmanager.app'])
        
        cmd.append(entry_file)
        
        # Run PyInstaller
        self._run_command(cmd)
        
        self._print("✅", f"Executable built: dist/{output_name}", Colors.GREEN)
        
        return self.config.project_root / 'dist' / output_name
    
    def create_windows_package(self, binary_path: Path):
        """Create Windows distribution package."""
        self._print("📦", "Creating Windows distribution...", Colors.YELLOW)
        
        dist_dir = self.config.project_root / f'dist/git-manager-{self.config.version}-windows-{self.config.arch}'
        dist_dir.mkdir(parents=True, exist_ok=True)
        
        # Copy binary
        shutil.copy2(binary_path, dist_dir / binary_path.name)
        
        # Copy additional files
        for file in ['README.md', 'LICENSE']:
            src = self.config.project_root / file
            if src.exists():
                shutil.copy2(src, dist_dir / file)
        
        # Create install script
        install_script = dist_dir / 'install.bat'
        install_script.write_text('''@echo off
echo Installing Git Manager...
echo.

set "INSTALL_DIR=%ProgramFiles%\\Git Manager"

if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
copy /Y git-manager.exe "%INSTALL_DIR%\\" >nul

echo.
echo Git Manager installed to: %INSTALL_DIR%
echo.
echo You can now run 'git-manager' from: %INSTALL_DIR%\\git-manager.exe
echo.
pause
''')
        
        # Create ZIP archive
        zip_path = self.config.project_root / f'dist/git-manager-{self.config.version}-windows-{self.config.arch}'
        shutil.make_archive(str(zip_path), 'zip', dist_dir.parent, dist_dir.name)
        
        self._print("✅", f"Windows package created: {zip_path.name}.zip", Colors.GREEN)
    
    def create_macos_package(self, binary_path: Path):
        """Create macOS distribution package (.app bundle and DMG)."""
        self._print("📦", "Creating macOS application bundle...", Colors.YELLOW)
        
        app_name = "Git Manager.app"
        app_dir = self.config.project_root / f'dist/{app_name}'
        
        # Create bundle structure
        (app_dir / 'Contents/MacOS').mkdir(parents=True, exist_ok=True)
        (app_dir / 'Contents/Resources').mkdir(parents=True, exist_ok=True)
        
        # Copy binary
        shutil.copy2(binary_path, app_dir / 'Contents/MacOS/git-manager')
        os.chmod(app_dir / 'Contents/MacOS/git-manager', 0o755)
        
        # Create Info.plist
        info_plist = app_dir / 'Contents/Info.plist'
        info_plist.write_text(f'''<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>git-manager</string>
    <key>CFBundleIdentifier</key>
    <string>com.gitmanager.app</string>
    <key>CFBundleName</key>
    <string>Git Manager</string>
    <key>CFBundleVersion</key>
    <string>{self.config.version}</string>
    <key>CFBundleShortVersionString</key>
    <string>{self.config.version}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
''')
        
        self._print("✅", f"App bundle created: dist/{app_name}", Colors.GREEN)
        
        # Create TAR.GZ archive
        tar_dir = self.config.project_root / f'dist/git-manager-{self.config.version}-macos-{self.config.arch}'
        tar_dir.mkdir(parents=True, exist_ok=True)
        
        shutil.copy2(binary_path, tar_dir / 'git-manager')
        
        for file in ['README.md', 'LICENSE']:
            src = self.config.project_root / file
            if src.exists():
                shutil.copy2(src, tar_dir / file)
        
        # Create install script
        install_script = tar_dir / 'install.sh'
        install_script.write_text('''#!/bin/bash
set -e

echo "Installing Git Manager..."

if [ "$EUID" -ne 0 ]; then 
    echo "Please run with sudo: sudo ./install.sh"
    exit 1
fi

cp git-manager /usr/local/bin/
chmod +x /usr/local/bin/git-manager

echo "✅ Git Manager installed successfully!"
echo "Run 'git-manager' to start"
''')
        os.chmod(install_script, 0o755)
        
        tar_path = self.config.project_root / f'dist/git-manager-{self.config.version}-macos-{self.config.arch}'
        shutil.make_archive(str(tar_path), 'gztar', tar_dir.parent, tar_dir.name)
        
        self._print("✅", f"macOS package created: {tar_path.name}.tar.gz", Colors.GREEN)
    
    def create_linux_package(self, binary_path: Path):
        """Create Linux distribution packages (DEB and TAR.GZ)."""
        self._print("📦", "Creating Linux distribution...", Colors.YELLOW)
        
        # Create TAR.GZ
        tar_dir = self.config.project_root / f'dist/git-manager-{self.config.version}-linux-{self.config.arch}'
        tar_dir.mkdir(parents=True, exist_ok=True)
        
        shutil.copy2(binary_path, tar_dir / 'git-manager')
        
        for file in ['README.md', 'LICENSE']:
            src = self.config.project_root / file
            if src.exists():
                shutil.copy2(src, tar_dir / file)
        
        # Create install script
        install_script = tar_dir / 'install.sh'
        install_script.write_text('''#!/bin/bash
set -e

echo "Installing Git Manager..."

sudo cp git-manager /usr/local/bin/
sudo chmod +x /usr/local/bin/git-manager

echo "✅ Git Manager installed successfully!"
echo "Run 'git-manager' to start"
''')
        os.chmod(install_script, 0o755)
        
        tar_path = self.config.project_root / f'dist/git-manager-{self.config.version}-linux-{self.config.arch}'
        shutil.make_archive(str(tar_path), 'gztar', tar_dir.parent, tar_dir.name)
        
        self._print("✅", f"TAR.GZ created: {tar_path.name}.tar.gz", Colors.GREEN)
        
        # Create DEB package
        self._create_deb_package(binary_path)
    
    def _create_deb_package(self, binary_path: Path):
        """Create Debian package."""
        deb_dir = self.config.project_root / 'deb'
        deb_dir.mkdir(exist_ok=True)
        
        # Create directory structure
        (deb_dir / 'DEBIAN').mkdir(exist_ok=True)
        (deb_dir / 'usr/local/bin').mkdir(parents=True, exist_ok=True)
        (deb_dir / 'usr/share/doc/git-manager').mkdir(parents=True, exist_ok=True)
        
        # Copy binary
        shutil.copy2(binary_path, deb_dir / 'usr/local/bin/git-manager')
        os.chmod(deb_dir / 'usr/local/bin/git-manager', 0o755)
        
        # Create control file
        control = deb_dir / 'DEBIAN/control'
        control.write_text(f'''Package: git-manager
Version: {self.config.version}
Section: utils
Priority: optional
Architecture: {self.config.arch}
Maintainer: Git Manager Team <team@gitmanager.dev>
Description: Modern Git repository manager
 Git Manager is a modern tool for managing Git repositories
 with an intuitive interface and powerful features.
''')
        
        # Build DEB
        deb_name = f'git-manager_{self.config.version}_{self.config.arch}.deb'
        
        try:
            self._run_command(['dpkg-deb', '--build', 'deb', deb_name])
            self._print("✅", f"DEB package created: {deb_name}", Colors.GREEN)
        except (subprocess.CalledProcessError, FileNotFoundError):
            self._print("⚠️", "dpkg-deb not found, skipping DEB creation", Colors.YELLOW)
    
    def build(self, package: bool = True):
        """Main build process."""
        self._print("🚀", f"Building Git Manager v{self.config.version}", Colors.BOLD + Colors.CYAN)
        self._print("", f"Platform: {self.config.platform} ({self.config.arch})", Colors.GRAY)
        print()
        
        # Clean up previous builds
        self.cleanup()
        
        # Install dependencies
        self.install_pyinstaller()
        
        # Build executable
        binary_path = self.build_pyinstaller()
        
        # Create distribution packages
        if package:
            if self.config.platform == 'windows':
                self.create_windows_package(binary_path)
            elif self.config.platform == 'macos':
                self.create_macos_package(binary_path)
            elif self.config.platform == 'linux':
                self.create_linux_package(binary_path)
        
        print()
        self._print("🎉", "Build completed successfully!", Colors.BOLD + Colors.GREEN)
        print()
        self._print("📦", "Distribution files:", Colors.CYAN)
        
        # List created files
        dist_dir = self.config.project_root / 'dist'
        if dist_dir.exists():
            for item in sorted(dist_dir.iterdir()):
                size = ''
                if item.is_file():
                    size_mb = item.stat().st_size / (1024 * 1024)
                    size = f" ({size_mb:.1f} MB)"
                self._print("  •", f"{item.name}{size}", Colors.GRAY)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description='Build Git Manager executables and packages')
    parser.add_argument('--clean', action='store_true', help='Clean build artifacts only')
    parser.add_argument('--no-package', action='store_true', help='Build binary only, skip packaging')
    parser.add_argument('--cli', action='store_true', help='Build CLI version')
    parser.add_argument('--desktop', action='store_true', help='Build desktop version')
    parser.add_argument('--web', action='store_true', help='Build web version')
    
    args = parser.parse_args()
    
    config = BuildConfig()
    builder = Builder(config)
    
    try:
        if args.clean:
            builder.cleanup()
        else:
            builder.build(package=not args.no_package)
    except Exception as e:
        builder._print("❌", f"Build failed: {e}", Colors.RED)
        sys.exit(1)


if __name__ == '__main__':
    main()