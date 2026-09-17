# src/git_manager/__main__.py
"""
Main entry point for the Git Multi-Account Manager.

This module serves as the entry point for both the CLI and GUI applications.
"""

import sys
import os
import argparse

# Add src directory to sys.path so git_manager can be imported
# This allows running the script directly: python src/git_manager/__main__.py
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

# CRITICAL: Monkeypatch sys.argv to ensure all elements are strings
# This fixes issues with Click parser when Path objects are in sys.argv
_original_argv = sys.argv
sys.argv = [str(arg) for arg in sys.argv]

def show_mode_selection():
    """Display interactive mode selection menu."""
    print("\n" + "="*50)
    print("  Git Multi-Account Manager")
    print("="*50)
    print("\nPlease select a mode to run:")
    print("\n  1. Command Line Interface (CLI)")
    print("  2. Desktop Application (GUI)")
    print("  3. Web Application")
    print("  4. Exit")
    print("\n" + "="*50)
    
    while True:
        try:
            choice = input("\nEnter your choice (1-4): ").strip()
            
            if choice == '1':
                return 'cli'
            elif choice == '2':
                return 'desktop'
            elif choice == '3':
                return 'web'
            elif choice == '4':
                print("\nExiting. Goodbye!")
                sys.exit(0)
            else:
                print("Invalid choice. Please enter a number between 1 and 4.")
        except KeyboardInterrupt:
            print("\n\nExiting. Goodbye!")
            sys.exit(0)
        except EOFError:
            print("\n\nExiting. Goodbye!")
            sys.exit(0)

def main():
    """Main entry point for the application."""
    # Ensure all sys.argv elements are strings (not Path objects) - fix for Click
    sys.argv = [str(arg) for arg in sys.argv]
    
    # Check if we're running in CLI mode (first argument is a known command or flag)
    if len(sys.argv) > 1:
        first_arg = sys.argv[1]
        if first_arg in ['--cli', '--desktop', '--web']:
            mode = sys.argv.pop(1).lstrip('--')
        elif first_arg.startswith('--') or first_arg.startswith('-'):
            # If it starts with - or --, it's a flag (like --help, --version), run in CLI mode
            mode = 'cli'
        else:
            # Assume it's a CLI command if it doesn't look like a path or interactive choice
            # This allows any Click command to work without hardcoding
            mode = 'cli'
    else:
        # Show interactive mode selection if no mode is specified
        mode = show_mode_selection()
    
    try:
        if mode == 'cli':
            print("\nStarting Command Line Interface...\n")
            from git_manager.cli.app import cli
            # If no additional arguments, run interactive mode
            # Only inject 'interactive' if there are no other commands
            if len(sys.argv) == 1:
                sys.argv.append('interactive')
            # Ensure sys.argv is all strings before calling Click (critical for Click parser)
            sys.argv = [str(arg) for arg in sys.argv]
            cli()
        elif mode == 'desktop':
            print("\nStarting Desktop Application...\n")
            from git_manager.desktop.app import run_desktop_app
            run_desktop_app()
        elif mode == 'web':
            print("\nStarting Web Application...")
            print("Access the app at: http://localhost:5000\n")
            
            # Check for production flag
            production = '--production' in sys.argv
            debug = '--debug' in sys.argv
            
            # Remove our flags from sys.argv before importing
            if '--production' in sys.argv:
                sys.argv.remove('--production')
            if '--debug' in sys.argv:
                sys.argv.remove('--debug')
            
            # Get host and port from environment or use defaults
            host = os.environ.get('GIT_MANAGER_WEB_HOST', '0.0.0.0')
            port = int(os.environ.get('GIT_MANAGER_WEB_PORT', 5000))
            
            if production:
                # Use production server with gevent-websocket
                from git_manager.web.app import run_production_server
                run_production_server(host, port)
            else:
                # Use development server with threading/polling fallback
                from git_manager.web.app import run_web_server
                run_web_server(host, port, debug=debug)
        else:
            print(f"Unknown mode: {mode}")
            print("Available modes: --cli, --desktop, --web")
            sys.exit(1)
    except ImportError as e:
        print(f"\nError importing module: {e}")
        print("Make sure all required dependencies are installed.")
        print("Current sys.path:", sys.path)
        sys.exit(1)
    except Exception as e:
        print(f"\nAn error occurred: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()