# src/git_manager/cli/ui/interactive.py
"""Interactive mode."""

from rich.console import Console
from rich.panel import Panel
from rich.prompt import Prompt
from typing import Optional
import re

from ...core.account_manager import AccountManager
from ...core.ssh import SSHWorkflowOrchestrator, SSHIntegrationLayer
from ...core.git_operations import GitOperations
from ...core.database_manager import DatabaseManager
from ...core.config_manager import ConfigManager
from ...core.platform_config import PlatformManager
from ...utils.log_config import LogCategory, get_logger as get_advanced_logger
from ...utils.log_utils import log_git_operation, log_ssh_operation, ContextLogger
from .color_schemes import (
    ColorScheme, ThemeType, DEFAULT_SCHEME, get_scheme,
    ALL_SCHEMES, list_schemes_by_type, get_all_scheme_names
)
from .theme_manager import ThemeManager


class InteractiveMode:
    """Interactive CLI mode."""
    
    def __init__(
        self,
        account_manager: AccountManager,
        ssh_orchestrator: SSHWorkflowOrchestrator,
        git_operations: GitOperations,
        console: Console,
        color_scheme: Optional[ColorScheme] = None,
        config_manager=None,
        database_manager=None
    ):
        self.account_manager = account_manager
        self.ssh_orchestrator = ssh_orchestrator
        self.git_operations = git_operations
        self.console = console
        self.color_scheme = color_scheme or DEFAULT_SCHEME
        self.config_manager = config_manager
        self.database_manager = database_manager
        self.logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)
        
        # Initialize SSH integration layer
        if database_manager and account_manager and config_manager:
            self.ssh_integration = SSHIntegrationLayer(database_manager, account_manager, config_manager)
        else:
            self.ssh_integration = None
    
    def run(self):
        """Run interactive mode."""
        theme_manager = ThemeManager()
        while True:
            # Reload theme on each iteration to pick up changes
            self.color_scheme = theme_manager.get_current_theme()
            
            self.show_menu()
            choice = Prompt.ask(
                "[primary]Select option[/primary]",
                choices=['1', '2', '3', '4', '5', '6', '7', '8'],
                default='8'
            )
            
            if choice == '8':
                self.console.print("[info]Goodbye![/info]")
                break
            
            self.handle_choice(choice)
    
    def show_menu(self):
        """Display main menu."""
        scheme = self.color_scheme
        menu = f"""
        [{scheme.accent}]═══ Repository Operations ═══[/{scheme.accent}]
        [{scheme.primary}][1][/{scheme.primary}] Clone a repository (8 platforms supported)
        [{scheme.primary}][2][/{scheme.primary}] Check current repository account
        [{scheme.primary}][3][/{scheme.primary}] Git push/pull/sync operations
        [{scheme.primary}][4][/{scheme.primary}] Set up repository for specific account

        [{scheme.accent}]═══ Account Management ═══[/{scheme.accent}]
        [{scheme.primary}][5][/{scheme.primary}] Show all accounts (8 platforms: GitHub, GitLab, Bitbucket, Azure DevOps, Self-Hosted, Cloud Storage, Local Path, SourceForge)
        [{scheme.primary}][6][/{scheme.primary}] Test SSH connections
        [{scheme.primary}][7][/{scheme.primary}] Generate and Manage SSH keys and PATs

        [{scheme.primary}][8][/{scheme.primary}] Exit
                """
        panel = Panel(menu, title="Main Menu", border_style=scheme.accent)
        self.console.print(panel)
    
    def handle_choice(self, choice: str):
        """Handle menu choice."""
        actions = {
            '1': self.clone_repo,
            '2': self.check_status,
            '3': self.git_push,
            '4': self.setup_repo,
            '5': self.list_accounts,
            '6': self.test_ssh,
            '7': self.generate_key
        }
        
        action = actions.get(choice)
        if action:
            try:
                action()
            except Exception as e:
                self.console.print(f"[error]Error: {e}[/error]")
        
        self.console.print()
    
    def clone_repo(self):
        """Clone repository with comprehensive workflow."""
        from rich.table import Table
        from rich.panel import Panel
        from pathlib import Path
        from ...core.clone import CloneWorkflow, URLParser
        
        with ContextLogger("clone_repository", category=LogCategory.GIT_OPERATION):
            try:
                # Initialize clone workflow
                clone_workflow = CloneWorkflow(self.account_manager, self.config_manager)
                
                # Step 1: Choose clone type
                self.console.print("\n[accent]═══ Clone Repository ═══[/accent]\n")
                self.console.print("[primary]Choose clone type:[/primary]")
                self.console.print("[1] Clone from external repository (any public/private repo)")
                self.console.print("[2] Clone from your personal repositories")
                self.console.print("[3] Back to main menu")
                
                clone_type = Prompt.ask("\nSelect option", choices=['1', '2', '3'], default='3')
                
                if clone_type == '3':
                    return
                
                # Handle external repository clone
                if clone_type == '1':
                    self._clone_external_repository(clone_workflow)
                
                # Handle personal repositories clone
                elif clone_type == '2':
                    self._clone_personal_repository(clone_workflow)
            
            except Exception as e:
                self.console.print(f"[error]✗ Error: {str(e)}[/error]")
                log_git_operation("clone", success=False, error_msg=str(e))
    
    def _handle_clone_error(self, error_msg: str, error_type: str = None):
        """
        Handle clone errors with helpful solutions.
        
        Args:
            error_msg: Error message from git/API
            error_type: Type of error ('auth', 'permission', 'not_found', 'network', 'disk', 'other')
        """
        from rich.panel import Panel
        
        # Auto-detect error type if not provided
        if not error_type:
            if 'permission' in error_msg.lower() or 'access denied' in error_msg.lower():
                error_type = 'permission'
            elif 'not found' in error_msg.lower() or '404' in error_msg:
                error_type = 'not_found'
            elif 'authentication' in error_msg.lower() or 'auth' in error_msg.lower() or '401' in error_msg or '403' in error_msg:
                error_type = 'auth'
            elif 'connection' in error_msg.lower() or 'network' in error_msg.lower() or 'timeout' in error_msg.lower():
                error_type = 'network'
            elif 'disk' in error_msg.lower() or 'space' in error_msg.lower():
                error_type = 'disk'
            else:
                error_type = 'other'
        
        # Display error with solutions
        if error_type == 'auth':
            self.console.print(Panel(
                "[error]❌ Authentication Failed[/error]\n\n"
                "[warning]Problem:[/warning] Could not authenticate with the Git platform\n\n"
                "[warning]Possible causes:[/warning]\n"
                "  • SSH key not added to your account\n"
                "  • SSH key has wrong permissions (should be 600)\n"
                "  • PAT is invalid or expired\n"
                "  • Wrong username/password\n\n"
                "[primary]💡 Solutions:[/primary]\n"
                "  [1] Test SSH connection: ssh -T git@github.com\n"
                "  [2] Add SSH key to your account settings\n"
                "  [3] Generate a new PAT with correct scopes\n"
                "  [4] Try a different authentication method",
                title="Authentication Error",
                border_style="red"
            ))
        
        elif error_type == 'permission':
            self.console.print(Panel(
                "[error]❌ Permission Denied[/error]\n\n"
                "[warning]Problem:[/warning] You don't have access to this repository\n\n"
                "[warning]Possible causes:[/warning]\n"
                "  • Repository is private and you're not a collaborator\n"
                "  • You're using the wrong account\n"
                "  • Organization requires SSO authentication\n\n"
                "[primary]💡 Solutions:[/primary]\n"
                "  [1] Request access from repository owner\n"
                "  [2] Check if you're using the correct account\n"
                "  [3] Enable SSO for your PAT if required\n"
                "  [4] Verify the repository URL is correct",
                title="Permission Error",
                border_style="red"
            ))
        
        elif error_type == 'not_found':
            self.console.print(Panel(
                "[error]❌ Repository Not Found[/error]\n\n"
                "[warning]Problem:[/warning] The repository could not be found\n\n"
                "[warning]Possible causes:[/warning]\n"
                "  • Repository doesn't exist\n"
                "  • Repository is private (appears as 'not found')\n"
                "  • URL is misspelled\n"
                "  • Repository was deleted or renamed\n\n"
                "[primary]💡 Solutions:[/primary]\n"
                "  [1] Verify URL on the Git platform website\n"
                "  [2] Check if you have access (may need authentication)\n"
                "  [3] Try with authentication enabled\n"
                "  [4] Search for the repository on the platform",
                title="Repository Not Found",
                border_style="red"
            ))
        
        elif error_type == 'network':
            self.console.print(Panel(
                "[error]❌ Network Error[/error]\n\n"
                "[warning]Problem:[/warning] Failed to connect to the Git platform\n\n"
                "[warning]Possible causes:[/warning]\n"
                "  • No internet connection\n"
                "  • Git platform is down\n"
                "  • Firewall blocking connection\n"
                "  • Proxy configuration needed\n\n"
                "[primary]💡 Solutions:[/primary]\n"
                "  [1] Check your internet connection\n"
                "  [2] Check platform status page\n"
                "  [3] Check firewall/proxy settings\n"
                "  [4] Try again in a moment",
                title="Network Error",
                border_style="red"
            ))
        
        elif error_type == 'disk':
            self.console.print(Panel(
                "[error]❌ Insufficient Disk Space[/error]\n\n"
                "[warning]Problem:[/warning] Not enough disk space to clone repository\n\n"
                "[primary]💡 Solutions:[/primary]\n"
                "  [1] Free up disk space\n"
                "  [2] Use shallow clone (--depth=1) to save space\n"
                "  [3] Clone to a different location with more space\n"
                "  [4] Use sparse checkout to clone specific folders",
                title="Disk Space Error",
                border_style="red"
            ))
        
        else:
            self.console.print(Panel(
                f"[error]❌ Clone Failed[/error]\n\n"
                f"[warning]Error:[/warning] {error_msg}",
                title="Clone Error",
                border_style="red"
            ))
    
    def _select_repository_with_pagination(self, repositories, account_name, platform=None):
        """
        Display repositories with pagination, search, and filter options.
        
        Supports all 8 platforms:
        - GitHub, GitLab, Bitbucket, Azure DevOps (with API repository lists)
        - Self-Hosted, Cloud Storage, Local Path, SourceForge (manual URL entry)
        
        Args:
            repositories: List of Repository objects (can be empty for platforms without API)
            account_name: Selected account name
            platform: Optional platform name for better UX messaging
        
        Returns:
            Selected Repository object or None
        """
        from rich.table import Table
        
        # Platforms that support API repository listing
        api_platforms = ['github', 'gitlab', 'bitbucket', 'azure_devops']
        
        # If no repositories and platform doesn't support API, go straight to manual URL
        if not repositories and platform and platform not in api_platforms:
            self.console.print(f"\n[warning]⚠️  {platform.replace('_', ' ').title()} doesn't support API repository listing[/warning]")
            self.console.print("[primary]Please enter the repository URL manually[/primary]")
            return self._handle_manual_url_input()
        
        # Filter state
        filtered_repos = repositories if repositories else []
        current_page = 0
        page_size = 10
        
        try:
            while True:
                # If no repositories, offer manual URL entry
                if not filtered_repos:
                    self.console.print(f"\n[warning]⚠️  No repositories found for {account_name}[/warning]")
                    self.console.print("\n[accent]Options:[/accent]")
                    self.console.print("[1] 🔗 Enter repository URL manually")
                    self.console.print("[2] ← Back to account selection")
                    
                    choice = Prompt.ask("\nSelect option", choices=['1', '2'], default='1')
                    
                    if choice == '1':
                        return self._handle_manual_url_input()
                    else:
                        return None
                
                # Calculate pagination
                total_pages = (len(filtered_repos) + page_size - 1) // page_size
                start_idx = current_page * page_size
                end_idx = start_idx + page_size
                page_repos = filtered_repos[start_idx:end_idx]
                
                # Display header
                self.console.print(f"\n[accent]📚 Your {account_name} repositories ({len(filtered_repos)} found)[/accent]")
                
                # Display table
                repo_table = Table(border_style="cyan", show_header=True)
                repo_table.add_column("ID", style="green")
                repo_table.add_column("Name", style="white")
                repo_table.add_column("Visibility", style="yellow")
                repo_table.add_column("Updated", style="white")
                repo_table.add_column("Description", style="white")
                
                for idx, repo in enumerate(page_repos, 1):
                    visibility_icon = "🔒" if hasattr(repo, 'visibility') and repo.visibility == 'private' else "🌍"
                    visibility = getattr(repo, 'visibility', 'unknown')
                    description = ""
                    if hasattr(repo, 'description') and repo.description:
                        description = (repo.description[:40] + "...") if len(repo.description) > 40 else repo.description
                    updated = ""
                    if hasattr(repo, 'updated_at') and repo.updated_at:
                        updated = repo.updated_at[:10] if len(repo.updated_at) >= 10 else repo.updated_at
                    
                    repo_table.add_row(
                        str(idx),
                        repo.name,
                        f"{visibility_icon} {visibility}",
                        updated,
                        description
                    )
                
                self.console.print(repo_table)
                
                # Display pagination info
                self.console.print(f"\n[accent]Showing {start_idx + 1}-{min(end_idx, len(filtered_repos))} of {len(filtered_repos)} repositories (Page {current_page + 1}/{total_pages})[/accent]")
                
                # Display options
                options = []
                if current_page > 0:
                    options.append("[P] Previous page")
                if current_page < total_pages - 1:
                    options.append("[N] Next page")
                options.extend(["[S] Search", "[F] Filter", "[M] Manual URL", "[B] Back"])
                
                self.console.print(f"\nOptions: {' | '.join(options)}")
                
                # Get user input
                valid_choices = [str(i) for i in range(1, len(page_repos) + 1)]
                if current_page > 0:
                    valid_choices.append('p')
                if current_page < total_pages - 1:
                    valid_choices.append('n')
                valid_choices.extend(['s', 'f', 'm', 'b'])
                
                choice = Prompt.ask(
                    "\n[primary]Select repository [1-10] or action [N/P/S/F/M/B][/primary]",
                    choices=valid_choices
                ).lower()
                
                # Handle pagination
                if choice == 'p' and current_page > 0:
                    current_page -= 1
                    continue
                elif choice == 'n' and current_page < total_pages - 1:
                    current_page += 1
                    continue
                
                # Handle search
                elif choice == 's':
                    search_term = Prompt.ask("[primary]Search repositories by name[/primary]").lower()
                    filtered_repos = [r for r in repositories if search_term in r.name.lower()]
                    current_page = 0
                    if not filtered_repos:
                        self.console.print("[warning]⚠️  No repositories found matching your search[/warning]")
                        filtered_repos = repositories
                    continue
                
                # Handle filter
                elif choice == 'f':
                    self.console.print("\n[accent]Filter options:[/accent]")
                    self.console.print("[1] Private repositories")
                    self.console.print("[2] Public repositories")
                    self.console.print("[3] All repositories")
                    filter_choice = Prompt.ask("\nSelect filter", choices=['1', '2', '3'], default='3')
                    
                    if filter_choice == '1':
                        filtered_repos = [r for r in repositories if hasattr(r, 'visibility') and r.visibility == 'private']
                    elif filter_choice == '2':
                        filtered_repos = [r for r in repositories if hasattr(r, 'visibility') and r.visibility == 'public']
                    else:
                        filtered_repos = repositories
                    
                    current_page = 0
                    if not filtered_repos:
                        self.console.print("[warning]⚠️  No repositories found with that filter[/warning]")
                        filtered_repos = repositories
                    continue
                
                # Handle manual URL
                elif choice == 'm':
                    return self._handle_manual_url_input()
                
                # Handle back
                elif choice == 'b':
                    return None
                
                # Handle repository selection
                else:
                    repo_idx = int(choice) - 1
                    return page_repos[repo_idx]
        
        except KeyboardInterrupt:
            self.console.print("\n[warning]⚠️  Operation cancelled by user[/warning]")
            return None
    
    def _handle_manual_url_input(self):
        """
        Handle manual URL input for all 8 platforms.
        
        Supports:
        - GitHub, GitLab, Bitbucket, Azure DevOps (with API validation)
        - Self-Hosted, Cloud Storage, Local Path, SourceForge (manual entry)
        
        Returns:
            Mock repository object with parsed URL or None
        """
        try:
            while True:
                manual_url = Prompt.ask("\n[primary]Enter repository URL[/primary]")
                
                # Validate URL format early
                is_valid, message, detected_platform = PlatformManager.validate_url(manual_url)
                
                if not is_valid:
                    self.console.print(f"[error]✗ {message}[/error]")
                    self.console.print("[warning]Please enter a valid repository URL[/warning]")
                    continue
                
                if detected_platform:
                    self.console.print(f"[green]✓ {message}[/green]")
                else:
                    self.console.print(f"[yellow]⚠ {message}[/yellow]")
                
                # Create a temporary repository object for manual URL
                try:
                    from git_manager.core.clone.ui.clone_url_parser import URLParser
                    parsed = URLParser.parse(manual_url)
                    
                    # Return a mock repository object
                    class ManualRepo:
                        def __init__(self, url, platform_name=None):
                            self.ssh_url = parsed.to_ssh()
                            self.https_url = parsed.to_https()
                            self.name = parsed.repo
                            self.visibility = 'unknown'
                            self.updated_at = ''
                            self.description = f'Manual URL ({platform_name or "Unknown"})' if platform_name else 'Manual URL'
                    
                    return ManualRepo(manual_url, detected_platform)
                
                except Exception as e:
                    self.console.print(f"[error]✗ Failed to parse URL: {str(e)}[/error]")
                    self.console.print("[warning]Please try again with a valid repository URL[/warning]")
                    continue
        except KeyboardInterrupt:
            self.console.print("\n[warning]⚠️  Operation cancelled by user[/warning]")
            return None
    
    def _test_pat_token(self, platform: str, pat_token: str) -> bool:
        """
        Test if a PAT token is valid by making a simple API call.
        
        Supports all platforms that have PAT capability:
        - GitHub
        - GitLab
        - Bitbucket
        - Azure DevOps
        
        Args:
            platform: Platform ID ('github', 'gitlab', 'bitbucket', 'azure_devops')
            pat_token: Personal Access Token to test
        
        Returns:
            True if token is valid, False otherwise
        """
        import requests
        
        try:
            if platform == 'github':
                # Test GitHub token by getting user info
                response = requests.get(
                    'https://api.github.com/user',
                    headers={
                        'Authorization': f'token {pat_token}',
                        'Accept': 'application/vnd.github.v3+json'
                    },
                    timeout=5
                )
                return response.status_code == 200
            
            elif platform == 'gitlab':
                # Test GitLab token by getting user info
                response = requests.get(
                    'https://gitlab.com/api/v4/user',
                    headers={'PRIVATE-TOKEN': pat_token},
                    timeout=5
                )
                return response.status_code == 200
            
            elif platform == 'bitbucket':
                # Test Bitbucket token by getting user info
                response = requests.get(
                    'https://api.bitbucket.org/2.0/user',
                    auth=('x-token-auth', pat_token),
                    timeout=5
                )
                return response.status_code == 200
            
            elif platform == 'azure_devops':
                # Test Azure DevOps token by getting user profile
                # Azure DevOps uses Basic Auth with PAT as password and empty username
                response = requests.get(
                    'https://dev.azure.com/_apis/profile/profiles/me',
                    auth=('', pat_token),
                    timeout=5
                )
                return response.status_code == 200
            
            else:
                # Platform doesn't support PAT or is unknown
                return False
        
        except Exception:
            return False
    
    def _clone_external_repository(self, clone_workflow):
        """Clone from external repository."""
        from rich.table import Table
        from rich.panel import Panel

        """
            Handle keyboard interrupts
        """
        try:
        
            # Step 1: Enter URL
            self.console.print("\n[accent]Step 1: Enter Repository URL[/accent]")
            self.console.print("[primary]Examples:[/primary]")
            self.console.print("  • github.com/user/repo")
            self.console.print("  • https://gitlab.com/user/project")
            self.console.print("  • git@bitbucket.org:user/repo.git")
            
            # Validate URL early with retry loop
            while True:
                while True:
                    repo_url = Prompt.ask("\n[primary]Repository URL[/primary]")
                    
                    is_valid, message, detected_platform = PlatformManager.validate_url(repo_url)
                    
                    if not is_valid:
                        self.console.print(f"[error]✗ {message}[/error]")
                        self.console.print("[warning]Please enter a valid repository URL[/warning]")
                        continue
                    
                    if detected_platform:
                        self.console.print(f"[green]✓ {message}[/green]")
                    else:
                        self.console.print(f"[yellow]⚠ {message}[/yellow]")
                    
                    break
                
                # Step 2: Analyze repository
                self.console.print("\n[accent]Step 2: Analyzing repository...[/accent]")
                analysis = clone_workflow.analyze_external_repository(repo_url)
                
                if not analysis['success']:
                    self.console.print(f"[error]✗ Invalid URL: {analysis['error']}[/error]")
                    
                    # Give user options to retry or go back
                    self.console.print("\n[warning]What would you like to do?[/warning]")
                    self.console.print("[1] 🔄 Try another URL")
                    self.console.print("[2] ← Back to main menu")
                    
                    choice = Prompt.ask("\nSelect option", choices=['1', '2'], default='1')
                    
                    if choice == '2':
                        return
                    # If choice == '1', loop continues to ask for URL again
                    continue
                
                # Analysis successful, break out of retry loop
                break
            
            parsed = analysis['parsed']
            platform = analysis['platform']
            
            # Display repository info
            info_table = Table(border_style="cyan", show_header=False)
            info_table.add_row("Platform", f"[primary]{platform.upper()}[/primary]")
            info_table.add_row("Owner", f"[primary]{parsed['owner']}[/primary]")
            info_table.add_row("Repository", f"[primary]{parsed['repo']}[/primary]")
            self.console.print(info_table)
            
            # Step 3: Determine intent (for external repos)
            self.console.print("\n[accent]Step 3: What do you want to do?[/accent]")
            self.console.print("[1] 📖 Study/Use the code (read-only)")
            self.console.print("[2] 🔧 Contribute to the project (fork + clone)")
            self.console.print("[3] 🏗️  Build upon it (clone + customize)")
            self.console.print("[4] ← Back")
            
            intent = Prompt.ask("\nSelect option", choices=['1', '2', '3', '4'], default='1')
            
            if intent == '4':
                return
            
            # Step 4: Account selection
            self.console.print("\n[accent]Step 4: Select Account[/accent]")
            accounts = self.account_manager.list_accounts()
            
            if not accounts:
                self.console.print("[error]✗ No accounts configured[/error]")
                return
            
            # Filter accounts for the detected platform
            platform_accounts = [
                acc for acc in accounts
                if hasattr(acc, 'platform') and acc.platform.value == platform
            ]
            
            if not platform_accounts:
                self.console.print(f"[warning]⚠ No accounts configured for {platform}[/warning]")
                # Allow anonymous clone
                auth_method = 'anonymous'
                account_name = None
            else:
                account_table = Table(border_style="cyan", show_header=True)
                account_table.add_column("ID", style="green")
                account_table.add_column("Account", style="white")
                account_table.add_column("Email", style="white")
                account_table.add_column("Status", style="white")
                
                valid_accounts = []
                for idx, acc in enumerate(platform_accounts, 1):
                    email_status = "N/A"
                    if acc.email:
                        is_valid, msg = PlatformManager.validate_email(acc.email, platform)
                        email_status = "[green]✓ Valid[/green]" if is_valid else f"[red]✗ Invalid[/red]"
                        if is_valid:
                            valid_accounts.append(acc)
                    else:
                        valid_accounts.append(acc)
                    
                    account_table.add_row(
                        str(idx),
                        acc.name,
                        acc.email or "N/A",
                        email_status
                    )
                
                self.console.print(account_table)
                
                account_idx = Prompt.ask(
                    "\n[green]Select account[/green]",
                    choices=[str(i) for i in range(1, len(platform_accounts) + 1)]
                )
                account_name = platform_accounts[int(account_idx) - 1].name
                
                # Step 5: Authentication method
                self.console.print("\n[cyan]Step 5: Authentication Method[/cyan]")
                self.console.print("[1] 🔑 SSH (Recommended)")
                self.console.print("[2] 🎫 HTTPS with Personal Access Token")
                
                if platform == 'gitlab':
                    self.console.print("[3] 🔐 HTTPS with Password")
                    auth_choices = ['1', '2', '3']
                else:
                    auth_choices = ['1', '2']
                
                auth_choice = Prompt.ask("\nSelect method", choices=auth_choices, default='1')
                
                if auth_choice == '1':
                    auth_method = 'ssh'
                elif auth_choice == '2':
                    auth_method = 'pat'
                else:
                    auth_method = 'password'
            
            # Handle fork workflow
            fork_url = None
            if intent == '2':  # Contribute
                self.console.print("\n[accent]Forking repository...[/accent]")
                fork_result = clone_workflow.fork_repository(repo_url, account_name, platform)
                
                if fork_result['success']:
                    self.console.print(f"[success]✓ {fork_result['message']}[/success]")
                    fork_url = fork_result['fork_url']
                    repo_url = fork_url  # Clone from fork
                else:
                    self.console.print(f"[error]✗ Fork failed: {fork_result['error']}[/error]")
                    return
            
            # Step 6: Clone options
            self.console.print("\n[accent]Step 6: Clone Options[/accent]")
            
            recursive = Prompt.ask("[primary]Clone submodules recursively?[/primary]", choices=['y', 'n'], default='n') == 'y'
            shallow = Prompt.ask("[primary]Shallow clone (--depth=1)?[/primary]", choices=['y', 'n'], default='n') == 'y'
            
            # Step 7: Clone destination
            self.console.print("\n[accent]Step 7: Clone Destination[/accent]")
            default_dest = str(clone_workflow.prepare_clone_destination(parsed['repo'], account_name))
            self.console.print(f"[primary]Default: {default_dest}[/primary]")
            
            custom = Prompt.ask("[primary]Use custom location?[/primary]", choices=['y', 'n'], default='n')
            
            if custom == 'y':
                destination = Prompt.ask("[primary]Enter path[/primary]")
            else:
                destination = default_dest
            
            # Step 8: Execute clone
            self.console.print("\n[accent]Step 8: Cloning Repository...[/accent]")
            
            result = clone_workflow.clone_repository(
                repo_url,
                account_name or 'default',
                auth_method=auth_method,
                destination=destination,
                recursive=recursive,
                shallow=shallow,
                upstream_url=analysis['https_url'] if intent == '2' else None
            )
            
            if result['success']:
                self.console.print(f"[success]✓ Clone successful![/success]")
                self.console.print(f"[success]Location: {result['destination']}[/success]")
                
                # Show next steps
                self.console.print("\n[accent]═══ Next Steps ═══[/accent]")
                self.console.print(f"1. cd {result['destination']}")
                self.console.print("2. Start working on the code")
                
                if intent == '2':
                    self.console.print("3. Create feature branch: git checkout -b my-feature")
                    self.console.print("4. Push to your fork: git push origin my-feature")
                    self.console.print("5. Create PR to original repository")
                
                log_git_operation("clone", repository=repo_url, success=True)
            else:
                self.console.print(f"[error]✗ Clone failed: {result['error']}[/error]")
                log_git_operation("clone", repository=repo_url, success=False, error_msg=result['error'])
    
        except KeyboardInterrupt:
            self.console.print("\n[warning]⚠️  Operation cancelled by user[/warning]")
            return
        except Exception as e:
            self.console.print(f"\n[error]✗ Error during account selection: {str(e)}[/error]")
            return
    
    def _clone_personal_repository(self, clone_workflow):
        """Clone from personal repositories."""
        from rich.table import Table
        
        try:
            # Step 1: Platform selection
            self.console.print("\n[accent]Step 1: Select Platform[/accent]")
            self.console.print("[1] 🐙 GitHub")
            self.console.print("[2] 🦊 GitLab")
            self.console.print("[3] 🗃️  Bitbucket")
            self.console.print("[4] ☁️  Azure DevOps")
            self.console.print("[5] 🏠 Self-Hosted Server")
            self.console.print("[6] ☁️  Cloud Storage")
            self.console.print("[7] 📁 Local Path")
            self.console.print("[8] 🔧 SourceForge")
            self.console.print("[9] ← Back")
            
            platform_choice = Prompt.ask("\nSelect platform", choices=['1', '2', '3', '4', '5', '6', '7', '8', '9'], default='9')
            
            if platform_choice == '9':
                return
            
            platform_map = {
                '1': 'github',
                '2': 'gitlab',
                '3': 'bitbucket',
                '4': 'azure_devops',
                '5': 'self_hosted',
                '6': 'cloud_storage',
                '7': 'local_path',
                '8': 'sourceforge'
            }
            platform = platform_map[platform_choice]
            
            # Step 2: Account selection
            self.console.print("\n[accent]Step 2: Select Account[/accent]")
            accounts = clone_workflow.get_accounts_for_platform(platform)
            
            if not accounts:
                self.console.print(f"[error]✗ No {platform} accounts configured[/error]")
                return
            
            account_table = Table(border_style="cyan", show_header=True)
            account_table.add_column("ID", style="green")
            account_table.add_column("Account", style="white")
            account_table.add_column("Email", style="white")
            
            for idx, acc in enumerate(accounts, 1):
                account_table.add_row(str(idx), acc.name, acc.email or "N/A")
            
            self.console.print(account_table)
            
            account_idx = Prompt.ask(
                "\n[primary]Select account[/primary]",
                choices=[str(i) for i in range(1, len(accounts) + 1)]
            )
            selected_account = accounts[int(account_idx) - 1]
            account_name = selected_account.name
        except KeyboardInterrupt:
            self.console.print("\n[warning]⚠️  Operation cancelled by user[/warning]")
            return
        except Exception as e:
            self.console.print(f"\n[error]✗ Error during account selection: {str(e)}[/error]")
            return
        
        try:
            # Check if account has PAT token, if not prompt for it
            if not selected_account.pat_token:
                self.console.print(f"\n[warning]⚠️  Account '{account_name}' has no Personal Access Token (PAT)[/warning]")
                self.console.print(f"[accent]To fetch your repositories, you need a PAT from {platform.upper()}[/accent]")
                
                # Use the comprehensive PAT setup method
                self._setup_pat()
                
                # Refresh account data after PAT setup
                selected_account = self.account_manager.get_account(account_name)
                if not selected_account or not selected_account.pat_token:
                    self.console.print("[warning]⚠️  PAT setup was not completed[/warning]")
                    return
        except KeyboardInterrupt:
            self.console.print("\n[yellow]⚠️  Operation cancelled by user[/yellow]")
            return
        except Exception as e:
            self.console.print(f"\n[red]✗ Error: {str(e)}[/red]")
            return
        
        # Step 3: Fetch repositories
        self.console.print("\n[cyan]Step 3: Fetching Your Repositories...[/cyan]")
        
        try:
            repositories = clone_workflow.fetch_personal_repositories(platform, account_name)
            self.console.print(f"[green]✓ Found {len(repositories)} repositories[/green]")
        except KeyboardInterrupt:
            self.console.print("\n[yellow]⚠️  Operation cancelled by user[/yellow]")
            return
        except Exception as e:
            error_msg = str(e)
            # Check if it's a 401 Unauthorized error
            if '401' in error_msg or 'Unauthorized' in error_msg:
                self.console.print(f"[yellow]⚠️  Invalid or expired Personal Access Token[/yellow]")
                self.console.print(f"[cyan]The PAT for account '{account_name}' is no longer valid[/cyan]")
                self.console.print(f"\n[green]How to get a new PAT:[/green]")
                if platform == 'github':
                    self.console.print("1. Go to: https://github.com/settings/tokens")
                    self.console.print("2. Click 'Generate new token' → 'Generate new token (classic)'")
                    self.console.print("3. Select scopes: repo, read:user")
                    self.console.print("4. Click 'Generate token' and copy it")
                elif platform == 'gitlab':
                    self.console.print("1. Go to: https://gitlab.com/-/profile/personal_access_tokens")
                    self.console.print("2. Fill in: Name, Expiration date")
                    self.console.print("3. Select scopes: api, read_api, read_repository")
                    self.console.print("4. Click 'Create personal access token' and copy it")
                elif platform == 'bitbucket':
                    self.console.print("1. Go to: https://bitbucket.org/account/settings/app-passwords/new")
                    self.console.print("2. Fill in: Label, Permissions (Repositories: read)")
                    self.console.print("3. Click 'Create' and copy the password")
                
                # Retry loop for new PAT input
                new_pat = None
                max_retries = 3
                retry_count = 0
                
                while retry_count < max_retries:
                    new_pat = Prompt.ask("\n[green]Enter your new Personal Access Token[/green]", password=True)
                    if not new_pat:
                        self.console.print("[red]✗ PAT is required to fetch repositories[/red]")
                        return
                    
                    # Test the new PAT token
                    self.console.print("\n[cyan]⏳ Testing Personal Access Token...[/cyan]")
                    if self._test_pat_token(platform, new_pat):
                        self.console.print("[green]✓ PAT is valid![/green]")
                        break
                    else:
                        retry_count += 1
                        if retry_count < max_retries:
                            self.console.print(f"[yellow]⚠️  PAT is invalid. Try again ({retry_count}/{max_retries})[/yellow]")
                        else:
                            self.console.print(f"[red]✗ Failed to validate PAT after {max_retries} attempts[/red]")
                            return
                
                # Update account with new PAT token
                self.account_manager.update_account(account_name, pat_token=new_pat)
                selected_account.pat_token = new_pat
                self.console.print(f"[green]✓ PAT updated for account '{account_name}'[/green]")
                
                # Retry fetching repositories
                self.console.print("\n[cyan]Step 3: Fetching Your Repositories...[/cyan]")
                try:
                    repositories = clone_workflow.fetch_personal_repositories(platform, account_name)
                    self.console.print(f"[green]✓ Found {len(repositories)} repositories[/green]")
                except Exception as retry_error:
                    self.console.print(f"[red]✗ Failed to fetch repositories: {str(retry_error)}[/red]")
                    return
            else:
                self.console.print(f"[red]✗ Failed to fetch repositories: {error_msg}[/red]")
                return
        
        # Step 4: Display and select repository with pagination
        self.console.print("\n[cyan]Step 4: Select Repository[/cyan]")
        
        try:
            # Repository selection with pagination
            selected_repo = self._select_repository_with_pagination(repositories, account_name)
            
            if not selected_repo:
                self.console.print("[yellow]⚠️  No repository selected[/yellow]")
                return
            
            # Step 5: Authentication method
            self.console.print("\n[cyan]Step 5: Authentication Method[/cyan]")
            self.console.print("[1] 🔑 SSH (Recommended)")
            self.console.print("[2] 🎫 HTTPS with Personal Access Token")
            
            auth_choice = Prompt.ask("\nSelect method", choices=['1', '2'], default='1')
            auth_method = 'ssh' if auth_choice == '1' else 'pat'
            
            # Step 6: Clone options
            self.console.print("\n[cyan]Step 6: Clone Options[/cyan]")
            
            recursive = Prompt.ask("[green]Clone submodules?[/green]", choices=['y', 'n'], default='n') == 'y'
            shallow = Prompt.ask("[green]Shallow clone?[/green]", choices=['y', 'n'], default='n') == 'y'
            
            # Step 7: Clone destination
            self.console.print("\n[cyan]Step 7: Clone Destination[/cyan]")
            default_dest = str(clone_workflow.prepare_clone_destination(
                selected_repo.name,
                account_name
            ))
            self.console.print(f"[green]Default: {default_dest}[/green]")
            
            custom = Prompt.ask("[green]Use custom location?[/green]", choices=['y', 'n'], default='n')
            
            if custom == 'y':
                destination = Prompt.ask("[green]Enter path[/green]")
            else:
                destination = default_dest
        except KeyboardInterrupt:
            self.console.print("\n[yellow]⚠️  Operation cancelled by user[/yellow]")
            return
        except Exception as e:
            self.console.print(f"\n[red]✗ Error: {str(e)}[/red]")
            return
        
        # Step 8: Execute clone
        self.console.print("\n[cyan]Step 8: Cloning Repository...[/cyan]")
        
        try:
            result = clone_workflow.clone_repository(
                selected_repo.ssh_url if auth_method == 'ssh' else selected_repo.https_url,
                account_name,
                auth_method=auth_method,
                destination=destination,
                recursive=recursive,
                shallow=shallow
            )
            
            if result['success']:
                from rich.panel import Panel
                
                self.console.print(Panel(
                    "[green]✓ Clone Successful![/green]",
                    border_style="green"
                ))
                self.console.print(f"[green]📁 Location: {result['destination']}[/green]")
                
                # Show next steps
                self.console.print("\n[cyan]═══ Next Steps ═══[/cyan]")
                self.console.print(f"1. cd {result['destination']}")
                self.console.print("2. Start working on the code")
                self.console.print("3. Use 'gitmanager push' to push your changes")
                self.console.print("4. Use 'gitmanager status' to check repository status")
                
                log_git_operation("clone", repository=selected_repo.ssh_url, success=True)
            else:
                # Use comprehensive error handler
                self._handle_clone_error(result['error'])
                log_git_operation("clone", success=False, error_msg=result['error'])
        except KeyboardInterrupt:
            self.console.print("\n[yellow]⚠️  Clone operation cancelled by user[/yellow]")
            log_git_operation("clone", success=False, error_msg="Operation cancelled by user")
        except Exception as e:
            self.console.print(f"\n[red]✗ Clone failed: {str(e)}[/red]")
            log_git_operation("clone", success=False, error_msg=str(e))
    
    def check_status(self):
        """Check repository status with account information."""
        from .tables import display_repository_status
        from rich.table import Table
        from pathlib import Path
        import subprocess
        
        try:
            status = self.git_operations.check_status()
            
            # Get remote URL to identify account
            result = subprocess.run(
                ['git', 'config', '--get', 'remote.origin.url'],
                capture_output=True,
                text=True,
                cwd=Path.cwd()
            )
            remote_url = result.stdout.strip() if result.returncode == 0 else "Unknown"
            
            # Try to match with configured accounts
            matched_account = None
            for account in self.account_manager.list_accounts():
                if account.username in remote_url or account.host in remote_url:
                    matched_account = account
                    break
            
            # Display account information
            self.console.print("\n[cyan]═══ Repository Account Information ═══[/cyan]\n")
            
            account_table = Table(border_style="cyan", show_header=True)
            account_table.add_column("Property", style="green")
            account_table.add_column("Value", style="white")
            
            if matched_account:
                account_table.add_row("Account Name", matched_account.name)
                account_table.add_row("Platform", matched_account.platform.value)
                account_table.add_row("Username", matched_account.username)
                account_table.add_row("Email", matched_account.email or "Not set")
                account_table.add_row("SSH Host", matched_account.host)
                account_table.add_row("Description", matched_account.description or "N/A")
            else:
                account_table.add_row("Status", "[yellow]⚠ No matching account configured[/yellow]")
                account_table.add_row("Remote URL", remote_url)
            
            self.console.print(account_table)
            
            # Display repository status
            self.console.print("\n[accent]═══ Repository Status ═══[/accent]\n")
            
            status_table = Table(border_style="cyan", show_header=True)
            status_table.add_column("Property", style="green")
            status_table.add_column("Value", style="white")
            
            status_table.add_row("Branch", status.current_branch)
            status_table.add_row("Remote URL", remote_url)
            
            # Uncommitted changes - get actual count from git
            if status.has_uncommitted_changes:
                result = subprocess.run(
                    ['git', 'status', '--porcelain', '-uall'],
                    capture_output=True,
                    text=True,
                    cwd=Path.cwd()
                )
                uncommitted_count = len([f for f in result.stdout.strip().split('\n') if f.strip()])
                status_table.add_row(
                    "Uncommitted Changes",
                    f"[warning]⚠ {uncommitted_count} file(s)[/warning]"
                )
            else:
                status_table.add_row("Uncommitted Changes", "[success]✓ None[/success]")
            
            # Commits ahead/behind
            if status.commits_ahead > 0 or status.commits_behind > 0:
                ahead_str = f"[warning]{status.commits_ahead}[/warning]" if status.commits_ahead > 0 else "0"
                behind_str = f"[warning]{status.commits_behind}[/warning]" if status.commits_behind > 0 else "0"
                status_table.add_row("Commits Ahead", ahead_str)
                status_table.add_row("Commits Behind", behind_str)
            else:
                status_table.add_row("Sync Status", "[success]✓ Up to date with remote[/success]")
            
            self.console.print(status_table)
            
            # Show uncommitted files if any
            if status.has_uncommitted_changes and status.uncommitted_files:
                self.console.print("\n[cyan]═══ Uncommitted Files ═══[/cyan]\n")
                
                # Get actual file count (not just directory count)
                result = subprocess.run(
                    ['git', 'status', '--porcelain', '-uall'],
                    capture_output=True,
                    text=True,
                    cwd=Path.cwd()
                )
                actual_files = [f for f in result.stdout.strip().split('\n') if f.strip()]
                actual_count = len(actual_files)
                
                files_table = Table(border_style="cyan", show_header=True)
                files_table.add_column("Status", style="yellow")
                files_table.add_column("File", style="white")
                
                # Show first 20 actual files
                for file_line in actual_files[:20]:
                    if file_line.strip():
                        status_code = file_line[:2]
                        file_path = file_line[3:]
                        files_table.add_row(status_code, file_path)
                
                if actual_count > 20:
                    files_table.add_row("...", f"... and {actual_count - 20} more files")
                
                self.console.print(files_table)
            
        except Exception as e:
            self.console.print(f"[red]✗ Error checking status: {str(e)}[/red]")
    
    def git_push(self):
        """Git push/pull/sync with 9 modular options."""
        from pathlib import Path
        from .git_operations_menu import GitOperationsMenu
        
        with ContextLogger("git_operations", category=LogCategory.GIT_OPERATION):
            try:
                repo_path = Path.cwd()
                menu = GitOperationsMenu(self.console, color_scheme=self.color_scheme)
                
                while True:
                    choice = menu.show_main_menu(repo_path)
                    
                    if choice == '8':
                        self.console.print("[info]Operation cancelled[/info]")
                        break
                    elif choice == '9':
                        break
                    elif choice == '1':
                        menu.handle_push(repo_path)
                    elif choice == '2':
                        menu.handle_pull(repo_path)
                    elif choice == '3':
                        menu.handle_sync(repo_path)
                    elif choice == '4':
                        menu.handle_status(repo_path)
                    elif choice == '5':
                        menu.handle_branch(repo_path)
                    elif choice == '6':
                        menu.handle_stage(repo_path)
                    elif choice == '7':
                        menu.handle_commit(repo_path)
                    
                    # Ask if user wants to continue
                    cont = Prompt.ask("\n[accent]Continue?[/accent]", choices=['yes', 'no'], default='yes')
                    if cont == 'no':
                        break
                
                return
            
            except Exception as e:
                self.console.print(f"[error]✗ Operation failed: {str(e)}[/error]")
                log_git_operation("operation", success=False, error_msg=str(e))
    
    def setup_repo(self):
        """
        Setup local repository with remote configuration.
        
        Supports all 8 platforms:
        - GitHub, GitLab, Bitbucket, Azure DevOps
        - Self-Hosted, Cloud Storage, Local Path, SourceForge
        
        Workflow:
        1. Select account (filters by platform support)
        2. Validate repository path
        3. Configure repository metadata
        4. Initialize Git repository
        5. Configure remote URL based on platform
        6. Verify setup
        """
        from ...core.repository_manager import RepositoryManager
        from ...core.platform_config import PlatformManager
        from pathlib import Path
        from rich.table import Table
        import subprocess
        
        with ContextLogger("setup_repository", category=LogCategory.GIT_OPERATION):
            try:
                # Get list of accounts
                accounts = self.account_manager.list_accounts()
                if not accounts:
                    self.console.print("[error]✗ No accounts configured. Please add an account first.[/error]")
                    return
                
                self.console.print("\n[accent]═══ Setup Local Project with Remote ═══[/accent]")
                
                # Step 1: Select account
                self.console.print("\n[accent]Step 1: Select Account[/accent]")
                account_table = Table(border_style="cyan", show_header=True)
                account_table.add_column("ID", style="green")
                account_table.add_column("Account", style="white")
                account_table.add_column("Platform", style="white")
                
                for idx, acc in enumerate(accounts, 1):
                    account_table.add_row(str(idx), acc.name, acc.platform.value)
                
                self.console.print(account_table)
                
                account_idx = Prompt.ask(
                    "\n[primary]Select account[/primary]",
                    choices=[str(i) for i in range(1, len(accounts) + 1)]
                )
                selected_account = accounts[int(account_idx) - 1]
                platform_key = selected_account.platform.value
                
                # Validate platform support
                platform_manager = PlatformManager()
                platform_info = platform_manager.PLATFORMS.get(platform_key)
                if not platform_info:
                    self.console.print(f"[error]✗ Platform '{platform_key}' is not supported[/error]")
                    return
                
                # Step 2: Get repository path
                self.console.print("\n[accent]Step 2: Repository Path[/accent]")
                while True:
                    repo_path = Prompt.ask("[primary]Enter repository path[/primary]")
                    repo_path = Path(repo_path).expanduser()
                    
                    # Validate path
                    if repo_path.exists() and list(repo_path.iterdir()):
                        overwrite = Prompt.ask(
                            f"[warning]Path exists and is not empty. Overwrite?[/warning]",
                            choices=['yes', 'no'],
                            default='no'
                        )
                        if overwrite == 'yes':
                            break
                    else:
                        repo_path.mkdir(parents=True, exist_ok=True)
                        break
                
                # Step 3: Get repository name
                self.console.print("\n[accent]Step 3: Repository Name[/accent]")
                default_name = repo_path.name
                repo_name = Prompt.ask(
                    "[primary]Repository name[/primary]",
                    default=default_name
                )
                
                # Validate repository name
                if not repo_name or not repo_name.replace('-', '').replace('_', '').isalnum():
                    self.console.print("[warning]⚠️  Repository name should be alphanumeric (hyphens/underscores allowed)[/warning]")
                    repo_name = Prompt.ask("[primary]Repository name[/primary]", default=default_name)
                
                # Step 4: Get description
                self.console.print("\n[accent]Step 4: Repository Description[/accent]")
                description = Prompt.ask("[primary]Description (optional)[/primary]", default="")
                
                # Step 5: Get branch name
                self.console.print("\n[accent]Step 5: Default Branch[/accent]")
                branch = Prompt.ask("[primary]Default branch[/primary]", default="main")
                
                # Step 6: Configure remote URL (platform-specific)
                self.console.print("\n[accent]Step 6: Configure Remote URL[/accent]")
                remote_url = self._get_remote_url(
                    platform_key,
                    selected_account,
                    repo_name
                )
                
                if remote_url:
                    self.console.print(f"[info]Remote URL: {remote_url}[/info]")
                else:
                    self.console.print("[warning]⚠️  Could not auto-generate remote URL. You can add it manually later.[/warning]")
                
                # Step 7: Initialize repository
                self.console.print("\n[accent]Step 7: Initializing Repository...[/accent]")
                
                # Create repository manager and setup
                repo_manager = RepositoryManager()
                result = repo_manager.setup_new_repository(
                    repo_path=repo_path,
                    account=selected_account,
                    repo_name=repo_name,
                    description=description,
                    branch=branch,
                    initialize_git=True
                )
                
                # Step 8: Configure remote if URL available
                if remote_url:
                    self.console.print("\n[accent]Step 8: Configuring Remote...[/accent]")
                    try:
                        subprocess.run(
                            ["git", "remote", "add", "origin", remote_url],
                            cwd=repo_path,
                            capture_output=True,
                            timeout=5,
                            check=True
                        )
                        self.console.print("[success]✓ Remote configured[/success]")
                    except subprocess.CalledProcessError:
                        self.console.print("[warning]⚠️  Remote already exists or configuration failed[/warning]")
                    except Exception as e:
                        self.console.print(f"[warning]⚠️  Could not configure remote: {str(e)}[/warning]")
                
                # Display summary
                self.console.print("\n[accent]═══ Setup Summary ═══[/accent]")
                self.console.print("[success]✓ Repository setup completed successfully![/success]")
                self.console.print(f"[success]📁 Location: {repo_path}[/success]")
                self.console.print(f"[success]📝 Name: {repo_name}[/success]")
                self.console.print(f"[success]👤 Account: {selected_account.name}[/success]")
                self.console.print(f"[success]🌿 Branch: {branch}[/success]")
                self.console.print(f"[success]🔗 Platform: {platform_key.upper()}[/success]")
                
                if remote_url:
                    self.console.print(f"[success]🌐 Remote: origin[/success]")
                
                self.console.print("\n[info]Next steps:[/info]")
                self.console.print(f"1. cd {repo_path}")
                self.console.print("2. Add files: git add .")
                self.console.print("3. Commit: git commit -m 'Initial commit'")
                self.console.print("4. Push: git push -u origin main")
                
                log_git_operation("setup_repository", repository=str(repo_path), success=True)
                
            except KeyboardInterrupt:
                self.console.print("\n[warning]⚠️  Operation cancelled by user[/warning]")
                log_git_operation("setup_repository", success=False, error_msg="Cancelled by user")
            except Exception as e:
                self.console.print(f"[error]✗ Setup failed: {str(e)}[/error]")
                log_git_operation("setup_repository", success=False, error_msg=str(e))
    
    def _get_remote_url(self, platform: str, account, repo_name: str) -> str:
        """
        Generate remote URL based on platform and account.
        
        Supports all 8 platforms with their specific URL formats.
        
        Args:
            platform: Platform key (github, gitlab, bitbucket, etc.)
            account: Account object with platform-specific details
            repo_name: Repository name
        
        Returns:
            SSH or HTTPS remote URL, or None if cannot generate
        """
        try:
            # Use SSH if available, fallback to HTTPS
            use_ssh = hasattr(account, 'ssh_key_path') and account.ssh_key_path
            
            # Platform-specific URL formats
            url_templates = {
                'github': {
                    'ssh': 'git@github.com:{username}/{repo}.git',
                    'https': 'https://github.com/{username}/{repo}.git'
                },
                'gitlab': {
                    'ssh': 'git@gitlab.com:{username}/{repo}.git',
                    'https': 'https://gitlab.com/{username}/{repo}.git'
                },
                'bitbucket': {
                    'ssh': 'git@bitbucket.org:{username}/{repo}.git',
                    'https': 'https://bitbucket.org/{username}/{repo}.git'
                },
                'azure_devops': {
                    'ssh': 'git@ssh.dev.azure.com:v3/{org}/{project}/{repo}',
                    'https': 'https://dev.azure.com/{org}/{project}/_git/{repo}'
                },
                'self_hosted': {
                    'ssh': 'git@{host}:{username}/{repo}.git',
                    'https': 'https://{host}/{username}/{repo}.git'
                },
                'sourceforge': {
                    'ssh': 'git@git.code.sf.net:p/{repo}/git',
                    'https': 'https://git.code.sf.net/p/{repo}/git'
                },
                'cloud_storage': {
                    'https': 'https://{bucket}/{repo}'
                },
                'local_path': {
                    'local': '{path}/{repo}'
                }
            }
            
            templates = url_templates.get(platform)
            if not templates:
                return None
            
            # Get username/identifier from account
            username = getattr(account, 'username', account.name)
            
            # Build URL based on platform
            if platform == 'azure_devops':
                org = getattr(account, 'organization', username)
                project = getattr(account, 'project', repo_name)
                url_key = 'ssh' if use_ssh else 'https'
                return templates[url_key].format(org=org, project=project, repo=repo_name)
            
            elif platform == 'self_hosted':
                host = getattr(account, 'host', 'git.example.com')
                url_key = 'ssh' if use_ssh else 'https'
                return templates[url_key].format(host=host, username=username, repo=repo_name)
            
            elif platform == 'cloud_storage':
                bucket = getattr(account, 'bucket', 'my-bucket')
                return templates['https'].format(bucket=bucket, repo=repo_name)
            
            elif platform == 'local_path':
                path = getattr(account, 'path', '/tmp/repos')
                return templates['local'].format(path=path, repo=repo_name)
            
            elif platform == 'sourceforge':
                url_key = 'ssh' if use_ssh else 'https'
                return templates[url_key].format(repo=repo_name)
            
            else:
                # GitHub, GitLab, Bitbucket
                url_key = 'ssh' if use_ssh else 'https'
                return templates[url_key].format(username=username, repo=repo_name)
        
        except Exception as e:
            self.logger.warning(f"Could not generate remote URL: {e}")
            return None
    
    def list_accounts(self):
        """List all accounts."""
        from .tables import display_accounts_table
        
        accounts = self.account_manager.list_accounts()
        display_accounts_table(accounts, self.console)
    
    def test_ssh(self):
        """Test SSH connections for all 8 platforms."""
        from .tables import display_accounts_table
        from ...models.account import Platform
        from ...core.platform_config import PlatformManager
        
        with ContextLogger("test_ssh_connections", category=LogCategory.SSH_OPERATION):
            try:
                accounts = self.account_manager.list_accounts()
                
                if not accounts:
                    self.console.print("[red]✗ No accounts configured[/red]")
                    return
                
                # Get platform manager for SSH support info
                platform_manager = PlatformManager()
                
                # If only one account, test it automatically
                if len(accounts) == 1:
                    self._test_single_account(accounts[0])
                    return
                
                # Multiple accounts - display all
                self.console.print("\n[accent]Available accounts:[/accent]")
                display_accounts_table(accounts, self.console)
                
                # Group accounts by platform
                platform_groups = {}
                for account in accounts:
                    platform_key = account.platform.value
                    if platform_key not in platform_groups:
                        platform_groups[platform_key] = []
                    platform_groups[platform_key].append(account)
                
                # Display platform options with SSH support info
                self.console.print("\n[accent]Available Platforms:[/accent]")
                platform_choices = []
                for idx, (platform_key, platform_accounts) in enumerate(platform_groups.items(), 1):
                    platform_info = platform_manager.PLATFORMS.get(platform_key)
                    ssh_support = "✓ SSH" if platform_info and platform_info.supports_ssh else "✗ No SSH"
                    self.console.print(f"[{idx}] {platform_key.upper()} ({len(platform_accounts)} account(s)) - {ssh_support}")
                    platform_choices.append(platform_key)
                
                self.console.print(f"[{len(platform_choices) + 1}] Test all platforms")
                self.console.print(f"[{len(platform_choices) + 2}] ← Back")
                
                # Ask which platform to test
                choice = Prompt.ask(
                    "\n[secondary]Select platform to test[/secondary]",
                    choices=[str(i) for i in range(1, len(platform_choices) + 3)],
                    default=str(len(platform_choices) + 2)
                )
                
                choice_idx = int(choice) - 1
                
                # Handle "Back" option
                if choice_idx == len(platform_choices) + 1:
                    return
                
                # Handle "Test all" option
                if choice_idx == len(platform_choices):
                    self.console.print("\n[secondary]Testing all platforms...[/secondary]")
                    for platform_key, platform_accounts in platform_groups.items():
                        platform_info = platform_manager.PLATFORMS.get(platform_key)
                        
                        # Check if platform supports SSH
                        if not platform_info or not platform_info.supports_ssh:
                            self.console.print(f"\n[warning]⚠️  {platform_key.upper()} does not support SSH[/warning]")
                            continue
                        
                        self.console.print(f"\n[secondary]Testing {platform_key.upper()} accounts...[/secondary]")
                        for account in platform_accounts:
                            self._test_single_account(account)
                    return
                
                # Test specific platform
                selected_platform = platform_choices[choice_idx]
                platform_info = platform_manager.PLATFORMS.get(selected_platform)
                
                # Check if platform supports SSH
                if not platform_info or not platform_info.supports_ssh:
                    self.console.print(f"\n[red]✗ {selected_platform.upper()} does not support SSH[/red]")
                    return
                
                accounts_to_test = platform_groups[selected_platform]
                
                # If multiple accounts, ask which one
                if len(accounts_to_test) > 1:
                    self.console.print(f"\n[info]Accounts for {selected_platform.upper()}:[/info]")
                    for i, account in enumerate(accounts_to_test, 1):
                        self.console.print(f"[{i}] {account.name}")
                    
                    choice = Prompt.ask(
                        "\n[info]Select account to test[/info]",
                        choices=[str(i) for i in range(1, len(accounts_to_test) + 1)],
                        default='1'
                    )
                    account = accounts_to_test[int(choice) - 1]
                    self._test_single_account(account)
                else:
                    self._test_single_account(accounts_to_test[0])
            
            except KeyboardInterrupt:
                self.console.print("\n[warning]⚠️  Operation cancelled by user[/warning]")
            except Exception as e:
                self.console.print(f"\n[error]✗ Error: {str(e)}[/error]")
    
    def _test_single_account(self, account):
        """Test SSH connection for a single account with retry logic.
        
        Args:
            account: Account object to test
        """
        self.console.print(f"\n[info]Testing {account.name} ({account.platform.value})...[/info]")
        try:
            from pathlib import Path
            max_retries = 3
            retry_count = 0
            
            while retry_count < max_retries:
                try:
                    # Use SSH orchestrator's _test_connection method for all 8 platforms
                    success, message, username = self.ssh_orchestrator._test_connection(
                        platform=account.host,
                        key_path=Path(account.ssh_key_path)
                    )
                    
                    if success:
                        if username:
                            self.console.print(f"[success]✓ Connection successful (user: {username})[/success]")
                        else:
                            self.console.print(f"[success]✓ Connection successful[/success]")
                        log_ssh_operation("test_connection", account=account.name, success=True)
                        return
                    else:
                        # Check if it's a timeout error - retry
                        if "timed out" in message.lower():
                            retry_count += 1
                            if retry_count < max_retries:
                                self.console.print(f"[warning]⚠️  Timeout, retrying... ({retry_count}/{max_retries})[/warning]")
                                continue
                        
                        # Non-timeout error or max retries reached
                        self.console.print(f"[error]✗ Connection failed: {message}[/error]")
                        log_ssh_operation("test_connection", account=account.name, success=False, error_msg=message)
                        return
                except Exception as e:
                    retry_count += 1
                    if retry_count < max_retries:
                        self.console.print(f"[warning]⚠️  Error, retrying... ({retry_count}/{max_retries})[/warning]")
                        continue
                    
                    self.console.print(f"[error]✗ Connection failed: {str(e)}[/error]")
                    log_ssh_operation("test_connection", account=account.name, success=False, error_msg=str(e))
                    return
            
        except KeyboardInterrupt:
            self.console.print("\n[warning]⚠️  Operation cancelled by user[/warning]")
        except Exception as e:
            self.console.print(f"[error]✗ Unexpected error during testing: {str(e)}[/error]")
            log_ssh_operation("test_connection", account=account.name, success=False, error_msg=str(e))
    
    def generate_key(self):
        """Generate SSH key or setup Personal Access Token."""
        from ...models.ssh_key import SSHKeyType
        from ...models.account import Platform
        
        with ContextLogger("account_setup", category=LogCategory.SSH_OPERATION):
            # Show menu for SSH key or PAT setup
            self.console.print("\n[accent]═══ Account Authentication Setup ═══[/accent]")
            self.console.print("[1] 🔑 Generate new SSH key")
            self.console.print("[2] 🎫 Setup Personal Access Token (PAT)")
            self.console.print("[3] ← Back to main menu")
            
            choice = Prompt.ask(
                "\n[green]Select option[/green]",
                choices=['1', '2', '3'],
                default='3'
            )
            
            if choice == '3':
                return
            
            elif choice == '1':
                self._generate_ssh_key()
            
            elif choice == '2':
                self._setup_pat()
    
    def _generate_ssh_key(self):
        """
        Generate SSH key for account on any of 8 supported platforms.
        
        Supports all 8 platforms:
        - GitHub, GitLab, Bitbucket, Azure DevOps
        - Self-Hosted, Cloud Storage, Local Path, SourceForge
        
        Workflow:
        1. Get account name and email
        2. Select platform (all 8 supported)
        3. Configure account type
        4. Generate SSH key
        5. Start SSH agent
        6. Add key to agent
        7. Configure SSH config
        8. Test connection
        """
        from rich.table import Table
        from ...core.platform_config import PlatformManager
        
        with ContextLogger("generate_ssh_key", category=LogCategory.SSH_OPERATION):
            try:
                # Step 1: Get account details
                self.console.print("\n[accent]═══ SSH Key Generation ═══[/accent]")
                self.console.print("\n[accent]Step 1: Account Details[/accent]")
                
                account_name = Prompt.ask("[primary]Account name[/primary]")
                if not account_name or not account_name.replace('-', '').replace('_', '').isalnum():
                    self.console.print("[warning]⚠️  Account name should be alphanumeric[/warning]")
                    account_name = Prompt.ask("[primary]Account name[/primary]")
                
                email = Prompt.ask("[primary]Email[/primary]")
                if not email or '@' not in email:
                    self.console.print("[warning]⚠️  Please enter a valid email[/warning]")
                    email = Prompt.ask("[primary]Email[/primary]")
                
                # Step 2: Select platform (all 8 supported)
                self.console.print("\n[accent]Step 2: Select Platform[/accent]")
                platform_manager = PlatformManager()
                
                platform_options = []
                for idx, (key, info) in enumerate(platform_manager.PLATFORMS.items(), 1):
                    ssh_support = "✓ SSH" if info.supports_ssh else "✗ No SSH"
                    self.console.print(f"[{idx}] {key.upper()} - {ssh_support}")
                    platform_options.append(key)
                
                platform_choice = Prompt.ask(
                    "\n[primary]Select platform[/primary]",
                    choices=[str(i) for i in range(1, len(platform_options) + 1)],
                    default='1'
                )
                
                platform_key = platform_options[int(platform_choice) - 1]
                platform_info = platform_manager.PLATFORMS[platform_key]
                
                # Check SSH support
                if not platform_info.supports_ssh:
                    self.console.print(f"[warning]⚠️  {platform_key.upper()} does not support SSH[/warning]")
                    self.console.print("[info]SSH keys are only supported for platforms with SSH capability[/info]")
                    return
                
                # Step 3: Account type
                self.console.print("\n[accent]Step 3: Account Type[/accent]")
                self.console.print("[1] personal")
                self.console.print("[2] school")
                self.console.print("[3] work")
                # self.console.print("[4] zanabuni")
                self.console.print("[4] organization")
                self.console.print("[5] custom (enter your own)")
                
                account_type_choice = Prompt.ask(
                    "[primary]Select account type[/primary]",
                    choices=["1", "2", "3", "4", "5"],
                    default="1"
                )
                
                account_type_map = {
                    "1": "personal",
                    "2": "school",
                    "3": "work",
                    "4": "organization",
                    "5": "custom"
                }
                
                if account_type_choice == "5":
                    account_type = Prompt.ask("[primary]Enter custom account type[/primary]")
                    if not account_type:
                        account_type = "personal"
                else:
                    account_type = account_type_map[account_type_choice]
                
                # Step 4: Key type selection
                self.console.print("\n[accent]Step 4: SSH Key Type[/accent]")
                self.console.print("[1] ed25519 (recommended - modern, secure)")
                self.console.print("[2] rsa (compatible - widely supported)")
                
                key_type_choice = Prompt.ask(
                    "[primary]Select key type[/primary]",
                    choices=["1", "2"],
                    default="1"
                )
                
                key_type = "ed25519" if key_type_choice == "1" else "rsa"
                
                # Step 5: Optional passphrase
                self.console.print("\n[accent]Step 5: Passphrase (Optional)[/accent]")
                self.console.print("[info]A passphrase adds extra security. Leave empty for no passphrase.[/info]")
                
                passphrase = Prompt.ask(
                    "[primary]Enter passphrase (or press Enter to skip)[/primary]",
                    password=True,
                    default=""
                )
                
                # Confirm passphrase if provided
                if passphrase:
                    while True:
                        passphrase_confirm = Prompt.ask(
                            "[primary]Confirm passphrase[/primary]",
                            password=True,
                            default=""
                        )
                        
                        if passphrase == passphrase_confirm:
                            self.console.print("[success]✓ Passphrase confirmed[/success]")
                            break
                        else:
                            self.console.print("[error]✗ Passphrases do not match. Please try again.[/error]")
                
                # Step 6: Confirmation
                self.console.print("\n[accent]Step 6: Confirm Setup[/accent]")
                self.console.print(f"[info]Account: {account_name}[/info]")
                self.console.print(f"[info]Email: {email}[/info]")
                self.console.print(f"[info]Platform: {platform_key.upper()}[/info]")
                self.console.print(f"[info]Type: {account_type}[/info]")
                self.console.print(f"[info]Key Type: {key_type}[/info]")
                self.console.print(f"[info]Passphrase: {'Yes' if passphrase else 'No'}[/info]")
                
                confirm = Prompt.ask(
                    "\n[primary]Proceed with key generation?[/primary]",
                    choices=['yes', 'no'],
                    default='yes'
                )
                
                if confirm == 'no':
                    self.console.print("[warning]⚠️  Operation cancelled[/warning]")
                    return
                
                # Step 7: Generate SSH key
                self.console.print("\n[accent]Step 7: Generating SSH Key...[/accent]")
                
                # Map platform key to domain for orchestrator
                platform_domain = platform_info.api_url or f"{platform_key}.com"
                
                result = self.orchestrator.setup_account(
                    name=account_name,
                    email=email,
                    platform=platform_domain,
                    account_type=account_type,
                    key_type=key_type,
                    passphrase=passphrase if passphrase else None
                )
                
                # Step 8: Display results
                if result['success']:
                    self.console.print("\n[accent]═══ SSH Setup Complete ═══[/accent]")
                    self.console.print("[success]✓ SSH Account Setup Successful![/success]")
                    
                    # Show step results table
                    table = Table(title="Setup Steps", show_header=True, border_style="accent")
                    table.add_column("Step", style="accent")
                    table.add_column("Status", style="bold")
                    table.add_column("Details", style="dim")
                    
                    step_names = {
                        "generate_key": "1. Generate SSH Key",
                        "start_agent": "2. Start SSH Agent",
                        "add_to_agent": "3. Add to Agent",
                        "configure_ssh": "4. Configure SSH",
                        "test_connection": "5. Test Connection"
                    }
                    
                    for step_key, step_name in step_names.items():
                        if step_key in result["steps"]:
                            step = result["steps"][step_key]
                            status = "[success]✓[/success]" if step["success"] else "[error]✗[/error]"
                            table.add_row(step_name, status, step["message"])
                    
                    self.console.print(table)
                    
                    # Display key information
                    if result.get("key_info"):
                        key_info = result["key_info"]
                        self.console.print("\n[accent]SSH Key Information:[/accent]")
                        self.console.print(f"  📝 Name: {key_info['name']}")
                        self.console.print(f"  🔑 SSH Alias: {key_info['ssh_host_alias']}")
                        self.console.print(f"  👆 Fingerprint: {key_info['fingerprint']}")
                        self.console.print(f"  📂 Location: {key_info['key_path']}")
                        
                        # Show connection test result
                        if result["steps"].get("test_connection", {}).get("success"):
                            username = result["steps"]["test_connection"].get("username")
                            if username:
                                self.console.print(f"\n[success]✓ Connected as: {username}[/success]")
                            else:
                                self.console.print("[success]✓ Connection test successful[/success]")
                    
                    # Save to database
                    if self.ssh_integration and result.get("key_info"):
                        try:
                            success, msg = self.ssh_integration.save_ssh_key_metadata(1, result['key_info'])
                            if success:
                                self.console.print(f"\n[success]✓ {msg}[/success]")
                        except Exception as e:
                            self.logger.warning(f"Could not save to database: {e}")
                    
                    # Next steps
                    self.console.print("\n[info]Next steps:[/info]")
                    self.console.print(f"1. Add public key to your {platform_key.upper()} account")
                    self.console.print(f"2. Test connection: ssh -T git@{platform_info.api_url or platform_key}.com")
                    self.console.print("3. Clone repositories using SSH URLs")
                    
                    log_ssh_operation("generate_key", account=account_name, success=True)
                
                else:
                    # Setup completed with warnings
                    self.console.print("\n[accent]═══ Setup Completed with Warnings ═══[/accent]")
                    self.console.print("[warning]⚠️  Setup completed but with some warnings[/warning]")
                    
                    for error in result.get("errors", []):
                        self.console.print(f"  [warning]⚠️  {error}[/warning]")
                    
                    if result.get("key_info"):
                        self.console.print(f"\n[info]Key was generated: {result['key_info']['name']}[/info]")
                    
                    log_ssh_operation("generate_key", account=account_name, success=False, error_msg="Setup completed with warnings")
            
            except KeyboardInterrupt:
                self.console.print("\n[warning]⚠️  Operation cancelled by user[/warning]")
                log_ssh_operation("generate_key", account="unknown", success=False, error_msg="Cancelled by user")
            except Exception as e:
                error_response = SSHExceptionHandler.handle(e, self.logger)
                self.console.print(f"[error]✗ {error_response['message']}[/error]")
                log_ssh_operation("generate_key", account="unknown", success=False, error_msg=str(e))
    
    def _setup_pat(self):
        """
        Setup Personal Access Token (PAT) for account on any of 8 supported platforms.
        
        Supports all 8 platforms:
        - GitHub, GitLab, Bitbucket, Azure DevOps
        - Self-Hosted, Cloud Storage, Local Path, SourceForge
        
        Workflow:
        1. Select platform (all 8 supported)
        2. Select or create account
        3. Show platform-specific PAT generation instructions
        4. Input and validate PAT
        5. Save PAT to account
        """
        from rich.table import Table
        from ...core.platform_config import PlatformManager
        from ...models.account import Platform as PlatformEnum
        
        with ContextLogger("setup_pat", category=LogCategory.ACTIVITY):
            try:
                # Step 1: Select platform (all 8 supported)
                self.console.print("\n[accent]═══ Personal Access Token Setup ═══[/accent]")
                self.console.print("\n[accent]Step 1: Select Platform[/accent]")
                
                platform_manager = PlatformManager()
                platform_options = []
                
                for idx, (key, info) in enumerate(platform_manager.PLATFORMS.items(), 1):
                    # Show platforms that support token-based auth
                    token_support = "✓ Token" if info.supports_pat else "✗ No Token"
                    self.console.print(f"[{idx}] {key.upper()} - {token_support}")
                    platform_options.append(key)
                
                platform_choice = Prompt.ask(
                    "\n[primary]Select platform[/primary]",
                    choices=[str(i) for i in range(1, len(platform_options) + 1)],
                    default=str(len(platform_options))
                )
                
                if platform_choice == str(len(platform_options)):
                    return
                
                platform_key = platform_options[int(platform_choice) - 1]
                platform_info = platform_manager.PLATFORMS[platform_key]
                
                # Check PAT support
                if not platform_info.supports_pat:
                    self.console.print(f"[warning]⚠️  {platform_key.upper()} does not support token-based authentication[/warning]")
                    self.console.print("[info]This platform uses SSH keys or other authentication methods[/info]")
                    return
                
                # Step 2: Select or create account
                self.console.print(f"\n[accent]Step 2: Select Account for {platform_key.upper()}[/accent]")
                accounts = self.account_manager.list_accounts()
                platform_accounts = [a for a in accounts if hasattr(a, 'platform') and a.platform.value == platform_key]
                
                selected_account = None
                
                if platform_accounts:
                    account_table = Table(border_style="cyan", show_header=True)
                    account_table.add_column("ID", style="green")
                    account_table.add_column("Account", style="white")
                    account_table.add_column("Username", style="white")
                    account_table.add_column("Email", style="white")
                    
                    for idx, acc in enumerate(platform_accounts, 1):
                        account_table.add_row(
                            str(idx),
                            acc.name,
                            getattr(acc, 'username', 'N/A'),
                            getattr(acc, 'email', 'N/A')
                        )
                    
                    self.console.print(account_table)
                    
                    account_idx = Prompt.ask(
                        "\n[primary]Select account[/primary]",
                        choices=[str(i) for i in range(1, len(platform_accounts) + 1)],
                        default='1'
                    )
                    selected_account = platform_accounts[int(account_idx) - 1]
                else:
                    self.console.print(f"[warning]⚠️  No {platform_key} accounts found[/warning]")
                    create_account = Prompt.ask(
                        "[primary]Create new account?[/primary]",
                        choices=['yes', 'no'],
                        default='yes'
                    )
                    
                    if create_account.lower() != 'yes':
                        return
                    
                    # Create new account
                    self.console.print("\n[accent]Creating New Account[/accent]")
                    account_name = Prompt.ask("[primary]Account name[/primary]")
                    username = Prompt.ask("[primary]Git username[/primary]")
                    email = Prompt.ask("[primary]Email[/primary]")
                    
                    try:
                        selected_account = self.account_manager.add_account(
                            name=account_name,
                            platform=PlatformEnum(platform_key),
                            username=username,
                            email=email,
                            description=f"{platform_key} account for {username}"
                        )
                        self.console.print(f"[success]✓ Account created: {account_name}[/success]")
                    except Exception as e:
                        self.console.print(f"[error]✗ Failed to create account: {str(e)}[/error]")
                        return
                
                # Step 3: Show platform-specific PAT instructions
                self.console.print(f"\n[accent]Step 3: Get Personal Access Token[/accent]")
                self._show_pat_instructions(platform_key, platform_info)
                
                # Step 4: Input PAT with validation
                self.console.print(f"\n[accent]Step 4: Enter Personal Access Token[/accent]")
                
                pat_token = None
                max_retries = 3
                retry_count = 0
                
                while retry_count < max_retries:
                    pat_token = Prompt.ask(
                        f"\n[primary]Paste your {platform_key.upper()} PAT[/primary]",
                        password=True
                    )
                    
                    if not pat_token or not pat_token.strip():
                        self.console.print("[error]✗ PAT is required[/error]")
                        continue
                    
                    # Test the PAT token
                    self.console.print(f"\n[info]⏳ Testing {platform_key.upper()} PAT...[/info]")
                    if self._test_pat_token(platform_key, pat_token):
                        self.console.print(f"[success]✓ PAT is valid![/success]")
                        break
                    else:
                        retry_count += 1
                        if retry_count < max_retries:
                            self.console.print(f"[warning]⚠️  PAT is invalid. Try again ({retry_count}/{max_retries})[/warning]")
                        else:
                            self.console.print(f"[error]✗ Failed to validate PAT after {max_retries} attempts[/error]")
                            return
                
                # Step 5: Save PAT to account
                self.console.print(f"\n[accent]Step 5: Saving PAT[/accent]")
                try:
                    self.account_manager.update_account(selected_account.name, pat_token=pat_token)
                    self.console.print(f"\n[accent]═══ PAT Setup Complete ═══[/accent]")
                    self.console.print(f"[success]✓ PAT saved for account '{selected_account.name}'[/success]")
                    self.console.print(f"[success]✓ Platform: {platform_key.upper()}[/success]")
                    self.console.print(f"[success]✓ Username: {getattr(selected_account, 'username', 'N/A')}[/success]")
                    self.console.print(f"\n[info]You can now use this account to clone repositories![/info]")
                    log_ssh_operation("setup_pat", account=selected_account.name, success=True)
                except Exception as e:
                    self.console.print(f"[error]✗ Failed to save PAT: {str(e)}[/error]")
                    log_ssh_operation("setup_pat", account=selected_account.name, success=False, error_msg=str(e))
            
            except KeyboardInterrupt:
                self.console.print("\n[warning]⚠️  Operation cancelled by user[/warning]")
                log_ssh_operation("setup_pat", account="unknown", success=False, error_msg="Cancelled by user")
            except Exception as e:
                self.console.print(f"[error]✗ Setup failed: {str(e)}[/error]")
                log_ssh_operation("setup_pat", account="unknown", success=False, error_msg=str(e))
    
    def _show_pat_instructions(self, platform: str, platform_info) -> None:
        """
        Show platform-specific PAT generation instructions.
        
        Args:
            platform: Platform key (github, gitlab, bitbucket, etc.)
            platform_info: Platform metadata from PlatformManager
        """
        self.console.print(f"\n[info]How to get a PAT for {platform.upper()}:[/info]\n")
        
        instructions = {
            'github': [
                "1. Go to: https://github.com/settings/tokens",
                "2. Click 'Generate new token' → 'Generate new token (classic)'",
                "3. Select scopes:",
                "   • repo (full control of private repositories)",
                "   • read:user (read user profile data)",
                "4. Click 'Generate token' and copy it immediately",
                "   ⚠️  You won't be able to see it again!"
            ],
            'gitlab': [
                "1. Go to: https://gitlab.com/-/profile/personal_access_tokens",
                "2. Fill in the form:",
                "   • Token name: e.g., 'git-manager'",
                "   • Expiration date: Choose appropriate date",
                "3. Select scopes:",
                "   • api (full API access)",
                "   • read_api (read API)",
                "   • read_repository (read repository)",
                "4. Click 'Create personal access token' and copy it"
            ],
            'bitbucket': [
                "1. Go to: https://bitbucket.org/account/settings/app-passwords/new",
                "2. Fill in the form:",
                "   • Label: e.g., 'git-manager'",
                "3. Select permissions:",
                "   • Repositories: Read",
                "   • Workspace membership: Read",
                "4. Click 'Create' and copy the password"
            ],
            'azure_devops': [
                "1. Go to: https://dev.azure.com/",
                "2. Click your profile icon → Personal access tokens",
                "3. Click '+ New Token'",
                "4. Fill in the form:",
                "   • Name: e.g., 'git-manager'",
                "   • Organization: Select your organization",
                "   • Expiration: Choose appropriate date",
                "5. Select scopes:",
                "   • Code (Read & Write)",
                "   • Identity (Read)",
                "6. Click 'Create' and copy the token"
            ],
            'self_hosted': [
                "1. Go to your self-hosted Git server settings",
                "2. Navigate to Personal Access Tokens or API Tokens section",
                "3. Create a new token with appropriate permissions",
                "4. Copy the token immediately (may not be visible again)"
            ],
            'sourceforge': [
                "1. Go to: https://sourceforge.net/auth/",
                "2. Click 'Account Settings'",
                "3. Navigate to 'API Tokens'",
                "4. Create a new token",
                "5. Copy the token immediately"
            ],
            'cloud_storage': [
                "1. Cloud storage platforms typically use API keys",
                "2. Refer to your cloud provider's documentation",
                "3. Generate an API key with read access",
                "4. Copy the key immediately"
            ],
            'local_path': [
                "1. Local path repositories don't require tokens",
                "2. Use SSH keys or file system permissions instead",
                "3. Consider using SSH authentication for this platform"
            ]
        }
        
        # Show instructions for platform
        if platform in instructions:
            for instruction in instructions[platform]:
                self.console.print(instruction)
        else:
            self.console.print(f"[warning]⚠️  No specific instructions for {platform}[/warning]")
            self.console.print("Please refer to your platform's documentation for PAT generation")