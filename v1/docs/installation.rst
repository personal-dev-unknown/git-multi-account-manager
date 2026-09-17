# docs/installation.rst
"""
Installation
============

Prerequisites
-------------

* Python 3.9 or higher
* Git 2.25 or higher
* SSH client

Installation Methods
--------------------

From PyPI
~~~~~~~~~

.. code-block:: bash

   pip install git-multi-account-manager

From Source
~~~~~~~~~~~

**Linux/macOS:**

.. code-block:: bash

   git clone https://github.com/yourusername/git-multi-account-manager.git
   cd git-multi-account-manager
   chmod +x scripts/install.sh
   ./scripts/install.sh
   source venv/bin/activate

**Windows:**

.. code-block:: powershell

   git clone https://github.com/yourusername/git-multi-account-manager.git
   cd git-multi-account-manager
   .\\scripts\\install.ps1
   .\\venv\\Scripts\\Activate.ps1

Development Installation
~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   pip install -e ".[dev]"

Verify Installation
-------------------

.. code-block:: bash

   git-manager --version

Configuration
-------------

After installation, configure your first account:

.. code-block:: bash

   git-manager account add \\
     --name work \\
     --platform github \\
     --username myuser \\
     --email work@example.com \\
     --ssh-key ~/.ssh/id_ed25519_work

Platform-Specific Notes
-----------------------

Linux
~~~~~

Install system dependencies:

.. code-block:: bash

   # Ubuntu/Debian
   sudo apt-get install python3-dev python3-venv

   # Fedora
   sudo dnf install python3-devel

macOS
~~~~~

.. code-block:: bash

   brew install python@3.9

Windows
~~~~~~~

Download Python from `python.org <https://www.python.org/downloads/>`_
and ensure it's added to PATH.
"""