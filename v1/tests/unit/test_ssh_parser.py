#!/usr/bin/env python3
"""Test SSH config parser."""

import sys
from pathlib import Path

# Add the source directory to path
sys.path.insert(0, str(Path(__file__).parent / 'git-multi-account-manager' / 'src'))

from git_manager.core.ssh_config_parser import SSHConfigParser
from git_manager.core.account_manager import AccountManager

# Test SSH config parser
print("Testing SSH Config Parser...")
parser = SSHConfigParser()
accounts = parser.parse_accounts()

print(f"\nFound {len(accounts)} accounts from SSH config:")
for account in accounts:
    print(f"  - {account.name} ({account.platform.value})")
    print(f"    Host: {account.host}")
    print(f"    SSH Key: {account.ssh_key_path}")
    print(f"    Email: {account.email}")
    print()

# Test AccountManager
print("\nTesting AccountManager...")
manager = AccountManager()
all_accounts = manager.list_accounts()

print(f"\nAccountManager loaded {len(all_accounts)} accounts:")
for account in all_accounts:
    print(f"  - {account.name} ({account.platform.value})")
    print(f"    Host: {account.host}")
    print(f"    SSH Key: {account.ssh_key_path}")
    print()
