# src/git_manager/desktop/windows/account_window.py
"""Account management widget."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QPushButton,
    QTableWidget, QTableWidgetItem, QDialog, QFormLayout,
    QLineEdit, QComboBox, QMessageBox
)


class AccountWidget(QWidget):
    """Account management widget."""
    
    def __init__(self, account_manager):
        super().__init__()
        self.account_manager = account_manager
        self.init_ui()
        self.load_accounts()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout(self)
        
        # Toolbar
        toolbar = QHBoxLayout()
        
        add_btn = QPushButton("Add Account")
        add_btn.clicked.connect(self.add_account)
        toolbar.addWidget(add_btn)
        
        refresh_btn = QPushButton("Refresh")
        refresh_btn.clicked.connect(self.load_accounts)
        toolbar.addWidget(refresh_btn)
        
        toolbar.addStretch()
        
        layout.addLayout(toolbar)
        
        # Table
        self.table = QTableWidget()
        self.table.setColumnCount(5)
        self.table.setHorizontalHeaderLabels(
            ['Name', 'Platform', 'Username', 'Email', 'Actions']
        )
        layout.addWidget(self.table)
    
    def load_accounts(self):
        """Load accounts into table."""
        accounts = self.account_manager.list_accounts()
        self.table.setRowCount(len(accounts))
        
        for i, account in enumerate(accounts):
            self.table.setItem(i, 0, QTableWidgetItem(account.name))
            self.table.setItem(i, 1, QTableWidgetItem(account.platform.value))
            self.table.setItem(i, 2, QTableWidgetItem(account.username))
            self.table.setItem(i, 3, QTableWidgetItem(account.email))
            
            # Actions
            actions_widget = QWidget()
            actions_layout = QHBoxLayout(actions_widget)
            
            test_btn = QPushButton("Test")
            test_btn.clicked.connect(lambda checked, n=account.name: self.test_account(n))
            actions_layout.addWidget(test_btn)
            
            delete_btn = QPushButton("Delete")
            delete_btn.clicked.connect(lambda checked, n=account.name: self.delete_account(n))
            actions_layout.addWidget(delete_btn)
            
            self.table.setCellWidget(i, 4, actions_widget)
    
    def add_account(self):
        """Show add account dialog."""
        dialog = AddAccountDialog(self.account_manager, self)
        if dialog.exec():
            self.load_accounts()
    
    def test_account(self, name):
        """Test account connection."""
        QMessageBox.information(self, "Test", f"Testing {name}...")
    
    def delete_account(self, name):
        """Delete account."""
        reply = QMessageBox.question(
            self, 'Delete Account',
            f'Are you sure you want to delete account {name}?',
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        
        if reply == QMessageBox.StandardButton.Yes:
            try:
                self.account_manager.remove_account(name)
                self.load_accounts()
                QMessageBox.information(self, "Success", f"Account {name} deleted")
            except Exception as e:
                QMessageBox.critical(self, "Error", str(e))


class AddAccountDialog(QDialog):
    """Dialog for adding accounts."""
    
    def __init__(self, account_manager, parent=None):
        super().__init__(parent)
        self.account_manager = account_manager
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        self.setWindowTitle("Add Account")
        layout = QFormLayout(self)
        
        self.name_input = QLineEdit()
        self.platform_input = QComboBox()
        self.platform_input.addItems(['github', 'gitlab'])
        self.username_input = QLineEdit()
        self.email_input = QLineEdit()
        self.ssh_key_input = QLineEdit()
        
        layout.addRow("Name:", self.name_input)
        layout.addRow("Platform:", self.platform_input)
        layout.addRow("Username:", self.username_input)
        layout.addRow("Email:", self.email_input)
        layout.addRow("SSH Key:", self.ssh_key_input)
        
        # Buttons
        buttons = QHBoxLayout()
        save_btn = QPushButton("Save")
        save_btn.clicked.connect(self.save)
        cancel_btn = QPushButton("Cancel")
        cancel_btn.clicked.connect(self.reject)
        
        buttons.addWidget(save_btn)
        buttons.addWidget(cancel_btn)
        layout.addRow(buttons)
    
    def save(self):
        """Save account."""
        from ...models.account import Platform
        
        try:
            self.account_manager.add_account(
                name=self.name_input.text(),
                platform=Platform(self.platform_input.currentText()),
                username=self.username_input.text(),
                email=self.email_input.text(),
                ssh_key_path=self.ssh_key_input.text()
            )
            self.accept()
        except Exception as e:
            QMessageBox.critical(self, "Error", str(e))