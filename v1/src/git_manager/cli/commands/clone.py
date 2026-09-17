# src/git_manager/cli/commands/clone.py
"""Clone command."""

import click
from rich.progress import Progress, SpinnerColumn, TextColumn
from pathlib import Path


@click.command()
@click.argument('url')
@click.option('--account', '-a', required=True, help='Account to use')
@click.option('--destination', '-d', type=click.Path(), help='Clone destination')
@click.option('--branch', '-b', help='Branch to clone')
@click.option('--personal', is_flag=True, help='Clone from personal repositories')
@click.pass_context
def clone(ctx, url, account, destination, branch, personal):
    """Clone a repository."""
    console = ctx.obj['console']
    git_ops = ctx.obj['git_operations']
    
    if personal:
        # Show personal repositories and let user select
        from ..ui.prompts import select_personal_repository
        url = select_personal_repository(
            ctx.obj['account_manager'],
            account,
            console
        )
        if not url:
            return
    
    dest = Path(destination) if destination else None
    
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console
    ) as progress:
        task = progress.add_task(f"[accent]Cloning {url}...", total=None)
        
        try:
            repo = git_ops.clone(url, account, dest, branch)
            progress.update(task, completed=True)
            console.print(f"\n[success]✓ Successfully cloned to {repo.path}[/success]")
        except Exception as e:
            progress.update(task, completed=True)
            console.print(f"\n[error]✗ Clone failed: {e}[/error]")