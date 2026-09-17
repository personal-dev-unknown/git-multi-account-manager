# setup.py
"""Setup script for Git Multi-Account Manager."""

from setuptools import setup, find_packages
from pathlib import Path

# Read README
readme_file = Path(__file__).parent / 'README.md'
long_description = readme_file.read_text() if readme_file.exists() else ''

setup(
    name='git-multi-account-manager',
    version='1.0.0',
    author='DevonionMoses',
    author_email='moses.murungi@strathmore.edu',
    description='Manage multiple Git accounts with ease',
    long_description=long_description,
    long_description_content_type='text/markdown',
    url='https://github.com/DevonionMoses/git-multi-account-manager',
    packages=find_packages(where='src'),
    package_dir={'': 'src'},
    classifiers=[
        'Development Status :: 4 - Beta',
        'Intended Audience :: Developers',
        'Topic :: Software Development :: Version Control :: Git',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3',
        'Programming Language :: Python :: 3.9',
        'Programming Language :: Python :: 3.10',
        'Programming Language :: Python :: 3.11',
        'Programming Language :: Python :: 3.12',
    ],
    python_requires='>=3.9',
    install_requires=[
        'click>=8.0.0',
        'rich>=13.0.0',
        'flask>=3.0.0',
        'flask-socketio>=5.3.0',
        'PyQt6>=6.6.0',
        'pexpect>=4.9.0',
        'pyyaml>=6.0',
    ],
    extras_require={
        'dev': [
            'pytest>=7.0.0',
            'pytest-cov>=4.0.0',
            'black>=23.0.0',
            'flake8>=6.0.0',
            'mypy>=1.0.0',
            'isort>=5.12.0',
            'pylint>=2.17.0',
            'pre-commit>=3.0.0',
            'sphinx>=6.0.0',
        ],
        'web': [
            'gunicorn>=21.0.0',
            'eventlet>=0.33.0',
        ]
    },
    entry_points={
        'console_scripts': [
            'git-manager=git_manager.cli.app:cli',
            'git-manager-web=git_manager.web.app:run_web_server',
            'git-manager-desktop=git_manager.desktop.app:run_desktop_app',
        ],
    },
    include_package_data=True,
    package_data={
        'git_manager': [
            'web/templates/**/*.html',
            'web/static/**/*',
            'desktop/resources/**/*',
        ],
    },
)
