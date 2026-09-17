# Desktop Integration Guide

This guide shows how to integrate the new interactive widgets into the main desktop application.

## Current Desktop Structure

```
src/git_manager/desktop/
├── app.py                          # Main application
├── interactive_manager.py          # NEW: Interactive manager
├── widgets/
│   ├── clone_widget.py            # Existing clone widget
│   ├── git_operations_widget.py   # Existing git operations widget
│   ├── interactive_accounts_widget.py      # NEW: Interactive accounts
│   ├── interactive_clone_widget.py         # NEW: Interactive clone
│   ├── interactive_git_widget.py           # NEW: Interactive git
│   └── ...
└── windows/
    ├── account_window.py
    └── ...
```

## Integration Steps

### 1. Update Main Window (app.py)

Replace the existing tab setup with the new interactive widgets:

```python
# In MainWindow.init_ui() method

from .widgets.interactive_accounts_widget import InteractiveAccountsWidget
from .widgets.interactive_clone_widget import InteractiveCloneWidget
from .widgets.interactive_git_widget import InteractiveGitWidget

# ... existing code ...

# Tab widget
tabs = QTabWidget()

# Add interactive widgets (recommended order)
tabs.addTab(InteractiveCloneWidget(), "Clone Repository")
tabs.addTab(InteractiveGitWidget(), "Git Operations")
tabs.addTab(InteractiveAccountsWidget(), "Accounts")

# Keep existing tabs if needed
# tabs.addTab(RepositoryWidget(self.git_operations), "Repositories")
# tabs.addTab(SSHWidget(self.ssh_orchestrator), "SSH Keys")
# tabs.addTab(ThemeWidget(self.theme_manager), "Theme")

layout.addWidget(tabs)
```

### 2. Update Widget Imports

Ensure all necessary imports are in place:

```python
# In desktop/widgets/__init__.py

from .interactive_accounts_widget import (
    InteractiveAccountsWidget,
    GenerateSSHKeyDialog,
    SetupPATDialog
)
from .interactive_clone_widget import (
    InteractiveCloneWidget,
    CloneWorkerThread,
    RepositoryFetchWorkerThread
)
from .interactive_git_widget import (
    InteractiveGitWidget,
    GitOperationWorkerThread
)

__all__ = [
    'InteractiveAccountsWidget',
    'InteractiveCloneWidget',
    'InteractiveGitWidget',
    'GenerateSSHKeyDialog',
    'SetupPATDialog',
    'CloneWorkerThread',
    'RepositoryFetchWorkerThread',
    'GitOperationWorkerThread',
]
```

### 3. Update Desktop App Initialization

Ensure the interactive manager is properly initialized:

```python
# In MainWindow.__init__()

from .interactive_manager import InteractiveDesktopManager

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        
        # Initialize interactive manager
        self.interactive_manager = InteractiveDesktopManager()
        
        # ... rest of initialization ...
```

### 4. Optional: Add Menu Items

Add menu items for quick access to interactive features:

```python
# In MainWindow.init_ui() or create_menu_bar()

from PyQt6.QtWidgets import QMenu, QAction

# Create Tools menu
tools_menu = self.menuBar().addMenu("Tools")

# Clone action
clone_action = QAction("Clone Repository", self)
clone_action.triggered.connect(lambda: self.switch_to_tab("Clone Repository"))
tools_menu.addAction(clone_action)

# Git operations action
git_action = QAction("Git Operations", self)
git_action.triggered.connect(lambda: self.switch_to_tab("Git Operations"))
tools_menu.addAction(git_action)

# Account management action
account_action = QAction("Manage Accounts", self)
account_action.triggered.connect(lambda: self.switch_to_tab("Accounts"))
tools_menu.addAction(account_action)

def switch_to_tab(self, tab_name):
    """Switch to specified tab."""
    for i in range(self.tabs.count()):
        if self.tabs.tabText(i) == tab_name:
            self.tabs.setCurrentIndex(i)
            break
```

### 5. Optional: Add Keyboard Shortcuts

Add keyboard shortcuts for quick access:

```python
# In MainWindow.init_ui()

from PyQt6.QtGui import QKeySequence

clone_action.setShortcut(QKeySequence("Ctrl+Shift+C"))
git_action.setShortcut(QKeySequence("Ctrl+Shift+G"))
account_action.setShortcut(QKeySequence("Ctrl+Shift+A"))
```

## Complete Example: Updated app.py

Here's a complete example of how to update the main app.py:

```python
# src/git_manager/desktop/app.py
"""Desktop GUI application using PyQt6."""

import sys
from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QTabWidget, QPushButton, QLabel, QMessageBox, QMenu, QAction
)
from PyQt6.QtCore import Qt
from PyQt6.QtGui import QIcon, QPalette, QColor, QKeySequence

from ..core.account_manager import AccountManager
from ..core.ssh import SSHWorkflowOrchestrator
from ..core.git_operations import GitOperations
from ..utils.constants import APP_NAME, APP_VERSION, COLORS
from ..utils.log_config import initialize_logging, LogLevel, LogCategory, get_logger as get_advanced_logger
from ..cli.ui.theme_manager import ThemeManager

# Import interactive widgets
from .widgets.interactive_accounts_widget import InteractiveAccountsWidget
from .widgets.interactive_clone_widget import InteractiveCloneWidget
from .widgets.interactive_git_widget import InteractiveGitWidget
from .interactive_manager import InteractiveDesktopManager


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
        self.interactive_manager = InteractiveDesktopManager()
        
        self.init_ui()
        self.create_menu_bar()
    
    def init_ui(self):
        """Initialize UI components."""
        self.setWindowTitle(f"{APP_NAME} v{APP_VERSION}")
        self.setGeometry(100, 100, 1200, 800)
        
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
        
        # Tab widget with interactive features
        self.tabs = QTabWidget()
        
        # Add interactive widgets
        self.tabs.addTab(InteractiveCloneWidget(), "Clone Repository")
        self.tabs.addTab(InteractiveGitWidget(), "Git Operations")
        self.tabs.addTab(InteractiveAccountsWidget(), "Accounts")
        
        # Optional: Add existing widgets if needed
        # from .windows.repository_window import RepositoryWidget
        # from .windows.ssh_window import SSHWidget
        # from .windows.theme_window import ThemeWidget
        # self.tabs.addTab(RepositoryWidget(self.git_operations), "Repositories")
        # self.tabs.addTab(SSHWidget(self.ssh_orchestrator), "SSH Keys")
        # self.tabs.addTab(ThemeWidget(self.theme_manager), "Theme")
        
        layout.addWidget(self.tabs)
        
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
    
    def create_menu_bar(self):
        """Create menu bar with shortcuts."""
        # File menu
        file_menu = self.menuBar().addMenu("File")
        
        exit_action = QAction("Exit", self)
        exit_action.setShortcut(QKeySequence("Ctrl+Q"))
        exit_action.triggered.connect(self.close)
        file_menu.addAction(exit_action)
        
        # Tools menu
        tools_menu = self.menuBar().addMenu("Tools")
        
        clone_action = QAction("Clone Repository", self)
        clone_action.setShortcut(QKeySequence("Ctrl+Shift+C"))
        clone_action.triggered.connect(lambda: self.switch_to_tab("Clone Repository"))
        tools_menu.addAction(clone_action)
        
        git_action = QAction("Git Operations", self)
        git_action.setShortcut(QKeySequence("Ctrl+Shift+G"))
        git_action.triggered.connect(lambda: self.switch_to_tab("Git Operations"))
        tools_menu.addAction(git_action)
        
        account_action = QAction("Manage Accounts", self)
        account_action.setShortcut(QKeySequence("Ctrl+Shift+A"))
        account_action.triggered.connect(lambda: self.switch_to_tab("Accounts"))
        tools_menu.addAction(account_action)
        
        # Help menu
        help_menu = self.menuBar().addMenu("Help")
        
        about_action = QAction("About", self)
        about_action.triggered.connect(self.show_about)
        help_menu.addAction(about_action)
    
    def switch_to_tab(self, tab_name):
        """Switch to specified tab."""
        for i in range(self.tabs.count()):
            if self.tabs.tabText(i) == tab_name:
                self.tabs.setCurrentIndex(i)
                break
    
    def show_about(self):
        """Show about dialog."""
        QMessageBox.about(
            self,
            f"About {APP_NAME}",
            f"{APP_NAME} v{APP_VERSION}\n\n"
            "Multi-account Git manager with support for 8 platforms.\n\n"
            "Supported Platforms:\n"
            "• GitHub\n"
            "• GitLab\n"
            "• Bitbucket\n"
            "• Azure DevOps\n"
            "• Self-Hosted\n"
            "• Cloud Storage\n"
            "• Local Path\n"
            "• SourceForge"
        )
    
    def set_application_style(self):
        """Set application color scheme based on theme."""
        palette = QPalette()
        
        # Color mapping
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
        
        def get_color(color_name):
            if color_name in color_map:
                return color_map[color_name]
            if color_name.startswith('rgb('):
                try:
                    rgb_str = color_name[4:-1]
                    r, g, b = map(int, rgb_str.split(','))
                    return QColor(r, g, b)
                except:
                    pass
            return QColor(50, 50, 50)
        
        theme = self.theme
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


if __name__ == '__main__':
    run_desktop_app()
```

## Testing the Integration

After integration, test the following:

1. **Launch Desktop App**
   ```bash
   git-manager --desktop
   ```

2. **Test Clone Tab**
   - Analyze external repository URL
   - Fetch personal repositories
   - Clone a repository

3. **Test Git Operations Tab**
   - Check repository status
   - Push/pull/sync changes
   - Setup new repository

4. **Test Accounts Tab**
   - List all accounts
   - Test SSH connections
   - Generate SSH keys
   - Setup PAT tokens

5. **Test Menu Shortcuts**
   - Ctrl+Shift+C → Clone Repository
   - Ctrl+Shift+G → Git Operations
   - Ctrl+Shift+A → Accounts

## Troubleshooting

### Import Errors
- Ensure all widget files are in `src/git_manager/desktop/widgets/`
- Check that `__init__.py` files exist in all directories

### Missing Dependencies
- Ensure PyQt6 is installed: `pip install PyQt6`
- Check that core modules are available

### Thread Issues
- Worker threads should be properly cleaned up
- Check logs for thread-related errors

### UI Layout Issues
- Adjust window size and widget proportions as needed
- Test on different screen resolutions

## Performance Optimization

For better performance:

1. **Lazy Load Widgets**
   ```python
   # Only create widget when tab is first accessed
   def on_tab_changed(self, index):
       if not self.tabs.widget(index):
           self.tabs.setTabText(index, "Loading...")
           # Create widget
   ```

2. **Cache Account Lists**
   ```python
   # Cache accounts to avoid repeated database queries
   self.cached_accounts = None
   self.cache_timestamp = 0
   ```

3. **Optimize Thread Pool**
   ```python
   # Limit concurrent operations
   self.max_concurrent_operations = 3
   ```

## Next Steps

1. Test the integration thoroughly
2. Gather user feedback
3. Optimize performance based on usage patterns
4. Add additional features as needed
5. Create web UI templates for interactive features
