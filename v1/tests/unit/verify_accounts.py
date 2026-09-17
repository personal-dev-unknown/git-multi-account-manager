#!/usr/bin/env python3
"""Verify accounts are loaded correctly."""

import sys
from pathlib import Path

# Add the source directory to path
sys.path.insert(0, str(Path(__file__).parent / 'git-multi-account-manager' / 'src'))

from git_manager.core.account_manager import AccountManager

# Test AccountManager
print("Loading accounts from ~/.git-manager/accounts.json...")
manager = AccountManager()
all_accounts = manager.list_accounts()

print(f"\n✓ Successfully loaded {len(all_accounts)} accounts:\n")

for account in all_accounts:
    print(f"  [{account.platform.value.upper()}] {account.name}")
    print(f"    Username: {account.username}")
    print(f"    Email: {account.email}")
    print(f"    Host: {account.host}")
    print(f"    SSH Key: {account.ssh_key_path}")
    print()

print("\n" + "="*60)
print("IMPORTANT: Email addresses need to be updated!")
print("="*60)
print("\nTo update emails, use:")
print("  python3 -m git_manager account update ACCOUNT_NAME --email YOUR_REAL_EMAIL@domain.com")
print("\nExample:")
print("  python3 -m git_manager account update devonionMoses --email devonion@gmail.com")
