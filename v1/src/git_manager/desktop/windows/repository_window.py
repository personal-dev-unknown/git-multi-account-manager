# src/git_manager/desktop/windows/repository_window.py
"""Repository management widget."""

from PyQt6.QtWidgets import QWidget, QVBoxLayout, QLabel


class RepositoryWidget(QWidget):
    """Repository management widget."""
    
    def __init__(self, git_operations):
        super().__init__()
        self.git_operations = git_operations
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout(self)
        label = QLabel("Repository Management")
        layout.addWidget(label)