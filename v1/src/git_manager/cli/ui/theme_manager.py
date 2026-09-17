# src/git_manager/cli/ui/theme_manager.py
"""Theme manager for handling color scheme preferences."""

import json
from pathlib import Path
from typing import Optional

from ...utils.config_paths import get_config_dir
from .color_schemes import ColorScheme, DEFAULT_SCHEME, get_scheme, ALL_SCHEMES


class ThemeManager:
    """Manages user's theme preferences."""
    
    def __init__(self):
        """Initialize theme manager."""
        self.config_dir = get_config_dir()
        self.theme_file = self.config_dir / 'theme.json'
        self._ensure_theme_file()
    
    def _ensure_theme_file(self) -> None:
        """Ensure theme configuration file exists."""
        if not self.theme_file.exists():
            self.set_theme(DEFAULT_SCHEME.name)
    
    def get_current_theme(self) -> ColorScheme:
        """Get the currently selected theme.
        
        Returns:
            ColorScheme object for the current theme
        """
        try:
            with open(self.theme_file, 'r') as f:
                data = json.load(f)
                theme_name = data.get('theme', DEFAULT_SCHEME.name)
                scheme = get_scheme(theme_name)
                return scheme or DEFAULT_SCHEME
        except (json.JSONDecodeError, FileNotFoundError):
            return DEFAULT_SCHEME
    
    def set_theme(self, theme_name: str) -> bool:
        """Set the user's preferred theme.
        
        Args:
            theme_name: Name of the theme to set
            
        Returns:
            True if successful, False otherwise
        """
        if theme_name not in ALL_SCHEMES:
            return False
        
        try:
            self.config_dir.mkdir(parents=True, exist_ok=True)
            with open(self.theme_file, 'w') as f:
                json.dump({'theme': theme_name}, f, indent=2)
            return True
        except Exception:
            return False
    
    def list_available_themes(self) -> dict:
        """List all available themes organized by type.
        
        Returns:
            Dictionary with theme types as keys and lists of theme names as values
        """
        themes_by_type = {
            'light': [],
            'dark': [],
            'colored': []
        }
        
        for name, scheme in ALL_SCHEMES.items():
            theme_type = scheme.theme_type.value
            themes_by_type[theme_type].append(name)
        
        return themes_by_type
