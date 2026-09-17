"""Platform API endpoints."""

from typing import Dict, List
from ..workflow import CloneWorkflow
from ..platforms import GitHubPlatform, GitLabPlatform, BitbucketPlatform


class PlatformAPI:
    """API for platform operations."""
    
    def __init__(self, account_manager, config_manager=None):
        self.workflow = CloneWorkflow(account_manager, config_manager)
        self.account_manager = account_manager
    
    def get_supported_platforms(self) -> Dict:
        """
        Get list of supported platforms.
        
        Returns:
            Supported platforms
        """
        
        return {
            'success': True,
            'platforms': [
                {
                    'id': 'github',
                    'name': 'GitHub',
                    'api_base': 'https://api.github.com',
                    'ssh_host': 'github.com',
                    'features': ['api', 'fork', 'ssh', 'pat']
                },
                {
                    'id': 'gitlab',
                    'name': 'GitLab',
                    'api_base': 'https://gitlab.com/api/v4',
                    'ssh_host': 'gitlab.com',
                    'features': ['api', 'fork', 'ssh', 'pat', 'password']
                },
                {
                    'id': 'bitbucket',
                    'name': 'Bitbucket',
                    'api_base': 'https://api.bitbucket.org/2.0',
                    'ssh_host': 'bitbucket.org',
                    'features': ['api', 'fork', 'ssh', 'pat']
                },
                {
                    'id': 'custom',
                    'name': 'Custom/Self-Hosted',
                    'api_base': None,
                    'ssh_host': 'custom',
                    'features': ['ssh', 'anonymous']
                }
            ]
        }
    
    def test_platform_connection(
        self,
        platform: str,
        account_name: str
    ) -> Dict:
        """
        Test connection to platform.
        
        Args:
            platform: Platform ID
            account_name: Account name
        
        Returns:
            Connection test result
        """
        
        try:
            account = self.account_manager.get_account(account_name)
            if not account:
                return {
                    'success': False,
                    'error': f"Account not found: {account_name}"
                }
            
            # Convert account to dict
            account_dict = {
                'username': account.username,
                'pat_token': getattr(account, 'pat_token', None),
            }
            
            # Get platform and test
            platform_obj = self.workflow.get_platform(platform)
            is_connected = platform_obj.test_connection(account_dict)
            
            return {
                'success': is_connected,
                'platform': platform,
                'account': account_name,
                'connected': is_connected,
                'message': 'Connection successful' if is_connected else 'Connection failed'
            }
        
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def get_platform_info(self, platform: str) -> Dict:
        """
        Get platform information.
        
        Args:
            platform: Platform ID
        
        Returns:
            Platform information
        """
        
        platforms_info = self.get_supported_platforms()
        
        for p in platforms_info['platforms']:
            if p['id'] == platform:
                return {
                    'success': True,
                    'platform': p
                }
        
        return {
            'success': False,
            'error': f"Platform not found: {platform}"
        }
    
    def get_platform_accounts(self, platform: str) -> Dict:
        """
        Get accounts for a platform.
        
        Args:
            platform: Platform ID
        
        Returns:
            Accounts for platform
        """
        
        try:
            accounts = self.workflow.get_accounts_for_platform(platform)
            
            return {
                'success': True,
                'platform': platform,
                'count': len(accounts),
                'accounts': [
                    {
                        'name': acc.name,
                        'username': acc.username,
                        'email': acc.email,
                        'platform': acc.platform.value if hasattr(acc.platform, 'value') else str(acc.platform)
                    }
                    for acc in accounts
                ]
            }
        
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def get_authentication_methods(self, platform: str) -> Dict:
        """
        Get available authentication methods for platform.
        
        Args:
            platform: Platform ID
        
        Returns:
            Available authentication methods
        """
        
        auth_methods = {
            'github': ['ssh', 'pat', 'anonymous'],
            'gitlab': ['ssh', 'pat', 'password', 'anonymous'],
            'bitbucket': ['ssh', 'pat', 'anonymous'],
            'custom': ['ssh', 'anonymous']
        }
        
        methods = auth_methods.get(platform, [])
        
        return {
            'success': True,
            'platform': platform,
            'methods': [
                {
                    'id': 'ssh',
                    'name': 'SSH Key',
                    'description': 'SSH key-based authentication',
                    'requires_key': True,
                    'supports_2fa': True
                } if m == 'ssh' else
                {
                    'id': 'pat',
                    'name': 'Personal Access Token',
                    'description': 'HTTPS with PAT',
                    'requires_key': False,
                    'supports_2fa': True
                } if m == 'pat' else
                {
                    'id': 'password',
                    'name': 'Username/Password',
                    'description': 'HTTPS with password',
                    'requires_key': False,
                    'supports_2fa': False
                } if m == 'password' else
                {
                    'id': 'anonymous',
                    'name': 'Anonymous',
                    'description': 'Read-only cloning',
                    'requires_key': False,
                    'supports_2fa': False
                }
                for m in methods
            ]
        }
