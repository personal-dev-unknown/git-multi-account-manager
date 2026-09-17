# src/git_manager/cli/app.py
"""Main CLI application."""

import sys
import click
import logging
from rich.console import Console
from rich.theme import Theme
from pathlib import Path

# Monkeypatch sys.argv to ensure all elements are strings
sys.argv = [str(arg) for arg in sys.argv]

from ..core.account_manager import AccountManager
from ..core.ssh import SSHWorkflowOrchestrator
from ..core.git_operations import GitOperations
from ..core.config_manager import ConfigManager
from ..core.database_manager import DatabaseManager
from ..utils.constants import COLORS, APP_NAME, APP_VERSION
from ..utils.logger import get_logger
from ..utils.log_config import initialize_logging, LogLevel, LogCategory, get_logger as get_advanced_logger
from .commands import clone, account, repository, ssh, config, logs, theme #, git

# Setup Rich console with custom theme
custom_theme = Theme({
    "primary": COLORS['primary'],
    "secondary": COLORS['secondary'],
    "accent": COLORS['accent'],
    "success": COLORS['success'],
    "warning": COLORS['warning'],
    "error": COLORS['error'],
    "info": COLORS['info']
})

console = Console(theme=custom_theme)


@click.group()
@click.version_option(version=APP_VERSION)
@click.option('--debug', is_flag=True, help='Enable debug mode')
@click.option('--json-logs', is_flag=True, help='Use JSON format for logs')
@click.pass_context
def cli(ctx, debug, json_logs):
    """Git Multi-Account Manager - Manage multiple Git accounts with ease."""
    # Ensure context object exists
    ctx.ensure_object(dict)
    
    # Initialize advanced logging system
    log_level = LogLevel.DEBUG if debug else LogLevel.INFO
    initialize_logging(
        log_level=log_level,
        use_json=json_logs,
        enable_console=True,
    )
    
    # Get logger for CLI
    logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
    logger.info(f"Starting {APP_NAME} v{APP_VERSION}")
    
    if debug:
        logger.debug("Debug mode enabled")
    
    # Initialize managers
    ctx.obj['database_manager'] = DatabaseManager()
    ctx.obj['account_manager'] = AccountManager()
    ctx.obj['ssh_orchestrator'] = SSHWorkflowOrchestrator()
    ctx.obj['git_operations'] = GitOperations(ctx.obj['account_manager'])
    ctx.obj['config_manager'] = ConfigManager()
    ctx.obj['console'] = console
    ctx.obj['logger'] = logger


# Register command groups
cli.add_command(clone.clone)
cli.add_command(account.account)
cli.add_command(repository.repository)
cli.add_command(ssh.ssh)
cli.add_command(config.config)
cli.add_command(logs.logs)
cli.add_command(theme.theme)
# cli.add_command(git.git)


@cli.command()
@click.pass_context
def interactive(ctx):
    """Start interactive mode."""
    from .ui.interactive import InteractiveMode
    
    console.print(f"\n[primary]{APP_NAME} v{APP_VERSION}[/primary]")
    console.print("[accent]Interactive Mode[/accent]\n")
    
    interactive_mode = InteractiveMode(
        ctx.obj['account_manager'],
        ctx.obj['ssh_orchestrator'],
        ctx.obj['git_operations'],
        console,
        config_manager=ctx.obj.get('config_manager'),
        database_manager=ctx.obj.get('database_manager')
    )
    interactive_mode.run()


@cli.command()
@click.pass_context
def status(ctx):
    """Check current repository status."""
    from .ui.tables import display_repository_status
    
    git_ops = ctx.obj['git_operations']
    try:
        status = git_ops.check_status()
        display_repository_status(status, console)
    except Exception as e:
        console.print(f"[error]Error: {e}[/error]")


if __name__ == '__main__':
    cli()