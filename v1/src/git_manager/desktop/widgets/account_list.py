# src/git_manager/desktop/widgets/account_list.py
"""Account list widget."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QTableWidget, QTableWidgetItem,
    QPushButton, QHBoxLayout, QHeaderView
)
from PyQt6.QtCore import Qt, pyqtSignal
from typing import List

from ...models.account import Account


class AccountListWidget(QWidget):
    """Widget for displaying account list."""
    
    account_selected = pyqtSignal(str)  # Emits account name
    account_double_clicked = pyqtSignal(str)
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.accounts: List[Account] = []
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout(self)
        
        # Table
        self.table = QTableWidget()
        self.table.setColumnCount(4)
        self.table.setHorizontalHeaderLabels(['Name', 'Platform', 'Username', 'Email'])
        self.table.horizontalHeader().setSectionResizeMode(QHeaderView.ResizeMode.Stretch)
        self.table.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)
        self.table.setSelectionMode(QTableWidget.SelectionMode.SingleSelection)
        self.table.itemSelectionChanged.connect(self.on_selection_changed)
        self.table.itemDoubleClicked.connect(self.on_double_clicked)
        
        layout.addWidget(self.table)
    
    def set_accounts(self, accounts: List[Account]):
        """Set accounts to display.
        
        Args:
            accounts: List of Account objects
        """
        self.accounts = accounts
        self.refresh()
    
    def refresh(self):
        """Refresh the table display."""
        self.table.setRowCount(len(self.accounts))
        
        for i, account in enumerate(self.accounts):
            self.table.setItem(i, 0, QTableWidgetItem(account.name))
            self.table.setItem(i, 1, QTableWidgetItem(account.platform.value))
            self.table.setItem(i, 2, QTableWidgetItem(account.username))
            self.table.setItem(i, 3, QTableWidgetItem(account.email))
    
    def get_selected_account(self) -> str:
        """Get selected account name.
        
        Returns:
            Account name or empty string
        """
        selected = self.table.selectedItems()
        if selected:
            row = selected[0].row()
            return self.table.item(row, 0).text()
        return ""
    
    def on_selection_changed(self):
        """Handle selection change."""
        account_name = self.get_selected_account()
        if account_name:
            self.account_selected.emit(account_name)
    
    def on_double_clicked(self, item):
        """Handle double click."""
        account_name = self.table.item(item.row(), 0).text()
        self.account_double_clicked.emit(account_name)