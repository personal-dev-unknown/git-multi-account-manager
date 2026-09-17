# src/git_manager/core/config_manager.py
"""Configuration manager."""

from typing import Dict, Any, Optional
from pathlib import Path
import json

from ..utils.logger import get_logger
from ..utils.file_operations import read_json, write_json
from ..utils.constants import CONFIG_FILE


logger = get_logger(__name__)


class ConfigManager:
    """Manages application configuration."""
    
    def __init__(self, config_path: Optional[Path] = None):
        """Initialize Config Manager.
        
        Args:
            config_path: Path to configuration file
        """
        self.config_path = config_path or CONFIG_FILE
        self.config: Dict[str, Any] = {}
        self._load_config()
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get configuration value.
        
        Args:
            key: Configuration key (supports dot notation)
            default: Default value if key not found
            
        Returns:
            Configuration value
        """
        keys = key.split('.')
        value = self.config
        
        for k in keys:
            if isinstance(value, dict):
                value = value.get(k)
                if value is None:
                    return default
            else:
                return default
        
        return value
    
    def set(self, key: str, value: Any) -> None:
        """Set configuration value.
        
        Args:
            key: Configuration key (supports dot notation)
            value: Value to set
        """
        keys = key.split('.')
        config = self.config
        
        for k in keys[:-1]:
            if k not in config:
                config[k] = {}
            config = config[k]
        
        config[keys[-1]] = value
        self._save_config()
    
    def update(self, updates: Dict[str, Any]) -> None:
        """Update multiple configuration values.
        
        Args:
            updates: Dictionary of updates
        """
        self.config.update(updates)
        self._save_config()
    
    def delete(self, key: str) -> None:
        """Delete configuration key.
        
        Args:
            key: Configuration key
        """
        keys = key.split('.')
        config = self.config
        
        for k in keys[:-1]:
            if k not in config:
                return
            config = config[k]
        
        if keys[-1] in config:
            del config[keys[-1]]
            self._save_config()
    
    def reset_to_defaults(self) -> None:
        """Reset configuration to defaults."""
        self.config = self._get_default_config()
        self._save_config()
    
    def export_config(self, path: Path) -> None:
        """Export configuration to file.
        
        Args:
            path: Export file path
        """
        write_json(path, self.config)
        logger.info(f"Configuration exported to {path}")
    
    def import_config(self, path: Path) -> None:
        """Import configuration from file.
        
        Args:
            path: Import file path
        """
        imported = read_json(path)
        self.config.update(imported)
        self._save_config()
        logger.info(f"Configuration imported from {path}")
    
    def _load_config(self) -> None:
        """Load configuration from file."""
        if not self.config_path.exists():
            self.config = self._get_default_config()
            self._save_config()
            return
        
        try:
            self.config = read_json(self.config_path)
            logger.debug(f"Configuration loaded from {self.config_path}")
        except Exception as e:
            logger.error(f"Failed to load configuration: {e}")
            self.config = self._get_default_config()
    
    def _save_config(self) -> None:
        """Save configuration to file."""
        try:
            write_json(self.config_path, self.config)
            logger.debug(f"Configuration saved to {self.config_path}")
        except Exception as e:
            logger.error(f"Failed to save configuration: {e}")
    
    def _get_default_config(self) -> Dict[str, Any]:
        """Get default configuration.
        
        Returns:
            Default configuration dictionary
        """
        return {
            "version": "1.0.0",
            "logging": {
                "level": "INFO",
                "file": str(self.config_path.parent / "git-manager.log")
            },
            "ssh": {
                "directory": "~/.ssh",
                "config_file": "~/.ssh/config",
                "key_type": "ed25519",
                "auto_add_to_agent": True
            },
            "git": {
                "default_branch": "main",
                "default_remote": "origin",
                "auto_fetch": False,
                "auto_pull": False
            },
            "ui": {
                "theme": "dark",
                "show_progress": True
            }
        }