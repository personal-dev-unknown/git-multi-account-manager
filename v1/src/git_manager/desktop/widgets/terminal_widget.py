# src/git_manager/desktop/widgets/terminal_widget.py
"""Terminal widget for executing commands."""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QTextEdit, QLineEdit,
    QPushButton, QHBoxLayout
)
from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QTextCursor, QFont
import subprocess


class TerminalWidget(QWidget):
    """Widget for terminal emulation."""
    
    command_executed = pyqtSignal(str, str)  # command, output
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.history: List[str] = []
        self.history_index = 0
        self.init_ui()
    
    def init_ui(self):
        """Initialize UI."""
        layout = QVBoxLayout(self)
        
        # Output area
        self.output = QTextEdit()
        self.output.setReadOnly(True)
        self.output.setFont(QFont("Courier", 10))
        self.output.setStyleSheet("background-color: #1e1e1e; color: #d4d4d4;")
        layout.addWidget(self.output)
        
        # Input area
        input_layout = QHBoxLayout()
        
        self.prompt_label = QPushButton("$")
        self.prompt_label.setEnabled(False)
        self.prompt_label.setFixedWidth(30)
        input_layout.addWidget(self.prompt_label)
        
        self.input = QLineEdit()
        self.input.setPlaceholderText("Enter command...")
        self.input.returnPressed.connect(self.execute_command)
        input_layout.addWidget(self.input)
        
        self.execute_btn = QPushButton("Execute")
        self.execute_btn.clicked.connect(self.execute_command)
        input_layout.addWidget(self.execute_btn)
        
        layout.addLayout(input_layout)
        
        # Initial message
        self.append_output("Git Manager Terminal\n", "info")
        self.append_output("Type 'help' for available commands\n\n", "info")
    
    def execute_command(self):
        """Execute the entered command."""
        command = self.input.text().strip()
        if not command:
            return
        
        # Add to history
        self.history.append(command)
        self.history_index = len(self.history)
        
        # Display command
        self.append_output(f"$ {command}\n", "command")
        
        # Execute based on command type
        if command.startswith("git-manager"):
            self.execute_git_manager_command(command)
        elif command.startswith("git"):
            self.execute_git_command(command)
        elif command == "clear":
            self.output.clear()
        elif command == "help":
            self.show_help()
        else:
            self.execute_shell_command(command)
        
        # Clear input
        self.input.clear()
    
    def execute_git_manager_command(self, command: str):
        """Execute git-manager command."""
        self.append_output("Executing Git Manager command...\n", "info")
        # Integration with core managers would go here
        self.append_output("Command completed\n\n", "success")
    
    def execute_git_command(self, command: str):
        """Execute git command."""
        try:
            result = subprocess.run(
                command.split(),
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.stdout:
                self.append_output(result.stdout, "output")
            if result.stderr:
                self.append_output(result.stderr, "error")
            
            self.command_executed.emit(command, result.stdout + result.stderr)
        except Exception as e:
            self.append_output(f"Error: {e}\n", "error")
    
    def execute_shell_command(self, command: str):
        """Execute shell command."""
        try:
            result = subprocess.run(
                command,
                shell=True,
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.stdout:
                self.append_output(result.stdout, "output")
            if result.stderr:
                self.append_output(result.stderr, "error")
            
            self.command_executed.emit(command, result.stdout + result.stderr)
        except Exception as e:
            self.append_output(f"Error: {e}\n", "error")
    
    def show_help(self):
        """Show help message."""
        help_text = """
Available Commands:
  git-manager account list    - List all accounts
  git-manager status          - Check repository status
  git <command>              - Execute git command
  clear                      - Clear terminal
  help                       - Show this help
  
Any other command will be executed as shell command
"""
        self.append_output(help_text, "info")
    
    def append_output(self, text: str, style: str = "output"):
        """Append text to output area.
        
        Args:
            text: Text to append
            style: Style type (command, output, error, info, success)
        """
        colors = {
            "command": "#569cd6",  # Blue
            "output": "#d4d4d4",   # White
            "error": "#f44747",    # Red
            "info": "#4ec9b0",     # Cyan
            "success": "#4ec9b0"   # Green
        }
        
        color = colors.get(style, colors["output"])
        
        cursor = self.output.textCursor()
        cursor.movePosition(QTextCursor.MoveOperation.End)
        cursor.insertHtml(f'<span style="color: {color};">{text}</span>')
        self.output.setTextCursor(cursor)
        self.output.ensureCursorVisible()
    
    def keyPressEvent(self, event):
        """Handle key press events."""
        if event.key() == Qt.Key.Key_Up:
            # Navigate history up
            if self.history_index > 0:
                self.history_index -= 1
                self.input.setText(self.history[self.history_index])
        elif event.key() == Qt.Key.Key_Down:
            # Navigate history down
            if self.history_index < len(self.history) - 1:
                self.history_index += 1
                self.input.setText(self.history[self.history_index])
            elif self.history_index == len(self.history) - 1:
                self.history_index = len(self.history)
                self.input.clear()
        else:
            super().keyPressEvent(event)