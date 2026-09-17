# scripts/hooks/hook-git_manager.py
# PyInstaller hook for git_manager package

from PyInstaller.utils.hooks import collect_submodules, collect_data_files

# Collect all submodules
hiddenimports = collect_submodules('git_manager')

# Collect data files
datas = collect_data_files('git_manager')

# Add any additional hidden imports that might be needed
hiddenimports.extend([
    'PyQt6',
    'PyQt6.QtCore',
    'PyQt6.QtGui',
    'PyQt6.QtWidgets',
    'flask',
    'flask_socketio',
    'pygit2',
    'cryptography',
    'paramiko',
    'click',
    'rich',
    'pyyaml',
    'toml',
    'requests',
    'urllib3',
    'packaging',
    'typing_extensions',
])
