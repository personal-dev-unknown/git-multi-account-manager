# src/git_manager/core/ssh/key_generator.py
"""
Enhanced SSH Key Generation Module - Modular & Creative

Handles:
✅ SSH key generation with custom names
✅ Multiple key types (ED25519, RSA)
✅ Passphrase support
✅ Key metadata storage
✅ Automatic backup
✅ Permission management
✅ Key validation
"""

import subprocess
from pathlib import Path
from typing import Optional, Tuple, Dict
from dataclasses import dataclass, asdict
import json
from datetime import datetime
import shutil


@dataclass
class KeyMetadata:
    """SSH key metadata for tracking and management."""
    name: str
    email: str
    platform: str
    account_type: str
    key_type: str  # ed25519, rsa
    created_at: str
    fingerprint: Optional[str] = None
    has_passphrase: bool = False
    backup_path: Optional[str] = None
    notes: Optional[str] = None


class SSHKeyGenerator:
    """
    Modular SSH key generation with creative features.
    
    Features:
    - Multiple key types (ED25519, RSA)
    - Automatic metadata tracking
    - Key backup system
    - Permission enforcement
    - Validation checks
    - Fingerprint generation
    """
    
    def __init__(self, ssh_dir: Optional[Path] = None):
        """
        Initialize key generator.
        
        Args:
            ssh_dir: SSH directory (defaults to ~/.ssh)
        """
        self.ssh_dir = ssh_dir or Path.home() / ".ssh"
        self.gitmanager_dir = self.ssh_dir / "gitmanager"
        self.metadata_file = self.gitmanager_dir / "keys_metadata.json"
        self.backup_dir = self.gitmanager_dir / "backups"
        
        # Ensure directories exist
        self._ensure_directories()
    
    def _ensure_directories(self):
        """Ensure all required directories exist with proper permissions."""
        self.ssh_dir.mkdir(parents=True, exist_ok=True)
        self.ssh_dir.chmod(0o700)
        
        self.gitmanager_dir.mkdir(parents=True, exist_ok=True)
        self.gitmanager_dir.chmod(0o700)
        
        self.backup_dir.mkdir(parents=True, exist_ok=True)
        self.backup_dir.chmod(0o700)
    
    def generate_key(
        self,
        name: str,
        email: str,
        platform: str,
        account_type: str,
        key_type: str = "ed25519",
        passphrase: Optional[str] = None,
        backup: bool = True
    ) -> Tuple[bool, str, Optional[Dict]]:
        """
        Generate SSH key with full metadata tracking.
        
        Args:
            name: Account name/identifier
            email: Email address for the key
            platform: Git platform (github, gitlab, bitbucket, etc.)
            account_type: Account type (school, work, personal, etc.)
            key_type: Key type (ed25519 or rsa)
            passphrase: Optional passphrase for the key
            backup: Whether to backup existing key if it exists
            
        Returns:
            Tuple of (success, message, key_info_dict)
            
        Example:
            success, msg, info = generator.generate_key(
                name="devonionMoses",
                email="moses@school.edu",
                platform="github",
                account_type="school"
            )
        """
        try:
            # Validate inputs
            if not self._validate_inputs(name, email, platform, account_type):
                return False, "Invalid input parameters", None
            
            # Create key filename
            key_filename = f"id_{key_type}_{name}"
            key_path = self.gitmanager_dir / key_filename
            pub_key_path = self.gitmanager_dir / f"{key_filename}.pub"
            
            # Check if key already exists
            if key_path.exists():
                if backup:
                    success, msg = self._backup_existing_key(key_path, pub_key_path)
                    if not success:
                        return False, f"Backup failed: {msg}", None
                else:
                    return False, f"Key already exists: {key_path}", None
            
            # Generate the key
            success, msg = self._run_keygen(
                key_path, email, key_type, passphrase
            )
            
            if not success:
                return False, f"Key generation failed: {msg}", None
            
            # Set proper permissions
            key_path.chmod(0o600)
            pub_key_path.chmod(0o644)
            
            # Read public key
            public_key = pub_key_path.read_text().strip()
            
            # Generate fingerprint
            fingerprint = self._get_fingerprint(key_path)
            
            # Create metadata
            metadata = KeyMetadata(
                name=name,
                email=email,
                platform=platform,
                account_type=account_type,
                key_type=key_type,
                created_at=datetime.now().isoformat(),
                fingerprint=fingerprint,
                has_passphrase=passphrase is not None,
                backup_path=str(self.backup_dir / f"{key_filename}.backup") if backup else None,
                notes=f"Generated for {account_type} account on {platform}"
            )
            
            # Save metadata
            self._save_metadata(metadata)
            
            # Return success with key information
            key_info = {
                "name": name,
                "key_path": str(key_path),
                "pub_key_path": str(pub_key_path),
                "public_key": public_key,
                "fingerprint": fingerprint,
                "ssh_host_alias": f"{platform}-{name}",
                "metadata": asdict(metadata)
            }
            
            return True, f"Key generated successfully: {key_path}", key_info
        
        except Exception as e:
            return False, f"Error generating key: {str(e)}", None
    
    def _validate_inputs(
        self,
        name: str,
        email: str,
        platform: str,
        account_type: str
    ) -> bool:
        """Validate input parameters."""
        if not name or not isinstance(name, str):
            return False
        if not email or "@" not in email:
            return False
        if not platform or not isinstance(platform, str):
            return False
        if not account_type or not isinstance(account_type, str):
            return False
        return True
    
    def _run_keygen(
        self,
        key_path: Path,
        email: str,
        key_type: str,
        passphrase: Optional[str]
    ) -> Tuple[bool, str]:
        """Run ssh-keygen command."""
        try:
            cmd = [
                "ssh-keygen",
                "-t", key_type,
                "-C", email,
                "-f", str(key_path),
            ]
            
            # Add key size for RSA
            if key_type == "rsa":
                cmd.extend(["-b", "4096"])
            
            # Add passphrase
            if passphrase:
                cmd.extend(["-N", passphrase])
            else:
                cmd.extend(["-N", ""])
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode != 0:
                return False, result.stderr
            
            return True, "Key generated successfully"
        
        except subprocess.TimeoutExpired:
            return False, "Key generation timed out"
        except Exception as e:
            return False, str(e)
    
    def _backup_existing_key(
        self,
        key_path: Path,
        pub_key_path: Path
    ) -> Tuple[bool, str]:
        """Backup existing key before overwriting."""
        try:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            backup_key = self.backup_dir / f"{key_path.name}.{timestamp}.backup"
            backup_pub = self.backup_dir / f"{pub_key_path.name}.{timestamp}.backup"
            
            shutil.copy2(key_path, backup_key)
            if pub_key_path.exists():
                shutil.copy2(pub_key_path, backup_pub)
            
            backup_key.chmod(0o600)
            backup_pub.chmod(0o644)
            
            return True, f"Backed up to {backup_key}"
        
        except Exception as e:
            return False, f"Backup failed: {str(e)}"
    
    def _get_fingerprint(self, key_path: Path) -> Optional[str]:
        """Get SSH key fingerprint."""
        try:
            result = subprocess.run(
                ["ssh-keygen", "-l", "-f", str(key_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                # Extract fingerprint (format: "256 SHA256:xxxxx email")
                parts = result.stdout.strip().split()
                if len(parts) >= 2:
                    return parts[1]
            
            return None
        
        except Exception:
            return None
    
    def _save_metadata(self, metadata: KeyMetadata):
        """Save key metadata to JSON file."""
        try:
            # Load existing metadata
            all_metadata = {}
            if self.metadata_file.exists():
                all_metadata = json.loads(self.metadata_file.read_text())
            
            # Add new metadata
            all_metadata[metadata.name] = asdict(metadata)
            
            # Save
            self.metadata_file.write_text(
                json.dumps(all_metadata, indent=2)
            )
            self.metadata_file.chmod(0o600)
        
        except Exception as e:
            print(f"Warning: Could not save metadata: {e}")
    
    def get_metadata(self, name: str) -> Optional[KeyMetadata]:
        """Get metadata for a specific key."""
        try:
            if not self.metadata_file.exists():
                return None
            
            all_metadata = json.loads(self.metadata_file.read_text())
            if name in all_metadata:
                data = all_metadata[name]
                return KeyMetadata(**data)
            
            return None
        
        except Exception:
            return None
    
    def list_keys(self) -> Dict[str, KeyMetadata]:
        """List all generated keys with metadata."""
        try:
            if not self.metadata_file.exists():
                return {}
            
            all_metadata = json.loads(self.metadata_file.read_text())
            return {
                name: KeyMetadata(**data)
                for name, data in all_metadata.items()
            }
        
        except Exception:
            return {}
    
    def validate_key(self, key_path: Path) -> Tuple[bool, str]:
        """Validate SSH key integrity."""
        try:
            if not key_path.exists():
                return False, f"Key not found: {key_path}"
            
            # Check permissions
            if key_path.stat().st_mode & 0o077:
                return False, "Key has incorrect permissions (should be 600)"
            
            # Check key format
            result = subprocess.run(
                ["ssh-keygen", "-l", "-f", str(key_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode != 0:
                return False, "Key validation failed"
            
            return True, "Key is valid"
        
        except Exception as e:
            return False, f"Validation error: {str(e)}"
    
    def delete_key(
        self,
        name: str,
        backup: bool = True
    ) -> Tuple[bool, str]:
        """
        Delete a key (with optional backup).
        
        Args:
            name: Key name to delete
            backup: Whether to backup before deleting
            
        Returns:
            Tuple of (success, message)
        """
        try:
            key_filename = f"id_ed25519_{name}"
            key_path = self.gitmanager_dir / key_filename
            pub_key_path = self.gitmanager_dir / f"{key_filename}.pub"
            
            if not key_path.exists():
                return False, f"Key not found: {key_path}"
            
            # Backup if requested
            if backup:
                success, msg = self._backup_existing_key(key_path, pub_key_path)
                if not success:
                    return False, f"Backup failed: {msg}"
            
            # Delete files
            key_path.unlink()
            if pub_key_path.exists():
                pub_key_path.unlink()
            
            # Remove metadata
            try:
                all_metadata = json.loads(self.metadata_file.read_text())
                if name in all_metadata:
                    del all_metadata[name]
                    self.metadata_file.write_text(json.dumps(all_metadata, indent=2))
            except Exception:
                pass
            
            return True, f"Key deleted: {name}"
        
        except Exception as e:
            return False, f"Error deleting key: {str(e)}"


# Example usage
if __name__ == "__main__":
    generator = SSHKeyGenerator()
    
    # Generate a key
    success, msg, info = generator.generate_key(
        name="devonionMoses",
        email="moses@school.edu",
        platform="github",
        account_type="school"
    )
    
    if success:
        print(f"✓ {msg}")
        print(f"  Fingerprint: {info['fingerprint']}")
        print(f"  SSH Alias: {info['ssh_host_alias']}")
    else:
        print(f"✗ {msg}")
    
    # List all keys
    keys = generator.list_keys()
    for name, metadata in keys.items():
        print(f"\n{name}:")
        print(f"  Platform: {metadata.platform}")
        print(f"  Type: {metadata.account_type}")
        print(f"  Created: {metadata.created_at}")
