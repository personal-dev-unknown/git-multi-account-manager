# src/git_manager/models/ssh_key.py
"""SSH Key model."""

from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Optional
from datetime import datetime


class SSHKeyType(Enum):
    """SSH key types."""
    ED25519 = "ed25519"
    RSA = "rsa"
    ECDSA = "ecdsa"


@dataclass
class SSHKey:
    """Represents an SSH key pair."""
    name: str
    key_type: SSHKeyType
    private_key_path: Path
    public_key_path: Path
    public_key: str
    email: str
    created_at: datetime = None
    passphrase_protected: bool = False
    
    def __post_init__(self):
        if self.created_at is None:
            self.created_at = datetime.now()