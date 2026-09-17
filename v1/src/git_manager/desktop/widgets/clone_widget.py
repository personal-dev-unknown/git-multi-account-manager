"""Clone repository widget for desktop."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QTabWidget, QLabel, QLineEdit,
    QPushButton, QComboBox, QCheckBox, QTableWidget, QTableWidgetItem,
    QMessageBox, QProgressBar, QTextEdit, QFileDialog
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal
from PyQt6.QtGui import QFont

from ...core.clone.api import CloneAPI, RepositoryAPI, PlatformAPI
from ...core.account_manager import AccountManager
from ...core.config_manager import ConfigManager
from ...utils.log_config import get_logger as get_advanced_logger, LogCategory


class CloneWorkerThread(QThread):
    """Worker thread for clone operations."""
    
    progress = pyqtSignal(str)
    finished = pyqtSignal(bool, str)
    
    def __init__(self, clone_api, operation_type, **kwargs):
        super().__init__()
        self.clone_api = clone_api
        self.operation_type = operation_type
        self.kwargs = kwargs
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
    
    def run(self):
        """Run clone operation."""
        try:
            self.progress.emit("Starting clone operation...")
            
            if self.operation_type == 'external':
                result = self.clone_api.clone_external_repository(**self.kwargs)
            elif self.operation_type == 'personal':
                result = self.clone_api.clone_personal_repository(**self.kwargs)
            else:
                result = {'success': False, 'error': 'Unknown operation'}
            
            if result['success']:
                self.progress.emit(f"Clone successful: {result.get('destination', 'N/A')}")
                self.finished.emit(True, result.get('message', 'Clone completed'))
            else:
                self.finished.emit(False, result.get('error', 'Clone failed'))
        
        except Exception as e:
            self.logger.error(f"Clone operation failed: {str(e)}", exc_info=True)
            self.finished.emit(False, str(e))


class CloneWidget(QWidget):
    """Clone repository widget."""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
        
        # Initialize managers and APIs
        self.account_manager = AccountManager()
        self.config_manager = ConfigManager()
        self.clone_api = CloneAPI(self.account_manager, self.config_manager)
        self.repo_api = RepositoryAPI(self.account_manager, self.config_manager)
        self.platform_api = PlatformAPI(self.account_manager, self.config_manager)
        
        self.clone_thread = None
        
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
        tabs.addTab(self.create_platforms_tab(), "Platforms")
        layout.addWidget(tabs)
        
        self.setLayout(layout)
    
    def create_external_tab(self):
        """Create external repository tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Repository URL
        layout.addWidget(QLabel("Repository URL:"))
        self.external_url = QLineEdit()
        self.external_url.setPlaceholderText("https://github.com/user/repo or git@github.com:user/repo.git")
        layout.addWidget(self.external_url)
        
        # Account selection
        layout.addWidget(QLabel("Account:"))
        self.external_account = QComboBox()
        self.update_account_combo(self.external_account)
        layout.addWidget(self.external_account)
        
        # Authentication method
        layout.addWidget(QLabel("Authentication Method:"))
        self.external_auth = QComboBox()
        self.external_auth.addItems(['ssh', 'pat', 'password', 'anonymous'])
        layout.addWidget(self.external_auth)
        
        # Clone options
        self.external_recursive = QCheckBox("Clone submodules recursively")
        layout.addWidget(self.external_recursive)
        
        self.external_shallow = QCheckBox("Shallow clone (--depth=1)")
        layout.addWidget(self.external_shallow)
        
        self.external_fork = QCheckBox("Fork before cloning (for contributions)")
        layout.addWidget(self.external_fork)
        
        # Destination
        layout.addWidget(QLabel("Destination (optional):"))
        dest_layout = QHBoxLayout()
        self.external_dest = QLineEdit()
        self.external_dest.setPlaceholderText("Leave empty for default: ~/projects/account/repo")
        dest_layout.addWidget(self.external_dest)
        browse_btn = QPushButton("Browse...")
        browse_btn.clicked.connect(self.browse_external_destination)
        dest_layout.addWidget(browse_btn)
        layout.addLayout(dest_layout)
        
        # Clone button
        clone_btn = QPushButton("Clone")
        clone_btn.clicked.connect(self.clone_external)
        layout.addWidget(clone_btn)
        
        # Progress
        self.external_progress = QProgressBar()
        self.external_progress.setVisible(False)
        layout.addWidget(self.external_progress)
        
        # Output
        self.external_output = QTextEdit()
        self.external_output.setReadOnly(True)
        self.external_output.setMaximumHeight(150)
        layout.addWidget(QLabel("Output:"))
        layout.addWidget(self.external_output)
        
        layout.addStretch()
        widget.setLayout(layout)
        return widget
    
    def create_personal_tab(self):
        """Create personal repository tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Platform selection
        layout.addWidget(QLabel("Platform:"))
        self.personal_platform = QComboBox()
        self.personal_platform.addItems([
            'github',
            'gitlab',
            'bitbucket',
            'azure_devops',
            'self_hosted',
            'cloud_storage',
            'local_path',
            'sourceforge'
        ])
        self.personal_platform.currentTextChanged.connect(self.update_personal_accounts)
        layout.addWidget(self.personal_platform)
        
        # Account selection
        layout.addWidget(QLabel("Account:"))
        self.personal_account = QComboBox()
        self.update_personal_accounts()
        layout.addWidget(self.personal_account)
        
        # Repository list
        layout.addWidget(QLabel("Your Repositories:"))
        self.personal_repos_table = QTableWidget()
        self.personal_repos_table.setColumnCount(4)
        self.personal_repos_table.setHorizontalHeaderLabels(['Name', 'Visibility', 'Language', 'Stars'])
        self.personal_repos_table.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)
        self.personal_repos_table.setSelectionMode(QTableWidget.SelectionMode.SingleSelection)
        layout.addWidget(self.personal_repos_table)
        
        # Refresh button
        refresh_btn = QPushButton("Refresh Repositories")
        refresh_btn.clicked.connect(self.refresh_personal_repos)
        layout.addWidget(refresh_btn)
        
        # Authentication method
        layout.addWidget(QLabel("Authentication Method:"))
        self.personal_auth = QComboBox()
        self.personal_auth.addItems(['ssh', 'pat', 'anonymous'])
        layout.addWidget(self.personal_auth)
        
        # Clone options
        self.personal_recursive = QCheckBox("Clone submodules recursively")
        layout.addWidget(self.personal_recursive)
        
        self.personal_shallow = QCheckBox("Shallow clone (--depth=1)")
        layout.addWidget(self.personal_shallow)
        
        # Destination
        layout.addWidget(QLabel("Destination (optional):"))
        dest_layout = QHBoxLayout()
        self.personal_dest = QLineEdit()
        self.personal_dest.setPlaceholderText("Leave empty for default")
        dest_layout.addWidget(self.personal_dest)
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
        self.personal_output = QTextEdit()
        self.personal_output.setReadOnly(True)
        self.personal_output.setMaximumHeight(100)
        layout.addWidget(QLabel("Output:"))
        layout.addWidget(self.personal_output)
        
        layout.addStretch()
        widget.setLayout(layout)
        return widget
    
    def create_platforms_tab(self):
        """Create platforms information tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Platforms table
        layout.addWidget(QLabel("Supported Platforms:"))
        self.platforms_table = QTableWidget()
        self.platforms_table.setColumnCount(4)
        self.platforms_table.setHorizontalHeaderLabels(['Platform', 'API', 'SSH Host', 'Features'])
        layout.addWidget(self.platforms_table)
        
        # Refresh button
        refresh_btn = QPushButton("Refresh Platforms")
        refresh_btn.clicked.connect(self.refresh_platforms)
        layout.addWidget(refresh_btn)
        
        # Test connection
        layout.addWidget(QLabel("Test Connection:"))
        test_layout = QHBoxLayout()
        
        self.test_platform = QComboBox()
        self.test_platform.addItems(['github', 'gitlab', 'bitbucket'])
        test_layout.addWidget(QLabel("Platform:"))
        test_layout.addWidget(self.test_platform)
        
        self.test_account = QComboBox()
        self.update_account_combo(self.test_account)
        test_layout.addWidget(QLabel("Account:"))
        test_layout.addWidget(self.test_account)
        
        test_btn = QPushButton("Test")
        test_btn.clicked.connect(self.test_platform_connection)
        test_layout.addWidget(test_btn)
        
        layout.addLayout(test_layout)
        
        # Output
        self.platforms_output = QTextEdit()
        self.platforms_output.setReadOnly(True)
        layout.addWidget(QLabel("Output:"))
        layout.addWidget(self.platforms_output)
        
        layout.addStretch()
        widget.setLayout(layout)
        
        # Load platforms
        self.refresh_platforms()
        
        return widget
    
    def update_account_combo(self, combo):
        """Update account combo box."""
        try:
            accounts = self.account_manager.list_accounts()
            combo.clear()
            for acc in accounts:
                combo.addItem(acc.name)
        except Exception as e:
            self.logger.error(f"Failed to load accounts: {str(e)}")
    
    def update_personal_accounts(self):
        """Update personal accounts based on platform."""
        platform = self.personal_platform.currentText()
        result = self.platform_api.get_platform_accounts(platform)
        
        self.personal_account.clear()
        if result['success']:
            for acc in result.get('accounts', []):
                self.personal_account.addItem(acc['name'])
    
    def refresh_personal_repos(self):
        """Refresh personal repositories list."""
        try:
            platform = self.personal_platform.currentText()
            account = self.personal_account.currentText()
            
            result = self.repo_api.list_personal_repositories(
                platform=platform,
                account_name=account,
                force_refresh=True
            )
            
            if result['success']:
                self.personal_repos_table.setRowCount(0)
                for repo in result.get('repositories', []):
                    row = self.personal_repos_table.rowCount()
                    self.personal_repos_table.insertRow(row)
                    
                    self.personal_repos_table.setItem(row, 0, QTableWidgetItem(repo['name']))
                    self.personal_repos_table.setItem(row, 1, QTableWidgetItem(repo['visibility']))
                    self.personal_repos_table.setItem(row, 2, QTableWidgetItem(repo['language']))
                    self.personal_repos_table.setItem(row, 3, QTableWidgetItem(str(repo['stars'])))
                
                self.personal_output.setText(f"Loaded {result['count']} repositories")
            else:
                QMessageBox.warning(self, "Error", result.get('error', 'Failed to load repositories'))
        
        except Exception as e:
            self.logger.error(f"Failed to refresh repositories: {str(e)}")
            QMessageBox.critical(self, "Error", str(e))
    
    def refresh_platforms(self):
        """Refresh platforms list."""
        try:
            result = self.platform_api.get_supported_platforms()
            
            if result['success']:
                self.platforms_table.setRowCount(0)
                for platform in result.get('platforms', []):
                    row = self.platforms_table.rowCount()
                    self.platforms_table.insertRow(row)
                    
                    self.platforms_table.setItem(row, 0, QTableWidgetItem(platform['name']))
                    self.platforms_table.setItem(row, 1, QTableWidgetItem(platform.get('api_base', 'N/A')))
                    self.platforms_table.setItem(row, 2, QTableWidgetItem(platform.get('ssh_host', 'N/A')))
                    self.platforms_table.setItem(row, 3, QTableWidgetItem(', '.join(platform.get('features', []))))
        
        except Exception as e:
            self.logger.error(f"Failed to refresh platforms: {str(e)}")
    
    def clone_external(self):
        """Clone external repository."""
        url = self.external_url.text().strip()
        if not url:
            QMessageBox.warning(self, "Error", "Please enter a repository URL")
            return
        
        account = self.external_account.currentText()
        if not account:
            QMessageBox.warning(self, "Error", "Please select an account")
            return
        
        self.external_progress.setVisible(True)
        self.external_progress.setValue(0)
        
        self.clone_thread = CloneWorkerThread(
            self.clone_api,
            'external',
            repo_url=url,
            account_name=account,
            auth_method=self.external_auth.currentText(),
            destination=self.external_dest.text() or None,
            recursive=self.external_recursive.isChecked(),
            shallow=self.external_shallow.isChecked(),
            fork=self.external_fork.isChecked()
        )
        
        self.clone_thread.progress.connect(self.on_clone_progress)
        self.clone_thread.finished.connect(self.on_clone_finished)
        self.clone_thread.start()
    
    def clone_personal(self):
        """Clone personal repository."""
        if self.personal_repos_table.currentRow() < 0:
            QMessageBox.warning(self, "Error", "Please select a repository")
            return
        
        repo_name = self.personal_repos_table.item(self.personal_repos_table.currentRow(), 0).text()
        account = self.personal_account.currentText()
        platform = self.personal_platform.currentText()
        
        self.personal_progress.setVisible(True)
        self.personal_progress.setValue(0)
        
        self.clone_thread = CloneWorkerThread(
            self.clone_api,
            'personal',
            platform=platform,
            account_name=account,
            repo_name=repo_name,
            auth_method=self.personal_auth.currentText(),
            destination=self.personal_dest.text() or None,
            recursive=self.personal_recursive.isChecked(),
            shallow=self.personal_shallow.isChecked()
        )
        
        self.clone_thread.progress.connect(self.on_clone_progress)
        self.clone_thread.finished.connect(self.on_clone_finished)
        self.clone_thread.start()
    
    def test_platform_connection(self):
        """Test platform connection."""
        try:
            platform = self.test_platform.currentText()
            account = self.test_account.currentText()
            
            result = self.platform_api.test_platform_connection(platform, account)
            
            if result['success']:
                self.platforms_output.setText(f"✓ Connection successful to {platform}")
            else:
                self.platforms_output.setText(f"✗ Connection failed: {result.get('error', 'Unknown error')}")
        
        except Exception as e:
            self.logger.error(f"Connection test failed: {str(e)}")
            self.platforms_output.setText(f"Error: {str(e)}")
    
    def browse_external_destination(self):
        """Browse for external destination."""
        path = QFileDialog.getExistingDirectory(self, "Select Clone Destination")
        if path:
            self.external_dest.setText(path)
    
    def browse_personal_destination(self):
        """Browse for personal destination."""
        path = QFileDialog.getExistingDirectory(self, "Select Clone Destination")
        if path:
            self.personal_dest.setText(path)
    
    def on_clone_progress(self, message):
        """Handle clone progress."""
        self.external_output.append(message)
        self.personal_output.append(message)
    
    def on_clone_finished(self, success, message):
        """Handle clone finished."""
        self.external_progress.setVisible(False)
        self.personal_progress.setVisible(False)
        
        if success:
            QMessageBox.information(self, "Success", message)
        else:
            QMessageBox.critical(self, "Error", message)
