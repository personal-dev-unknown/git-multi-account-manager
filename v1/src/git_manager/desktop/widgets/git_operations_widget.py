"""Git Operations Widget - Desktop UI for push/pull/sync operations."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QTabWidget, QPushButton, QLabel,
    QComboBox, QTextEdit, QMessageBox, QProgressBar, QTableWidget, QTableWidgetItem
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal
from PyQt6.QtGui import QFont
from pathlib import Path

from ...core.sync import (
    PushOperations, PullOperations, SyncOperations,
    GitStatus, GitBranch, GitStage, GitCommit
)
from ...utils.logger import get_logger
from ...utils.log_config import LogCategory, get_logger as get_advanced_logger


logger = get_advanced_logger(__name__, category=LogCategory.GIT_OPERATION)


class GitOperationWorker(QThread):
    """Worker thread for git operations."""
    
    finished = pyqtSignal()
    error = pyqtSignal(str)
    result = pyqtSignal(dict)
    
    def __init__(self, operation, repo_path, strategy=None):
        super().__init__()
        self.operation = operation
        self.repo_path = repo_path
        self.strategy = strategy
    
    def run(self):
        """Run git operation in background."""
        try:
            repo_path = Path(self.repo_path)
            
            if self.operation == 'push':
                push_ops = PushOperations()
                if self.strategy == 'safe':
                    result = push_ops.safe_push(repo_path)
                elif self.strategy == 'force_lease':
                    result = push_ops.push_with_lease(repo_path)
                elif self.strategy == 'dry_run':
                    result = push_ops.dry_run_push(repo_path)
                else:
                    result = push_ops.safe_push(repo_path)
                
            elif self.operation == 'pull':
                pull_ops = PullOperations()
                if self.strategy == 'safe':
                    result = pull_ops.safe_pull(repo_path)
                elif self.strategy == 'smart':
                    result = pull_ops.smart_pull(repo_path)
                elif self.strategy == 'rebase':
                    result = pull_ops.pull_rebase(repo_path)
                else:
                    result = pull_ops.safe_pull(repo_path)
                
            elif self.operation == 'sync':
                sync_ops = SyncOperations()
                if self.strategy == 'smart':
                    result = sync_ops.smart_sync(repo_path)
                elif self.strategy == 'conservative':
                    result = sync_ops.conservative_sync(repo_path)
                elif self.strategy == 'rebase':
                    result = sync_ops.rebase_sync(repo_path)
                else:
                    result = sync_ops.smart_sync(repo_path)
                
            elif self.operation == 'status':
                status_ops = GitStatus()
                status = status_ops.get_status(repo_path)
                result = {
                    'success': True,
                    'message': f"Branch: {status.branch}, Local: +{status.local_ahead}, Remote: +{status.remote_ahead}",
                    'details': {
                        'branch': status.branch,
                        'local_ahead': status.local_ahead,
                        'remote_ahead': status.remote_ahead,
                        'uncommitted': status.uncommitted_count
                    }
                }
            
            else:
                raise ValueError(f"Unknown operation: {self.operation}")
            
            self.result.emit({
                'success': result.success if hasattr(result, 'success') else True,
                'message': result.message if hasattr(result, 'message') else str(result),
                'details': result.details if hasattr(result, 'details') else {}
            })
            
        except Exception as e:
            logger.error(f"Error in git operation: {e}")
            self.error.emit(str(e))
        finally:
            self.finished.emit()


class GitOperationsWidget(QWidget):
    """Widget for git push/pull/sync operations."""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.repo_path = str(Path.cwd())
        self.worker = None
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout()
        
        # Repository path
        path_layout = QHBoxLayout()
        path_layout.addWidget(QLabel("Repository:"))
        path_label = QLabel(self.repo_path)
        path_label.setWordWrap(True)
        path_layout.addWidget(path_label)
        layout.addLayout(path_layout)
        
        # Tab widget
        tabs = QTabWidget()
        
        # Push tab
        tabs.addTab(self.create_push_tab(), "Push")
        
        # Pull tab
        tabs.addTab(self.create_pull_tab(), "Pull")
        
        # Sync tab
        tabs.addTab(self.create_sync_tab(), "Sync")
        
        # Status tab
        tabs.addTab(self.create_status_tab(), "Status")
        
        # Branches tab
        tabs.addTab(self.create_branches_tab(), "Branches")
        
        layout.addWidget(tabs)
        
        # Progress bar
        self.progress = QProgressBar()
        self.progress.setVisible(False)
        layout.addWidget(self.progress)
        
        # Result display
        self.result_text = QTextEdit()
        self.result_text.setReadOnly(True)
        self.result_text.setMaximumHeight(150)
        layout.addWidget(QLabel("Result:"))
        layout.addWidget(self.result_text)
        
        self.setLayout(layout)
    
    def create_push_tab(self):
        """Create push tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Strategy selection
        strategy_layout = QHBoxLayout()
        strategy_layout.addWidget(QLabel("Strategy:"))
        self.push_strategy = QComboBox()
        self.push_strategy.addItems(["Safe Push", "Force Lease", "Dry Run"])
        strategy_layout.addWidget(self.push_strategy)
        layout.addLayout(strategy_layout)
        
        # Buttons
        button_layout = QHBoxLayout()
        push_btn = QPushButton("Execute Push")
        push_btn.clicked.connect(self.execute_push)
        button_layout.addWidget(push_btn)
        layout.addLayout(button_layout)
        
        layout.addStretch()
        widget.setLayout(layout)
        return widget
    
    def create_pull_tab(self):
        """Create pull tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Strategy selection
        strategy_layout = QHBoxLayout()
        strategy_layout.addWidget(QLabel("Strategy:"))
        self.pull_strategy = QComboBox()
        self.pull_strategy.addItems(["Safe Pull", "Smart Pull", "Rebase"])
        strategy_layout.addWidget(self.pull_strategy)
        layout.addLayout(strategy_layout)
        
        # Buttons
        button_layout = QHBoxLayout()
        pull_btn = QPushButton("Execute Pull")
        pull_btn.clicked.connect(self.execute_pull)
        button_layout.addWidget(pull_btn)
        layout.addLayout(button_layout)
        
        layout.addStretch()
        widget.setLayout(layout)
        return widget
    
    def create_sync_tab(self):
        """Create sync tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Strategy selection
        strategy_layout = QHBoxLayout()
        strategy_layout.addWidget(QLabel("Strategy:"))
        self.sync_strategy = QComboBox()
        self.sync_strategy.addItems(["Smart Sync", "Conservative", "Rebase"])
        strategy_layout.addWidget(self.sync_strategy)
        layout.addLayout(strategy_layout)
        
        # Buttons
        button_layout = QHBoxLayout()
        sync_btn = QPushButton("Execute Sync")
        sync_btn.clicked.connect(self.execute_sync)
        button_layout.addWidget(sync_btn)
        layout.addLayout(button_layout)
        
        layout.addStretch()
        widget.setLayout(layout)
        return widget
    
    def create_status_tab(self):
        """Create status tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Buttons
        button_layout = QHBoxLayout()
        status_btn = QPushButton("Refresh Status")
        status_btn.clicked.connect(self.get_status)
        button_layout.addWidget(status_btn)
        layout.addLayout(button_layout)
        
        # Status display
        self.status_table = QTableWidget()
        self.status_table.setColumnCount(2)
        self.status_table.setHorizontalHeaderLabels(["Property", "Value"])
        layout.addWidget(self.status_table)
        
        widget.setLayout(layout)
        return widget
    
    def create_branches_tab(self):
        """Create branches tab."""
        widget = QWidget()
        layout = QVBoxLayout()
        
        # Buttons
        button_layout = QHBoxLayout()
        refresh_btn = QPushButton("Refresh Branches")
        refresh_btn.clicked.connect(self.list_branches)
        button_layout.addWidget(refresh_btn)
        layout.addLayout(button_layout)
        
        # Branches display
        self.branches_table = QTableWidget()
        self.branches_table.setColumnCount(2)
        self.branches_table.setHorizontalHeaderLabels(["Branch", "Current"])
        layout.addWidget(self.branches_table)
        
        widget.setLayout(layout)
        return widget
    
    def execute_push(self):
        """Execute push operation."""
        strategy_map = {
            "Safe Push": "safe",
            "Force Lease": "force_lease",
            "Dry Run": "dry_run"
        }
        strategy = strategy_map.get(self.push_strategy.currentText(), "safe")
        self.run_operation("push", strategy)
    
    def execute_pull(self):
        """Execute pull operation."""
        strategy_map = {
            "Safe Pull": "safe",
            "Smart Pull": "smart",
            "Rebase": "rebase"
        }
        strategy = strategy_map.get(self.pull_strategy.currentText(), "safe")
        self.run_operation("pull", strategy)
    
    def execute_sync(self):
        """Execute sync operation."""
        strategy_map = {
            "Smart Sync": "smart",
            "Conservative": "conservative",
            "Rebase": "rebase"
        }
        strategy = strategy_map.get(self.sync_strategy.currentText(), "smart")
        self.run_operation("sync", strategy)
    
    def get_status(self):
        """Get repository status."""
        self.run_operation("status")
    
    def list_branches(self):
        """List branches."""
        try:
            branch_ops = GitBranch()
            branches = branch_ops.list_branches(Path(self.repo_path))
            
            self.branches_table.setRowCount(len(branches))
            for row, branch in enumerate(branches):
                self.branches_table.setItem(row, 0, QTableWidgetItem(branch.name))
                current = "✓" if branch.is_current else ""
                self.branches_table.setItem(row, 1, QTableWidgetItem(current))
            
            self.result_text.setText(f"Loaded {len(branches)} branches")
        except Exception as e:
            logger.error(f"Error listing branches: {e}")
            self.result_text.setText(f"Error: {str(e)}")
    
    def run_operation(self, operation, strategy=None):
        """Run git operation in background."""
        if self.worker and self.worker.isRunning():
            QMessageBox.warning(self, "In Progress", "Operation already in progress")
            return
        
        self.progress.setVisible(True)
        self.progress.setValue(0)
        
        self.worker = GitOperationWorker(operation, self.repo_path, strategy)
        self.worker.result.connect(self.on_operation_result)
        self.worker.error.connect(self.on_operation_error)
        self.worker.finished.connect(self.on_operation_finished)
        self.worker.start()
    
    def on_operation_result(self, result):
        """Handle operation result."""
        success = result.get('success', False)
        message = result.get('message', '')
        details = result.get('details', {})
        
        if success:
            self.result_text.setText(f"✓ {message}")
            if details:
                self.result_text.append("\nDetails:")
                for key, value in details.items():
                    self.result_text.append(f"  {key}: {value}")
        else:
            self.result_text.setText(f"✗ {message}")
    
    def on_operation_error(self, error):
        """Handle operation error."""
        self.result_text.setText(f"✗ Error: {error}")
        QMessageBox.critical(self, "Error", error)
    
    def on_operation_finished(self):
        """Handle operation finished."""
        self.progress.setVisible(False)
