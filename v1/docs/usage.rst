# docs/usage.rst
"""
Usage Guide
===========

Terminal (CLI)
--------------

Interactive Mode
~~~~~~~~~~~~~~~~

Start interactive mode for guided operations:

.. code-block:: bash

   git-manager

Direct Commands
~~~~~~~~~~~~~~~

**Account Management:**

.. code-block:: bash

   # Add account
   git-manager account add --name work --platform github \\
     --username myuser --email work@example.com \\
     --ssh-key ~/.ssh/id_ed25519_work

   # List accounts
   git-manager account list

   # Show account details
   git-manager account show work

   # Remove account
   git-manager account remove work

**SSH Operations:**

.. code-block:: bash

   # Generate SSH key
   git-manager ssh generate --email work@example.com --name work-key

   # Test connection
   git-manager ssh test --account work

**Repository Operations:**

.. code-block:: bash

   # Clone repository
   git-manager clone https://github.com/user/repo --account work

   # Check status
   git-manager status

   # Pull changes
   git-manager pull

   # Push changes
   git-manager push

   # Setup repository
   git-manager repository setup --account work

Web Application
---------------

Starting the Server
~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Default (localhost:5000)
   git-manager-web

   # Custom host and port
   git-manager-web --host 0.0.0.0 --port 8080

   # Debug mode
   git-manager-web --debug

Web Interface Features
~~~~~~~~~~~~~~~~~~~~~~

1. **Dashboard**: Overview of accounts and repositories
2. **Account Management**: Add, edit, delete accounts
3. **Repository Operations**: Clone, pull, push via UI
4. **SSH Key Management**: Generate and test SSH keys
5. **Real-time Updates**: WebSocket-based progress tracking

Desktop Application
-------------------

Launching the App
~~~~~~~~~~~~~~~~~

.. code-block:: bash

   git-manager-desktop

Features
~~~~~~~~

* **Account Panel**: Manage all your Git accounts
* **Repository Explorer**: Browse and manage repositories
* **SSH Key Manager**: Generate and test keys
* **Integrated Terminal**: Execute Git commands
* **System Tray**: Quick access from system tray

Configuration
-------------

Config File Location
~~~~~~~~~~~~~~~~~~~~

* Linux/macOS: ``~/.git-manager/config.json``
* Windows: ``%USERPROFILE%\\.git-manager\\config.json``

Example Configuration
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: json

   {
     "ssh": {
       "directory": "~/.ssh",
       "key_type": "ed25519"
     },
     "git": {
       "default_branch": "main"
     },
     "ui": {
       "theme": "dark"
     }
   }

Common Workflows
----------------

Setting Up a New Account
~~~~~~~~~~~~~~~~~~~~~~~~

1. Generate SSH key
2. Add key to GitHub/GitLab
3. Configure account in Git Manager
4. Test connection

.. code-block:: bash

   git-manager ssh generate --email work@example.com --name work-key
   # Copy public key to GitHub/GitLab
   git-manager account add --name work --platform github \\
     --username myuser --email work@example.com \\
     --ssh-key ~/.ssh/id_ed25519_work
   git-manager ssh test --account work

Cloning a Repository
~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   git-manager clone https://github.com/user/repo --account work

Working with Existing Repository
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   cd /path/to/repo
   git-manager repository setup --account work
   git-manager pull
   # Make changes
   git-manager push

Troubleshooting
---------------

Common Issues
~~~~~~~~~~~~~

**SSH Connection Failed:**

.. code-block:: bash

   # Check SSH key permissions
   chmod 600 ~/.ssh/id_ed25519_work
   
   # Test connection manually
   ssh -T git@github.com

**Account Not Found:**

.. code-block:: bash

   # List all accounts
   git-manager account list
   
   # Verify account name

**Permission Denied:**

.. code-block:: bash

   # Check file ownership
   ls -la ~/.git-manager
   
   # Fix permissions
   chmod 600 ~/.git-manager/accounts.json
"""