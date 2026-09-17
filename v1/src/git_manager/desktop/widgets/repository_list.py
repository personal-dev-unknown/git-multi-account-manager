# src/git_manager/desktop/widgets/repository_list.py
"""Repository list widget."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QTableWidget, QTableWidgetItem,
    QHeaderView, QLabel
)
from PyQt6.QtCore import Qt, pyqtSignal
from typing import List
from pathlib import Path


class RepositoryListWidget(QWidget):
    """Widget for displaying repository list."""
    
    repository_selected = pyqtSignal(str)  # Emits repository path
    repository_double_clicked = pyqtSignal(str)
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.repositories: List = []
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout(self)
        
        # Info label
        self.info_label = QLabel("No repositories tracked")
        layout.addWidget(self.info_label)
        
        # Table
        self.table = QTableWidget()
        self.table.setColumnCount(5)
        self.table.setHorizontalHeaderLabels([
            'Name', 'Path', 'Branch', 'Account', 'Status'
        ])
        self.table.horizontalHeader().setSectionResizeMode(QHeaderView.ResizeMode.Stretch)
        self.table.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)
        self.table.itemSelectionChanged.connect(self.on_selection_changed)
        self.table.itemDoubleClicked.connect(self.on_double_clicked)
        
        layout.addWidget(self.table)
    
    def set_repositories(self, repositories: List):
        """Set repositories to display.
        
        Args:
            repositories: List of Repository objects
        """
        self.repositories = repositories
        self.refresh()
    
    def refresh(self):
        """Refresh the table display."""
        self.table.setRowCount(len(self.repositories))
        
        if not self.repositories:
            self.info_label.setText("No repositories tracked")
            return
        
        self.info_label.setText(f"{len(self.repositories)} repositories")
        
        for i, repo in enumerate(self.repositories):
            self.table.setItem(i, 0, QTableWidgetItem(repo.name))
            self.table.setItem(i, 1, QTableWidgetItem(str(repo.path)))
            self.table.setItem(i, 2, QTableWidgetItem(repo.branch))
            self.table.setItem(i, 3, QTableWidgetItem(repo.account.name))
            self.table.setItem(i, 4, QTableWidgetItem("Clean"))  # Placeholder
    
    def get_selected_repository(self) -> str:
        """Get selected repository path.
        
        Returns:
            Repository path or empty string
        """
        selected = self.table.selectedItems()
        if selected:
            row = selected[0].row()
            return self.table.item(row, 1).text()
        return ""
    
    def on_selection_changed(self):
        """Handle selection change."""
        repo_path = self.get_selected_repository()
        if repo_path:
            self.repository_selected.emit(repo_path)
    
    def on_double_clicked(self, item):
        """Handle double click."""
        repo_path = self.table.item(item.row(), 1).text()
        self.repository_double_clicked.emit(repo_path)