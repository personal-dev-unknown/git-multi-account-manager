"""Git operations commands - push, pull, sync, status, branch, stage, commit."""

import click
from pathlib import Path
from rich.table import Table

from ...core.sync import (
    PushOperations, PullOperations, SyncOperations,
    GitStatus, GitBranch, GitStage, GitCommit
)
from ...utils.logger import get_logger
from ...utils.log_config import LogCategory, get_logger as get_advanced_logger


logger = get_advanced_logger(__name__, category=LogCategory.GIT_OPERATION)


@click.group()
def git():
    """Git operations - push, pull, sync, status, branch, stage, commit."""
    pass


# ==================== PUSH COMMANDS ====================

@git.group()
def push():
    """Push operations."""
    pass


@push.command('safe')
@click.option('--repo', default='.', help='Repository path')
def push_safe(repo):
    """Safe push with pre-flight checks."""
    try:
        push_ops = PushOperations()
        result = push_ops.safe_push(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Safe push successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Safe push failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Push error: {e}")


@push.command('force-lease')
@click.option('--repo', default='.', help='Repository path')
def push_force_lease(repo):
    """Push with force-lease."""
    try:
        push_ops = PushOperations()
        result = push_ops.push_with_lease(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Force-lease push successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Force-lease push failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Force-lease push error: {e}")


@push.command('all')
@click.option('--repo', default='.', help='Repository path')
def push_all(repo):
    """Push all branches."""
    try:
        push_ops = PushOperations()
        result = push_ops.push_all_branches(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Push all successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Push all failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Push all error: {e}")


@push.command('tags')
@click.option('--repo', default='.', help='Repository path')
def push_tags(repo):
    """Push with tags."""
    try:
        push_ops = PushOperations()
        result = push_ops.push_with_tags(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Push tags successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Push tags failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Push tags error: {e}")


@push.command('dry-run')
@click.option('--repo', default='.', help='Repository path')
def push_dry_run(repo):
    """Dry run push."""
    try:
        push_ops = PushOperations()
        result = push_ops.dry_run_push(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Dry-run push: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Dry-run push failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Dry-run push error: {e}")


# ==================== PULL COMMANDS ====================

@git.group()
def pull():
    """Pull operations."""
    pass


@pull.command('safe')
@click.option('--repo', default='.', help='Repository path')
def pull_safe(repo):
    """Safe pull with auto-stash."""
    try:
        pull_ops = PullOperations()
        result = pull_ops.safe_pull(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Safe pull successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Safe pull failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Pull error: {e}")


@pull.command('smart')
@click.option('--repo', default='.', help='Repository path')
def pull_smart(repo):
    """Smart pull with intelligent strategy."""
    try:
        pull_ops = PullOperations()
        result = pull_ops.smart_pull(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Smart pull successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Smart pull failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Smart pull error: {e}")


@pull.command('rebase')
@click.option('--repo', default='.', help='Repository path')
def pull_rebase(repo):
    """Pull with rebase."""
    try:
        pull_ops = PullOperations()
        result = pull_ops.pull_rebase(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Pull rebase successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Pull rebase failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Pull rebase error: {e}")


@pull.command('fetch')
@click.option('--repo', default='.', help='Repository path')
def pull_fetch(repo):
    """Fetch only (no merge)."""
    try:
        pull_ops = PullOperations()
        result = pull_ops.fetch_only(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Fetch successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Fetch failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Fetch error: {e}")


# ==================== SYNC COMMANDS ====================

@git.group()
def sync():
    """Sync operations."""
    pass


@sync.command('smart')
@click.option('--repo', default='.', help='Repository path')
def sync_smart(repo):
    """Smart sync (recommended)."""
    try:
        sync_ops = SyncOperations()
        result = sync_ops.smart_sync(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Smart sync successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Smart sync failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Sync error: {e}")


@sync.command('conservative')
@click.option('--repo', default='.', help='Repository path')
def sync_conservative(repo):
    """Conservative sync with confirmations."""
    try:
        sync_ops = SyncOperations()
        result = sync_ops.conservative_sync(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Conservative sync successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Conservative sync failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Conservative sync error: {e}")


@sync.command('rebase')
@click.option('--repo', default='.', help='Repository path')
def sync_rebase(repo):
    """Sync with rebase."""
    try:
        sync_ops = SyncOperations()
        result = sync_ops.rebase_sync(Path(repo))
        
        if result.success:
            click.secho(f"✓ {result.message}", fg='green')
            logger.info(f"Rebase sync successful: {result.message}")
        else:
            click.secho(f"✗ {result.message}", fg='red')
            logger.error(f"Rebase sync failed: {result.message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Rebase sync error: {e}")


# ==================== STATUS COMMANDS ====================

@git.command('status')
@click.option('--repo', default='.', help='Repository path')
def git_status(repo):
    """Show repository status."""
    try:
        status_ops = GitStatus()
        status = status_ops.get_status(Path(repo))
        
        # Display status
        click.secho(f"\nBranch: {status.branch}", fg='cyan')
        click.secho(f"Remote: {status.remote}/{status.branch}", fg='cyan')
        click.secho(f"\nCommits:", fg='green')
        click.echo(f"  Local ahead:  {status.local_ahead}")
        click.echo(f"  Remote ahead: {status.remote_ahead}")
        click.secho(f"\nWorking Directory:", fg='green')
        click.echo(f"  Uncommitted: {status.uncommitted_count} files")
        click.echo(f"  Status: {'✓ Clean' if status.is_clean else '⚠️  Dirty'}")
        
        logger.info(f"Status retrieved: {status.branch}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Status error: {e}")


# ==================== BRANCH COMMANDS ====================

@git.group()
def branch():
    """Branch operations."""
    pass


@branch.command('list')
@click.option('--repo', default='.', help='Repository path')
@click.option('--remote', is_flag=True, help='Show remote branches')
def branch_list(repo, remote):
    """List branches."""
    try:
        branch_ops = GitBranch()
        branches = branch_ops.list_branches(Path(repo), remote=remote)
        
        branch_type = "Remote" if remote else "Local"
        click.secho(f"\n{branch_type} Branches:\n", fg='cyan')
        
        for branch in branches:
            marker = "* " if branch.is_current else "  "
            click.echo(f"{marker}{branch.name}")
        
        logger.info(f"Listed {len(branches)} branches")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Branch list error: {e}")


@branch.command('create')
@click.argument('name')
@click.option('--repo', default='.', help='Repository path')
def branch_create(name, repo):
    """Create new branch."""
    try:
        branch_ops = GitBranch()
        success, message = branch_ops.create_branch(Path(repo), name)
        
        if success:
            click.secho(f"✓ {message}", fg='green')
            logger.info(f"Branch created: {name}")
        else:
            click.secho(f"✗ {message}", fg='red')
            logger.error(f"Branch creation failed: {message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Branch create error: {e}")


@branch.command('switch')
@click.argument('name')
@click.option('--repo', default='.', help='Repository path')
def branch_switch(name, repo):
    """Switch to branch."""
    try:
        branch_ops = GitBranch()
        success, message = branch_ops.switch_branch(Path(repo), name)
        
        if success:
            click.secho(f"✓ {message}", fg='green')
            logger.info(f"Switched to branch: {name}")
        else:
            click.secho(f"✗ {message}", fg='red')
            logger.error(f"Branch switch failed: {message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Branch switch error: {e}")


# ==================== STAGE COMMANDS ====================

@git.group()
def stage():
    """Stage operations."""
    pass


@stage.command('all')
@click.option('--repo', default='.', help='Repository path')
def stage_all(repo):
    """Stage all changes."""
    try:
        stage_ops = GitStage()
        success, message = stage_ops.add_all(Path(repo))
        
        if success:
            click.secho(f"✓ {message}", fg='green')
            logger.info(f"Staged all changes: {message}")
        else:
            click.secho(f"✗ {message}", fg='red')
            logger.error(f"Stage all failed: {message}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Stage all error: {e}")


# ==================== COMMIT COMMANDS ====================

@git.group()
def commit():
    """Commit operations."""
    pass


@commit.command('create')
@click.argument('message')
@click.option('--repo', default='.', help='Repository path')
def commit_create(message, repo):
    """Create commit."""
    try:
        commit_ops = GitCommit()
        success, msg = commit_ops.commit(Path(repo), message)
        
        if success:
            click.secho(f"✓ {msg}", fg='green')
            logger.info(f"Commit created: {message}")
        else:
            click.secho(f"✗ {msg}", fg='red')
            logger.error(f"Commit failed: {msg}")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Commit error: {e}")


@commit.command('log')
@click.option('--repo', default='.', help='Repository path')
@click.option('--count', default=10, help='Number of commits to show')
def commit_log(repo, count):
    """Show commit log."""
    try:
        commit_ops = GitCommit()
        logs = commit_ops.get_log(Path(repo), max_count=count)
        
        click.secho(f"\nRecent Commits ({len(logs)}):\n", fg='cyan')
        for log in logs:
            click.echo(f"  {log}")
        
        logger.info(f"Retrieved {len(logs)} commits")
    except Exception as e:
        click.secho(f"✗ Error: {e}", fg='red')
        logger.error(f"Commit log error: {e}")
