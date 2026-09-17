# src/git_manager/utils/file_operations.py
"""File operation utilities."""

import json
import shutil
from pathlib import Path
from typing import Any, Dict, Optional
import yaml


def read_json(file_path: Path) -> Dict:
    """Read JSON file.
    
    Args:
        file_path: Path to JSON file
        
    Returns:
        Parsed JSON data
    """
    with open(file_path, 'r') as f:
        return json.load(f)


def write_json(file_path: Path, data: Dict, indent: int = 2) -> None:
    """Write JSON file.
    
    Args:
        file_path: Path to JSON file
        data: Data to write
        indent: JSON indentation
    """
    file_path.parent.mkdir(parents=True, exist_ok=True)
    with open(file_path, 'w') as f:
        json.dump(data, f, indent=indent)


def read_yaml(file_path: Path) -> Dict:
    """Read YAML file.
    
    Args:
        file_path: Path to YAML file
        
    Returns:
        Parsed YAML data
    """
    with open(file_path, 'r') as f:
        return yaml.safe_load(f)


def write_yaml(file_path: Path, data: Dict) -> None:
    """Write YAML file.
    
    Args:
        file_path: Path to YAML file
        data: Data to write
    """
    file_path.parent.mkdir(parents=True, exist_ok=True)
    with open(file_path, 'w') as f:
        yaml.dump(data, f, default_flow_style=False)


def ensure_directory(path: Path, mode: int = 0o755) -> None:
    """Ensure directory exists with correct permissions.
    
    Args:
        path: Directory path
        mode: Directory permissions
    """
    path.mkdir(parents=True, exist_ok=True)
    path.chmod(mode)


def copy_file(src: Path, dst: Path, preserve_permissions: bool = True) -> None:
    """Copy file.
    
    Args:
        src: Source file path
        dst: Destination file path
        preserve_permissions: Preserve file permissions
    """
    dst.parent.mkdir(parents=True, exist_ok=True)
    if preserve_permissions:
        shutil.copy2(src, dst)
    else:
        shutil.copy(src, dst)


def delete_path(path: Path, recursive: bool = False) -> None:
    """Delete file or directory.
    
    Args:
        path: Path to delete
        recursive: Delete recursively for directories
    """
    if not path.exists():
        return
    
    if path.is_file():
        path.unlink()
    elif path.is_dir() and recursive:
        shutil.rmtree(path)