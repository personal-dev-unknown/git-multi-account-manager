# src/git_manager/utils/config_paths.py
"""Configuration path management following XDG Base Directory specification."""

from pathlib import Path
import os
import platform as sys_platform


def get_config_dir() -> Path:
    """Get configuration directory following XDG Base Directory spec.
    
    Returns:
        Path to configuration directory
    """
    system = sys_platform.system()
    
    if system == "Linux" or system == "Unix":
        # XDG Base Directory Specification
        xdg_config = os.environ.get('XDG_CONFIG_HOME')
        if xdg_config:
            config_dir = Path(xdg_config) / 'git-manager'
        else:
            config_dir = Path.home() / '.config' / 'git-manager'
    elif system == "Darwin":  # macOS
        config_dir = Path.home() / 'Library' / 'Application Support' / 'git-manager'
    elif system == "Windows":
        appdata = os.environ.get('APPDATA')
        if appdata:
            config_dir = Path(appdata) / 'git-manager'
        else:
            config_dir = Path.home() / 'AppData' / 'Roaming' / 'git-manager'
    else:
        # Fallback for unknown systems
        config_dir = Path.home() / '.git-manager'
    
    return config_dir


def get_cache_dir() -> Path:
    """Get cache directory following XDG Base Directory spec.
    
    Returns:
        Path to cache directory
    """
    system = sys_platform.system()
    
    if system == "Linux" or system == "Unix":
        xdg_cache = os.environ.get('XDG_CACHE_HOME')
        if xdg_cache:
            cache_dir = Path(xdg_cache) / 'git-manager'
        else:
            cache_dir = Path.home() / '.cache' / 'git-manager'
    elif system == "Darwin":  # macOS
        cache_dir = Path.home() / 'Library' / 'Caches' / 'git-manager'
    elif system == "Windows":
        localappdata = os.environ.get('LOCALAPPDATA')
        if localappdata:
            cache_dir = Path(localappdata) / 'git-manager' / 'cache'
        else:
            cache_dir = Path.home() / 'AppData' / 'Local' / 'git-manager' / 'cache'
    else:
        # Fallback for unknown systems
        cache_dir = Path.home() / '.git-manager' / 'cache'
    
    return cache_dir


def get_accounts_file() -> Path:
    """Get path to accounts configuration file.
    
    Returns:
        Path to accounts.json
    """
    return get_config_dir() / 'accounts.json'


def get_config_file() -> Path:
    """Get path to main configuration file.
    
    Returns:
        Path to config.yaml
    """
    return get_config_dir() / 'config.yaml'


def get_ssh_keys_dir() -> Path:
    """Get SSH keys directory for gitmanager.
    
    Returns:
        Path to ~/.ssh/gitmanager/
    """
    ssh_dir = Path.home() / '.ssh' / 'gitmanager'
    return ssh_dir


def get_ssh_config_file() -> Path:
    """Get SSH config file for gitmanager.
    
    Returns:
        Path to ~/.ssh/gitmanager/config
    """
    return get_ssh_keys_dir() / 'config'


def ensure_config_dirs() -> None:
    """Ensure all configuration directories exist."""
    get_config_dir().mkdir(parents=True, exist_ok=True)
    get_cache_dir().mkdir(parents=True, exist_ok=True)
    
    # Ensure SSH keys directory with proper permissions
    ssh_keys_dir = get_ssh_keys_dir()
    ssh_keys_dir.mkdir(parents=True, exist_ok=True)
    
    # Set proper permissions (700 = rwx------)
    import stat
    ssh_keys_dir.chmod(stat.S_IRWXU)
