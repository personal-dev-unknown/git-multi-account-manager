#!/usr/bin/env python3
"""
Demonstration of the Git Multi-Account Manager logging system.

This script shows various logging capabilities and best practices.
"""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from git_manager.utils.log_config import (
    initialize_logging,
    LogLevel,
    LogCategory,
    get_logger,
    AuditLogger,
    PerformanceLogger,
    LogStorageManager,
)
from git_manager.utils.log_utils import (
    log_operation,
    log_with_timing,
    log_errors,
    log_git_operation,
    log_ssh_operation,
    log_security_event,
    ContextLogger,
    get_log_directory_info,
    list_log_files,
)
import time


def demo_basic_logging():
    """Demonstrate basic logging."""
    print("\n" + "="*60)
    print("DEMO 1: Basic Logging")
    print("="*60)
    
    logger = get_logger(__name__, category=LogCategory.ACTIVITY)
    
    logger.debug("This is a debug message")
    logger.info("This is an info message")
    logger.warning("This is a warning message")
    logger.error("This is an error message")
    
    print("✓ Check ~/.git-manager/logs/activity.log")


def demo_git_operations():
    """Demonstrate Git operation logging."""
    print("\n" + "="*60)
    print("DEMO 2: Git Operations Logging")
    print("="*60)
    
    # Successful operation
    log_git_operation("clone", repository="https://github.com/user/repo", success=True)
    print("✓ Logged successful clone operation")
    
    # Failed operation
    log_git_operation(
        "push",
        repository="my-repo",
        success=False,
        error_msg="Permission denied (publickey)"
    )
    print("✓ Logged failed push operation")
    
    print("✓ Check ~/.git-manager/logs/git_operation.log")


def demo_ssh_operations():
    """Demonstrate SSH operation logging."""
    print("\n" + "="*60)
    print("DEMO 3: SSH Operations Logging")
    print("="*60)
    
    # Successful connection
    log_ssh_operation("connect", account="github", success=True)
    print("✓ Logged successful SSH connection")
    
    # Failed key generation
    log_ssh_operation(
        "generate_key",
        account="gitlab",
        success=False,
        error_msg="Invalid email format"
    )
    print("✓ Logged failed key generation")
    
    print("✓ Check ~/.git-manager/logs/ssh_operation.log")


def demo_security_events():
    """Demonstrate security event logging."""
    print("\n" + "="*60)
    print("DEMO 4: Security Events Logging")
    print("="*60)
    
    log_security_event(
        "ssh_key_generated",
        {"key_type": "ed25519", "key_name": "github", "bits": 256}
    )
    print("✓ Logged SSH key generation")
    
    log_security_event(
        "account_configured",
        {"platform": "github", "account_name": "john_doe"}
    )
    print("✓ Logged account configuration")
    
    print("✓ Check ~/.git-manager/logs/security.log")


def demo_decorators():
    """Demonstrate logging decorators."""
    print("\n" + "="*60)
    print("DEMO 5: Logging Decorators")
    print("="*60)
    
    @log_operation(category=LogCategory.GIT_OPERATION)
    def simulated_clone():
        """Simulated clone operation."""
        time.sleep(0.1)
        return "cloned"
    
    @log_with_timing(threshold_ms=50)
    def simulated_push():
        """Simulated push operation."""
        time.sleep(0.2)
        return "pushed"
    
    @log_errors(category=LogCategory.ERROR)
    def simulated_error():
        """Simulated operation with error."""
        raise ValueError("Simulated error for demo")
    
    # Test log_operation
    result = simulated_clone()
    print(f"✓ Logged operation: {result}")
    
    # Test log_with_timing
    result = simulated_push()
    print(f"✓ Logged operation with timing: {result}")
    
    # Test log_errors
    try:
        simulated_error()
    except ValueError:
        print("✓ Logged error operation")
    
    print("✓ Check ~/.git-manager/logs/git_operation.log and performance.log")


def demo_context_manager():
    """Demonstrate context manager logging."""
    print("\n" + "="*60)
    print("DEMO 6: Context Manager Logging")
    print("="*60)
    
    with ContextLogger("complex_operation", category=LogCategory.GIT_OPERATION):
        print("  Performing complex operation...")
        time.sleep(0.1)
        print("  Operation completed successfully")
    
    print("✓ Logged context operation")
    
    # Demonstrate error handling
    try:
        with ContextLogger("failing_operation", category=LogCategory.GIT_OPERATION):
            print("  Performing operation that will fail...")
            time.sleep(0.05)
            raise RuntimeError("Simulated failure")
    except RuntimeError:
        print("✓ Logged failing context operation")
    
    print("✓ Check ~/.git-manager/logs/git_operation.log")


def demo_audit_logger():
    """Demonstrate audit logger."""
    print("\n" + "="*60)
    print("DEMO 7: Audit Logger")
    print("="*60)
    
    audit_logger = AuditLogger()
    
    audit_logger.log_operation(
        operation="account_created",
        status="success",
        user="john_doe",
        details={"platform": "github", "account": "john"}
    )
    print("✓ Logged account creation")
    
    audit_logger.log_operation(
        operation="ssh_key_rotated",
        status="success",
        user="jane_doe",
        details={"key_type": "ed25519", "old_key_id": "abc123"}
    )
    print("✓ Logged SSH key rotation")
    
    print("✓ Check ~/.git-manager/logs/audit.log")


def demo_performance_logger():
    """Demonstrate performance logger."""
    print("\n" + "="*60)
    print("DEMO 8: Performance Logger")
    print("="*60)
    
    perf_logger = PerformanceLogger()
    
    # Fast operation
    perf_logger.log_operation_time(
        operation="quick_clone",
        duration_ms=250,
        threshold_ms=1000
    )
    print("✓ Logged fast operation")
    
    # Slow operation (will trigger warning)
    perf_logger.log_operation_time(
        operation="large_clone",
        duration_ms=2500,
        threshold_ms=1000
    )
    print("✓ Logged slow operation (warning threshold exceeded)")
    
    print("✓ Check ~/.git-manager/logs/performance.log")


def demo_storage_info():
    """Demonstrate storage information."""
    print("\n" + "="*60)
    print("DEMO 9: Storage Information")
    print("="*60)
    
    storage_info = get_log_directory_info()
    print(f"Log Directory: {storage_info['log_directory']}")
    print(f"Installation Type: {'System-wide' if storage_info['is_system_wide'] else 'User'}")
    print(f"Available Space: {storage_info['available_space_mb']:.2f} MB")
    
    print("\nLog Files:")
    log_files = list_log_files()
    for log_file in log_files:
        print(f"  - {log_file['name']}: {log_file['size_kb']:.2f} KB")


def demo_all_categories():
    """Demonstrate all log categories."""
    print("\n" + "="*60)
    print("DEMO 10: All Log Categories")
    print("="*60)
    
    categories = [
        (LogCategory.ACTIVITY, "General activity"),
        (LogCategory.ERROR, "Error message"),
        (LogCategory.SECURITY, "Security event"),
        (LogCategory.PERFORMANCE, "Performance metric"),
        (LogCategory.GIT_OPERATION, "Git operation"),
        (LogCategory.SSH_OPERATION, "SSH operation"),
        (LogCategory.AUDIT, "Audit event"),
    ]
    
    for category, message in categories:
        logger = get_logger(__name__, category=category)
        logger.info(f"Demo message: {message}")
        print(f"✓ Logged to {category.value}.log")


def main():
    """Run all demos."""
    print("\n" + "="*60)
    print("Git Multi-Account Manager - Logging System Demo")
    print("="*60)
    
    # Initialize logging
    initialize_logging(
        log_level=LogLevel.DEBUG,
        use_json=False,
        enable_console=True,
    )
    
    # Run demos
    demo_basic_logging()
    demo_git_operations()
    demo_ssh_operations()
    demo_security_events()
    demo_decorators()
    demo_context_manager()
    demo_audit_logger()
    demo_performance_logger()
    demo_storage_info()
    demo_all_categories()
    
    print("\n" + "="*60)
    print("Demo Complete!")
    print("="*60)
    print("\nView logs with:")
    print("  git-manager --cli logs view")
    print("  git-manager --cli logs list")
    print("  git-manager --cli logs info")
    print("  git-manager --cli logs categories")
    print("\nExport logs with:")
    print("  git-manager --cli logs export")
    print("\nClear logs with:")
    print("  git-manager --cli logs clear")


if __name__ == "__main__":
    main()
