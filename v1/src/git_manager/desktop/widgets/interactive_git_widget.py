"""Interactive git operations widget for desktop."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QLineEdit,
    QTextEdit, QMessageBox, QProgressBar, QFileDialog, QTabWidget,
    QTableWidget, QTableWidgetItem, QComboBox, QCheckBox, QSpinBox
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal, QTimer
from PyQt6.QtGui import QFont, QColor

from pathlib import Path
from typing import Optional, Dict, Any

from ..interactive_manager import InteractiveDesktopManager
from ...utils.log_config import get_logger as get_advanced_logger, LogCategory


class GitOperationWorkerThread(QThread):
    """Worker thread for git operations."""
    
    progress = pyqtSignal(str)
    finished = pyqtSignal(bool, str)
    
    def __init__(
        self,
        manager: InteractiveDesktopManager,
        operation: str,
        repo_path: str,
        **kwargs
    ):
        super().__init__()
        self.manager = manager
        self.operation = operation
        self.repo_path = repo_path
        self.kwargs = kwargs
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
    
    def run(self):
        """Run git operation."""
        try:
            self.progress.emit(f"Running {self.operation}...")
            
            if self.operation == 'status':
                result = self.manager.check_repository_status(self.repo_path)
            elif self.operation == 'push':
                result = self.manager.git_push(self.repo_path)
            elif self.operation == 'pull':
                result = self.manager.git_pull(self.repo_path)
            elif self.operation == 'sync':
                result = self.manager.git_sync(self.repo_path)
            else:
                result = {'success': False, 'error': 'Unknown operation'}
            
            if result['success']:
                self.finished.emit(True, result.get('message', 'Operation completed'))
            else:
                self.finished.emit(False, result.get('error', 'Operation failed'))
        except Exception as e:
            self.logger.error(f"Git operation failed: {e}")
            self.finished.emit(False, str(e))


class InteractiveGitWidget(QWidget):
    """Interactive git operations widget."""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
        self.manager = InteractiveDesktopManager()
        self.current_repo_path = None
        self.git_thread = None
        
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout()
        
        # Title
        title = QLabel("Git Operations")
        title_font = QFont()
        title_font.setPointSize(14)
        title_font.setBold(True)
        title.setFont(title_font)
        layout.addWidget(title)
        
        # Repository path selection
        path_layout = QHBoxLayout()
        path_layout.addWidget(QLabel("Repository Path:"))
        
        self.repo_path_input = QLineEdit()
        self.repo_path_input.setText(str(Path.cwd()))
        path_layout.addWidget(self.repo_path_input)
        
        browse_btn = QPushButton("Browse...")
        browse_btn.clicked.connect(self.browse_repository)
        path_layout.addWidget(browse_btn)
        
        layout.addLayout(path_layout)
        
        # Tab widget
        tabs = QTabWidget()
        tabs.addTab(self.create_status_tab(), "Status")
        tabs.addTab(self.create_operations_tab(), "Operations")
        tabs.addTab(self.create_setup_tab(), "Setup")
        layout.addWidget(tabs)
        
        self.setLayout(layout)
    
    def create_status_tab(self) -> QWidget:
        """Create status tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Status button
        check_status_btn = QPushButton("Check Status")
        check_status_btn.clicked.connect(self.check_status)
        layout.addWidget(check_status_btn)
        
        # Status display
        layout.addWidget(QLabel("Repository Status:"))
        self.status_text = QTextEdit()
        self.status_text.setReadOnly(True)
        layout.addWidget(self.status_text)
        
        # Progress bar
        self.status_progress = QProgressBar()
        self.status_progress.setVisible(False)
        layout.addWidget(self.status_progress)
        
        widget.setLayout(layout)
        return widget
    
    def create_operations_tab(self) -> QWidget:
        """Create operations tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Operation buttons
        button_layout = QHBoxLayout()
        
        push_btn = QPushButton("📤 Push")
        push_btn.clicked.connect(lambda: self.run_git_operation('push'))
        button_layout.addWidget(push_btn)
        
        pull_btn = QPushButton("📥 Pull")
        pull_btn.clicked.connect(lambda: self.run_git_operation('pull'))
        button_layout.addWidget(pull_btn)
        
        sync_btn = QPushButton("🔄 Sync")
        sync_btn.clicked.connect(lambda: self.run_git_operation('sync'))
        button_layout.addWidget(sync_btn)
        
        layout.addLayout(button_layout)
        
        # Operation output
        layout.addWidget(QLabel("Operation Output:"))
        self.operation_text = QTextEdit()
        self.operation_text.setReadOnly(True)
        layout.addWidget(self.operation_text)
        
        # Progress bar
        self.operation_progress = QProgressBar()
        self.operation_progress.setVisible(False)
        layout.addWidget(self.operation_progress)
        
        widget.setLayout(layout)
        return widget
    
    def create_setup_tab(self) -> QWidget:
        """Create repository setup tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Repository name
        layout.addWidget(QLabel("Repository Name:"))
        self.setup_repo_name = QLineEdit()
        layout.addWidget(self.setup_repo_name)
        
        # Account selection
        layout.addWidget(QLabel("Account:"))
        self.setup_account_combo = QComboBox()
        self.refresh_accounts_combo()
        layout.addWidget(self.setup_account_combo)
        
        # Description
        layout.addWidget(QLabel("Description (optional):"))
        self.setup_description = QTextEdit()
        self.setup_description.setMaximumHeight(80)
        layout.addWidget(self.setup_description)
        
        # Branch
        layout.addWidget(QLabel("Default Branch:"))
        self.setup_branch = QLineEdit()
        self.setup_branch.setText("main")
        layout.addWidget(self.setup_branch)
        
        # Setup button
        setup_btn = QPushButton("Setup Repository")
        setup_btn.clicked.connect(self.setup_repository)
        layout.addWidget(setup_btn)
        
        layout.addStretch()
        
        widget.setLayout(layout)
        return widget
    
    def refresh_accounts_combo(self):
        """Refresh accounts combo box."""
        try:
            accounts = self.manager.list_all_accounts()
            self.setup_account_combo.clear()
            self.setup_account_combo.addItems([a['name'] for a in accounts])
        except Exception as e:
            self.logger.error(f"Error refreshing accounts: {e}")
    
    def browse_repository(self):
        """Browse for repository path."""
        path = QFileDialog.getExistingDirectory(
            self,
            "Select Repository Path",
            str(Path.home())
        )
        
        if path:
            self.repo_path_input.setText(path)
            self.current_repo_path = path
    
    def check_status(self):
        """Check repository status."""
        repo_path = self.repo_path_input.text().strip()
        
        if not repo_path:
            QMessageBox.warning(self, "Validation Error", "Please select a repository path")
            return
        
        self.status_progress.setVisible(True)
        self.status_progress.setValue(0)
        
        self.git_thread = GitOperationWorkerThread(
            self.manager,
            'status',
            repo_path
        )
        self.git_thread.progress.connect(self.on_git_progress)
        self.git_thread.finished.connect(self.on_status_finished)
        self.git_thread.start()
    
    def run_git_operation(self, operation: str):
        """Run git operation."""
        repo_path = self.repo_path_input.text().strip()
        
        if not repo_path:
            QMessageBox.warning(self, "Validation Error", "Please select a repository path")
            return
        
        self.operation_progress.setVisible(True)
        self.operation_progress.setValue(0)
        
        self.git_thread = GitOperationWorkerThread(
            self.manager,
            operation,
            repo_path
        )
        self.git_thread.progress.connect(self.on_git_progress)
        self.git_thread.finished.connect(self.on_operation_finished)
        self.git_thread.start()
    
    def on_git_progress(self, message: str):
        """Handle git operation progress."""
        self.status_progress.setValue(50)
        self.operation_progress.setValue(50)
    
    def on_status_finished(self, success: bool, message: str):
        """Handle status check completion."""
        self.status_progress.setVisible(False)
        
        if success:
            self.status_text.setText(message)
        else:
            self.status_text.setText(f"Error: {message}")
            QMessageBox.critical(self, "Status Check Failed", message)
    
    def on_operation_finished(self, success: bool, message: str):
        """Handle git operation completion."""
        self.operation_progress.setVisible(False)
        
        if success:
            self.operation_text.setText(message)
            QMessageBox.information(self, "Success", message)
        else:
            self.operation_text.setText(f"Error: {message}")
            QMessageBox.critical(self, "Operation Failed", message)
    
    def setup_repository(self):
        """Setup new repository."""
        repo_path = self.repo_path_input.text().strip()
        repo_name = self.setup_repo_name.text().strip()
        account_name = self.setup_account_combo.currentText()
        description = self.setup_description.toPlainText().strip()
        branch = self.setup_branch.text().strip() or "main"
        
        if not repo_path or not repo_name:
            QMessageBox.warning(self, "Validation Error", "Repository path and name are required")
            return
        
        try:
            result = self.manager.setup_new_repository(
                repo_path=repo_path,
                account_name=account_name,
                repo_name=repo_name,
                description=description,
                branch=branch
            )
            
            if result['success']:
                QMessageBox.information(
                    self,
                    "Success",
                    f"✓ Repository setup completed\n\nPath: {repo_path}"
                )
            else:
                QMessageBox.critical(
                    self,
                    "Setup Failed",
                    f"✗ {result.get('error', 'Unknown error')}"
                )
        except Exception as e:
            self.logger.error(f"Repository setup failed: {e}")
            QMessageBox.critical(self, "Error", f"Setup failed: {str(e)}")
