# docs/contributing.rst
"""
Contributing Guide
==================

Welcome! We're excited that you're interested in contributing to
Git Multi-Account Manager.

Getting Started
---------------

1. Fork the repository
2. Clone your fork
3. Create a feature branch
4. Make your changes
5. Submit a pull request

Development Setup
-----------------

.. code-block:: bash

   # Clone repository
   git clone https://github.com/yourusername/git-multi-account-manager.git
   cd git-multi-account-manager

   # Create virtual environment
   python -m venv venv
   source venv/bin/activate

   # Install in development mode
   pip install -e ".[dev]"

   # Install pre-commit hooks
   pre-commit install

Code Standards
--------------

Style Guide
~~~~~~~~~~~

* Follow PEP 8
* Use type hints
* Write docstrings (Google style)
* Maximum line length: 100 characters

Code Quality Tools
~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Format code
   black src/ tests/

   # Sort imports
   isort src/ tests/

   # Lint
   flake8 src/ tests/
   pylint src/

   # Type check
   mypy src/

Testing
-------

Writing Tests
~~~~~~~~~~~~~

* Write unit tests for all new code
* Aim for 80%+ code coverage
* Use pytest fixtures
* Mock external dependencies

Running Tests
~~~~~~~~~~~~~

.. code-block:: bash

   # Run all tests
   pytest

   # Run with coverage
   pytest --cov=git_manager --cov-report=html

   # Run specific test
   pytest tests/unit/test_account_manager.py

   # Run integration tests
   pytest tests/integration/

Documentation
-------------

Building Docs
~~~~~~~~~~~~~

.. code-block:: bash

   cd docs
   make html
   open _build/html/index.html

Writing Docs
~~~~~~~~~~~~

* Update relevant .rst files
* Add docstrings to all public APIs
* Include code examples
* Update CHANGELOG.md

Pull Request Process
--------------------

1. **Branch Naming**: ``feature/description`` or ``fix/description``
2. **Commit Messages**: Clear and descriptive
3. **Code Quality**: All checks must pass
4. **Tests**: Add tests for new features
5. **Documentation**: Update relevant docs
6. **Review**: Request review from maintainers

Example Commit Message
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   feat: add support for BitBucket accounts

   - Implement BitBucket platform support
   - Add BitBucket API integration
   - Update documentation
   - Add unit tests

   Closes #123

Types of Contributions
----------------------

Bug Reports
~~~~~~~~~~~

* Use GitHub Issues
* Include steps to reproduce
* Provide system information
* Attach logs if applicable

Feature Requests
~~~~~~~~~~~~~~~~

* Use GitHub Discussions
* Explain use case
* Describe proposed solution
* Consider alternatives

Code Contributions
~~~~~~~~~~~~~~~~~~

* Bug fixes
* New features
* Performance improvements
* Documentation improvements

Code Review Guidelines
----------------------

For Contributors
~~~~~~~~~~~~~~~~

* Keep PRs focused and small
* Write clear descriptions
* Respond to feedback promptly
* Update based on reviews

For Reviewers
~~~~~~~~~~~~~

* Be constructive and respectful
* Focus on code quality
* Test changes locally
* Approve when ready

Release Process
---------------

1. Update version in all files
2. Update CHANGELOG.md
3. Run full test suite
4. Build documentation
5. Create release tag
6. Deploy to PyPI

Community
---------

* **Discussions**: GitHub Discussions
* **Chat**: Discord Server
* **Email**: dev@gitmanager.dev

Code of Conduct
---------------

We follow the Contributor Covenant Code of Conduct.
Please read CODE_OF_CONDUCT.md for details.

License
-------

By contributing, you agree that your contributions will be
licensed under the MIT License.

Questions?
----------

Feel free to reach out:

* GitHub Discussions
* Email: dev@gitmanager.dev
* Discord: gitmanager.dev/discord

Thank you for contributing! 🎉
"""