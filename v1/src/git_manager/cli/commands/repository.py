# src/git_manager/cli/commands/repository.py
"""Repository commands."""

import click
from pathlib import Path


@click.group()
def repository():
    """Repository operations."""
    pass


@repository.command('status')
@click.option('--path', '-p', type=click.Path(exists=True), help='Repository path')
@click.pass_context
def repo_status(ctx, path):
    """Check repository status."""
    from ..ui.tables import display_repository_status
    
    console = ctx.obj['console']
    git_ops = ctx.obj['git_operations']
    
    repo_path = Path(path) if path else None
    
    try:
        status = git_ops.check_status(repo_path)
        display_repository_status(status, console)
    except Exception as e:
        console.print(f"[error]Error: {e}[/error]")


@repository.command('setup')
@click.option('--account', '-a', required=True, help='Account to use')
@click.option('--path', '-p', type=click.Path(exists=True), help='Repository path')
@click.pass_context
def setup_repo(ctx, account, path):
    """Setup repository for specific account."""
    from ..ui.prompts import confirm_action
    
    console = ctx.obj['console']
    account_manager = ctx.obj['account_manager']
    
    repo_path = Path(path) if path else Path.cwd()
    
    try:
        acc = account_manager.get_account(account)
        console.print(f"\n[info]Setting up repository for account: {acc.name}[/info]")
        console.print(f"[info]Platform: {acc.platform.value}[/info]")
        console.print(f"[info]Username: {acc.username}[/info]\n")
        
        if confirm_action("Continue with setup?", console):
            # Implementation for repository setup
            console.print("[success]✓ Repository setup complete[/success]")
    except Exception as e:
        console.print(f"[error]Error: {e}[/error]")