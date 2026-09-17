# src/git_manager/cli/commands/account.py
"""Account management commands."""

import click
from rich.table import Table
from rich.panel import Panel


@click.group()
def account():
    """Manage Git accounts."""
    pass


@account.command('platforms')
@click.pass_context
def list_platforms(ctx):
    """List all supported Git platforms."""
    from ...core.platform_config import PlatformManager
    
    console = ctx.obj['console']
    
    platforms = PlatformManager.list_platforms()
    
    table = Table(title="Supported Git Platforms", show_header=True, border_style="cyan")
    table.add_column("Platform", style="cyan", width=20)
    table.add_column("Name", style="bold")
    table.add_column("SSH", style="green")
    table.add_column("HTTPS", style="green")
    table.add_column("PAT", style="yellow")
    table.add_column("Description", style="dim")
    
    for platform in platforms:
        ssh_support = "✓" if platform.supports_ssh else "✗"
        https_support = "✓" if platform.supports_https else "✗"
        pat_support = "✓" if platform.supports_pat else "✗"
        
        table.add_row(
            platform.key,
            platform.name,
            ssh_support,
            https_support,
            pat_support,
            platform.description
        )
    
    console.print(table)


@account.command('list')
@click.option('--platform', type=click.Choice(['github', 'gitlab', 'bitbucket', 'azure_devops', 'self_hosted', 'cloud_storage', 'local_path', 'sourceforge']), help='Filter by platform')
@click.pass_context
def list_accounts(ctx, platform):
    """List all accounts."""
    from ..ui.tables import display_accounts_table
    from ...models.account import Platform
    
    console = ctx.obj['console']
    account_manager = ctx.obj['account_manager']
    
    platform_filter = Platform(platform) if platform else None
    accounts = account_manager.list_accounts(platform_filter)
    
    display_accounts_table(accounts, console)


@account.command('add')
@click.option('--name', '-n', required=True, help='Account name')
@click.option('--platform', '-p', type=click.Choice(['github', 'gitlab', 'bitbucket', 'azure_devops', 'self_hosted', 'cloud_storage', 'local_path', 'sourceforge']), required=True)
@click.option('--username', '-u', required=True, help='Platform username')
@click.option('--ssh-key', '-k', required=True, help='SSH key path')
@click.option('--host', '-h', help='Custom SSH host')
@click.option('--email', '-e', help='Email address (optional, captured during SSH key generation)')
@click.option('--description', '-d', help='Account description')
@click.pass_context
def add_account(ctx, name, platform, username, ssh_key, host, email, description):
    """Add a new account."""
    from ...models.account import Platform
    
    console = ctx.obj['console']
    account_manager = ctx.obj['account_manager']
    
    try:
        account = account_manager.add_account(
            name=name,
            platform=Platform(platform),
            username=username,
            email=email,
            ssh_key_path=ssh_key,
            host=host,
            description=description
        )
        console.print(f"[success]✓ Added account: {account.name}[/success]")
    except Exception as e:
        console.print(f"[error]✗ Failed to add account: {e}[/error]")


@account.command('remove')
@click.argument('name')
@click.confirmation_option(prompt='Are you sure you want to delete this account?')
@click.pass_context
def remove_account(ctx, name):
    """Remove an account."""
    console = ctx.obj['console']
    account_manager = ctx.obj['account_manager']
    
    try:
        account_manager.remove_account(name)
        console.print(f"[success]✓ Removed account: {name}[/success]")
    except Exception as e:
        console.print(f"[error]✗ Failed to remove account: {e}[/error]")


@account.command('show')
@click.argument('name')
@click.pass_context
def show_account(ctx, name):
    """Show account details."""
    from ..ui.tables import display_account_details
    
    console = ctx.obj['console']
    account_manager = ctx.obj['account_manager']
    
    try:
        account = account_manager.get_account(name)
        display_account_details(account, console)
    except Exception as e:
        console.print(f"[error]✗ Account not found: {e}[/error]")


@account.command('update')
@click.argument('name')
@click.option('--email', '-e', help='Update email address')
@click.option('--username', '-u', help='Update username')
@click.option('--description', '-d', help='Update description')
@click.pass_context
def update_account(ctx, name, email, username, description):
    """Update account details."""
    console = ctx.obj['console']
    account_manager = ctx.obj['account_manager']
    
    try:
        updates = {}
        if email:
            updates['email'] = email
        if username:
            updates['username'] = username
        if description:
            updates['description'] = description
        
        if not updates:
            console.print("[warning]No updates specified[/warning]")
            return
        
        account = account_manager.update_account(name, **updates)
        console.print(f"[success]✓ Updated account: {account.name}[/success]")
    except Exception as e:
        console.print(f"[error]✗ Failed to update account: {e}[/error]")