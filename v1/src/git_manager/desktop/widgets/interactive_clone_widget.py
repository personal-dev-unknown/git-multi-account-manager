"""Interactive clone widget for desktop."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QLineEdit,
    QComboBox, QCheckBox, QTableWidget, QTableWidgetItem, QMessageBox,
    QProgressBar, QTextEdit, QFileDialog, QTabWidget, QDialog,
    QSpinBox, QListWidget, QListWidgetItem
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal
from PyQt6.QtGui import QFont

from pathlib import Path
from typing import Optional, Dict, List, Any

from ..interactive_manager import InteractiveDesktopManager
from ...utils.log_config import get_logger as get_advanced_logger, LogCategory


class CloneWorkerThread(QThread):
    """Worker thread for clone operations."""
    
    progress = pyqtSignal(str)
    finished = pyqtSignal(bool, str, str)  # success, message, destination
    
    def __init__(
        self,
        manager: InteractiveDesktopManager,
        repo_url: str,
        account_name: str,
        destination: str,
        auth_method: str = 'ssh',
        recursive: bool = False,
        shallow: bool = False
    ):
        super().__init__()
        self.manager = manager
        self.repo_url = repo_url
        self.account_name = account_name
        self.destination = destination
        self.auth_method = auth_method
        self.recursive = recursive
        self.shallow = shallow
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
    
    def run(self):
        """Run clone operation."""
        try:
            self.progress.emit(f"Cloning {self.repo_url}...")
            
            result = self.manager.clone_repository(
                repo_url=self.repo_url,
                account_name=self.account_name,
                destination=self.destination,
                auth_method=self.auth_method,
                recursive=self.recursive,
                shallow=self.shallow
            )
            
            if result['success']:
                dest = result.get('destination', self.destination)
                self.finished.emit(True, "Clone completed successfully", dest)
            else:
                self.finished.emit(False, result.get('error', 'Clone failed'), '')
        except Exception as e:
            self.logger.error(f"Clone failed: {e}")
            self.finished.emit(False, str(e), '')


class RepositoryFetchWorkerThread(QThread):
    """Worker thread for fetching repositories."""
    
    progress = pyqtSignal(str)
    finished = pyqtSignal(bool, list)  # success, repositories
    
    def __init__(
        self,
        manager: InteractiveDesktopManager,
        platform: str,
        account_name: str
    ):
        super().__init__()
        self.manager = manager
        self.platform = platform
        self.account_name = account_name
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
    
    def run(self):
        """Fetch repositories."""
        try:
            self.progress.emit(f"Fetching repositories for {self.account_name}...")
            
            result = self.manager.get_personal_repositories(
                platform=self.platform,
                account_name=self.account_name
            )
            
            if result['success']:
                self.finished.emit(True, result.get('repositories', []))
            else:
                self.finished.emit(False, [])
        except Exception as e:
            self.logger.error(f"Repository fetch failed: {e}")
            self.finished.emit(False, [])


class InteractiveCloneWidget(QWidget):
    """Interactive clone widget."""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
        self.manager = InteractiveDesktopManager()
        self.clone_thread = None
        self.fetch_thread = None
        
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout()
        
        # Title
        title = QLabel("Clone Repository")
        title_font = QFont()
        title_font.setPointSize(14)
        title_font.setBold(True)
        title.setFont(title_font)
        layout.addWidget(title)
        
        # Tab widget
        tabs = QTabWidget()
        tabs.addTab(self.create_external_tab(), "External Repository")
        tabs.addTab(self.create_personal_tab(), "Personal Repository")
        layout.addWidget(tabs)
        
        self.setLayout(layout)
    
    def create_external_tab(self) -> QWidget:
        """Create external repository tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Repository URL
        layout.addWidget(QLabel("Repository URL:"))
        self.external_url = QLineEdit()
        self.external_url.setPlaceholderText("e.g., https://github.com/user/repo.git")
        layout.addWidget(self.external_url)
        
        # Analyze button
        analyze_btn = QPushButton("Analyze URL")
        analyze_btn.clicked.connect(self.analyze_external_url)
        layout.addWidget(analyze_btn)
        
        # URL analysis result
        layout.addWidget(QLabel("Repository Info:"))
        self.external_info_text = QTextEdit()
        self.external_info_text.setReadOnly(True)
        self.external_info_text.setMaximumHeight(100)
        layout.addWidget(self.external_info_text)
        
        # Account selection
        layout.addWidget(QLabel("Account:"))
        self.external_account_combo = QComboBox()
        self.refresh_accounts_combo(self.external_account_combo)
        layout.addWidget(self.external_account_combo)
        
        # Authentication method
        layout.addWidget(QLabel("Authentication Method:"))
        self.external_auth_combo = QComboBox()
        self.external_auth_combo.addItems(['SSH', 'HTTPS with PAT'])
        layout.addWidget(self.external_auth_combo)
        
        # Clone options
        options_layout = QHBoxLayout()
        
        self.external_recursive = QCheckBox("Clone submodules recursively")
        options_layout.addWidget(self.external_recursive)
        
        self.external_shallow = QCheckBox("Shallow clone (--depth=1)")
        options_layout.addWidget(self.external_shallow)
        
        layout.addLayout(options_layout)
        
        # Destination
        layout.addWidget(QLabel("Destination Path:"))
        dest_layout = QHBoxLayout()
        
        self.external_destination = QLineEdit()
        self.external_destination.setText(str(Path.home() / "repositories"))
        dest_layout.addWidget(self.external_destination)
        
        browse_btn = QPushButton("Browse...")
        browse_btn.clicked.connect(self.browse_external_destination)
        dest_layout.addWidget(browse_btn)
        
        layout.addLayout(dest_layout)
        
        # Clone button
        clone_btn = QPushButton("Clone Repository")
        clone_btn.clicked.connect(self.clone_external)
        layout.addWidget(clone_btn)
        
        # Progress
        self.external_progress = QProgressBar()
        self.external_progress.setVisible(False)
        layout.addWidget(self.external_progress)
        
        # Output
        layout.addWidget(QLabel("Output:"))
        self.external_output = QTextEdit()
        self.external_output.setReadOnly(True)
        layout.addWidget(self.external_output)
        
        widget.setLayout(layout)
        return widget
    
    def create_personal_tab(self) -> QWidget:
        """Create personal repository tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Platform selection
        layout.addWidget(QLabel("Platform:"))
        self.personal_platform_combo = QComboBox()
        platforms = self.manager.platform_manager.PLATFORMS.keys()
        self.personal_platform_combo.addItems(platforms)
        self.personal_platform_combo.currentTextChanged.connect(self.on_platform_changed)
        layout.addWidget(self.personal_platform_combo)
        
        # Account selection
        layout.addWidget(QLabel("Account:"))
        self.personal_account_combo = QComboBox()
        layout.addWidget(self.personal_account_combo)
        
        # Fetch repositories button
        fetch_btn = QPushButton("Fetch Repositories")
        fetch_btn.clicked.connect(self.fetch_personal_repositories)
        layout.addWidget(fetch_btn)
        
        # Repositories list
        layout.addWidget(QLabel("Available Repositories:"))
        self.personal_repos_list = QListWidget()
        layout.addWidget(self.personal_repos_list)
        
        # Authentication method
        layout.addWidget(QLabel("Authentication Method:"))
        self.personal_auth_combo = QComboBox()
        self.personal_auth_combo.addItems(['SSH', 'HTTPS with PAT'])
        layout.addWidget(self.personal_auth_combo)
        
        # Clone options
        options_layout = QHBoxLayout()
        
        self.personal_recursive = QCheckBox("Clone submodules recursively")
        options_layout.addWidget(self.personal_recursive)
        
        self.personal_shallow = QCheckBox("Shallow clone (--depth=1)")
        options_layout.addWidget(self.personal_shallow)
        
        layout.addLayout(options_layout)
        
        # Destination
        layout.addWidget(QLabel("Destination Path:"))
        dest_layout = QHBoxLayout()
        
        self.personal_destination = QLineEdit()
        self.personal_destination.setText(str(Path.home() / "repositories"))
        dest_layout.addWidget(self.personal_destination)
        
        browse_btn = QPushButton("Browse...")
        browse_btn.clicked.connect(self.browse_personal_destination)
        dest_layout.addWidget(browse_btn)
        
        layout.addLayout(dest_layout)
        
        # Clone button
        clone_btn = QPushButton("Clone Selected Repository")
        clone_btn.clicked.connect(self.clone_personal)
        layout.addWidget(clone_btn)
        
        # Progress
        self.personal_progress = QProgressBar()
        self.personal_progress.setVisible(False)
        layout.addWidget(self.personal_progress)
        
        # Output
        layout.addWidget(QLabel("Output:"))
        self.personal_output = QTextEdit()
        self.personal_output.setReadOnly(True)
        layout.addWidget(self.personal_output)
        
        widget.setLayout(layout)
        return widget
    
    def refresh_accounts_combo(self, combo: QComboBox):
        """Refresh accounts combo box."""
        try:
            accounts = self.manager.list_all_accounts()
            combo.clear()
            combo.addItems([a['name'] for a in accounts])
        except Exception as e:
            self.logger.error(f"Error refreshing accounts: {e}")
    
    def on_platform_changed(self):
        """Handle platform change."""
        self.refresh_accounts_combo(self.personal_account_combo)
        self.personal_repos_list.clear()
    
    def analyze_external_url(self):
        """Analyze external repository URL."""
        url = self.external_url.text().strip()
        
        if not url:
            QMessageBox.warning(self, "Validation Error", "Please enter a repository URL")
            return
        
        try:
            result = self.manager.analyze_repository_url(url)
            
            if result['success']:
                info = f"Platform: {result.get('platform', 'Unknown').upper()}\n"
                info += f"Owner: {result.get('parsed', {}).get('owner', 'Unknown')}\n"
                info += f"Repository: {result.get('parsed', {}).get('repo', 'Unknown')}"
                self.external_info_text.setText(info)
            else:
                QMessageBox.critical(self, "Analysis Failed", result.get('error', 'Unknown error'))
        except Exception as e:
            self.logger.error(f"URL analysis failed: {e}")
            QMessageBox.critical(self, "Error", f"Analysis failed: {str(e)}")
    
    def browse_external_destination(self):
        """Browse for external destination."""
        path = QFileDialog.getExistingDirectory(
            self,
            "Select Destination Directory",
            str(Path.home())
        )
        
        if path:
            self.external_destination.setText(path)
    
    def browse_personal_destination(self):
        """Browse for personal destination."""
        path = QFileDialog.getExistingDirectory(
            self,
            "Select Destination Directory",
            str(Path.home())
        )
        
        if path:
            self.personal_destination.setText(path)
    
    def fetch_personal_repositories(self):
        """Fetch personal repositories."""
        platform = self.personal_platform_combo.currentText()
        account = self.personal_account_combo.currentText()
        
        if not account:
            QMessageBox.warning(self, "Validation Error", "Please select an account")
            return
        
        self.personal_progress.setVisible(True)
        self.personal_progress.setValue(0)
        
        self.fetch_thread = RepositoryFetchWorkerThread(
            self.manager,
            platform,
            account
        )
        self.fetch_thread.progress.connect(self.on_fetch_progress)
        self.fetch_thread.finished.connect(self.on_fetch_finished)
        self.fetch_thread.start()
    
    def on_fetch_progress(self, message: str):
        """Handle fetch progress."""
        self.personal_progress.setValue(50)
    
    def on_fetch_finished(self, success: bool, repositories: List[Dict]):
        """Handle fetch completion."""
        self.personal_progress.setVisible(False)
        
        if success:
            self.personal_repos_list.clear()
            for repo in repositories:
                item = QListWidgetItem(f"{repo['name']} ({repo['visibility']})")
                item.setData(Qt.ItemDataRole.UserRole, repo)
                self.personal_repos_list.addItem(item)
            
            self.personal_output.setText(f"✓ Found {len(repositories)} repositories")
        else:
            QMessageBox.critical(self, "Fetch Failed", "Failed to fetch repositories")
    
    def clone_external(self):
        """Clone external repository."""
        url = self.external_url.text().strip()
        account = self.external_account_combo.currentText()
        destination = self.external_destination.text().strip()
        auth_method = 'ssh' if self.external_auth_combo.currentText() == 'SSH' else 'pat'
        recursive = self.external_recursive.isChecked()
        shallow = self.external_shallow.isChecked()
        
        if not url or not account or not destination:
            QMessageBox.warning(self, "Validation Error", "Please fill in all required fields")
            return
        
        self.external_progress.setVisible(True)
        self.external_progress.setValue(0)
        
        self.clone_thread = CloneWorkerThread(
            self.manager,
            url,
            account,
            destination,
            auth_method,
            recursive,
            shallow
        )
        self.clone_thread.progress.connect(self.on_clone_progress)
        self.clone_thread.finished.connect(self.on_external_clone_finished)
        self.clone_thread.start()
    
    def clone_personal(self):
        """Clone personal repository."""
        selected_items = self.personal_repos_list.selectedItems()
        
        if not selected_items:
            QMessageBox.warning(self, "Validation Error", "Please select a repository")
            return
        
        repo = selected_items[0].data(Qt.ItemDataRole.UserRole)
        account = self.personal_account_combo.currentText()
        destination = self.personal_destination.text().strip()
        auth_method = 'ssh' if self.personal_auth_combo.currentText() == 'SSH' else 'pat'
        recursive = self.personal_recursive.isChecked()
        shallow = self.personal_shallow.isChecked()
        
        # Use SSH URL or HTTPS URL based on auth method
        repo_url = repo['ssh_url'] if auth_method == 'ssh' else repo['https_url']
        
        self.personal_progress.setVisible(True)
        self.personal_progress.setValue(0)
        
        self.clone_thread = CloneWorkerThread(
            self.manager,
            repo_url,
            account,
            destination,
            auth_method,
            recursive,
            shallow
        )
        self.clone_thread.progress.connect(self.on_clone_progress)
        self.clone_thread.finished.connect(self.on_personal_clone_finished)
        self.clone_thread.start()
    
    def on_clone_progress(self, message: str):
        """Handle clone progress."""
        self.external_progress.setValue(50)
        self.personal_progress.setValue(50)
    
    def on_external_clone_finished(self, success: bool, message: str, destination: str):
        """Handle external clone completion."""
        self.external_progress.setVisible(False)
        
        if success:
            self.external_output.setText(f"✓ {message}\n\nLocation: {destination}")
            QMessageBox.information(self, "Success", f"✓ {message}")
        else:
            self.external_output.setText(f"✗ {message}")
            QMessageBox.critical(self, "Clone Failed", message)
    
    def on_personal_clone_finished(self, success: bool, message: str, destination: str):
        """Handle personal clone completion."""
        self.personal_progress.setVisible(False)
        
        if success:
            self.personal_output.setText(f"✓ {message}\n\nLocation: {destination}")
            QMessageBox.information(self, "Success", f"✓ {message}")
        else:
            self.personal_output.setText(f"✗ {message}")
            QMessageBox.critical(self, "Clone Failed", message)
