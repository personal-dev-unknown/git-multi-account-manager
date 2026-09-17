"""Git Operations Menu - Modular submenu handlers for push/pull/sync/status/branch/stage/commit."""

from pathlib import Path
from rich.prompt import Prompt
from rich.table import Table
from rich.console import Console
from typing import Optional

from ...core.sync import (
    PushOperations, PullOperations, SyncOperations,
    GitStatus, GitBranch, GitStage, GitCommit
)
from ...utils.log_utils import log_git_operation
from .color_schemes import ColorScheme, DEFAULT_SCHEME


class GitOperationsMenu:
    """Modular git operations menu with 9 main options and submenus."""
    
    def __init__(self, console: Console, color_scheme: Optional[ColorScheme] = None):
        """Initialize menu with console and color scheme.
        
        Args:
            console: Rich Console instance
            color_scheme: Optional ColorScheme for styling (uses DEFAULT_SCHEME if not provided)
        """
        self.console = console
        self.color_scheme = color_scheme or DEFAULT_SCHEME
    
    def show_main_menu(self, repo_path: Path):
        """Show main 9-option menu with table format."""
        scheme = self.color_scheme
        self.console.print(f"\n[{scheme.accent}]═══ Git Operations Menu ═══[/{scheme.accent}]\n")
        
        # Initialize operations
        status_ops = GitStatus()
        
        # Show repository status
        self.console.print(status_ops.show_status(repo_path))
        
        # Show 9 main options in table format
        self.console.print(f"\n[{scheme.success}]Available Operations:[/{scheme.success}]\n")
        
        table = Table(
            title="Git Operations",
            show_header=True,
            header_style=f"bold {scheme.accent}",
            border_style=scheme.accent
        )
        table.add_column("Option", style=scheme.accent, width=8)
        table.add_column("Operation", style=scheme.success, width=20)
        table.add_column("Description", style=scheme.text)
        
        operations = [
            ("1", "🟢 Git Push", "Push commits to remote"),
            ("2", "🟢 Git Pull", "Pull changes from remote"),
            ("3", "🟢 Git Sync", "Bidirectional synchronization"),
            ("4", "📊 Git Status", "Show detailed repository status"),
            ("5", "🌿 Git Branch", "Manage branches"),
            ("6", "📝 Git Stage/Stash", "Stage changes or stash"),
            ("7", "💾 Git Commit", "Create commits"),
            ("8", "❌ Cancel", "Do nothing"),
            ("9", "⬅️  Go Back", "Return to main menu"),
        ]
        
        for code, name, desc in operations:
            table.add_row(f"[{code}]", name, desc)
        
        self.console.print(table)
        
        choice = Prompt.ask(
            f"\n[{scheme.primary}]Select operation[/{scheme.primary}]",
            choices=[str(i) for i in range(1, 10)],
            default='9'
        )
        
        return choice
    
    def handle_push(self, repo_path: Path):
        """Handle git push submenu."""
        scheme = self.color_scheme
        push_ops = PushOperations()
        self.console.print(f"\n[{scheme.accent}]═══ Git Push Options ═══[/{scheme.accent}]\n")
        
        table = Table(
            show_header=True,
            header_style=f"bold {scheme.accent}",
            border_style=scheme.accent
        )
        table.add_column("Option", style=scheme.accent, width=8)
        table.add_column("Operation", style=scheme.success, width=25)
        table.add_column("Description", style=scheme.text)
        
        options = [
            ("1", "🟢 Safe Push", "Push with pre-flight checks"),
            ("2", "🟡 Push with Force-Lease", "Safer force push"),
            ("3", "🟠 Force Push", "⚠️  Dangerous - requires confirmation"),
            ("4", "🔵 Push All Branches", "Push all local branches"),
            ("5", "🔵 Push with Tags", "Push commits and tags"),
            ("6", "🟣 Dry Run Push", "Preview what would be pushed"),
            ("7", "❌ Cancel", "Do nothing"),
        ]
        
        for code, name, desc in options:
            table.add_row(f"[{code}]", name, desc)
        
        self.console.print(table)
        
        choice = Prompt.ask(
            f"\n[{scheme.primary}]Select push option[/{scheme.primary}]",
            choices=[str(i) for i in range(1, 8)],
            default='1'
        )
        
        if choice == '7':
            return
        
        if choice == '1':
            result = push_ops.safe_push(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("push", repository=str(repo_path), success=result.success)
        elif choice == '2':
            self.console.print("\n[warning]⚠️  Force-Lease will overwrite remote history if needed[/warning]")
            confirm = Prompt.ask("[warning]Proceed?[/warning]", choices=['yes', 'no'], default='no')
            if confirm == 'yes':
                result = push_ops.push_with_lease(repo_path)
                self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
                log_git_operation("push_force_lease", repository=str(repo_path), success=result.success)
        elif choice == '3':
            self.console.print("\n[error]🚨 WARNING: Force push will overwrite remote history![/error]")
            confirm = Prompt.ask("[error]Type 'FORCE PUSH' to confirm[/error]", default="cancel")
            if confirm == "FORCE PUSH":
                result = push_ops.force_push(repo_path)
                self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
                log_git_operation("force_push", repository=str(repo_path), success=result.success)
        elif choice == '4':
            result = push_ops.push_all_branches(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("push_all", repository=str(repo_path), success=result.success)
        elif choice == '5':
            result = push_ops.push_with_tags(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("push_tags", repository=str(repo_path), success=result.success)
        elif choice == '6':
            result = push_ops.dry_run_push(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("push_dry_run", repository=str(repo_path), success=result.success)
    
    def handle_pull(self, repo_path: Path):
        """Handle git pull submenu."""
        scheme = self.color_scheme
        pull_ops = PullOperations()
        self.console.print(f"\n[{scheme.accent}]═══ Git Pull Options ═══[/{scheme.accent}]\n")
        
        table = Table(
            show_header=True,
            header_style=f"bold {scheme.accent}",
            border_style=scheme.accent
        )
        table.add_column("Option", style=scheme.accent, width=8)
        table.add_column("Operation", style=scheme.success, width=25)
        table.add_column("Description", style=scheme.text)
        
        options = [
            ("1", "🟢 Safe Pull", "Pull with auto-stash"),
            ("2", "🟢 Smart Pull", "Intelligent strategy selection"),
            ("3", "🔵 Pull with Rebase", "Clean linear history"),
            ("4", "🔵 Fast-Forward Only", "Safest pull option"),
            ("5", "🟡 Pull with Autostash", "Auto stash/unstash"),
            ("6", "🟠 Force Pull", "Reset to remote (DESTRUCTIVE)"),
            ("7", "🟣 Fetch Only", "Download without integrating"),
            ("8", "❌ Cancel", "Do nothing"),
        ]
        
        for code, name, desc in options:
            table.add_row(f"[{code}]", name, desc)
        
        self.console.print(table)
        
        choice = Prompt.ask(
            f"\n[{scheme.primary}]Select pull option[/{scheme.primary}]",
            choices=[str(i) for i in range(1, 9)],
            default='1'
        )
        
        if choice == '8':
            return
        
        if choice == '1':
            result = pull_ops.safe_pull(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("pull", repository=str(repo_path), success=result.success)
        elif choice == '2':
            result = pull_ops.smart_pull(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("smart_pull", repository=str(repo_path), success=result.success)
        elif choice == '3':
            result = pull_ops.pull_rebase(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("pull_rebase", repository=str(repo_path), success=result.success)
        elif choice == '4':
            result = pull_ops.pull_ff_only(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("pull_ff", repository=str(repo_path), success=result.success)
        elif choice == '5':
            result = pull_ops.pull_autostash(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("pull_autostash", repository=str(repo_path), success=result.success)
        elif choice == '6':
            self.console.print("\n[error]🚨 WARNING: Force pull will discard ALL local changes![/error]")
            confirm = Prompt.ask("[error]Type 'DELETE MY WORK' to confirm[/error]", default="cancel")
            if confirm == "DELETE MY WORK":
                result = pull_ops.force_pull(repo_path)
                self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
                log_git_operation("force_pull", repository=str(repo_path), success=result.success)
        elif choice == '7':
            result = pull_ops.fetch_only(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("fetch", repository=str(repo_path), success=result.success)
    
    def handle_sync(self, repo_path: Path):
        """Handle git sync submenu."""
        scheme = self.color_scheme
        sync_ops = SyncOperations()
        self.console.print(f"\n[{scheme.accent}]═══ Git Sync Options ═══[/{scheme.accent}]\n")
        
        table = Table(
            show_header=True,
            header_style=f"bold {scheme.accent}",
            border_style=scheme.accent
        )
        table.add_column("Option", style=scheme.accent, width=8)
        table.add_column("Operation", style=scheme.success, width=25)
        table.add_column("Description", style=scheme.text)
        
        options = [
            ("1", "🟢 Smart Sync", "Intelligent bidirectional sync (RECOMMENDED)"),
            ("2", "🟢 Conservative Sync", "Extra-safe with confirmations"),
            ("3", "🔵 Rebase Sync", "Sync with rebase for clean history"),
            ("4", "🔵 Merge Sync", "Sync with merge commits"),
            ("5", "🟡 Aggressive Sync", "Auto-resolve conflicts"),
            ("6", "🟣 Dry Run Sync", "Preview what would happen"),
            ("7", "❌ Cancel", "Do nothing"),
        ]
        
        for code, name, desc in options:
            table.add_row(f"[{code}]", name, desc)
        
        self.console.print(table)
        
        choice = Prompt.ask(
            f"\n[{scheme.primary}]Select sync option[/{scheme.primary}]",
            choices=[str(i) for i in range(1, 8)],
            default='1'
        )
        
        if choice == '7':
            return
        
        if choice == '1':
            result = sync_ops.smart_sync(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("sync", repository=str(repo_path), success=result.success)
        elif choice == '2':
            result = sync_ops.conservative_sync(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("conservative_sync", repository=str(repo_path), success=result.success)
        elif choice == '3':
            result = sync_ops.rebase_sync(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("rebase_sync", repository=str(repo_path), success=result.success)
        elif choice == '4':
            result = sync_ops.merge_sync(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("merge_sync", repository=str(repo_path), success=result.success)
        elif choice == '5':
            result = sync_ops.aggressive_sync(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("aggressive_sync", repository=str(repo_path), success=result.success)
        elif choice == '6':
            result = sync_ops.dry_run_sync(repo_path)
            self.console.print(f"[primary]✓ {result.message}[/primary]" if result.success else f"[error]✗ {result.message}[/error]")
            log_git_operation("dry_run_sync", repository=str(repo_path), success=result.success)
    
    def handle_status(self, repo_path: Path):
        """Handle git status submenu."""
        status_ops = GitStatus()
        self.console.print(f"\n[{self.color_scheme.accent}]═══ Git Status ═══[/{self.color_scheme.accent}]")
        self.console.print(f"[{self.color_scheme.secondary}]{status_ops.show_status(repo_path)}[/{self.color_scheme.secondary}]")
    
    def handle_branch(self, repo_path: Path):
        """Handle git branch submenu."""
        branch_ops = GitBranch()
        self.console.print(f"\n[{self.color_scheme.accent}]═══ Git Branch Options ═══[/{self.color_scheme.accent}]\n")
        
        options = [
            ("1", "📋 List Branches", "Show all local branches"),
            ("2", "🌿 Create Branch", "Create a new branch"),
            ("3", "🔀 Switch Branch", "Switch to different branch"),
            ("4", "🗑️  Delete Branch", "Delete a branch"),
            ("5", "✏️  Rename Branch", "Rename a branch"),
            ("6", "ℹ️  Branch Info", "Show branch information"),
            ("7", "❌ Cancel", "Do nothing"),
        ]
        
        for code, name, desc in options:
            self.console.print(f"  [{code}] {name:<25} - {desc}")
        
        choice = Prompt.ask(f"\n[{self.color_scheme.primary}]Select branch option[/{self.color_scheme.primary}]", choices=[str(i) for i in range(1, 8)], default='1')
        
        if choice == '7':
            return
        
        if choice == '1':
            branches = branch_ops.list_branches(repo_path)
            self.console.print(f"\n[{self.color_scheme.accent}]Local Branches: [/{self.color_scheme.accent}]")
            for branch in branches:
                marker = "* " if branch.is_current else "  "
                self.console.print(f"{marker}{branch.name}")
        elif choice == '2':
            name = Prompt.ask("Enter new branch name")
            success, msg = branch_ops.create_branch(repo_path, name)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("create_branch", repository=str(repo_path), success=success)
        elif choice == '3':
            name = Prompt.ask("Enter branch name to switch to")
            success, msg = branch_ops.switch_branch(repo_path, name)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("switch_branch", repository=str(repo_path), success=success)
        elif choice == '4':
            name = Prompt.ask("Enter branch name to delete")
            success, msg = branch_ops.delete_branch(repo_path, name)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("delete_branch", repository=str(repo_path), success=success)
        elif choice == '5':
            old_name = Prompt.ask("Enter current branch name")
            new_name = Prompt.ask("Enter new branch name")
            success, msg = branch_ops.rename_branch(repo_path, old_name, new_name)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("rename_branch", repository=str(repo_path), success=success)
        elif choice == '6':
            info = branch_ops.get_branch_info(repo_path)
            self.console.print(f"\n[{self.color_scheme.accent}]Branch: {info['name']}[/{self.color_scheme.accent}]")
            self.console.print(f"[{self.color_scheme.secondary}]Tracking: {info.get('tracking', 'None')}[/{self.color_scheme.secondary}]")
            self.console.print(f"[{self.color_scheme.secondary}]Ahead: {info.get('ahead', 0)}[/{self.color_scheme.secondary}]")
            self.console.print(f"[{self.color_scheme.secondary}]Behind: {info.get('behind', 0)}[/{self.color_scheme.secondary}]")
    
    def handle_stage(self, repo_path: Path):
        """Handle git stage/stash submenu."""
        stage_ops = GitStage()
        self.console.print(f"\n[{self.color_scheme.accent}]═══ Git Stage/Stash Options ═══[/{self.color_scheme.accent}]\n")
        
        options = [
            ("1", "📝 Stage All", "Stage all changes (git add .)"),
            ("2", "📄 Stage File", "Stage specific file"),
            ("3", "🔄 Reset All", "Unstage all changes"),
            ("4", "🔄 Reset File", "Unstage specific file"),
            ("5", "📦 Stash Save", "Stash changes"),
            ("6", "📦 Stash Pop", "Pop stashed changes"),
            ("7", "📦 Stash List", "List all stashes"),
            ("8", "❌ Cancel", "Do nothing"),
        ]
        
        for code, name, desc in options:
            self.console.print(f"  [{code}] {name:<25} - {desc}")
        
        choice = Prompt.ask(f"\n[{self.color_scheme.primary}]Select stage/stash option[/{self.color_scheme.primary}]", choices=[str(i) for i in range(1, 9)], default='1')
        
        if choice == '8':
            return
        
        if choice == '1':
            success, msg = stage_ops.add_all(repo_path)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("stage_all", repository=str(repo_path), success=success)
        elif choice == '2':
            file_path = Prompt.ask("Enter file path")
            success, msg = stage_ops.add_file(repo_path, file_path)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("stage_file", repository=str(repo_path), success=success)
        elif choice == '3':
            success, msg = stage_ops.reset_all(repo_path)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("reset_all", repository=str(repo_path), success=success)
        elif choice == '4':
            file_path = Prompt.ask("Enter file path")
            success, msg = stage_ops.reset_file(repo_path, file_path)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("reset_file", repository=str(repo_path), success=success)
        elif choice == '5':
            message = Prompt.ask("Enter stash message (optional)", default="")
            success, msg = stage_ops.stash_save(repo_path, message or None)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("stash_save", repository=str(repo_path), success=success)
        elif choice == '6':
            success, msg = stage_ops.stash_pop(repo_path)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("stash_pop", repository=str(repo_path), success=success)
        elif choice == '7':
            success, stashes = stage_ops.stash_list(repo_path)
            if success and stashes:
                self.console.print(f"\n[{self.color_scheme.accent}]Stashes:[/{self.color_scheme.accent}]")
                for stash in stashes:
                    self.console.print(f"  [{self.color_scheme.secondary}]{stash}[/{self.color_scheme.secondary}]")
            else:
                self.console.print(f"[{self.color_scheme.info}]No stashes found[/{self.color_scheme.info}]")
    
    def handle_commit(self, repo_path: Path):
        """Handle git commit submenu."""
        commit_ops = GitCommit()
        self.console.print(f"\n[{self.color_scheme.accent}]═══ Git Commit Options ═══[/{self.color_scheme.accent}]\n")
        
        options = [
            ("1", "💾 Commit", "Create commit from staged changes"),
            ("2", "💾 Commit All", "Stage and commit all changes"),
            ("3", "✏️  Amend", "Amend last commit"),
            ("4", "📋 Log", "Show commit log"),
            ("5", "↩️  Revert", "Revert a commit"),
            ("6", "🔄 Reset", "Reset to a commit"),
            ("7", "❌ Cancel", "Do nothing"),
        ]
        
        for code, name, desc in options:
            self.console.print(f"  [{self.color_scheme.accent}]{code}[/{self.color_scheme.accent}] {name:<25} - {desc}")
        
        choice = Prompt.ask(f"\n[{self.color_scheme.primary}]Select commit option[/{self.color_scheme.primary}]", choices=[str(i) for i in range(1, 8)], default='1')
        
        if choice == '7':
            return
        
        if choice == '1':
            message = Prompt.ask("Enter commit message")
            success, msg = commit_ops.commit(repo_path, message)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("commit", repository=str(repo_path), success=success)
        elif choice == '2':
            message = Prompt.ask("Enter commit message")
            success, msg = commit_ops.commit_all(repo_path, message)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("commit_all", repository=str(repo_path), success=success)
        elif choice == '3':
            message = Prompt.ask("Enter new commit message (or leave empty to keep)", default="")
            success, msg = commit_ops.amend(repo_path, message or None, no_edit=not message)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("amend", repository=str(repo_path), success=success)
        elif choice == '4':
            logs = commit_ops.get_log(repo_path, max_count=10)
            self.console.print(f"\n[{self.color_scheme.accent}]Recent Commits:[/{self.color_scheme.accent}]")
            for log in logs:
                self.console.print(f"  [{self.color_scheme.secondary}]{log}[/{self.color_scheme.secondary}]")
        elif choice == '5':
            commit_hash = Prompt.ask("Enter commit hash to revert")
            success, msg = commit_ops.revert(repo_path, commit_hash)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("revert", repository=str(repo_path), success=success)
        elif choice == '6':
            commit_hash = Prompt.ask("Enter commit hash to reset to")
            mode = Prompt.ask("Reset mode", choices=['soft', 'mixed', 'hard'], default='mixed')
            success, msg = commit_ops.reset_to_commit(repo_path, commit_hash, mode)
            self.console.print(f"[{self.color_scheme.success}]✓ {msg}[/{self.color_scheme.success}]" if success else f"[{self.color_scheme.error}]✗ {msg}[/{self.color_scheme.error}]")
            log_git_operation("reset", repository=str(repo_path), success=success)
