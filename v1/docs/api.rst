# docs/api.rst
"""
API Reference
=============

Core Modules
------------

AccountManager
~~~~~~~~~~~~~~

.. automodule:: git_manager.core.account_manager
   :members:
   :undoc-members:
   :show-inheritance:

SSHManager
~~~~~~~~~~

.. automodule:: git_manager.core.ssh_manager
   :members:
   :undoc-members:
   :show-inheritance:

GitOperations
~~~~~~~~~~~~~

.. automodule:: git_manager.core.git_operations
   :members:
   :undoc-members:
   :show-inheritance:

RepositoryManager
~~~~~~~~~~~~~~~~~

.. automodule:: git_manager.core.repository_manager
   :members:
   :undoc-members:
   :show-inheritance:

ConfigManager
~~~~~~~~~~~~~

.. automodule:: git_manager.core.config_manager
   :members:
   :undoc-members:
   :show-inheritance:

Models
------

Account
~~~~~~~

.. automodule:: git_manager.models.account
   :members:
   :undoc-members:
   :show-inheritance:

Repository
~~~~~~~~~~

.. automodule:: git_manager.models.repository
   :members:
   :undoc-members:
   :show-inheritance:

SSHKey
~~~~~~

.. automodule:: git_manager.models.ssh_key
   :members:
   :undoc-members:
   :show-inheritance:

Utilities
---------

Validators
~~~~~~~~~~

.. automodule:: git_manager.utils.validators
   :members:
   :undoc-members:

Logger
~~~~~~

.. automodule:: git_manager.utils.logger
   :members:
   :undoc-members:

File Operations
~~~~~~~~~~~~~~~

.. automodule:: git_manager.utils.file_operations
   :members:
   :undoc-members:

Git Helpers
~~~~~~~~~~~

.. automodule:: git_manager.utils.git_helpers
   :members:
   :undoc-members:

SSH Helpers
~~~~~~~~~~~

.. automodule:: git_manager.utils.ssh_helpers
   :members:
   :undoc-members:

Exceptions
----------

.. automodule:: git_manager.core.exceptions
   :members:
   :undoc-members:
   :show-inheritance:

REST API
--------

Endpoints
~~~~~~~~~

**Accounts**

* ``GET /api/v1/accounts`` - List all accounts
* ``POST /api/v1/accounts`` - Create account
* ``GET /api/v1/accounts/:id`` - Get account
* ``PUT /api/v1/accounts/:id`` - Update account
* ``DELETE /api/v1/accounts/:id`` - Delete account
* ``POST /api/v1/accounts/:id/test`` - Test connection

**Repositories**

* ``GET /api/v1/repositories`` - List repositories
* ``POST /api/v1/repositories/clone`` - Clone repository
* ``GET /api/v1/repositories/:id`` - Get repository
* ``POST /api/v1/repositories/:id/pull`` - Pull changes
* ``POST /api/v1/repositories/:id/push`` - Push changes
* ``GET /api/v1/repositories/:id/status`` - Get status

**SSH**

* ``GET /api/v1/ssh/keys`` - List SSH keys
* ``POST /api/v1/ssh/keys/generate`` - Generate key
* ``POST /api/v1/ssh/keys/test`` - Test connection
* ``GET /api/v1/ssh/config`` - Get SSH config

Example Usage
~~~~~~~~~~~~~

.. code-block:: python

   import requests

   # Get accounts
   response = requests.get('http://localhost:5000/api/v1/accounts')
   accounts = response.json()

   # Add account
   data = {
       'name': 'work',
       'platform': 'github',
       'username': 'myuser',
       'email': 'work@example.com',
       'ssh_key_path': '~/.ssh/id_ed25519_work'
   }
   response = requests.post('http://localhost:5000/api/v1/accounts', json=data)

   # Clone repository
   data = {
       'url': 'https://github.com/user/repo',
       'account': 'work'
   }
   response = requests.post('http://localhost:5000/api/v1/repositories/clone', json=data)
"""