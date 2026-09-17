# src/git_manager/utils/platform_helpers.py
"""Platform-specific helper functions."""

from typing import Optional, Tuple
from pathlib import Path
import re

from ..core.platform_config import PlatformManager


def get_platform_ssh_host(platform_key: str) -> Optional[str]:
    """Get SSH host for a platform.
    
    Args:
        platform_key: Platform key (e.g., 'github', 'gitlab')
        
    Returns:
        SSH host or None if not supported
    """
    return PlatformManager.get_ssh_host(platform_key)


def get_platform_https_host(platform_key: str) -> Optional[str]:
    """Get HTTPS host for a platform.
    
    Args:
        platform_key: Platform key
        
    Returns:
        HTTPS host or None if not supported
    """
    return PlatformManager.get_https_host(platform_key)


def get_platform_api_url(platform_key: str) -> Optional[str]:
    """Get API URL for a platform.
    
    Args:
        platform_key: Platform key
        
    Returns:
        API URL or None if not available
    """
    return PlatformManager.get_api_url(platform_key)


def convert_url_to_ssh(url: str, platform_key: str, account_name: str) -> Optional[str]:
    """Convert HTTPS URL to SSH URL for a platform.
    
    Args:
        url: Repository URL (HTTPS or short format)
        platform_key: Platform key
        account_name: Account name for SSH host alias
        
    Returns:
        SSH URL or None if conversion not possible
    """
    platform = PlatformManager.get_platform(platform_key)
    if not platform or not platform.supports_ssh:
        return None
    
    # Handle different URL formats
    if url.startswith('http://') or url.startswith('https://'):
        # Extract owner and repo from HTTPS URL
        match = re.search(r'https?://[^/]+/([^/]+)/([^/]+?)(?:\.git)?/?$', url)
        if match:
            owner, repo = match.groups()
            if not repo.endswith('.git'):
                repo = f"{repo}.git"
            
            # Generate SSH URL with account-specific host
            ssh_host = f"{platform.ssh_host}-{account_name}" if platform.ssh_host else None
            if ssh_host:
                return f"git@{ssh_host}:{owner}/{repo}"
    
    # Handle short format (owner/repo)
    elif '/' in url and not url.startswith('/'):
        owner, repo = url.split('/', 1)
        if not repo.endswith('.git'):
            repo = f"{repo}.git"
        
        ssh_host = f"{platform.ssh_host}-{account_name}" if platform.ssh_host else None
        if ssh_host:
            return f"git@{ssh_host}:{owner}/{repo}"
    
    return None


def convert_url_to_https(url: str, platform_key: str) -> Optional[str]:
    """Convert SSH URL to HTTPS URL for a platform.
    
    Args:
        url: Repository URL (SSH or short format)
        platform_key: Platform key
        
    Returns:
        HTTPS URL or None if conversion not possible
    """
    platform = PlatformManager.get_platform(platform_key)
    if not platform or not platform.supports_https:
        return None
    
    # Handle SSH URLs
    if url.startswith('git@'):
        # Extract owner and repo from SSH URL
        match = re.search(r'git@[^:]+:([^/]+)/([^/]+?)(?:\.git)?$', url)
        if match:
            owner, repo = match.groups()
            if not repo.endswith('.git'):
                repo = f"{repo}.git"
            
            https_host = platform.https_host
            if https_host:
                return f"https://{https_host}/{owner}/{repo}"
    
    # Handle short format (owner/repo)
    elif '/' in url and not url.startswith('/'):
        owner, repo = url.split('/', 1)
        if not repo.endswith('.git'):
            repo = f"{repo}.git"
        
        https_host = platform.https_host
        if https_host:
            return f"https://{https_host}/{owner}/{repo}"
    
    return None


def is_local_path(url: str) -> bool:
    """Check if URL is a local path.
    
    Args:
        url: URL to check
        
    Returns:
        True if URL is a local path
    """
    if url.startswith('/') or url.startswith('~') or url.startswith('file://'):
        return True
    
    # Check if it's a valid local path
    try:
        path = Path(url).expanduser()
        return path.exists() or path.parent.exists()
    except (ValueError, OSError):
        return False


def is_self_hosted_url(url: str) -> bool:
    """Check if URL is for a self-hosted Git server.
    
    Args:
        url: URL to check
        
    Returns:
        True if URL appears to be self-hosted
    """
    # Self-hosted URLs typically have custom domains
    if url.startswith('git@') and not any(host in url for host in ['github.com', 'gitlab.com', 'bitbucket.org', 'dev.azure.com', 'git.code.sf.net']):
        return True
    
    if url.startswith('https://') and not any(host in url for host in ['github.com', 'gitlab.com', 'bitbucket.org', 'dev.azure.com', 'git.code.sf.net']):
        return True
    
    return False


def detect_platform_from_url(url: str) -> Optional[str]:
    """Detect platform from repository URL.
    
    Args:
        url: Repository URL
        
    Returns:
        Platform key or None if not detected
    """
    url_lower = url.lower()
    
    if 'github.com' in url_lower:
        return 'github'
    elif 'gitlab.com' in url_lower:
        return 'gitlab'
    elif 'bitbucket.org' in url_lower:
        return 'bitbucket'
    elif 'dev.azure.com' or 'ssh.dev.azure.com' in url_lower:
        return 'azure_devops'
    elif 'git.code.sf.net' in url_lower:
        return 'sourceforge'
    elif is_local_path(url):
        return 'local_path'
    elif is_self_hosted_url(url):
        return 'self_hosted'
    
    return None


def get_platform_display_name(platform_key: str) -> str:
    """Get human-readable platform name.
    
    Args:
        platform_key: Platform key
        
    Returns:
        Platform display name
    """
    name = PlatformManager.get_platform_name(platform_key)
    return name or platform_key.replace('_', ' ').title()


def get_platform_description(platform_key: str) -> str:
    """Get platform description.
    
    Args:
        platform_key: Platform key
        
    Returns:
        Platform description
    """
    description = PlatformManager.get_platform_description(platform_key)
    return description or f"Git platform: {platform_key}"


def supports_ssh(platform_key: str) -> bool:
    """Check if platform supports SSH.
    
    Args:
        platform_key: Platform key
        
    Returns:
        True if platform supports SSH
    """
    platform = PlatformManager.get_platform(platform_key)
    return platform.supports_ssh if platform else False


def supports_https(platform_key: str) -> bool:
    """Check if platform supports HTTPS.
    
    Args:
        platform_key: Platform key
        
    Returns:
        True if platform supports HTTPS
    """
    platform = PlatformManager.get_platform(platform_key)
    return platform.supports_https if platform else False


def supports_pat(platform_key: str) -> bool:
    """Check if platform supports Personal Access Tokens.
    
    Args:
        platform_key: Platform key
        
    Returns:
        True if platform supports PAT
    """
    platform = PlatformManager.get_platform(platform_key)
    return platform.supports_pat if platform else False
