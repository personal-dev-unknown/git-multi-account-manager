# src/git_manager/core/ssh/__init__.py
"""
SSH Management Module - Modular & Creative

Complete SSH workflow system with:
✅ Key generation (key_generator.py)
✅ Agent management (agent_manager.py)
✅ Config management (config_manager.py)
✅ Workflow orchestration (orchestrator.py)
✅ Exception handling (exceptions.py)
✅ System integration (integration.py)

Supports all 8 Git platforms:
- GitHub
- GitLab
- Bitbucket
- Azure DevOps
- Self-Hosted
- Cloud Storage
- Local Path
- SourceForge

Usage:
    from git_manager.core.ssh import SSHWorkflowOrchestrator
    
    orchestrator = SSHWorkflowOrchestrator()
    result = orchestrator.setup_account(
        name="devonionMoses",
        email="moses@school.edu",
        platform="github",  # Any of the 8 platforms
        account_type="school"
    )
"""

from .key_generator import SSHKeyGenerator, KeyMetadata
from .agent_manager import SSHAgentManager
from .config_manager import SSHConfigManager, SSHHostEntry, SSHConfigParser
from .orchestrator import SSHWorkflowOrchestrator
from .integration import SSHIntegrationLayer

# Import exceptions from core module (consolidated)
from ..exceptions import (
    SSHError,
    SSHKeyGenerationError,
    SSHKeyNotFoundError,
    SSHKeyAlreadyExistsError,
    SSHKeyPermissionError,
    SSHKeyValidationError,
    SSHAgentError,
    SSHAgentNotRunningError,
    SSHConfigError,
    SSHConfigParseError,
    SSHConnectionTestError,
    SSHURLConversionError,
    SSHMetadataError,
    SSHBackupError,
    SSHIntegrationError,
    SSHDatabaseError,
)

__all__ = [
    # Key Generator
    "SSHKeyGenerator",
    "KeyMetadata",
    
    # Agent Manager
    "SSHAgentManager",
    
    # Config Manager
    "SSHConfigManager",
    "SSHHostEntry",
    "SSHConfigParser",
    
    # Orchestrator
    "SSHWorkflowOrchestrator",
    
    # Integration
    "SSHIntegrationLayer",
    
    # Exceptions (imported from core)
    "SSHError",
    "SSHKeyGenerationError",
    "SSHKeyNotFoundError",
    "SSHKeyAlreadyExistsError",
    "SSHKeyPermissionError",
    "SSHKeyValidationError",
    "SSHAgentError",
    "SSHAgentNotRunningError",
    "SSHConfigError",
    "SSHConfigParseError",
    "SSHConnectionTestError",
    "SSHURLConversionError",
    "SSHMetadataError",
    "SSHBackupError",
    "SSHIntegrationError",
    "SSHDatabaseError",
]
