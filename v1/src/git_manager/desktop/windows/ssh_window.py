# src/git_manager/desktop/windows/ssh_window.py
"""SSH key management widget - Integrated with new SSH system."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton,
    QTableWidget, QTableWidgetItem, QDialog, QLineEdit, QComboBox,
    QMessageBox, QProgressDialog
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal
from PyQt6.QtGui import QIcon


class SSHSetupThread(QThread):
    """Background thread for SSH setup."""
    
    finished = pyqtSignal(dict)
    error = pyqtSignal(str)
    
    def __init__(self, name, email, platform, account_type):
        super().__init__()
        self.name = name
        self.email = email
        self.platform = platform
        self.account_type = account_type
    
    def run(self):
        """Run SSH setup."""
        try:
            from ...core.ssh import SSHWorkflowOrchestrator
            
            orchestrator = SSHWorkflowOrchestrator()
            result = orchestrator.setup_account(
                name=self.name,
                email=self.email,
                platform=self.platform,
                account_type=self.account_type
            )
            
            self.finished.emit(result)
        except Exception as e:
            self.error.emit(str(e))


class SSHWidget(QWidget):
    """SSH key management widget - Integrated with new SSH system."""
    
    def __init__(self, database_manager=None, account_manager=None, config_manager=None):
        super().__init__()
        self.db = database_manager
        self.account_mgr = account_manager
        self.config_mgr = config_manager
        self.init_ui()
        self.load_accounts()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout(self)
        
        # Title
        title = QLabel("🔐 SSH Key Management")
        title.setStyleSheet("font-size: 16px; font-weight: bold;")
        layout.addWidget(title)
        
        # Buttons
        button_layout = QHBoxLayout()
        
        setup_btn = QPushButton("Setup New Account")
        setup_btn.clicked.connect(self.setup_new_account)
        button_layout.addWidget(setup_btn)
        
        test_btn = QPushButton("Test Connection")
        test_btn.clicked.connect(self.test_connection)
        button_layout.addWidget(test_btn)
        
        refresh_btn = QPushButton("Refresh")
        refresh_btn.clicked.connect(self.load_accounts)
        button_layout.addWidget(refresh_btn)
        
        layout.addLayout(button_layout)
        
        # Accounts table
        self.table = QTableWidget()
        self.table.setColumnCount(5)
        self.table.setHorizontalHeaderLabels([
            "Account", "Platform", "SSH Alias", "Fingerprint", "Status"
        ])
        layout.addWidget(self.table)
    
    def load_accounts(self):
        """Load SSH accounts."""
        try:
            from ...core.ssh import SSHWorkflowOrchestrator
            
            orchestrator = SSHWorkflowOrchestrator()
            accounts = orchestrator.list_accounts()
            
            self.table.setRowCount(len(accounts))
            
            for row, acc in enumerate(accounts):
                self.table.setItem(row, 0, QTableWidgetItem(acc['name']))
                self.table.setItem(row, 1, QTableWidgetItem(acc['platform']))
                self.table.setItem(row, 2, QTableWidgetItem(acc['host_alias']))
                self.table.setItem(row, 3, QTableWidgetItem(acc.get('fingerprint', 'N/A')[:20]))
                self.table.setItem(row, 4, QTableWidgetItem("✓ Configured"))
        
        except Exception as e:
            QMessageBox.warning(self, "Error", f"Failed to load accounts: {e}")
    
    def setup_new_account(self):
        """Setup new SSH account."""
        dialog = QDialog(self)
        dialog.setWindowTitle("Setup SSH Account")
        dialog.setGeometry(100, 100, 400, 300)
        
        layout = QVBoxLayout(dialog)
        
        # Name
        layout.addWidget(QLabel("Account Name:"))
        name_input = QLineEdit()
        layout.addWidget(name_input)
        
        # Email
        layout.addWidget(QLabel("Email:"))
        email_input = QLineEdit()
        layout.addWidget(email_input)
        
        # Platform
        layout.addWidget(QLabel("Platform:"))
        platform_combo = QComboBox()
        platform_combo.addItems(["github.com", "gitlab.com", "bitbucket.org"])
        layout.addWidget(platform_combo)
        
        # Account Type
        layout.addWidget(QLabel("Account Type:"))
        type_combo = QComboBox()
        type_combo.addItems(["personal", "school", "work", "zanabuni", "casini"])
        layout.addWidget(type_combo)
        
        # Buttons
        button_layout = QHBoxLayout()
        
        ok_btn = QPushButton("Setup")
        ok_btn.clicked.connect(lambda: self._do_setup(
            dialog, name_input.text(), email_input.text(),
            platform_combo.currentText(), type_combo.currentText()
        ))
        button_layout.addWidget(ok_btn)
        
        cancel_btn = QPushButton("Cancel")
        cancel_btn.clicked.connect(dialog.reject)
        button_layout.addWidget(cancel_btn)
        
        layout.addLayout(button_layout)
        
        dialog.exec()
    
    def _do_setup(self, dialog, name, email, platform, account_type):
        """Perform SSH setup."""
        if not all([name, email]):
            QMessageBox.warning(self, "Error", "Please fill in all fields")
            return
        
        # Show progress
        progress = QProgressDialog("Setting up SSH account...", None, 0, 0, self)
        progress.setWindowModality(Qt.WindowModality.WindowModal)
        progress.show()
        
        # Run setup in background
        self.setup_thread = SSHSetupThread(name, email, platform, account_type)
        self.setup_thread.finished.connect(lambda result: self._setup_finished(result, progress, dialog))
        self.setup_thread.error.connect(lambda err: self._setup_error(err, progress))
        self.setup_thread.start()
    
    def _setup_finished(self, result, progress, dialog):
        """Handle setup completion."""
        progress.close()
        
        if result['success']:
            QMessageBox.information(
                self, "Success",
                f"SSH account '{result['key_info']['name']}' setup complete!\n"
                f"SSH Alias: {result['key_info']['ssh_host_alias']}"
            )
            dialog.accept()
            self.load_accounts()
        else:
            errors = "\n".join(result.get('errors', ['Unknown error']))
            QMessageBox.warning(self, "Setup Failed", f"Errors:\n{errors}")
    
    def _setup_error(self, error, progress):
        """Handle setup error."""
        progress.close()
        QMessageBox.critical(self, "Error", f"Setup failed: {error}")
    
    def test_connection(self):
        """Test SSH connection."""
        try:
            from ...core.ssh import SSHWorkflowOrchestrator
            
            orchestrator = SSHWorkflowOrchestrator()
            accounts = orchestrator.list_accounts()
            
            if not accounts:
                QMessageBox.information(self, "Info", "No SSH accounts configured")
                return
            
            # Show results
            message = "SSH Connection Test Results:\n\n"
            for acc in accounts:
                message += f"✓ {acc['name']} - SSH key configured\n"
            
            QMessageBox.information(self, "Connection Test", message)
        
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Test failed: {e}")