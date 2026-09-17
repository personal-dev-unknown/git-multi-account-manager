# src/git_manager/cli/commands/logs.py
"""Commands for managing and viewing logs."""

import click
import sys
import os
import glob as glob_module
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from pathlib import Path
from datetime import datetime

from ...utils.log_config import LogStorageManager, LogCategory
from ...utils.log_utils import get_log_directory_info, list_log_files


@click.group()
def logs():
    """Manage application logs."""
    # Ensure sys.argv is all strings (fix for Path objects)
    sys.argv = [str(arg) for arg in sys.argv]
    pass


@logs.command()
@click.pass_context
def info(ctx):
    """Show log storage information."""
    console = ctx.obj.get('console', Console())
    
    storage_info = get_log_directory_info()
    
    panel_content = f"""
[primary]Log Directory:[/primary] {storage_info['log_directory']}
[primary]Installation Type:[/primary] {'System-wide' if storage_info['is_system_wide'] else 'User'}
[primary]Available Space:[/primary] {storage_info['available_space_mb']:.2f} MB
    """
    
    panel = Panel(panel_content, title="Log Storage Info", border_style="primary")
    console.print(panel)


@logs.command()
@click.pass_context
def list(ctx):
    """List all log files."""
    console = ctx.obj.get('console', Console())
    
    log_files = list_log_files()
    
    if not log_files:
        console.print("[warning]No log files found[/warning]")
        return
    
    table = Table(title="Log Files", border_style="primary")
    table.add_column("File Name", style="accent")
    table.add_column("Size (KB)", justify="right", style="info")
    table.add_column("Last Modified", style="secondary")
    
    for log_file in log_files:
        modified = datetime.fromtimestamp(log_file['modified']).strftime('%Y-%m-%d %H:%M:%S')
        table.add_row(
            log_file['name'],
            f"{log_file['size_kb']:.2f}",
            modified,
        )
    
    console.print(table)


@logs.command()
@click.option('--category', type=click.Choice([c.value for c in LogCategory]), help='Filter by category')
@click.option('--lines', default=50, help='Number of lines to show')
@click.pass_context
def view(ctx, category, lines):
    """View log file contents."""
    console = ctx.obj.get('console', Console())
    
    log_dir = LogStorageManager.get_log_directory()
    
    if category:
        log_file = log_dir / f"{category}.log"
    else:
        # Show activity log by default
        log_file = log_dir / f"{LogCategory.ACTIVITY.value}.log"
    
    if not log_file.exists():
        console.print(f"[error]Log file not found: {log_file}[/error]")
        return
    
    try:
        with open(log_file, 'r') as f:
            all_lines = f.readlines()
        
        # Get last N lines
        recent_lines = all_lines[-lines:] if len(all_lines) > lines else all_lines
        
        content = ''.join(recent_lines)
        
        panel = Panel(
            content,
            title=f"Log: {log_file.name} (last {len(recent_lines)} lines)",
            border_style="primary",
            expand=False,
        )
        console.print(panel)
    
    except Exception as e:
        console.print(f"[error]Error reading log file: {e}[/error]")


@logs.command()
@click.option('--category', type=click.Choice([c.value for c in LogCategory]), help='Clear specific category')
@click.confirmation_option(prompt='Are you sure you want to clear logs?')
@click.pass_context
def clear(ctx, category):
    """Clear log files."""
    console = ctx.obj.get('console', Console())
    
    log_dir = LogStorageManager.get_log_directory()
    
    if category:
        log_files = [log_dir / f"{category}.log"]
    else:
        log_files = list(log_dir.glob("*.log*"))
    
    cleared_count = 0
    for log_file in log_files:
        try:
            if log_file.exists():
                log_file.unlink()
                cleared_count += 1
                console.print(f"[success]✓ Cleared: {log_file.name}[/success]")
        except Exception as e:
            console.print(f"[error]✗ Failed to clear {log_file.name}: {e}[/error]")
    
    console.print(f"\n[info]Cleared {cleared_count} log file(s)[/info]")


@logs.command()
@click.option('--output', type=str, default=None, help='Output file path')
@click.pass_context
def export(ctx, output):
    """Export all logs to a file."""
    try:
        console = ctx.obj.get('console', Console()) if ctx.obj else Console()
        
        log_dir = LogStorageManager.get_log_directory()
        # Use glob module instead of Path.glob() to avoid Click issues
        log_dir_str = str(log_dir)
        log_files_paths = glob_module.glob(os.path.join(log_dir_str, "*.log*"))
        log_files = [Path(f) for f in log_files_paths]
        
        if not log_files:
            console.print("[warning]No log files to export[/warning]")
            return
        
        if not output:
            output = f"git-manager-logs-{datetime.now().strftime('%Y%m%d_%H%M%S')}.txt"
        
        output_path = str(Path(output))
        # Convert to string to avoid Path object issues
        output_path_str = str(output_path)
        
        with open(output_path_str, 'w') as out_file:
            for log_file in sorted(log_files, key=lambda x: x.name):
                out_file.write(f"\n{'='*80}\n")
                out_file.write(f"File: {log_file.name}\n")
                out_file.write(f"{'='*80}\n\n")
                
                with open(str(log_file), 'r') as in_file:
                    out_file.write(in_file.read())
                
                out_file.write(f"\n\n")
        
        console.print(f"[success]✓ Logs exported to: {output_path_str}[/success]")
    
    except Exception as e:
        console = ctx.obj.get('console', Console()) if ctx.obj else Console()
        console.print(f"[error]✗ Failed to export logs: {e}[/error]")


@logs.command()
@click.pass_context
def categories(ctx):
    """Show available log categories."""
    console = ctx.obj.get('console', Console())
    
    table = Table(title="Log Categories", border_style="primary")
    table.add_column("Category", style="accent")
    table.add_column("Description", style="info")
    
    descriptions = {
        LogCategory.ACTIVITY.value: "General application operations",
        LogCategory.ERROR.value: "Errors and exceptions",
        LogCategory.SECURITY.value: "Authentication and account operations",
        LogCategory.PERFORMANCE.value: "Performance metrics and timing",
        LogCategory.GIT_OPERATION.value: "Git-specific operations",
        LogCategory.SSH_OPERATION.value: "SSH-specific operations",
        LogCategory.AUDIT.value: "Audit trail for important actions",
    }
    
    for category in LogCategory:
        table.add_row(
            category.value,
            descriptions.get(category.value, ""),
        )
    
    console.print(table)
