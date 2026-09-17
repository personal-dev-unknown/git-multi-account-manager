code part 1

# src/git_manager/core/ssh_workflow.py
"""
Complete SSH key workflow: Generate → Configure → Upload → Test
Handles multi-account setup for GitHub, GitLab, Bitbucket, Gitea, and custom Git servers.
"""

import subprocess
import re
from pathlib import Path
from typing import Optional, Dict, Tuple, List
from dataclasses import dataclass
from enum import Enum

from ..api.github_client import GitHubAPIClient
from ..api.gitlab_client import GitLabAPIClient


class Platform(Enum):
    """Supported Git platforms."""
    GITHUB = "github.com"
    GITLAB = "gitlab.com"
    BITBUCKET = "bitbucket.org"
    GITEA = "gitea"
    CUSTOM = "custom"


@dataclass
class SSHKeyInfo:
    """SSH key information."""
    name: str
    email: str
    platform: Platform
    account_type: str  # e.g., "school", "work", "personal", "zanabuni"
    key_path: Path
    pub_key_path: Path
    ssh_host_alias: str  # e.g., "github.com-school"
    public_key: Optional[str] = None


@dataclass
class SSHTestResult:
    """SSH connection test result."""
    success: bool
    message: str
    username: Optional[str] = None


class SSHWorkflowManager:
    """Manages complete SSH workflow for multiple accounts and platforms."""
    
    def __init__(self, ssh_dir: Optional[Path] = None):
        """
        Initialize SSH workflow manager.
        
        Args:
            ssh_dir: SSH directory (defaults to ~/.ssh)
        """
        self.ssh_dir = ssh_dir or Path.home() / ".ssh"
        self.gitmanager_dir = self.ssh_dir / "gitmanager"
        self.config_file = self.ssh_dir / "config"
        
        # API clients (initialized when tokens available)
        self.github_client: Optional[GitHubAPIClient] = None
        self.gitlab_client: Optional[GitLabAPIClient] = None
    
    def set_github_token(self, token: str):
        """Set GitHub API token."""
        self.github_client = GitHubAPIClient(token)
    
    def set_gitlab_token(self, token: str):
        """Set GitLab API token."""
        self.gitlab_client = GitLabAPIClient(token)
    
    # ========== Step 1: Generate SSH Key ==========
    
    def generate_ssh_key(
        self,
        name: str,
        email: str,
        platform: Platform,
        account_type: str,
        key_type: str = "ed25519",
        passphrase: Optional[str] = None
    ) -> SSHKeyInfo:
        """
        Generate new SSH key for an account.
        
        Args:
            name: Account name/identifier
            email: Email address for the key
            platform: Git platform
            account_type: Account type (school, work, personal, etc.)
            key_type: Key type (ed25519 or rsa)
            passphrase: Optional passphrase for the key
            
        Returns:
            SSHKeyInfo with key details
            
        Example:
            generate_ssh_key(
                name="drmuranja",
                email="drmuranja@example.com",
                platform=Platform.GITHUB,
                account_type="zanabuni"
            )
        """
        # Ensure gitmanager directory exists
        self.gitmanager_dir.mkdir(parents=True, exist_ok=True)
        self.gitmanager_dir.chmod(0o700)
        
        # Create key filename
        key_filename = f"id_{key_type}_{name}"
        key_path = self.gitmanager_dir / key_filename
        pub_key_path = self.gitmanager_dir / f"{key_filename}.pub"
        
        # Check if key already exists
        if key_path.exists():
            raise FileExistsError(f"Key already exists: {key_path}")
        
        # Build ssh-keygen command
        cmd = [
            "ssh-keygen",
            "-t", key_type,
            "-C", email,
            "-f", str(key_path),
        ]
        
        # Add key size for RSA
        if key_type == "rsa":
            cmd.extend(["-b", "4096"])
        
        # Add passphrase handling
        if passphrase:
            cmd.extend(["-N", passphrase])
        else:
            cmd.extend(["-N", ""])  # No passphrase
        
        # Generate key
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Failed to generate SSH key: {result.stderr}")
        
        # Read public key
        public_key = pub_key_path.read_text().strip()
        
        # Create SSH host alias
        ssh_host_alias = f"{platform.value}-{name}"
        
        key_info = SSHKeyInfo(
            name=name,
            email=email,
            platform=platform,
            account_type=account_type,
            key_path=key_path,
            pub_key_path=pub_key_path,
            ssh_host_alias=ssh_host_alias,
            public_key=public_key
        )
        
        return key_info
    
    # ========== Step 2: Start SSH Agent & Add Key ==========
    
    def start_ssh_agent(self) -> Tuple[bool, str]:
        """
        Start SSH agent if not already running.
        
        Returns:
            Tuple of (success, message)
        """
        # Check if agent is running
        try:
            result = subprocess.run(
                ["ssh-add", "-l"],
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                return True, "SSH agent already running"
        except FileNotFoundError:
            return False, "ssh-add command not found"
        
        # Start agent
        result = subprocess.run(
            ["ssh-agent", "-s"],
            capture_output=True,
            text=True,
            shell=True
        )
        
        if result.returncode == 0:
            return True, "SSH agent started"
        else:
            return False, f"Failed to start SSH agent: {result.stderr}"
    
    def add_key_to_agent(self, key_path: Path) -> Tuple[bool, str]:
        """
        Add SSH key to SSH agent.
        
        Args:
            key_path: Path to private key
            
        Returns:
            Tuple of (success, message)
        """
        # Ensure agent is running
        success, msg = self.start_ssh_agent()
        if not success:
            return False, msg
        
        # Add key
        result = subprocess.run(
            ["ssh-add", str(key_path)],
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0:
            return True, f"Key added to agent: {key_path.name}"
        else:
            return False, f"Failed to add key: {result.stderr}"
    
    # ========== Step 3: Configure SSH Config ==========
    
    def configure_ssh_config(self, key_info: SSHKeyInfo) -> bool:
        """
        Add SSH config entry for the key.
        
        Args:
            key_info: SSH key information
            
        Returns:
            True if successful
        """
        # Ensure config file exists
        if not self.config_file.exists():
            self.config_file.touch()
            self.config_file.chmod(0o600)
        
        # Read existing config
        existing_config = self.config_file.read_text()
        
        # Check if host already exists
        if f"Host {key_info.ssh_host_alias}" in existing_config:
            # Update existing entry
            return self._update_ssh_config_entry(key_info)
        
        # Create new entry
        config_entry = f"""
# {key_info.account_type.capitalize()} account ({key_info.name})
Host {key_info.ssh_host_alias}
  HostName {key_info.platform.value}
  User git
  IdentityFile {key_info.key_path}
  IdentitiesOnly yes

"""
        
        # Append to config
        with open(self.config_file, 'a') as f:
            f.write(config_entry)
        
        return True
    
    def _update_ssh_config_entry(self, key_info: SSHKeyInfo) -> bool:
        """Update existing SSH config entry."""
        config_text = self.config_file.read_text()
        
        # Pattern to match the host block
        pattern = rf"Host {re.escape(key_info.ssh_host_alias)}.*?(?=\nHost |\Z)"
        
        new_entry = f"""Host {key_info.ssh_host_alias}
  HostName {key_info.platform.value}
  User git
  IdentityFile {key_info.key_path}
  IdentitiesOnly yes

"""
        
        updated_config = re.sub(pattern, new_entry, config_text, flags=re.DOTALL)
        self.config_file.write_text(updated_config)
        
        return True
    
    # ========== Step 4: Upload Key to Platform ==========
    
    def upload_key_to_platform(
        self,
        key_info: SSHKeyInfo,
        title: Optional[str] = None
    ) -> Tuple[bool, str]:
        """
        Upload SSH key to Git platform via API.
        
        Args:
            key_info: SSH key information
            title: Optional custom title for the key
            
        Returns:
            Tuple of (success, message)
        """
        if not key_info.public_key:
            key_info.public_key = key_info.pub_key_path.read_text().strip()
        
        key_title = title or f"{key_info.name}-{key_info.account_type}"
        
        try:
            if key_info.platform == Platform.GITHUB:
                return self._upload_to_github(key_info.public_key, key_title)
            
            elif key_info.platform == Platform.GITLAB:
                return self._upload_to_gitlab(key_info.public_key, key_title)
            
            else:
                return False, f"Platform {key_info.platform.value} requires manual key upload"
        
        except Exception as e:
            return False, f"Failed to upload key: {str(e)}"
    
    def _upload_to_github(self, public_key: str, title: str) -> Tuple[bool, str]:
        """Upload key to GitHub."""
        if not self.github_client:
            return False, (
                "GitHub token not configured. Please:\n"
                "1. Create token at: https://github.com/settings/tokens\n"
                "2. Add token to app settings\n"
                "3. Retry key upload"
            )
        
        try:
            key = self.github_client.add_ssh_key(title, public_key)
            return True, f"Key added to GitHub (ID: {key.id})"
        except Exception as e:
            return False, f"GitHub upload failed: {str(e)}"
    
    def _upload_to_gitlab(self, public_key: str, title: str) -> Tuple[bool, str]:
        """Upload key to GitLab."""
        if not self.gitlab_client:
            return False, (
                "GitLab token not configured. Please:\n"
                "1. Create token at: https://gitlab.com/-/profile/personal_access_tokens\n"
                "2. Add token to app settings\n"
                "3. Retry key upload"
            )
        
        try:
            key = self.gitlab_client.add_ssh_key(title, public_key)
            return True, f"Key added to GitLab (ID: {key.id})"
        except Exception as e:
            return False, f"GitLab upload failed: {str(e)}"
    
    # ========== Step 5: Test SSH Connection ==========
    
    def test_ssh_connection(self, key_info: SSHKeyInfo) -> SSHTestResult:
        """
        Test SSH connection to platform.
        
        Args:
            key_info: SSH key information
            
        Returns:
            SSHTestResult with connection status
        """
        # Build test command
        cmd = [
            "ssh",
            "-T",
            "-i", str(key_info.key_path),
            f"git@{key_info.platform.value}"
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        # Check response
        output = result.stdout + result.stderr
        
        # GitHub successful response
        if "successfully authenticated" in output.lower():
            # Extract username
            username_match = re.search(r"Hi (.+?)!", output)
            username = username_match.group(1) if username_match else None
            
            return SSHTestResult(
                success=True,
                message="Connection successful!",
                username=username
            )
        
        # GitLab successful response
        elif "Welcome to GitLab" in output:
            username_match = re.search(r"@(.+?)!", output)
            username = username_match.group(1) if username_match else None
            
            return SSHTestResult(
                success=True,
                message="Connection successful!",
                username=username
            )
        
        else:
            return SSHTestResult(
                success=False,
                message=f"Connection failed: {output}"
            )
    
    # ========== Repository Management ==========
    
    def convert_https_to_ssh(
        self,
        https_url: str,
        account_name: str
    ) -> Optional[str]:
        """
        Convert HTTPS URL to SSH URL with correct host alias.
        
        Args:
            https_url: HTTPS repository URL
            account_name: Account name to use
            
        Returns:
            SSH URL or None if invalid
            
        Example:
            convert_https_to_ssh(
                "https://github.com/Zanabuni/react-frontend.git",
                "drmuranja"
            )
            → "git@github.com-drmuranja:Zanabuni/react-frontend.git"
        """
        # Pattern to match repository URLs
        patterns = [
            # HTTPS URLs
            r"https://(?P<platform>github\.com|gitlab\.com|bitbucket\.org)/(?P<path>.+)",
            # SSH URLs (convert to use alias)
            r"git@(?P<platform>github\.com|gitlab\.com|bitbucket\.org):(?P<path>.+)",
        ]
        
        for pattern in patterns:
            match = re.match(pattern, https_url)
            if match:
                platform = match.group('platform')
                path = match.group('path')
                
                # Remove .git if present
                if not path.endswith('.git'):
                    path = path + '.git'
                
                # Build SSH URL with alias
                ssh_url = f"git@{platform}-{account_name}:{path}"
                return ssh_url
        
        return None
    
    def fix_remote_url(
        self,
        repo_path: Path,
        new_url: str,
        remote_name: str = "origin"
    ) -> Tuple[bool, str]:
        """
        Fix repository remote URL.
        
        Args:
            repo_path: Path to repository
            new_url: New remote URL (SSH)
            remote_name: Remote name (default: origin)
            
        Returns:
            Tuple of (success, message)
        """
        try:
            # Set new URL
            result = subprocess.run(
                ["git", "remote", "set-url", remote_name, new_url],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                return True, f"Remote '{remote_name}' updated to: {new_url}"
            else:
                return False, f"Failed to update remote: {result.stderr}"
        
        except Exception as e:
            return False, f"Error updating remote: {str(e)}"
    
    def get_remote_url(
        self,
        repo_path: Path,
        remote_name: str = "origin"
    ) -> Optional[str]:
        """Get current remote URL."""
        try:
            result = subprocess.run(
                ["git", "remote", "get-url", remote_name],
                cwd=repo_path,
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                return result.stdout.strip()
            return None
        
        except Exception:
            return None
    
    # ========== Complete Workflow ==========
    
    def setup_new_account(
        self,
        name: str,
        email: str,
        platform: Platform,
        account_type: str,
        upload_key: bool = True,
        passphrase: Optional[str] = None
    ) -> Dict[str, any]:
        """
        Complete workflow: Generate → Configure → Upload → Test
        
        Args:
            name: Account name
            email: Email address
            platform: Git platform
            account_type: Account type (school, work, personal, etc.)
            upload_key: Whether to upload key via API
            passphrase: Optional key passphrase
            
        Returns:
            Dict with workflow results
        """
        results = {
            "success": False,
            "key_info": None,
            "steps": {}
        }
        
        try:
            # Step 1: Generate key
            key_info = self.generate_ssh_key(
                name=name,
                email=email,
                platform=platform,
                account_type=account_type,
                passphrase=passphrase
            )
            results["key_info"] = key_info
            results["steps"]["generate"] = {
                "success": True,
                "message": f"Key generated: {key_info.key_path}"
            }
            
            # Step 2: Add to agent
            success, msg = self.add_key_to_agent(key_info.key_path)
            results["steps"]["agent"] = {"success": success, "message": msg}
            if not success:
                return results
            
            # Step 3: Configure SSH config
            success = self.configure_ssh_config(key_info)
            results["steps"]["config"] = {
                "success": success,
                "message": "SSH config updated" if success else "Failed to update config"
            }
            if not success:
                return results
            
            # Step 4: Upload key (if requested and token available)
            if upload_key:
                success, msg = self.upload_key_to_platform(key_info)
                results["steps"]["upload"] = {"success": success, "message": msg}
            else:
                results["steps"]["upload"] = {
                    "success": True,
                    "message": "Key upload skipped (manual upload required)"
                }
            
            # Step 5: Test connection
            test_result = self.test_ssh_connection(key_info)
            results["steps"]["test"] = {
                "success": test_result.success,
                "message": test_result.message,
                "username": test_result.username
            }
            
            results["success"] = all(
                step["success"] for step in results["steps"].values()
            )
            
        except Exception as e:
            results["error"] = str(e)
        
        return results


# Example usage
if __name__ == "__main__":
    manager = SSHWorkflowManager()
    
    # Example: Setup school account
    result = manager.setup_new_account(
        name="devonionMoses",
        email="devonion@school.edu",
        platform=Platform.GITHUB,
        account_type="school",
        upload_key=False  # Manual upload first time
    )
    
    print(f"Setup {'successful' if result['success'] else 'failed'}")
    for step, details in result["steps"].items():
        status = "✓" if details["success"] else "✗"
        print(f"{status} {step}: {details['message']}")



code part 2

# src/git_manager/cli/commands/ssh_workflow.py
"""
CLI commands for SSH workflow management.
"""

import click
from rich.console import Console
from rich.panel import Panel
from rich.prompt import Prompt, Confirm
from rich.table import Table
from pathlib import Path

from ...core.ssh_workflow import (
    SSHWorkflowManager,
    Platform,
    SSHKeyInfo
)


console = Console()


@click.group()
def ssh():
    """SSH key management and workflow."""
    pass


@ssh.command()
@click.option('--name', '-n', help='Account name (e.g., devonionMoses)')
@click.option('--email', '-e', help='Email address')
@click.option('--platform', '-p',
              type=click.Choice(['github', 'gitlab', 'bitbucket', 'gitea', 'custom']),
              help='Git platform')
@click.option('--account-type', '-t',
              help='Account type (school, work, personal, company name, etc.)')
@click.option('--upload/--no-upload', default=True,
              help='Upload key to platform via API')
@click.option('--passphrase', is_flag=True,
              help='Use passphrase for key')
def setup_account(
    name: str,
    email: str,
    platform: str,
    account_type: str,
    upload: bool,
    passphrase: bool
):
    """
    Complete SSH account setup workflow.
    
    This will:
    1. Generate SSH key
    2. Add to SSH agent
    3. Configure ~/.ssh/config
    4. Upload to platform (optional)
    5. Test connection
    """
    console.print(Panel(
        "[cyan]SSH Account Setup Wizard[/cyan]\n"
        "Setting up multi-account SSH for Git",
        title="🔐 SSH Setup",
        border_style="cyan"
    ))
    
    # Interactive prompts if options not provided
    if not name:
        name = Prompt.ask("[accent]Account name/username[/accent]")
    
    if not email:
        email = Prompt.ask("[accent]Email address[/accent]")
    
    if not platform:
        console.print("\n[cyan]Select platform:[/cyan]")
        console.print("[1] GitHub")
        console.print("[2] GitLab")
        console.print("[3] Bitbucket")
        console.print("[4] Gitea")
        console.print("[5] Custom")
        platform_choice = Prompt.ask("Choice", choices=['1','2','3','4','5'])
        platform_map = {'1': 'github', '2': 'gitlab', '3': 'bitbucket', '4': 'gitea', '5': 'custom'}
        platform = platform_map[platform_choice]
    
    if not account_type:
        account_type = Prompt.ask(
            "[accent]Account type[/accent]",
            default="personal"
        )
    
    # Convert platform string to enum
    platform_enum = Platform[platform.upper()]
    
    # Get passphrase if requested
    key_passphrase = None
    if passphrase or Confirm.ask("\n[accent]Use passphrase for extra security?[/accent]", default=False):
        import getpass
        key_passphrase = getpass.getpass("Enter passphrase: ")
        key_passphrase_confirm = getpass.getpass("Confirm passphrase: ")
        if key_passphrase != key_passphrase_confirm:
            console.print("[error]Passphrases don't match![/error]")
            return
    
    # Initialize manager
    manager = SSHWorkflowManager()
    
    # Check for API tokens if upload requested
    if upload:
        if platform == 'github':
            token = Prompt.ask(
                "\n[accent]GitHub Personal Access Token[/accent] (leave empty to skip upload)",
                default=""
            )
            if token:
                manager.set_github_token(token)
            else:
                upload = False
        
        elif platform == 'gitlab':
            token = Prompt.ask(
                "\n[accent]GitLab Personal Access Token[/accent] (leave empty to skip upload)",
                default=""
            )
            if token:
                manager.set_gitlab_token(token)
            else:
                upload = False
    
    # Run setup workflow
    console.print("\n[cyan]Starting setup workflow...[/cyan]\n")
    
    with console.status("[bold cyan]Setting up SSH account..."):
        result = manager.setup_new_account(
            name=name,
            email=email,
            platform=platform_enum,
            account_type=account_type,
            upload_key=upload,
            passphrase=key_passphrase
        )
    
    # Display results
    console.print()
    if result["success"]:
        console.print(Panel(
            "[green]✓ SSH Account Setup Complete![/green]",
            border_style="green"
        ))
    else:
        console.print(Panel(
            "[yellow]⚠ Setup completed with warnings[/yellow]",
            border_style="yellow"
        ))
    
    # Show step results
    table = Table(title="Setup Steps", show_header=True)
    table.add_column("Step", style="cyan")
    table.add_column("Status", style="bold")
    table.add_column("Details", style="dim")
    
    step_names = {
        "generate": "1. Generate SSH Key",
        "agent": "2. Add to SSH Agent",
        "config": "3. Configure SSH Config",
        "upload": "4. Upload to Platform",
        "test": "5. Test Connection"
    }
    
    for step_key, step_name in step_names.items():
        if step_key in result["steps"]:
            step = result["steps"][step_key]
            status = "[green]✓[/green]" if step["success"] else "[red]✗[/red]"
            table.add_row(step_name, status, step["message"])
    
    console.print(table)
    
    # Display key information
    if result.get("key_info"):
        key_info = result["key_info"]
        console.print("\n[cyan]SSH Key Information:[/cyan]")
        console.print(f"  Name: {key_info.name}")
        console.print(f"  Email: {key_info.email}")
        console.print(f"  Platform: {key_info.platform.value}")
        console.print(f"  Account Type: {key_info.account_type}")
        console.print(f"  Key Path: {key_info.key_path}")
        console.print(f"  SSH Alias: {key_info.ssh_host_alias}")
        
        if result["steps"].get("test", {}).get("username"):
            console.print(f"\n[green]✓ Connected as: {result['steps']['test']['username']}[/green]")
    
    # Show next steps
    console.print("\n[cyan]Next Steps:[/cyan]")
    
    if not upload:
        console.print(f"\n[yellow]Manual key upload required:[/yellow]")
        console.print(f"1. Copy your public key:")
        console.print(f"   [dim]cat {result['key_info'].pub_key_path}[/dim]")
        console.print(f"2. Go to {platform.capitalize()} SSH settings:")
        
        if platform == 'github':
            console.print(f"   [link]https://github.com/settings/keys[/link]")
        elif platform == 'gitlab':
            console.print(f"   [link]https://gitlab.com/-/profile/keys[/link]")
        
        console.print(f"3. Click 'New SSH key' and paste the key")
    
    console.print(f"\n[green]To use this account, clone with:[/green]")
    console.print(f"  [dim]git clone git@{key_info.ssh_host_alias}:username/repo.git[/dim]")


@ssh.command()
@click.option('--repo-path', '-r', type=click.Path(exists=True), default='.',
              help='Repository path')
@click.option('--account', '-a', help='Account name to use')
def fix_remote(repo_path: str, account: str):
    """Fix repository remote to use correct SSH account."""
    repo_path = Path(repo_path)
    manager = SSHWorkflowManager()
    
    # Get current remote
    current_url = manager.get_remote_url(repo_path)
    
    if not current_url:
        console.print("[error]No remote 'origin' found in repository[/error]")
        return
    
    console.print(f"\n[cyan]Current remote:[/cyan] {current_url}")
    
    # Prompt for account if not provided
    if not account:
        account = Prompt.ask("[accent]Which account should this repo use?[/accent]")
    
    # Convert URL
    new_url = manager.convert_https_to_ssh(current_url, account)
    
    if not new_url:
        console.print("[error]Could not convert URL. Please check the format.[/error]")
        return
    
    console.print(f"[cyan]New remote:[/cyan] {new_url}")
    
    if Confirm.ask("\n[accent]Update remote URL?[/accent]", default=True):
        success, message = manager.fix_remote_url(repo_path, new_url)
        
        if success:
            console.print(f"[success]✓ {message}[/success]")
            
            # Verify
            verify_url = manager.get_remote_url(repo_path)
            console.print(f"\n[info]Verified remote:[/info] {verify_url}")
        else:
            console.print(f"[error]✗ {message}[/error]")


@ssh.command()
@click.argument('url')
@click.option('--account', '-a', required=True, help='Account name to use')
def convert_url(url: str, account: str):
    """Convert HTTPS URL to SSH URL with account alias."""
    manager = SSHWorkflowManager()
    
    ssh_url = manager.convert_https_to_ssh(url, account)
    
    if ssh_url:
        console.print(f"\n[cyan]Original URL:[/cyan]")
        console.print(f"  {url}")
        console.print(f"\n[green]SSH URL (with account '{account}'):[/green]")
        console.print(f"  {ssh_url}")
        console.print(f"\n[info]Use this to clone:[/info]")
        console.print(f"  [dim]git clone {ssh_url}[/dim]")
    else:
        console.print("[error]Could not convert URL. Invalid format?[/error]")


@ssh.command()
@click.option('--account', '-a', help='Test specific account')
def test_connection(account: str):
    """Test SSH connection to Git platforms."""
    manager = SSHWorkflowManager()
    
    # List available accounts from SSH config
    ssh_config = manager.config_file
    
    if not ssh_config.exists():
        console.print("[error]No SSH config found. Set up an account first.[/error]")
        return
    
    console.print(Panel(
        "[cyan]Testing SSH Connections[/cyan]",
        border_style="cyan"
    ))
    
    # Parse config for git hosts
    config_text = ssh_config.read_text()
    hosts = []
    
    for line in config_text.split('\n'):
        if line.strip().startswith('Host ') and 'github.com' in line or 'gitlab.com' in line:
            host = line.split()[1]
            hosts.append(host)
    
    if not hosts:
        console.print("[warning]No Git SSH hosts found in config[/warning]")
        return
    
    # Test each host
    table = Table(title="Connection Tests", show_header=True)
    table.add_column("Account", style="cyan")
    table.add_column("Status", style="bold")
    table.add_column("Username", style="green")
    table.add_column("Details", style="dim")
    
    for host in hosts:
        # Create dummy key info for testing
        key_info = SSHKeyInfo(
            name=host,
            email="",
            platform=Platform.GITHUB if 'github' in host else Platform.GITLAB,
            account_type="",
            key_path=Path.home() / ".ssh" / "id_ed25519",
            pub_key_path=Path.home() / ".ssh" / "id_ed25519.pub",
            ssh_host_alias=host
        )
        
        with console.status(f"Testing {host}..."):
            result = manager.test_ssh_connection(key_info)
        
        status = "[green]✓ Connected[/green]" if result.success else "[red]✗ Failed[/red]"
        username = result.username or "N/A"
        
        table.add_row(
            host,
            status,
            username,
            result.message[:50]
        )
    
    console.print(table)


@ssh.command()
def list_accounts():
    """List all configured SSH accounts."""
    manager = SSHWorkflowManager()
    
    if not manager.config_file.exists():
        console.print("[warning]No SSH config found[/warning]")
        return
    
    console.print(Panel(
        "[cyan]Configured SSH Accounts[/cyan]",
        border_style="cyan"
    ))
    
    # Parse SSH config
    config_text = manager.config_file.read_text()
    accounts = []
    current_account = {}
    
    for line in config_text.split('\n'):
        line = line.strip()
        
        if line.startswith('Host ') and ('github' in line or 'gitlab' in line):
            if current_account:
                accounts.append(current_account)
            current_account = {'host': line.split()[1]}
        
        elif line.startswith('HostName '):
            current_account['hostname'] = line.split()[1]
        
        elif line.startswith('IdentityFile '):
            current_account['key'] = line.split()[1]
    
    if current_account:
        accounts.append(current_account)
    
    if not accounts:
        console.print("[warning]No Git accounts found in SSH config[/warning]")
        return
    
    # Display table
    table = Table(show_header=True)
    table.add_column("Alias", style="cyan")
    table.add_column("Platform", style="blue")
    table.add_column("SSH Key", style="green")
    table.add_column("Status", style="yellow")
    
    for account in accounts:
        key_path = Path(account.get('key', '').replace('~', str(Path.home())))
        key_exists = "✓" if key_path.exists() else "✗ Missing"
        
        table.add_row(
            account['host'],
            account.get('hostname', 'Unknown'),
            account.get('key', 'N/A'),
            key_exists
        )
    
    console.print(table)
    
    console.print(f"\n[info]Total: {len(accounts)} accounts configured[/info]")


# Integration with main CLI
def setup_ssh_commands(cli_app):
    """Add SSH commands to main CLI."""
    cli_app.add_command(ssh)


if __name__ == "__main__":
    ssh()


code part 3

# src/git_manager/cli/ui/interactive_with_ssh.py
"""
Enhanced interactive mode with integrated SSH workflow.
"""

from rich.console import Console
from rich.panel import Panel
from rich.prompt import Prompt, Confirm
from rich.table import Table

from ...core.account_manager import AccountManager
from ...core.ssh_manager import SSHManager
from ...core.git_operations import GitOperations
from ...core.ssh_workflow import SSHWorkflowManager, Platform


class EnhancedInteractiveMode:
    """Interactive CLI mode with SSH workflow integration."""
    
    def __init__(
        self,
        account_manager: AccountManager,
        ssh_manager: SSHManager,
        git_operations: GitOperations,
        console: Console
    ):
        self.account_manager = account_manager
        self.ssh_manager = ssh_manager
        self.git_operations = git_operations
        self.console = console
        self.ssh_workflow = SSHWorkflowManager()
    
    def run(self):
        """Run interactive mode."""
        while True:
            self.show_menu()
            choice = Prompt.ask(
                "[accent]Select option[/accent]",
                choices=['1', '2', '3', '4', '5', '6', '7', '8', '9', '10'],
                default='10'
            )
            
            if choice == '10':
                self.console.print("[info]Goodbye![/info]")
                break
            
            self.handle_choice(choice)
    
    def show_menu(self):
        """Display enhanced main menu."""
        menu = """
        [primary]═══ Repository Operations ═══[/primary]
        [accent][1][/accent] Clone repository (with smart account selection)
        [accent][2][/accent] Check current repository account
        [accent][3][/accent] Git pull (with account verification)
        [accent][4][/accent] Git push (with account verification)
        [accent][5][/accent] Fix repository remote URL

        [primary]═══ SSH Account Management ═══[/primary]
        [accent][6][/accent] Setup new SSH account (complete workflow)
        [accent][7][/accent] Test SSH connections
        [accent][8][/accent] List configured accounts
        [accent][9][/accent] Convert URL to SSH

        [accent]10.[/accent] Exit
        """
        panel = Panel(menu, title="Main Menu", border_style="primary")
        self.console.print(panel)
    
    def handle_choice(self, choice: str):
        """Handle menu choice."""
        actions = {
            '1': self.clone_repo_smart,
            '2': self.check_status,
            '3': self.git_pull,
            '4': self.git_push,
            '5': self.fix_remote,
            '6': self.setup_ssh_account,
            '7': self.test_ssh_connections,
            '8': self.list_accounts,
            '9': self.convert_url
        }
        
        action = actions.get(choice)
        if action:
            try:
                action()
            except Exception as e:
                self.console.print(f"[error]Error: {e}[/error]")
        
        self.console.print()
    
    # ========== Enhanced Repository Operations ==========
    
    def clone_repo_smart(self):
        """Smart clone with automatic URL conversion."""
        self.console.print("\n[primary]═══ Clone Repository ═══[/primary]\n")
        
        # Step 1: Get repository URL (any format)
        url = Prompt.ask("[accent]Enter repository URL[/accent]")
        
        # Step 2: List available accounts
        accounts = self.account_manager.list_accounts()
        
        if not accounts:
            self.console.print("[warning]No accounts configured. Setting up first account...[/warning]")
            self.setup_ssh_account()
            return
        
        # Display accounts
        table = Table(title="Available Accounts", show_header=True)
        table.add_column("#", style="cyan", width=3)
        table.add_column("Account", style="green")
        table.add_column("Platform", style="blue")
        table.add_column("Type", style="yellow")
        
        for i, account in enumerate(accounts, 1):
            platform = "GitHub" if "github" in account.host else "GitLab"
            table.add_row(
                str(i),
                account.name,
                platform,
                account.description or "N/A"
            )
        
        self.console.print(table)
        
        # Step 3: Select account
        choice = Prompt.ask(
            "\n[accent]Select account to use[/accent]",
            choices=[str(i) for i in range(1, len(accounts) + 1)]
        )
        selected_account = accounts[int(choice) - 1]
        
        # Step 4: Convert URL to SSH
        ssh_url = self.ssh_workflow.convert_https_to_ssh(url, selected_account.name)
        
        if not ssh_url:
            self.console.print("[error]Could not convert URL. Using original URL...[/error]")
            ssh_url = url
        
        # Display conversion
        self.console.print(f"\n[info]Original URL:[/info] {url}")
        self.console.print(f"[success]SSH URL:[/success] {ssh_url}")
        
        # Step 5: Clone
        if Confirm.ask(f"\n[accent]Clone with account '{selected_account.name}'?[/accent]", default=True):
            self.console.print(f"\n[cyan]Cloning...[/cyan]")
            
            try:
                import subprocess
                result = subprocess.run(
                    ["git", "clone", ssh_url],
                    capture_output=True,
                    text=True
                )
                
                if result.returncode == 0:
                    self.console.print(f"[success]✓ Repository cloned successfully![/success]")
                else:
                    self.console.print(f"[error]✗ Clone failed: {result.stderr}[/error]")
            
            except Exception as e:
                self.console.print(f"[error]Error: {e}[/error]")
    
    def fix_remote(self):
        """Fix repository remote URL."""
        self.console.print("\n[primary]═══ Fix Remote URL ═══[/primary]\n")
        
        from pathlib import Path
        repo_path = Path.cwd()
        
        # Get current remote
        current_url = self.ssh_workflow.get_remote_url(repo_path)
        
        if not current_url:
            self.console.print("[error]Not in a git repository or no remote 'origin' found[/error]")
            return
        
        self.console.print(f"[info]Current remote:[/info] {current_url}\n")
        
        # Check if it's already SSH with account alias
        if "@" in current_url and "-" in current_url.split("@")[1].split(":")[0]:
            self.console.print("[success]Remote already using SSH with account alias![/success]")
            
            if not Confirm.ask("[accent]Change to different account?[/accent]", default=False):
                return
        
        # Select account
        accounts = self.account_manager.list_accounts()
        
        table = Table(title="Available Accounts", show_header=True)
        table.add_column("#", style="cyan", width=3)
        table.add_column("Account", style="green")
        table.add_column("Platform", style="blue")
        
        for i, account in enumerate(accounts, 1):
            platform = "GitHub" if "github" in account.host else "GitLab"
            table.add_row(str(i), account.name, platform)
        
        self.console.print(table)
        
        choice = Prompt.ask(
            "\n[accent]Select account for this repository[/accent]",
            choices=[str(i) for i in range(1, len(accounts) + 1)]
        )
        selected_account = accounts[int(choice) - 1]
        
        # Convert URL
        new_url = self.ssh_workflow.convert_https_to_ssh(current_url, selected_account.name)
        
        if not new_url:
            self.console.print("[error]Could not convert URL[/error]")
            return
        
        self.console.print(f"\n[success]New remote:[/success] {new_url}")
        
        if Confirm.ask("\n[accent]Update remote URL?[/accent]", default=True):
            success, message = self.ssh_workflow.fix_remote_url(repo_path, new_url)
            
            if success:
                self.console.print(f"[success]✓ {message}[/success]")
                
                # Verify
                verify_url = self.ssh_workflow.get_remote_url(repo_path)
                self.console.print(f"[info]Verified:[/info] {verify_url}")
            else:
                self.console.print(f"[error]✗ {message}[/error]")
    
    # ========== SSH Account Management ==========
    
    def setup_ssh_account(self):
        """Setup new SSH account with complete workflow."""
        self.console.print(Panel(
            "[cyan]SSH Account Setup Wizard[/cyan]\n"
            "This will generate SSH key, configure it, and test connection",
            title="🔐 SSH Setup",
            border_style="cyan"
        ))
        
        # Collect information
        name = Prompt.ask("\n[accent]Account name/username[/accent]")
        email = Prompt.ask("[accent]Email address[/accent]")
        
        # Platform selection
        self.console.print("\n[cyan]Select platform:[/cyan]")
        self.console.print("[1] GitHub")
        self.console.print("[2] GitLab")
        self.console.print("[3] Bitbucket")
        self.console.print("[4] Other")
        
        platform_choice = Prompt.ask("Choice", choices=['1','2','3','4'])
        platform_map = {
            '1': Platform.GITHUB,
            '2': Platform.GITLAB,
            '3': Platform.BITBUCKET,
            '4': Platform.CUSTOM
        }
        platform = platform_map[platform_choice]
        
        account_type = Prompt.ask(
            "\n[accent]Account type[/accent] (school, work, personal, company, etc.)",
            default="personal"
        )
        
        # Passphrase
        use_passphrase = Confirm.ask(
            "\n[accent]Use passphrase for extra security?[/accent]",
            default=False
        )
        
        passphrase = None
        if use_passphrase:
            import getpass
            passphrase = getpass.getpass("Enter passphrase: ")
            confirm = getpass.getpass("Confirm passphrase: ")
            if passphrase != confirm:
                self.console.print("[error]Passphrases don't match![/error]")
                return
        
        # API token for upload
        upload_key = False
        if platform in [Platform.GITHUB, Platform.GITLAB]:
            upload_key = Confirm.ask(
                "\n[accent]Upload key to platform via API?[/accent]",
                default=False
            )
            
            if upload_key:
                token = Prompt.ask(
                    f"[accent]Enter {platform.name} Personal Access Token[/accent]",
                    password=True
                )
                
                if platform == Platform.GITHUB:
                    self.ssh_workflow.set_github_token(token)
                else:
                    self.ssh_workflow.set_gitlab_token(token)
        
        # Run workflow
        self.console.print("\n[cyan]Starting SSH setup workflow...[/cyan]\n")
        
        with self.console.status("[bold cyan]Setting up account..."):
            result = self.ssh_workflow.setup_new_account(
                name=name,
                email=email,
                platform=platform,
                account_type=account_type,
                upload_key=upload_key,
                passphrase=passphrase
            )
        
        # Display results
        self.console.print()
        if result["success"]:
            self.console.print(Panel(
                "[green]✓ SSH Account Setup Complete![/green]",
                border_style="green"
            ))
        else:
            self.console.print(Panel(
                "[yellow]⚠ Setup completed with warnings[/yellow]",
                border_style="yellow"
            ))
        
        # Show steps
        table = Table(title="Setup Steps", show_header=True)
        table.add_column("Step", style="cyan")
        table.add_column("Status", style="bold")
        table.add_column("Details", style="dim")
        
        step_names = {
            "generate": "1. Generate SSH Key",
            "agent": "2. Add to SSH Agent",
            "config": "3. Configure SSH Config",
            "upload": "4. Upload to Platform",
            "test": "5. Test Connection"
        }
        
        for step_key, step_name in step_names.items():
            if step_key in result["steps"]:
                step = result["steps"][step_key]
                status = "[green]✓[/green]" if step["success"] else "[red]✗[/red]"
                table.add_row(step_name, status, step["message"])
        
        self.console.print(table)
        
        # Show key info
        if result.get("key_info"):
            key_info = result["key_info"]
            self.console.print("\n[cyan]Account Information:[/cyan]")
            self.console.print(f"  Name: {key_info.name}")
            self.console.print(f"  Alias: {key_info.ssh_host_alias}")
            self.console.print(f"  Key: {key_info.key_path}")
            
            if result["steps"].get("test", {}).get("username"):
                self.console.print(f"\n[green]✓ Connected as: {result['steps']['test']['username']}[/green]")
            
            self.console.print(f"\n[info]To clone repos, use:[/info]")
            self.console.print(f"  git clone git@{key_info.ssh_host_alias}:username/repo.git")
    
    def test_ssh_connections(self):
        """Test all SSH connections."""
        self.console.print("\n[primary]═══ Test SSH Connections ═══[/primary]\n")
        
        accounts = self.account_manager.list_accounts()
        
        if not accounts:
            self.console.print("[warning]No accounts configured[/warning]")
            return
        
        table = Table(title="Connection Tests", show_header=True)
        table.add_column("Account", style="cyan")
        table.add_column("Platform", style="blue")
        table.add_column("Status", style="bold")
        table.add_column("Username", style="green")
        
        for account in accounts:
            with self.console.status(f"Testing {account.name}..."):
                # Determine platform
                if "github" in account.host:
                    platform = Platform.GITHUB
                elif "gitlab" in account.host:
                    platform = Platform.GITLAB
                else:
                    platform = Platform.CUSTOM
                
                # Create key info for testing
                from pathlib import Path
                key_info = type('obj', (object,), {
                    'name': account.name,
                    'email': '',
                    'platform': platform,
                    'account_type': account.description or '',
                    'key_path': Path(account.ssh_key_path),
                    'pub_key_path': Path(account.ssh_key_path + '.pub'),
                    'ssh_host_alias': account.host
                })()
                
                result = self.ssh_workflow.test_ssh_connection(key_info)
                
                status = "[green]✓ Connected[/green]" if result.success else "[red]✗ Failed[/red]"
                platform_name = "GitHub" if "github" in account.host else "GitLab"
                username = result.username or "N/A"
                
                table.add_row(account.name, platform_name, status, username)
        
        self.console.print(table)
    
    def list_accounts(self):
        """List all configured accounts."""
        self.console.print("\n[primary]═══ Configured Accounts ═══[/primary]\n")
        
        accounts = self.account_manager.list_accounts()
        
        if not accounts:
            self.console.print("[warning]No accounts configured[/warning]")
            self.console.print("[info]Use option [6] to setup a new account[/info]")
            return
        
        table = Table(show_header=True)
        table.add_column("#", style="cyan", width=3)
        table.add_column("Name", style="green")
        table.add_column("Platform", style="blue")
        table.add_column("Type", style="yellow")
        table.add_column("SSH Alias", style="magenta")
        table.add_column("Key", style="dim")
        
        for i, account in enumerate(accounts, 1):
            platform = "GitHub" if "github" in account.host else "GitLab"
            from pathlib import Path
            key_exists = "✓" if Path(account.ssh_key_path).exists() else "✗"
            
            table.add_row(
                str(i),
                account.name,
                platform,
                account.description or "N/A",
                account.host,
                f"{key_exists} {Path(account.ssh_key_path).name}"
            )
        
        self.console.print(table)
        self.console.print(f"\n[info]Total: {len(accounts)} accounts[/info]")
    
    def convert_url(self):
        """Convert URL to SSH format."""
        self.console.print("\n[primary]═══ Convert URL to SSH ═══[/primary]\n")
        
        url = Prompt.ask("[accent]Enter repository URL[/accent]")
        
        accounts = self.account_manager.list_accounts()
        
        if not accounts:
            self.console.print("[warning]No accounts configured[/warning]")
            return
        
        # Show accounts
        for i, account in enumerate(accounts, 1):
            self.console.print(f"[{i}] {account.name}")
        
        choice = Prompt.ask(
            "\n[accent]Select account[/accent]",
            choices=[str(i) for i in range(1, len(accounts) + 1)]
        )
        selected_account = accounts[int(choice) - 1]
        
        # Convert
        ssh_url = self.ssh_workflow.convert_https_to_ssh(url, selected_account.name)
        
        if ssh_url:
            self.console.print(f"\n[cyan]Original:[/cyan] {url}")
            self.console.print(f"[green]SSH URL:[/green] {ssh_url}")
            self.console.print(f"\n[info]Use this to clone:[/info]")
            self.console.print(f"  git clone {ssh_url}")
        else:
            self.console.print("[error]Could not convert URL[/error]")
    
    # ========== Standard Git Operations ==========
    
    def check_status(self):
        """Check repository status."""
        from .tables import display_repository_status
        
        status = self.git_operations.check_status()
        display_repository_status(status, self.console)
    
    def git_pull(self):
        """Git pull."""
        success, message = self.git_operations.pull()
        if success:
            self.console.print(f"[success]✓ {message}[/success]")
        else:
            self.console.print(f"[error]✗ {message}[/error]")
    
    def git_push(self):
        """Git push."""
        success, message = self.git_operations.push()
        if success:
            self.console.print(f"[success]✓ {message}[/success]")
        else:
            self.console.print(f"[error]✗ {message}[/error]")