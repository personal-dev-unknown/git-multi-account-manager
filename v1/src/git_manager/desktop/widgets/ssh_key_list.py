# src/git_manager/desktop/widgets/ssh_key_list.py
"""SSH key list widget."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QTableWidget, QTableWidgetItem,
    QHeaderView, QLabel
)
from PyQt6.QtCore import Qt, pyqtSignal
from typing import List


class SSHKeyListWidget(QWidget):
    """Widget for displaying SSH key list."""
    
    key_selected = pyqtSignal(str)  # Emits key name
    key_double_clicked = pyqtSignal(str)
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.keys: List = []
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout(self)
        
        # Info label
        self.info_label = QLabel("No SSH keys found")
        layout.addWidget(self.info_label)
        
        # Table
        self.table = QTableWidget()
        self.table.setColumnCount(4)
        self.table.setHorizontalHeaderLabels([
            'Name', 'Type', 'Path', 'Fingerprint'
        ])
        self.table.horizontalHeader().setSectionResizeMode(QHeaderView.ResizeMode.Stretch)
        self.table.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)
        self.table.itemSelectionChanged.connect(self.on_selection_changed)
        self.table.itemDoubleClicked.connect(self.on_double_clicked)
        
        layout.addWidget(self.table)
    
    def set_keys(self, keys: List):
        """Set SSH keys to display.
        
        Args:
            keys: List of SSHKey objects
        """
        self.keys = keys
        self.refresh()
    
    def refresh(self):
        """Refresh the table display."""
        self.table.setRowCount(len(self.keys))
        
        if not self.keys:
            self.info_label.setText("No SSH keys found")
            return
        
        self.info_label.setText(f"{len(self.keys)} SSH keys")
        
        for i, key in enumerate(self.keys):
            self.table.setItem(i, 0, QTableWidgetItem(key.name))
            self.table.setItem(i, 1, QTableWidgetItem(key.key_type.value))
            self.table.setItem(i, 2, QTableWidgetItem(str(key.private_key_path)))
            self.table.setItem(i, 3, QTableWidgetItem("SHA256:..."))  # Placeholder
    
    def get_selected_key(self) -> str:
        """Get selected key name.
        
        Returns:
            Key name or empty string
        """
        selected = self.table.selectedItems()
        if selected:
            row = selected[0].row()
            return self.table.item(row, 0).text()
        return ""
    
    def on_selection_changed(self):
        """Handle selection change."""
        key_name = self.get_selected_key()
        if key_name:
            self.key_selected.emit(key_name)
    
    def on_double_clicked(self, item):
        """Handle double click."""
        key_name = self.table.item(item.row(), 0).text()
        self.key_double_clicked.emit(key_name)