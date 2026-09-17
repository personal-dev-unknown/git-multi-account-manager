# src/git_manager/desktop/app.py
"""Desktop GUI application using PyQt6."""

import sys
from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QTabWidget, QPushButton, QLabel, QMessageBox
)
from PyQt6.QtCore import Qt
from PyQt6.QtGui import QIcon, QPalette, QColor

from ..core.account_manager import AccountManager
from ..core.ssh import SSHWorkflowOrchestrator
from ..core.git_operations import GitOperations
from ..utils.constants import APP_NAME, APP_VERSION, COLORS
from ..utils.log_config import initialize_logging, LogLevel, LogCategory, get_logger as get_advanced_logger
from ..cli.ui.theme_manager import ThemeManager


class MainWindow(QMainWindow):
    """Main application window."""
    
    def __init__(self):
        super().__init__()
        
        # Initialize logging
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
        self.logger.info(f"Starting {APP_NAME} v{APP_VERSION} (Desktop)")
        
        # Initialize theme manager and get current theme
        self.theme_manager = ThemeManager()
        self.theme = self.theme_manager.get_current_theme()
        self.logger.info(f"Using theme: {self.theme.name}")
        
        # Initialize managers
        self.account_manager = AccountManager()
        self.ssh_orchestrator = SSHWorkflowOrchestrator()
        self.git_operations = GitOperations(self.account_manager)
        
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI components."""
        self.setWindowTitle(f"{APP_NAME} v{APP_VERSION}")
        self.setGeometry(100, 100, 1000, 700)
        
        # Set application style
        self.set_application_style()
        
        # Central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        
        # Main layout
        layout = QVBoxLayout(central_widget)
        
        # Header
        header = self.create_header()
        layout.addWidget(header)
        
        # Tab widget
        tabs = QTabWidget()
        
        # Add tabs
        from .windows.account_window import AccountWidget

        # Add tabs - HYBRID APPROACH: Simple + Advanced
        from .windows.repository_window import RepositoryWidget
        from .windows.ssh_window import SSHWidget
        from .windows.theme_window import ThemeWidget
        

        # from .widgets.interactive_clone_widget import InteractiveCloneWidget
        # from .widgets.interactive_git_widget import InteractiveGitWidget
        # from .widgets.interactive_accounts_widget import InteractiveAccountsWidget

        from .widgets.clone_widget import CloneWidget
        from .widgets.git_operations_widget import GitOperationsWidget
        
        tabs.addTab(CloneWidget(), "Clone")
        tabs.addTab(GitOperationsWidget(), "Git Operations")

        # Simple/Interactive tabs (for general users)
        # tabs.addTab(InteractiveCloneWidget(), "Clone")
        # tabs.addTab(InteractiveGitWidget(), "Git Operations")
        # tabs.addTab(InteractiveAccountsWidget(), "Accounts")
        
        # # Advanced tabs (for power users)
        # tabs.addTab(CloneWidget(), "Clone Advanced")
        # tabs.addTab(GitOperationsWidget(), "Git Advanced")
        
        # # Specialized tabs

        tabs.addTab(AccountWidget(self.account_manager), "Accounts")
        tabs.addTab(RepositoryWidget(self.git_operations), "Repositories")
        tabs.addTab(SSHWidget(self.ssh_orchestrator), "SSH Keys")
        tabs.addTab(ThemeWidget(self.theme_manager), "Theme")
        
        layout.addWidget(tabs)
        
        # Status bar
        self.statusBar().showMessage('Ready')
    
    def create_header(self):
        """Create header widget."""
        header = QWidget()
        header.setStyleSheet(f"background-color: {COLORS['primary']}; padding: 10px;")
        
        layout = QHBoxLayout(header)
        
        title = QLabel(f"{APP_NAME} v{APP_VERSION}")
        title.setStyleSheet("color: white; font-size: 18px; font-weight: bold;")
        layout.addWidget(title)
        
        layout.addStretch()
        
        return header
    
    def set_application_style(self):
        """Set application color scheme based on theme."""
        palette = QPalette()
        
        # Convert Rich color names to QColor
        # For Rich color names, use sensible defaults
        theme = self.theme
        
        # Map Rich color names to RGB values
        color_map = {
            'black': QColor(0, 0, 0),
            'white': QColor(255, 255, 255),
            'bright_white': QColor(255, 255, 255),
            'bright_black': QColor(128, 128, 128),
            'red': QColor(205, 49, 49),
            'bright_red': QColor(255, 85, 85),
            'green': QColor(19, 161, 14),
            'bright_green': QColor(85, 255, 85),
            'yellow': QColor(229, 229, 16),
            'bright_yellow': QColor(255, 255, 85),
            'blue': QColor(36, 114, 200),
            'bright_blue': QColor(85, 85, 255),
            'magenta': QColor(188, 63, 60),
            'bright_magenta': QColor(255, 85, 255),
            'cyan': QColor(17, 168, 205),
            'bright_cyan': QColor(85, 255, 255),
            'grey0': QColor(0, 0, 0),
            'grey100': QColor(255, 255, 255),
        }
        
        # Get colors from theme, with fallbacks
        def get_color(color_name):
            if color_name in color_map:
                return color_map[color_name]
            # Try to parse RGB format
            if color_name.startswith('rgb('):
                try:
                    rgb_str = color_name[4:-1]
                    r, g, b = map(int, rgb_str.split(','))
                    return QColor(r, g, b)
                except:
                    pass
            # Default fallback
            return QColor(50, 50, 50)
        
        bg_color = get_color(theme.background)
        primary_color = get_color(theme.primary)
        text_color = get_color(theme.text)
        accent_color = get_color(theme.accent)
        
        palette.setColor(QPalette.ColorRole.Window, bg_color)
        palette.setColor(QPalette.ColorRole.WindowText, text_color)
        palette.setColor(QPalette.ColorRole.Base, bg_color)
        palette.setColor(QPalette.ColorRole.AlternateBase, accent_color)
        palette.setColor(QPalette.ColorRole.Button, primary_color)
        palette.setColor(QPalette.ColorRole.ButtonText, text_color)
        palette.setColor(QPalette.ColorRole.Link, accent_color)
        palette.setColor(QPalette.ColorRole.Highlight, accent_color)
        
        self.setPalette(palette)


def run_desktop_app():
    """Run the desktop application."""
    # Initialize logging system for desktop
    initialize_logging(
        log_level=LogLevel.INFO,
        use_json=False,
        enable_console=False,  # Disable console for GUI
    )
    
    logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
    logger.info("Desktop application starting")
    
    app = QApplication(sys.argv)
    app.setApplicationName(APP_NAME)
    
    try:
        window = MainWindow()
        window.show()
        logger.info("Desktop window displayed")
        sys.exit(app.exec())
    except Exception as e:
        logger.error(f"Desktop application error: {str(e)}", exc_info=True)
        raise