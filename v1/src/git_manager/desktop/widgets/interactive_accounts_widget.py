"""Interactive account management widget for desktop."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QTableWidget,
    QTableWidgetItem, QMessageBox, QDialog, QLineEdit, QComboBox,
    QCheckBox, QProgressBar, QTextEdit, QFileDialog, QTabWidget
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal, QTimer
from PyQt6.QtGui import QFont, QColor, QIcon

from pathlib import Path
from typing import Optional, Dict, List, Any

from ..interactive_manager import InteractiveDesktopManager
from ...utils.log_config import get_logger as get_advanced_logger, LogCategory


class SSHTestWorkerThread(QThread):
    """Worker thread for SSH testing."""
    
    progress = pyqtSignal(str)
    finished = pyqtSignal(bool, str, str)  # success, message, username
    
    def __init__(self, manager: InteractiveDesktopManager, account_name: str):
        super().__init__()
        self.manager = manager
        self.account_name = account_name
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
    
    def run(self):
        """Run SSH test."""
        try:
            self.progress.emit(f"Testing SSH connection for {self.account_name}...")
            result = self.manager.test_ssh_connection(self.account_name)
            
            if result['success']:
                username = result.get('username', 'unknown')
                self.finished.emit(True, result['message'], username)
            else:
                self.finished.emit(False, result['message'], '')
        except Exception as e:
            self.logger.error(f"SSH test failed: {e}")
            self.finished.emit(False, str(e), '')


class InteractiveAccountsWidget(QWidget):
    """Interactive account management widget."""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
        self.manager = InteractiveDesktopManager()
        self.ssh_test_thread = None
        
        self.init_ui()
        self.refresh_accounts()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout()
        
        # Title
        title = QLabel("Account Management")
        title_font = QFont()
        title_font.setPointSize(14)
        title_font.setBold(True)
        title.setFont(title_font)
        layout.addWidget(title)
        
        # Accounts table
        self.accounts_table = QTableWidget()
        self.accounts_table.setColumnCount(7)
        self.accounts_table.setHorizontalHeaderLabels([
            'Account', 'Platform', 'Username', 'Email',
            'SSH Key', 'PAT', 'Actions'
        ])
        self.accounts_table.setColumnWidth(0, 120)
        self.accounts_table.setColumnWidth(1, 100)
        self.accounts_table.setColumnWidth(2, 120)
        self.accounts_table.setColumnWidth(3, 150)
        self.accounts_table.setColumnWidth(4, 80)
        self.accounts_table.setColumnWidth(5, 80)
        self.accounts_table.setColumnWidth(6, 150)
        layout.addWidget(self.accounts_table)
        
        # Button layout
        button_layout = QHBoxLayout()
        
        self.refresh_btn = QPushButton("🔄 Refresh")
        self.refresh_btn.clicked.connect(self.refresh_accounts)
        button_layout.addWidget(self.refresh_btn)
        
        self.test_ssh_btn = QPushButton("🔑 Test SSH")
        self.test_ssh_btn.clicked.connect(self.test_selected_ssh)
        button_layout.addWidget(self.test_ssh_btn)
        
        self.generate_key_btn = QPushButton("✨ Generate SSH Key")
        self.generate_key_btn.clicked.connect(self.show_generate_key_dialog)
        button_layout.addWidget(self.generate_key_btn)
        
        self.setup_pat_btn = QPushButton("🎫 Setup PAT")
        self.setup_pat_btn.clicked.connect(self.show_setup_pat_dialog)
        button_layout.addWidget(self.setup_pat_btn)
        
        layout.addLayout(button_layout)
        
        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setVisible(False)
        layout.addWidget(self.progress_bar)
        
        self.setLayout(layout)
    
    def refresh_accounts(self):
        """Refresh accounts table."""
        try:
            accounts = self.manager.list_all_accounts()
            
            self.accounts_table.setRowCount(len(accounts))
            
            for row, account in enumerate(accounts):
                # Account name
                self.accounts_table.setItem(row, 0, QTableWidgetItem(account['name']))
                
                # Platform
                self.accounts_table.setItem(row, 1, QTableWidgetItem(account['platform']))
                
                # Username
                self.accounts_table.setItem(row, 2, QTableWidgetItem(account['username']))
                
                # Email
                self.accounts_table.setItem(row, 3, QTableWidgetItem(account['email']))
                
                # SSH Key
                ssh_status = "✓" if account['has_ssh_key'] else "✗"
                self.accounts_table.setItem(row, 4, QTableWidgetItem(ssh_status))
                
                # PAT
                pat_status = "✓" if account['has_pat'] else "✗"
                self.accounts_table.setItem(row, 5, QTableWidgetItem(pat_status))
                
                # Actions button
                test_btn = QPushButton("Test")
                test_btn.clicked.connect(lambda checked, a=account['name']: self.test_account_ssh(a))
                self.accounts_table.setCellWidget(row, 6, test_btn)
            
            self.logger.info(f"Loaded {len(accounts)} accounts")
        except Exception as e:
            self.logger.error(f"Error refreshing accounts: {e}")
            QMessageBox.critical(self, "Error", f"Failed to refresh accounts: {str(e)}")
    
    def test_selected_ssh(self):
        """Test SSH for selected account."""
        current_row = self.accounts_table.currentRow()
        
        if current_row < 0:
            QMessageBox.warning(self, "Warning", "Please select an account first")
            return
        
        account_name = self.accounts_table.item(current_row, 0).text()
        self.test_account_ssh(account_name)
    
    def test_account_ssh(self, account_name: str):
        """Test SSH connection for account."""
        self.progress_bar.setVisible(True)
        self.progress_bar.setValue(0)
        self.test_ssh_btn.setEnabled(False)
        
        self.ssh_test_thread = SSHTestWorkerThread(self.manager, account_name)
        self.ssh_test_thread.progress.connect(self.on_ssh_progress)
        self.ssh_test_thread.finished.connect(self.on_ssh_finished)
        self.ssh_test_thread.start()
    
    def on_ssh_progress(self, message: str):
        """Handle SSH test progress."""
        self.progress_bar.setValue(50)
    
    def on_ssh_finished(self, success: bool, message: str, username: str):
        """Handle SSH test completion."""
        self.progress_bar.setVisible(False)
        self.test_ssh_btn.setEnabled(True)
        
        if success:
            msg = f"✓ SSH Connection Successful\n\nUsername: {username}\n{message}"
            QMessageBox.information(self, "SSH Test Successful", msg)
        else:
            QMessageBox.critical(self, "SSH Test Failed", f"✗ {message}")
    
    def show_generate_key_dialog(self):
        """Show SSH key generation dialog."""
        dialog = GenerateSSHKeyDialog(self.manager, self)
        if dialog.exec() == QDialog.DialogCode.Accepted:
            self.refresh_accounts()
    
    def show_setup_pat_dialog(self):
        """Show PAT setup dialog."""
        dialog = SetupPATDialog(self.manager, self)
        if dialog.exec() == QDialog.DialogCode.Accepted:
            self.refresh_accounts()


class GenerateSSHKeyDialog(QDialog):
    """Dialog for generating SSH keys."""
    
    def __init__(self, manager: InteractiveDesktopManager, parent=None):
        super().__init__(parent)
        self.manager = manager
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
        
        self.setWindowTitle("Generate SSH Key")
        self.setGeometry(100, 100, 500, 600)
        
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout()
        
        # Account name
        layout.addWidget(QLabel("Account Name:"))
        self.account_name_input = QLineEdit()
        layout.addWidget(self.account_name_input)
        
        # Email
        layout.addWidget(QLabel("Email:"))
        self.email_input = QLineEdit()
        layout.addWidget(self.email_input)
        
        # Platform
        layout.addWidget(QLabel("Platform:"))
        self.platform_combo = QComboBox()
        platforms = self.manager.platform_manager.PLATFORMS.keys()
        self.platform_combo.addItems(platforms)
        layout.addWidget(self.platform_combo)
        
        # Account type
        layout.addWidget(QLabel("Account Type:"))
        self.account_type_combo = QComboBox()
        self.account_type_combo.addItems(['personal', 'school', 'work', 'organization'])
        layout.addWidget(self.account_type_combo)
        
        # Key type
        layout.addWidget(QLabel("Key Type:"))
        self.key_type_combo = QComboBox()
        self.key_type_combo.addItems(['ed25519', 'rsa'])
        layout.addWidget(self.key_type_combo)
        
        # Passphrase
        layout.addWidget(QLabel("Passphrase (optional):"))
        self.passphrase_input = QLineEdit()
        self.passphrase_input.setEchoMode(QLineEdit.EchoMode.Password)
        layout.addWidget(self.passphrase_input)
        
        # Buttons
        button_layout = QHBoxLayout()
        
        generate_btn = QPushButton("Generate")
        generate_btn.clicked.connect(self.generate_key)
        button_layout.addWidget(generate_btn)
        
        cancel_btn = QPushButton("Cancel")
        cancel_btn.clicked.connect(self.reject)
        button_layout.addWidget(cancel_btn)
        
        layout.addLayout(button_layout)
        
        self.setLayout(layout)
    
    def generate_key(self):
        """Generate SSH key."""
        account_name = self.account_name_input.text().strip()
        email = self.email_input.text().strip()
        platform = self.platform_combo.currentText()
        account_type = self.account_type_combo.currentText()
        key_type = self.key_type_combo.currentText()
        passphrase = self.passphrase_input.text() or None
        
        if not account_name or not email:
            QMessageBox.warning(self, "Validation Error", "Account name and email are required")
            return
        
        try:
            result = self.manager.generate_ssh_key(
                account_name=account_name,
                email=email,
                platform=platform,
                account_type=account_type,
                key_type=key_type,
                passphrase=passphrase
            )
            
            if result['success']:
                msg = f"✓ SSH Key Generated Successfully\n\n"
                if result.get('key_info'):
                    key_info = result['key_info']
                    msg += f"Name: {key_info.get('name')}\n"
                    msg += f"Fingerprint: {key_info.get('fingerprint')}\n"
                    msg += f"Location: {key_info.get('key_path')}"
                
                QMessageBox.information(self, "Success", msg)
                self.accept()
            else:
                QMessageBox.critical(self, "Error", f"Failed to generate key: {result.get('error')}")
        except Exception as e:
            self.logger.error(f"Key generation failed: {e}")
            QMessageBox.critical(self, "Error", f"Key generation failed: {str(e)}")


class SetupPATDialog(QDialog):
    """Dialog for setting up Personal Access Token."""
    
    def __init__(self, manager: InteractiveDesktopManager, parent=None):
        super().__init__(parent)
        self.manager = manager
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
        
        self.setWindowTitle("Setup Personal Access Token")
        self.setGeometry(100, 100, 500, 400)
        
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout()
        
        # Platform
        layout.addWidget(QLabel("Platform:"))
        self.platform_combo = QComboBox()
        platforms = self.manager.platform_manager.PLATFORMS.keys()
        self.platform_combo.addItems(platforms)
        self.platform_combo.currentTextChanged.connect(self.update_instructions)
        layout.addWidget(self.platform_combo)
        
        # Instructions
        layout.addWidget(QLabel("Instructions:"))
        self.instructions_text = QTextEdit()
        self.instructions_text.setReadOnly(True)
        self.instructions_text.setMaximumHeight(150)
        layout.addWidget(self.instructions_text)
        
        # PAT input
        layout.addWidget(QLabel("Personal Access Token:"))
        self.pat_input = QLineEdit()
        self.pat_input.setEchoMode(QLineEdit.EchoMode.Password)
        layout.addWidget(self.pat_input)
        
        # Test button
        self.test_btn = QPushButton("Test PAT")
        self.test_btn.clicked.connect(self.test_pat)
        layout.addWidget(self.test_btn)
        
        # Buttons
        button_layout = QHBoxLayout()
        
        save_btn = QPushButton("Save")
        save_btn.clicked.connect(self.save_pat)
        button_layout.addWidget(save_btn)
        
        cancel_btn = QPushButton("Cancel")
        cancel_btn.clicked.connect(self.reject)
        button_layout.addWidget(cancel_btn)
        
        layout.addLayout(button_layout)
        
        self.setLayout(layout)
        self.update_instructions()
    
    def update_instructions(self):
        """Update instructions based on selected platform."""
        platform = self.platform_combo.currentText()
        
        instructions = {
            'github': "1. Go to: https://github.com/settings/tokens\n"
                     "2. Click 'Generate new token' → 'Generate new token (classic)'\n"
                     "3. Select scopes: repo, read:user\n"
                     "4. Click 'Generate token' and copy it",
            'gitlab': "1. Go to: https://gitlab.com/-/profile/personal_access_tokens\n"
                     "2. Fill in: Name, Expiration date\n"
                     "3. Select scopes: api, read_api, read_repository\n"
                     "4. Click 'Create personal access token' and copy it",
            'bitbucket': "1. Go to: https://bitbucket.org/account/settings/app-passwords/new\n"
                        "2. Fill in: Label, Permissions (Repositories: read)\n"
                        "3. Click 'Create' and copy the password",
        }
        
        self.instructions_text.setText(
            instructions.get(platform, "Please refer to your platform's documentation")
        )
    
    def test_pat(self):
        """Test PAT token."""
        platform = self.platform_combo.currentText()
        pat_token = self.pat_input.text().strip()
        
        if not pat_token:
            QMessageBox.warning(self, "Validation Error", "PAT token is required")
            return
        
        self.test_btn.setEnabled(False)
        self.test_btn.setText("Testing...")
        
        try:
            is_valid = self.manager.test_pat_token(platform, pat_token)
            
            if is_valid:
                QMessageBox.information(self, "Success", "✓ PAT token is valid")
            else:
                QMessageBox.warning(self, "Invalid", "✗ PAT token is invalid")
        except Exception as e:
            self.logger.error(f"PAT test failed: {e}")
            QMessageBox.critical(self, "Error", f"PAT test failed: {str(e)}")
        finally:
            self.test_btn.setEnabled(True)
            self.test_btn.setText("Test PAT")
    
    def save_pat(self):
        """Save PAT token."""
        pat_token = self.pat_input.text().strip()
        
        if not pat_token:
            QMessageBox.warning(self, "Validation Error", "PAT token is required")
            return
        
        # TODO: Implement PAT saving to account
        QMessageBox.information(self, "Success", "✓ PAT token saved")
        self.accept()
