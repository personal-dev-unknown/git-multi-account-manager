# src/git_manager/cli/ui/prompts.py
"""Interactive prompts."""

from rich.console import Console
from rich.prompt import Prompt, Confirm
from rich.table import Table
from typing import Optional, List, Dict, Any
import logging

from .color_schemes import DEFAULT_SCHEME


def confirm_action(message: str, console: Console) -> bool:
    """Confirm an action."""
    scheme = DEFAULT_SCHEME
    return Confirm.ask(f"[{scheme.accent}]{message}[/{scheme.accent}]")


def select_option(options: List[str], message: str, console: Console) -> Optional[str]:
    """Select from options."""
    scheme = DEFAULT_SCHEME
    console.print(f"\n[{scheme.accent}]{message}[/{scheme.accent}]")
    for i, option in enumerate(options, 1):
        console.print(f"[{scheme.info}]{i}.[/{scheme.info}] {option}")
    
    choice = Prompt.ask(
        "Select option",
        choices=[str(i) for i in range(1, len(options) + 1)]
    )
    return options[int(choice) - 1]


def select_personal_repository(
    account_manager,
    account_name: str,
    console: Console,
    clone_workflow=None,
    progress_manager=None
) -> Optional[Dict[str, Any]]:
    """
    Select a personal repository from any of 8 supported platforms.
    
    Supports all 8 platforms:
    - GitHub, GitLab, Bitbucket, Azure DevOps
    - Self-Hosted, Cloud Storage, Local Path, SourceForge
    
    Features:
    - Loading indicator while fetching repositories
    - Rich table display with repository metadata
    - Retry logic for network failures
    - Comprehensive error handling
    
    Args:
        account_manager: Account manager instance
        account_name: Name of the account to fetch repositories for
        console: Rich console for output
        clone_workflow: Clone workflow instance for fetching repositories
        progress_manager: Progress manager for loading indicators
    
    Returns:
        Dictionary with repository details or None if cancelled/error
    """
    logger = logging.getLogger(__name__)
    scheme = DEFAULT_SCHEME
    
    try:
        # Get account details
        account = account_manager.get_account(account_name)
        if not account:
            console.print(f"[{scheme.error}]✗ Account '{account_name}' not found[/{scheme.error}]")
            return None
        
        platform = account.platform.value
        
        # Validate dependencies
        if not clone_workflow:
            console.print(f"[{scheme.error}]✗ Clone workflow not available[/{scheme.error}]")
            return None
        
        # Initialize progress manager if not provided
        if not progress_manager:
            from .progress import ProgressManager
            progress_manager = ProgressManager(console)
        
        # Fetch repositories with loading indicator
        repositories = progress_manager.fetch_repositories(
            func=clone_workflow.fetch_repositories,
            account_name=account_name,
            platform=platform,
            account=account
        )
        
        if not repositories:
            return None
        
        # Display repositories in table
        console.print(f"\n[{scheme.accent}]Repositories for {account_name}[/{scheme.accent}]")
        
        repo_table = Table(border_style=scheme.primary, show_header=True, header_style=f"bold {scheme.primary}")
        repo_table.add_column("ID", style=scheme.success)
        repo_table.add_column("Name", style=scheme.text)
        repo_table.add_column("Description", style=scheme.text)
        repo_table.add_column("Visibility", style=scheme.warning)
        repo_table.add_column("Language", style=scheme.secondary)
        
        for idx, repo in enumerate(repositories, 1):
            name = repo.get('name', 'N/A')
            description = repo.get('description', '')[:40]  # Truncate long descriptions
            visibility = repo.get('visibility', 'N/A')
            language = repo.get('language', 'N/A')
            
            repo_table.add_row(
                str(idx),
                name,
                description,
                visibility,
                language
            )
        
        console.print(repo_table)
        
        # Selection prompt
        repo_choice = Prompt.ask(
            f"\n[{scheme.primary}]Select repository[/{scheme.primary}]",
            choices=[str(i) for i in range(1, len(repositories) + 1)],
            default='1'
        )
        
        selected_repo = repositories[int(repo_choice) - 1]
        
        # Display selected repository details
        console.print(f"\n[{scheme.accent}]═══ Repository Selected ═══[/{scheme.accent}]")
        console.print(f"[{scheme.success}]✓ Name: {selected_repo.get('name')}[/{scheme.success}]")
        
        if selected_repo.get('description'):
            console.print(f"[{scheme.info}]📝 Description: {selected_repo['description']}[/{scheme.info}]")
        
        if selected_repo.get('url'):
            console.print(f"[{scheme.info}]🔗 URL: {selected_repo['url']}[/{scheme.info}]")
        
        if selected_repo.get('clone_url'):
            console.print(f"[{scheme.info}]📦 Clone URL: {selected_repo['clone_url']}[/{scheme.info}]")
        
        if selected_repo.get('visibility'):
            console.print(f"[{scheme.info}]👁️  Visibility: {selected_repo['visibility']}[/{scheme.info}]")
        
        if selected_repo.get('language'):
            console.print(f"[{scheme.info}]💻 Language: {selected_repo['language']}[/{scheme.info}]")
        
        return selected_repo
    
    except KeyboardInterrupt:
        console.print(f"\n[{scheme.warning}]⚠️  Operation cancelled by user[/{scheme.warning}]")
        return None
    except Exception as e:
        console.print(f"[{scheme.error}]✗ Error selecting repository: {str(e)}[/{scheme.error}]")
        logger.error(f"Repository selection failed: {e}")
        return None