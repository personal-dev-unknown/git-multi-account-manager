# src/git_manager/core/platform_config.py
"""Platform configuration and management."""

from typing import Dict, Optional, List, Tuple
from enum import Enum
from dataclasses import dataclass
import re


@dataclass
class PlatformInfo:
    """Information about a Git platform."""
    key: str
    name: str
    api_url: Optional[str]
    ssh_host: Optional[str]
    https_host: Optional[str]
    supports_ssh: bool
    supports_https: bool
    supports_pat: bool
    description: str


class PlatformManager:
    """Manages platform configurations and metadata."""
    
    # Platform definitions
    PLATFORMS = {
        'github': PlatformInfo(
            key='github',
            name='GitHub',
            api_url='https://api.github.com',
            ssh_host='github.com',
            https_host='github.com',
            supports_ssh=True,
            supports_https=True,
            supports_pat=True,
            description='GitHub - Popular open-source and private development platform'
        ),
        'gitlab': PlatformInfo(
            key='gitlab',
            name='GitLab',
            api_url='https://gitlab.com/api/v4',
            ssh_host='gitlab.com',
            https_host='gitlab.com',
            supports_ssh=True,
            supports_https=True,
            supports_pat=True,
            description='GitLab - Integrated CI/CD pipelines and DevOps platform'
        ),
        'bitbucket': PlatformInfo(
            key='bitbucket',
            name='Bitbucket',
            api_url='https://api.bitbucket.org/2.0',
            ssh_host='bitbucket.org',
            https_host='bitbucket.org',
            supports_ssh=True,
            supports_https=True,
            supports_pat=True,
            description='Bitbucket - Jira and Trello integration, free private repos'
        ),
        'azure_devops': PlatformInfo(
            key='azure_devops',
            name='Azure DevOps',
            api_url='https://dev.azure.com',
            ssh_host='ssh.dev.azure.com',
            https_host='dev.azure.com',
            supports_ssh=True,
            supports_https=True,
            supports_pat=True,
            description='Azure DevOps - Microsoft enterprise Git hosting with CI/CD'
        ),
        'self_hosted': PlatformInfo(
            key='self_hosted',
            name='Self-Hosted Server',
            api_url=None,
            ssh_host=None,
            https_host=None,
            supports_ssh=True,
            supports_https=True,
            supports_pat=False,
            description='Self-hosted Git server on private machine or VPS'
        ),
        'cloud_storage': PlatformInfo(
            key='cloud_storage',
            name='Cloud Storage',
            api_url=None,
            ssh_host=None,
            https_host=None,
            supports_ssh=False,
            supports_https=True,
            supports_pat=False,
            description='Cloud storage services (S3, etc.) with Git configurations'
        ),
        'local_path': PlatformInfo(
            key='local_path',
            name='Local Path',
            api_url=None,
            ssh_host=None,
            https_host=None,
            supports_ssh=False,
            supports_https=False,
            supports_pat=False,
            description='Local directory or network share as Git remote'
        ),
        'sourceforge': PlatformInfo(
            key='sourceforge',
            name='SourceForge',
            api_url='https://sourceforge.net/api',
            ssh_host='git.code.sf.net',
            https_host='git.code.sf.net',
            supports_ssh=True,
            supports_https=True,
            supports_pat=False,
            description='SourceForge - Open-source project hosting'
        )
    }
    
    @classmethod
    def get_platform(cls, key: str) -> Optional[PlatformInfo]:
        """Get platform information by key.
        
        Args:
            key: Platform key (e.g., 'github', 'gitlab')
            
        Returns:
            PlatformInfo object or None if not found
        """
        return cls.PLATFORMS.get(key)
    
    @classmethod
    def list_platforms(cls) -> List[PlatformInfo]:
        """Get list of all platforms.
        
        Returns:
            List of PlatformInfo objects
        """
        return list(cls.PLATFORMS.values())
    
    @classmethod
    def list_platform_keys(cls) -> List[str]:
        """Get list of all platform keys.
        
        Returns:
            List of platform keys
        """
        return list(cls.PLATFORMS.keys())
    
    @classmethod
    def get_ssh_platforms(cls) -> List[PlatformInfo]:
        """Get platforms that support SSH.
        
        Returns:
            List of PlatformInfo objects supporting SSH
        """
        return [p for p in cls.PLATFORMS.values() if p.supports_ssh]
    
    @classmethod
    def get_https_platforms(cls) -> List[PlatformInfo]:
        """Get platforms that support HTTPS.
        
        Returns:
            List of PlatformInfo objects supporting HTTPS
        """
        return [p for p in cls.PLATFORMS.values() if p.supports_https]
    
    @classmethod
    def get_pat_platforms(cls) -> List[PlatformInfo]:
        """Get platforms that support Personal Access Tokens.
        
        Returns:
            List of PlatformInfo objects supporting PAT
        """
        return [p for p in cls.PLATFORMS.values() if p.supports_pat]
    
    @classmethod
    def validate_platform(cls, key: str) -> bool:
        """Validate if platform key exists.
        
        Args:
            key: Platform key to validate
            
        Returns:
            True if platform exists, False otherwise
        """
        return key in cls.PLATFORMS
    
    @classmethod
    def get_platform_name(cls, key: str) -> Optional[str]:
        """Get human-readable platform name.
        
        Args:
            key: Platform key
            
        Returns:
            Platform name or None if not found
        """
        platform = cls.get_platform(key)
        return platform.name if platform else None
    
    @classmethod
    def get_platform_description(cls, key: str) -> Optional[str]:
        """Get platform description.
        
        Args:
            key: Platform key
            
        Returns:
            Platform description or None if not found
        """
        platform = cls.get_platform(key)
        return platform.description if platform else None
    
    @classmethod
    def get_ssh_host(cls, key: str) -> Optional[str]:
        """Get SSH host for platform.
        
        Args:
            key: Platform key
            
        Returns:
            SSH host or None if not supported
        """
        platform = cls.get_platform(key)
        return platform.ssh_host if platform else None
    
    @classmethod
    def get_https_host(cls, key: str) -> Optional[str]:
        """Get HTTPS host for platform.
        
        Args:
            key: Platform key
            
        Returns:
            HTTPS host or None if not supported
        """
        platform = cls.get_platform(key)
        return platform.https_host if platform else None
    
    @classmethod
    def get_api_url(cls, key: str) -> Optional[str]:
        """Get API URL for platform.
        
        Args:
            key: Platform key
            
        Returns:
            API URL or None if not available
        """
        platform = cls.get_platform(key)
        return platform.api_url if platform else None
    
    # @classmethod
    # def validate_email(cls, email: str, platform_key: Optional[str] = None) -> Tuple[bool, str]:
    #     """Validate email format for a specific platform or general validation.
        
    #     Args:
    #         email: Email address to validate
    #         platform_key: Optional platform key for platform-specific validation.
    #                      If None, performs general email validation.
            
    #     Returns:
    #         Tuple of (is_valid: bool, message: str)
    #         - is_valid: True if email is valid for the platform
    #         - message: Validation message or error description
    #     """
    #     # Basic email format validation
    #     email_pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
        
    #     if not email or not isinstance(email, str):
    #         return False, "Email cannot be empty"
        
    #     email = email.strip()
        
    #     if not re.match(email_pattern, email):
    #         return False, "Invalid email format"
        
    #     # Platform-specific validation rules
    #     if platform_key:
    #         if not cls.validate_platform(platform_key):
    #             return False, f"Unknown platform: {platform_key}"
            
    #         platform = cls.get_platform(platform_key)
            
    #         # Platform-specific email rules
    #         if platform_key == 'github':
    #             if len(email) > 254:
    #                 return False, "Email too long for GitHub (max 254 chars)"
    #             return True, "Valid GitHub email"
            
    #         elif platform_key == 'gitlab':
    #             if len(email) > 255:
    #                 return False, "Email too long for GitLab (max 255 chars)"
    #             return True, "Valid GitLab email"
            
    #         elif platform_key == 'bitbucket':
    #             if len(email) > 254:
    #                 return False, "Email too long for Bitbucket (max 254 chars)"
    #             return True, "Valid Bitbucket email"
            
    #         elif platform_key == 'azure_devops':
    #             if len(email) > 254:
    #                 return False, "Email too long for Azure DevOps (max 254 chars)"
    #             return True, "Valid Azure DevOps email"
            
    #         elif platform_key == 'sourceforge':
    #             if len(email) > 254:
    #                 return False, "Email too long for SourceForge (max 254 chars)"
    #             return True, "Valid SourceForge email"
            
    #         elif platform_key in ['self_hosted', 'cloud_storage', 'local_path']:
    #             # These platforms have flexible email requirements
    #             return True, f"Valid {platform.name} email"
            
    #         return True, f"Valid {platform.name} email"
        
    #     # General validation (no platform specified)
    #     if len(email) > 254:
    #         return False, "Email too long (max 254 chars)"
        
    #     return True, "Valid email format"
    
    @classmethod
    def validate_url(cls, url: str, platform_key: Optional[str] = None) -> Tuple[bool, str, Optional[str]]:
        """Validate repository URL format for a specific platform or general validation.
        
        Args:
            url: Repository URL to validate
            platform_key: Optional platform key for platform-specific validation.
                         If None, performs general URL validation.
            
        Returns:
            Tuple of (is_valid: bool, message: str, detected_platform: Optional[str])
            - is_valid: True if URL is valid for the platform
            - message: Validation message or error description
            - detected_platform: Detected platform from URL (if applicable)
        """
        if not url or not isinstance(url, str):
            return False, "URL cannot be empty", None
        
        url = url.strip()
        
        # URL format patterns for different platforms
        platform_patterns = {
            'github': [
                r'^(https?://)?github\.com/[\w\-\.]+/[\w\-\.]+/?$',  # https://github.com/user/repo
                r'^git@github\.com:[\w\-\.]+/[\w\-\.]+\.git$',  # git@github.com:user/repo.git
                r'^github\.com/[\w\-\.]+/[\w\-\.]+/?$',  # github.com/user/repo
            ],
            'gitlab': [
                r'^(https?://)?gitlab\.com/[\w\-\.]+/[\w\-\.]+/?$',  # https://gitlab.com/user/repo
                r'^git@gitlab\.com:[\w\-\.]+/[\w\-\.]+\.git$',  # git@gitlab.com:user/repo.git
                r'^gitlab\.com/[\w\-\.]+/[\w\-\.]+/?$',  # gitlab.com/user/repo
            ],
            'bitbucket': [
                r'^(https?://)?bitbucket\.org/[\w\-\.]+/[\w\-\.]+/?$',  # https://bitbucket.org/user/repo
                r'^git@bitbucket\.org:[\w\-\.]+/[\w\-\.]+\.git$',  # git@bitbucket.org:user/repo.git
                r'^bitbucket\.org/[\w\-\.]+/[\w\-\.]+/?$',  # bitbucket.org/user/repo
            ],
            'azure_devops': [
                r'^(https?://)?dev\.azure\.com/[\w\-\.]+/[\w\-\.]+/_git/[\w\-\.]+/?$',  # https://dev.azure.com/org/project/_git/repo
                r'^git@ssh\.dev\.azure\.com:v3/[\w\-\.]+/[\w\-\.]+/[\w\-\.]+$',  # git@ssh.dev.azure.com:v3/org/project/repo
            ],
            'sourceforge': [
                r'^(https?://)?git\.code\.sf\.net/p/[\w\-\.]+/[\w\-\.]+/?$',  # https://git.code.sf.net/p/project/repo
                r'^git@git\.code\.sf\.net:/p/[\w\-\.]+/[\w\-\.]+\.git$',  # git@git.code.sf.net:/p/project/repo.git
            ],
            'self_hosted': [
                r'^(https?://)?[\w\-\.]+\.[a-z]{2,}/.*\.git/?$',  # https://git.example.com/repo.git
                r'^git@[\w\-\.]+:[\w\-\.]+/[\w\-\.]+\.git$',  # git@git.example.com:user/repo.git
            ],
        }
        
        # Detect platform from URL
        detected_platform = None
        for platform, patterns in platform_patterns.items():
            for pattern in patterns:
                if re.match(pattern, url, re.IGNORECASE):
                    detected_platform = platform
                    break
            if detected_platform:
                break
        
        # If platform_key is specified, validate against it
        if platform_key:
            if not cls.validate_platform(platform_key):
                return False, f"Unknown platform: {platform_key}", None
            
            platform = cls.get_platform(platform_key)
            
            # Check if URL matches the specified platform
            if platform_key in platform_patterns:
                patterns = platform_patterns[platform_key]
                for pattern in patterns:
                    if re.match(pattern, url, re.IGNORECASE):
                        return True, f"Valid {platform.name} URL", platform_key
                
                return False, f"URL does not match {platform.name} format", None
            
            # For platforms without specific patterns (cloud_storage, local_path)
            if platform_key in ['cloud_storage', 'local_path']:
                # These are more flexible
                if url.startswith('/') or url.startswith('s3://') or url.startswith('gs://'):
                    return True, f"Valid {platform.name} URL", platform_key
                return False, f"Invalid {platform.name} URL format", None
            
            return True, f"Valid {platform.name} URL", platform_key
        
        # General validation (no platform specified)
        if detected_platform:
            platform = cls.get_platform(detected_platform)
            return True, f"Valid {platform.name} URL", detected_platform
        
        # Check for common URL patterns
        if re.match(r'^(https?://|git@|/|s3://|gs://)', url):
            return True, "Valid repository URL format", None
        
        return False, "Invalid repository URL format", None
