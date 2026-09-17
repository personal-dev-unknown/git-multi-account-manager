# src/git_manager/models/repository.py
"""Repository model."""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional, List
from datetime import datetime


@dataclass
class RepositoryStatus:
    """Repository status information."""
    has_uncommitted_changes: bool = False
    current_branch: str = ''
    commits_ahead: int = 0
    commits_behind: int = 0
    uncommitted_files: List[str] = field(default_factory=list)
    
    @property
    def is_clean(self) -> bool:
        """Check if repository is clean."""
        return not self.has_uncommitted_changes
    
    @property
    def is_synced(self) -> bool:
        """Check if repository is synced with remote."""
        return self.commits_ahead == 0 and self.commits_behind == 0


@dataclass
class Repository:
    """Represents a Git repository."""
    name: str
    path: Path
    remote_url: str
    account: 'Account'
    branch: str = 'main'
    description: Optional[str] = None
    last_updated: Optional[datetime] = None
    
    def __post_init__(self):
        if self.last_updated is None:
            self.last_updated = datetime.now()