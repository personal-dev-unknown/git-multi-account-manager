# src/git_manager/core/ssh/orchestrator.py
"""
SSH Workflow Orchestrator - Complete Integration

Orchestrates the complete SSH workflow:
✅ Key Generation
✅ SSH Agent Management
✅ SSH Config Management
✅ Connection Testing
✅ URL Conversion
✅ Error Recovery

This is the main entry point for SSH operations.
"""

from pathlib import Path
from typing import Dict, Tuple, Optional, List
import subprocess
import re

from .key_generator import SSHKeyGenerator, KeyMetadata
from .agent_manager import SSHAgentManager
from .config_manager import SSHConfigManager, SSHHostEntry


class SSHWorkflowOrchestrator:
    """
    Complete SSH workflow orchestration.
    
    Coordinates:
    - Key generation
    - Agent management
    - Config management
    - Connection testing
    - URL conversion
    """
    
    def __init__(self, ssh_dir: Optional[Path] = None):
        """
        Initialize orchestrator.
        
        Args:
            ssh_dir: SSH directory (defaults to ~/.ssh)
        """
        self.ssh_dir = ssh_dir or Path.home() / ".ssh"
        
        # Initialize managers
        self.key_generator = SSHKeyGenerator(ssh_dir)
        self.agent_manager = SSHAgentManager()
        self.config_manager = SSHConfigManager(ssh_dir)
    
    def setup_account(
        self,
        name: str,
        email: str,
        platform: str,
        account_type: str,
        key_type: str = "ed25519",
        passphrase: Optional[str] = None,
        upload_key: bool = False
    ) -> Dict[str, any]:
        """
        Complete account setup workflow.
        
        This orchestrates:
        1. Generate SSH key
        2. Start SSH agent
        3. Add key to agent
        4. Configure SSH config
        5. Test connection
        
        Args:
            name: Account name
            email: Email address
            platform: Git platform (github, gitlab, bitbucket, etc.)
            account_type: Account type (school, work, personal, etc.)
            key_type: Key type (ed25519 or rsa)
            passphrase: Optional passphrase
            upload_key: Whether to upload key via API (not implemented here)
            
        Returns:
            Dict with workflow results
        """
        results = {
            "success": False,
            "steps": {},
            "key_info": None,
            "ssh_host_alias": None,
            "errors": []
        }
        
        try:
            # Step 1: Generate SSH key
            success, msg, key_info = self.key_generator.generate_key(
                name=name,
                email=email,
                platform=platform,
                account_type=account_type,
                key_type=key_type,
                passphrase=passphrase
            )
            
            results["steps"]["generate_key"] = {
                "success": success,
                "message": msg
            }
            
            if not success:
                results["errors"].append(f"Key generation failed: {msg}")
                return results
            
            results["key_info"] = key_info
            key_path = Path(key_info["key_path"])
            ssh_host_alias = key_info["ssh_host_alias"]
            results["ssh_host_alias"] = ssh_host_alias
            
            # Step 2: Start SSH agent
            success, msg = self.agent_manager.start()
            results["steps"]["start_agent"] = {
                "success": success,
                "message": msg
            }
            
            if not success:
                results["errors"].append(f"Agent start failed: {msg}")
                # Continue anyway, might already be running
            
            # Step 3: Add key to agent
            success, msg = self.agent_manager.add_key(key_path, passphrase)
            results["steps"]["add_to_agent"] = {
                "success": success,
                "message": msg
            }
            
            if not success:
                results["errors"].append(f"Add to agent failed: {msg}")
                # Continue anyway, key can be used without agent
            
            # Step 4: Configure SSH config
            success, msg = self.config_manager.add_host_entry(
                host=ssh_host_alias,
                hostname=platform,
                identity_file=str(key_path),
                user="git",
                comment=f"{account_type.capitalize()} account ({name})"
            )
            
            results["steps"]["configure_ssh"] = {
                "success": success,
                "message": msg
            }
            
            if not success:
                results["errors"].append(f"SSH config failed: {msg}")
                return results
            
            # Step 5: Test connection
            success, msg, username = self._test_connection(platform, key_path)
            results["steps"]["test_connection"] = {
                "success": success,
                "message": msg,
                "username": username
            }
            
            if not success:
                results["errors"].append(f"Connection test failed: {msg}")
                # Don't fail completely, key might still work
            
            # Overall success
            results["success"] = all(
                step.get("success", False)
                for step in results["steps"].values()
            )
            
            return results
        
        except Exception as e:
            results["errors"].append(f"Unexpected error: {str(e)}")
            return results
    
    def _test_connection(
        self,
        platform: str,
        key_path: Path
    ) -> Tuple[bool, str, Optional[str]]:
        """
        Test SSH connection to platform for all 8 supported platforms.
        
        Supports:
        - GitHub (github.com)
        - GitLab (gitlab.com)
        - Bitbucket (bitbucket.org)
        - Azure DevOps (ssh.dev.azure.com)
        - Self-Hosted (custom domain)
        - SourceForge (git.code.sf.net)
        - Cloud Storage (not SSH-based, returns False)
        - Local Path (not SSH-based, returns False)
        
        Args:
            platform: Platform domain (e.g., 'github.com', 'gitlab.com')
            key_path: Path to SSH private key
        
        Returns:
            Tuple of (success, message, username)
        """
        try:
            # Map platforms to their SSH hosts and response patterns
            platform_config = {
                'github.com': {
                    'host': 'github.com',
                    'patterns': [r"Hi (.+?)!", r"Hi (.+?),"],
                    'keywords': ['successfully authenticated', 'hi ']
                },
                'gitlab.com': {
                    'host': 'gitlab.com',
                    'patterns': [r"Welcome to GitLab, @(.+?)!", r"@(.+?)!"],
                    'keywords': ['welcome to gitlab', 'gitlab']
                },
                'bitbucket.org': {
                    'host': 'bitbucket.org',
                    'patterns': [r"authenticated as (.+?)"],
                    'keywords': ['authenticated']
                },
                'ssh.dev.azure.com': {
                    'host': 'ssh.dev.azure.com',
                    'patterns': [r"authenticated as (.+?)"],
                    'keywords': ['authenticated', 'azure']
                },
                'git.code.sf.net': {
                    'host': 'git.code.sf.net',
                    'patterns': [r"authenticated as (.+?)"],
                    'keywords': ['authenticated', 'sourceforge']
                }
            }
            
            # Get host configuration
            host_config = platform_config.get(platform)
            if not host_config:
                # For self-hosted or unknown platforms, use the platform directly
                host_config = {
                    'host': platform,
                    'patterns': [r"authenticated as (.+?)", r"Hi (.+?)!", r"@(.+?)!"],
                    'keywords': ['authenticated', 'successfully', 'welcome', 'hi ']
                }
            
            # Build SSH command with increased timeout
            cmd = [
                "ssh",
                "-T",
                "-i", str(key_path),
                f"git@{host_config['host']}"
            ]
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=30  # Increased from 10 to 30 seconds
            )
            
            output = result.stdout + result.stderr
            username = None
            
            # Check for successful authentication using keywords
            output_lower = output.lower()
            success = any(keyword in output_lower for keyword in host_config['keywords'])
            
            if success:
                # Try to extract username from response
                for pattern in host_config['patterns']:
                    match = re.search(pattern, output, re.IGNORECASE)
                    if match:
                        username = match.group(1)
                        break
                
                return True, "Connection successful!", username
            else:
                return False, f"Connection failed: {output[:100]}", None
        
        except subprocess.TimeoutExpired:
            return False, "Connection test timed out", None
        except Exception as e:
            return False, f"Connection test error: {str(e)}", None
    
    def test_connection(
        self,
        account_name: str,
        key_path: Optional[Path] = None
    ) -> Tuple[bool, str]:
        """
        Public method to test SSH connection for an account.
        
        Args:
            account_name: Name of the account to test
            key_path: Optional path to SSH key (uses default if not provided)
        
        Returns:
            Tuple of (success, message)
        """
        try:
            # Get account metadata
            metadata = self.key_generator.get_metadata(account_name)
            if not metadata:
                return False, f"Account '{account_name}' not found"
            
            # Determine key path
            if not key_path:
                key_path = Path(metadata.key_path)
            
            if not key_path.exists():
                return False, f"SSH key not found: {key_path}"
            
            # Get platform from metadata
            platform = metadata.platform
            
            # Test connection
            success, message, username = self._test_connection(platform, key_path)
            
            return success, message
        
        except Exception as e:
            return False, f"Error testing connection: {str(e)}"
    
    def convert_https_to_ssh(
        self,
        https_url: str,
        account_name: str
    ) -> Optional[str]:
        """
        Convert HTTPS URL to SSH URL with account alias.
        
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
        try:
            # Pattern to match repository URLs
            patterns = [
                # HTTPS URLs
                r"https://(?P<platform>github\.com|gitlab\.com|bitbucket\.org)/(?P<path>.+)",
                # SSH URLs (convert to use alias)
                r"git@(?P<platform>github\.com|gitlab\.com|bitbucket\.org):(?P<path>.+)",
                # Short form
                r"(?P<platform>github\.com|gitlab\.com|bitbucket\.org)[:/](?P<path>.+)",
            ]
            
            for pattern in patterns:
                match = re.match(pattern, https_url)
                if match:
                    platform = match.group('platform')
                    path = match.group('path')
                    
                    # Ensure .git suffix
                    if not path.endswith('.git'):
                        path = path + '.git'
                    
                    # Build SSH URL with alias
                    ssh_url = f"git@{platform}-{account_name}:{path}"
                    return ssh_url
            
            return None
        
        except Exception:
            return None
    
    def fix_remote_url(
        self,
        repo_path: Path,
        account_name: str,
        remote_name: str = "origin"
    ) -> Tuple[bool, str]:
        """
        Fix repository remote URL to use correct account.
        
        Args:
            repo_path: Path to repository
            account_name: Account name to use
            remote_name: Remote name (default: origin)
            
        Returns:
            Tuple of (success, message)
        """
        try:
            # Get current remote
            result = subprocess.run(
                ["git", "remote", "get-url", remote_name],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=5
            )
            
            if result.returncode != 0:
                return False, f"Remote '{remote_name}' not found"
            
            current_url = result.stdout.strip()
            
            # Convert to SSH
            new_url = self.convert_https_to_ssh(current_url, account_name)
            
            if not new_url:
                return False, f"Could not convert URL: {current_url}"
            
            # Update remote
            result = subprocess.run(
                ["git", "remote", "set-url", remote_name, new_url],
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=5
            )
            
            if result.returncode == 0:
                return True, f"Remote updated: {new_url}"
            else:
                return False, f"Failed to update remote: {result.stderr}"
        
        except Exception as e:
            return False, f"Error fixing remote: {str(e)}"
    
    def list_accounts(self) -> List[Dict[str, any]]:
        """
        List all configured SSH accounts.
        
        Returns:
            List of account info dicts
        """
        accounts = []
        
        try:
            # Get SSH config entries
            entries = self.config_manager.list_host_entries()
            
            for entry in entries:
                # Get metadata if available
                # Extract account name from host alias
                # Format: platform-name (e.g., github.com-devonionMoses)
                parts = entry.host.split('-', 1)
                if len(parts) == 2:
                    account_name = parts[1]
                    metadata = self.key_generator.get_metadata(account_name)
                    
                    accounts.append({
                        "name": account_name,
                        "host_alias": entry.host,
                        "platform": entry.hostname,
                        "identity_file": entry.identity_file,
                        "metadata": metadata.__dict__ if metadata else None
                    })
        
        except Exception:
            pass
        
        return accounts
    
    def get_account_status(self, account_name: str) -> Dict[str, any]:
        """
        Get detailed status for an account.
        
        Args:
            account_name: Account name
            
        Returns:
            Dict with account status
        """
        status = {
            "name": account_name,
            "configured": False,
            "key_exists": False,
            "in_agent": False,
            "metadata": None,
            "errors": []
        }
        
        try:
            # Get metadata
            metadata = self.key_generator.get_metadata(account_name)
            if metadata:
                status["metadata"] = metadata.__dict__
            
            # Check if key exists
            key_path = self.key_generator.gitmanager_dir / f"id_ed25519_{account_name}"
            if key_path.exists():
                status["key_exists"] = True
            
            # Check if in agent
            success, keys = self.agent_manager.list_keys()
            if success:
                for key in keys:
                    if account_name in key.get("comment", ""):
                        status["in_agent"] = True
                        break
            
            # Check if configured in SSH config
            accounts = self.list_accounts()
            for acc in accounts:
                if acc["name"] == account_name:
                    status["configured"] = True
                    break
            
            return status
        
        except Exception as e:
            status["errors"].append(str(e))
            return status


# Example usage
if __name__ == "__main__":
    orchestrator = SSHWorkflowOrchestrator()
    
    # Setup account
    result = orchestrator.setup_account(
        name="devonionMoses",
        email="moses@school.edu",
        platform="github.com",
        account_type="school"
    )
    
    if result["success"]:
        print("✓ Account setup successful!")
        print(f"  SSH Alias: {result['ssh_host_alias']}")
    else:
        print("✗ Account setup failed")
        for error in result["errors"]:
            print(f"  Error: {error}")
    
    # List accounts
    accounts = orchestrator.list_accounts()
    print(f"\nConfigured accounts: {len(accounts)}")
    for acc in accounts:
        print(f"  {acc['name']} → {acc['platform']}")
