# src/git_manager/desktop/windows/theme_window.py
"""Theme management widget for desktop application."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, 
    QGridLayout, QScrollArea, QMessageBox, QGroupBox
)
from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QColor, QFont

from ...cli.ui.theme_manager import ThemeManager
from ...cli.ui.color_schemes import ThemeType


class ThemeWidget(QWidget):
    """Widget for managing application themes."""
    
    theme_changed = pyqtSignal(str)
    
    def __init__(self, theme_manager: ThemeManager):
        super().__init__()
        self.theme_manager = theme_manager
        self.current_theme = theme_manager.get_current_theme()
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI components."""
        layout = QVBoxLayout(self)
        
        # Current theme info
        current_group = QGroupBox("Current Theme")
        current_layout = QVBoxLayout()
        
        info_layout = QHBoxLayout()
        info_layout.addWidget(QLabel("Theme:"))
        theme_name_label = QLabel(self.current_theme.name)
        theme_name_label.setFont(QFont("Arial", 12, QFont.Weight.Bold))
        info_layout.addWidget(theme_name_label)
        info_layout.addWidget(QLabel(f"({self.current_theme.theme_type.value})"))
        info_layout.addStretch()
        
        current_layout.addLayout(info_layout)
        
        # Color preview
        color_layout = QHBoxLayout()
        color_layout.addWidget(QLabel("Colors:"))
        
        colors = [
            ("Primary", self.current_theme.primary),
            ("Accent", self.current_theme.accent),
            ("Success", self.current_theme.success),
            ("Error", self.current_theme.error),
        ]
        
        for color_name, color_value in colors:
            color_box = self.create_color_box(color_name, color_value)
            color_layout.addWidget(color_box)
        
        color_layout.addStretch()
        current_layout.addLayout(color_layout)
        current_group.setLayout(current_layout)
        layout.addWidget(current_group)
        
        # Available themes
        themes_group = QGroupBox("Available Themes")
        themes_layout = QVBoxLayout()
        
        # Scroll area for themes
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll_widget = QWidget()
        scroll_layout = QVBoxLayout(scroll_widget)
        
        # Get themes by type
        themes_by_type = self.theme_manager.list_available_themes()
        
        for theme_type in ['light', 'dark', 'colored']:
            if theme_type in themes_by_type:
                type_label = QLabel(f"{theme_type.upper()} THEMES")
                type_label.setFont(QFont("Arial", 10, QFont.Weight.Bold))
                scroll_layout.addWidget(type_label)
                
                # Create grid for themes
                grid = QGridLayout()
                themes = themes_by_type[theme_type]
                
                for idx, theme_name in enumerate(sorted(themes)):
                    row = idx // 3
                    col = idx % 3
                    
                    btn = QPushButton(self.format_theme_name(theme_name))
                    btn.setMinimumHeight(50)
                    
                    # Highlight current theme
                    if theme_name == self.current_theme.name.lower().replace(' ', '_'):
                        btn.setStyleSheet("""
                            QPushButton {
                                background-color: #667eea;
                                color: white;
                                font-weight: bold;
                                border: 2px solid #667eea;
                                border-radius: 4px;
                            }
                            QPushButton:hover {
                                background-color: #5568d3;
                            }
                        """)
                    else:
                        btn.setStyleSheet("""
                            QPushButton {
                                background-color: #f0f0f0;
                                border: 1px solid #ddd;
                                border-radius: 4px;
                            }
                            QPushButton:hover {
                                background-color: #e0e0e0;
                                border: 1px solid #667eea;
                            }
                        """)
                    
                    btn.clicked.connect(lambda checked, name=theme_name: self.set_theme(name))
                    grid.addWidget(btn, row, col)
                
                scroll_layout.addLayout(grid)
                scroll_layout.addSpacing(10)
        
        scroll_layout.addStretch()
        scroll.setWidget(scroll_widget)
        themes_layout.addWidget(scroll)
        themes_group.setLayout(themes_layout)
        layout.addWidget(themes_group)
    
    def create_color_box(self, label: str, color_name: str) -> QWidget:
        """Create a color preview box."""
        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(5, 5, 5, 5)
        
        # Color swatch
        swatch = QWidget()
        swatch.setMinimumSize(40, 40)
        swatch.setMaximumSize(40, 40)
        swatch.setStyleSheet(f"background-color: {self.color_to_hex(color_name)}; border: 1px solid #ccc; border-radius: 4px;")
        
        layout.addWidget(swatch, alignment=Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(QLabel(label), alignment=Qt.AlignmentFlag.AlignCenter)
        
        return container
    
    def color_to_hex(self, color_name: str) -> str:
        """Convert Rich color name to hex."""
        color_map = {
            'black': '#000000',
            'white': '#FFFFFF',
            'bright_white': '#FFFFFF',
            'bright_black': '#808080',
            'red': '#CD3131',
            'bright_red': '#FF5555',
            'green': '#13A10E',
            'bright_green': '#55FF55',
            'yellow': '#E5E510',
            'bright_yellow': '#FFFF55',
            'blue': '#2472C8',
            'bright_blue': '#5555FF',
            'magenta': '#BC3F3C',
            'bright_magenta': '#FF55FF',
            'cyan': '#11A8CD',
            'bright_cyan': '#55FFFF',
            'grey0': '#000000',
            'grey100': '#FFFFFF',
        }
        
        if color_name.startswith('rgb('):
            try:
                rgb_str = color_name[4:-1]
                r, g, b = map(int, rgb_str.split(','))
                return f'#{r:02x}{g:02x}{b:02x}'
            except:
                return '#CCCCCC'
        
        return color_map.get(color_name, '#CCCCCC')
    
    def format_theme_name(self, theme_name: str) -> str:
        """Format theme name for display."""
        return ' '.join(word.capitalize() for word in theme_name.split('_'))
    
    def set_theme(self, theme_name: str):
        """Set the application theme."""
        if self.theme_manager.set_theme(theme_name):
            self.current_theme = self.theme_manager.get_current_theme()
            self.theme_changed.emit(theme_name)
            QMessageBox.information(
                self, 
                "Theme Changed", 
                f"Theme changed to {self.format_theme_name(theme_name)}.\n\nPlease restart the application for full effect."
            )
            # Refresh UI
            self.init_ui()
        else:
            QMessageBox.warning(
                self, 
                "Error", 
                f"Failed to change theme to {theme_name}"
            )
