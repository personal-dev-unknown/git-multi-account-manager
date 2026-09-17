# src/git_manager/cli/ui/tables.py
"""Rich table displays."""

from rich.table import Table
from rich.console import Console
from typing import List

from ...models.account import Account
from ...models.repository import RepositoryStatus
from .color_schemes import DEFAULT_SCHEME


def display_accounts_table(accounts: List[Account], console: Console):
    """Display accounts in a table."""
    if not accounts:
        console.print(f"[{DEFAULT_SCHEME.warning}]No accounts configured[/{DEFAULT_SCHEME.warning}]")
        return
    
    scheme = DEFAULT_SCHEME
    table = Table(title="Git Accounts", show_header=True, header_style=f"bold {scheme.primary}")
    table.add_column("Name", style=scheme.success, no_wrap=True)
    table.add_column("Platform", style=scheme.primary)
    table.add_column("Username", style=scheme.accent)
    table.add_column("Email", style=scheme.text)
    table.add_column("Description", style=scheme.text)
    
    for account in accounts:
        table.add_row(
            account.name,
            account.platform.value,
            account.username,
            account.email,
            account.description or "-"
        )
    
    console.print(table)


def display_account_details(account: Account, console: Console):
    """Display detailed account information."""
    scheme = DEFAULT_SCHEME
    console.print(f"\n[{scheme.secondary}]Account: {account.name}[/{scheme.secondary}]")
    console.print(f"Platform: [{scheme.primary}]{account.platform.value}[/{scheme.primary}]")
    console.print(f"Username: [{scheme.accent}]{account.username}[/{scheme.accent}]")
    console.print(f"Email: [{scheme.text}]{account.email}[/{scheme.text}]")
    console.print(f"SSH Host: [{scheme.success}]{account.host}[/{scheme.success}]")
    console.print(f"SSH Key: [{scheme.text}]{account.ssh_key_path}[/{scheme.text}]")
    console.print(f"Description: [{scheme.text}]{account.description or '-'}[/{scheme.text}]\n")


def display_repository_status(status: RepositoryStatus, console: Console):
    """Display repository status."""
    scheme = DEFAULT_SCHEME
    console.print(f"\n[{scheme.secondary}]Repository Status[/{scheme.secondary}]")
    console.print(f"Branch: [{scheme.success}]{status.current_branch}[/{scheme.success}]")
    
    if status.is_clean:
        console.print(f"[{scheme.success}]✓ Working tree clean[/{scheme.success}]")
    else:
        console.print(f"[{scheme.warning}]⚠ Uncommitted changes detected[/{scheme.warning}]")
        console.print(f"Changed files: [{scheme.text}]{len(status.uncommitted_files)}[/{scheme.text}]")
    
    if status.is_synced:
        console.print(f"[{scheme.success}]✓ Up to date with remote[/{scheme.success}]")
    else:
        if status.commits_ahead > 0:
            console.print(f"[{scheme.info}]↑ {status.commits_ahead} commits ahead[/{scheme.info}]")
        if status.commits_behind > 0:
            console.print(f"[{scheme.warning}]↓ {status.commits_behind} commits behind[/{scheme.warning}]")
    
    console.print()


def display_config(console: Console):
    """Display configuration."""
    scheme = DEFAULT_SCHEME
    table = Table(title="Configuration", show_header=True, header_style=f"bold {scheme.primary}")
    table.add_column("Setting", style=scheme.success)
    table.add_column("Value", style=scheme.accent)
    
    # Add configuration items
    from ...utils.constants import CONFIG_DIR, LOG_FILE
    
    table.add_row("Config Directory", str(CONFIG_DIR))
    table.add_row("Log File", str(LOG_FILE))
    
    console.print(table)