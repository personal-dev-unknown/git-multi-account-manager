#!/usr/bin/env python3
"""CLI entry point for Git Multi-Account Manager."""

import sys
from git_manager.cli.app import cli

if __name__ == "__main__":
    sys.exit(cli())
