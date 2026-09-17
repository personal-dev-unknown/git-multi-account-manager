# src/git_manager/models/account.py
"""Account model."""

from dataclasses import dataclass, asdict
from enum import Enum
from pathlib import Path
from typing import Optional, Dict


class Platform(Enum):
    """Git platform types."""
    GITHUB = "github"
    GITLAB = "gitlab"
    BITBUCKET = "bitbucket"
    AZURE_DEVOPS = "azure_devops"
    SELF_HOSTED = "self_hosted"
    CLOUD_STORAGE = "cloud_storage"
    LOCAL_PATH = "local_path"
    SOURCEFORGE = "sourceforge"


@dataclass
class Account:
    """Represents a Git account."""
    name: str
    platform: Platform
    username: str
    ssh_key_path: Path
    host: str
    email: Optional[str] = None
    description: Optional[str] = None
    pat_token: Optional[str] = None
    
    def to_dict(self) -> Dict:
        """Convert to dictionary."""
        return {
            'name': self.name,
            'platform': self.platform.value,
            'username': self.username,
            'email': self.email,
            'ssh_key_path': str(self.ssh_key_path),
            'host': self.host,
            'description': self.description,
            'pat_token': self.pat_token
        }
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'Account':
        """Create from dictionary."""
        return cls(
            name=data['name'],
            platform=Platform(data['platform']),
            username=data['username'],
            ssh_key_path=Path(data['ssh_key_path']),
            host=data['host'],
            email=data.get('email'),
            description=data.get('description'),
            pat_token=data.get('pat_token')
        )