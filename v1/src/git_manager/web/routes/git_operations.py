"""Git Operations Routes - Web API for push/pull/sync operations."""

from flask import Blueprint, request, jsonify, current_app
from flask_socketio import emit, join_room, leave_room
from pathlib import Path
import json

from ...core.sync import (
    PushOperations, PullOperations, SyncOperations,
    GitStatus, GitBranch, GitStage, GitCommit
)
from ...utils.logger import get_logger
from ...utils.log_config import LogCategory, get_logger as get_advanced_logger


logger = get_advanced_logger(__name__, category=LogCategory.GIT_OPERATION)

git_ops_bp = Blueprint('git_operations', __name__, url_prefix='/api/v1/git')


@git_ops_bp.route('/status', methods=['GET'])
def get_status():
    """Get repository status."""
    try:
        repo_path = request.args.get('repo_path', default=str(Path.cwd()))
        repo_path = Path(repo_path)
        
        status_ops = GitStatus()
        status = status_ops.get_status(repo_path)
        
        return jsonify({
            'success': True,
            'branch': status.branch,
            'remote': status.remote,
            'local_ahead': status.local_ahead,
            'remote_ahead': status.remote_ahead,
            'uncommitted_count': status.uncommitted_count,
            'is_clean': status.is_clean,
            'details': status.details
        })
    except Exception as e:
        logger.error(f"Error getting status: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@git_ops_bp.route('/push', methods=['POST'])
def push():
    """Execute push operation."""
    try:
        data = request.get_json()
        repo_path = Path(data.get('repo_path', Path.cwd()))
        strategy = data.get('strategy', 'safe')  # safe, force_lease, force, all, tags, dry_run
        
        push_ops = PushOperations()
        
        if strategy == 'safe':
            result = push_ops.safe_push(repo_path)
        elif strategy == 'force_lease':
            result = push_ops.push_with_lease(repo_path)
        elif strategy == 'force':
            result = push_ops.force_push(repo_path)
        elif strategy == 'all':
            result = push_ops.push_all_branches(repo_path)
        elif strategy == 'tags':
            result = push_ops.push_with_tags(repo_path)
        elif strategy == 'dry_run':
            result = push_ops.dry_run_push(repo_path)
        else:
            return jsonify({'success': False, 'error': 'Unknown strategy'}), 400
        
        logger.info(f"Push operation ({strategy}): {result.message}")
        return jsonify({
            'success': result.success,
            'message': result.message,
            'details': result.details
        })
    except Exception as e:
        logger.error(f"Error during push: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@git_ops_bp.route('/pull', methods=['POST'])
def pull():
    """Execute pull operation."""
    try:
        data = request.get_json()
        repo_path = Path(data.get('repo_path', Path.cwd()))
        strategy = data.get('strategy', 'safe')  # safe, smart, rebase, ff_only, autostash, force, fetch
        
        pull_ops = PullOperations()
        
        if strategy == 'safe':
            result = pull_ops.safe_pull(repo_path)
        elif strategy == 'smart':
            result = pull_ops.smart_pull(repo_path)
        elif strategy == 'rebase':
            result = pull_ops.pull_rebase(repo_path)
        elif strategy == 'ff_only':
            result = pull_ops.pull_ff_only(repo_path)
        elif strategy == 'autostash':
            result = pull_ops.pull_autostash(repo_path)
        elif strategy == 'force':
            result = pull_ops.force_pull(repo_path)
        elif strategy == 'fetch':
            result = pull_ops.fetch_only(repo_path)
        else:
            return jsonify({'success': False, 'error': 'Unknown strategy'}), 400
        
        logger.info(f"Pull operation ({strategy}): {result.message}")
        return jsonify({
            'success': result.success,
            'message': result.message,
            'details': result.details
        })
    except Exception as e:
        logger.error(f"Error during pull: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@git_ops_bp.route('/sync', methods=['POST'])
def sync():
    """Execute sync operation."""
    try:
        data = request.get_json()
        repo_path = Path(data.get('repo_path', Path.cwd()))
        strategy = data.get('strategy', 'smart')  # smart, conservative, rebase, merge, aggressive, dry_run
        
        sync_ops = SyncOperations()
        
        if strategy == 'smart':
            result = sync_ops.smart_sync(repo_path)
        elif strategy == 'conservative':
            result = sync_ops.conservative_sync(repo_path)
        elif strategy == 'rebase':
            result = sync_ops.rebase_sync(repo_path)
        elif strategy == 'merge':
            result = sync_ops.merge_sync(repo_path)
        elif strategy == 'aggressive':
            result = sync_ops.aggressive_sync(repo_path)
        elif strategy == 'dry_run':
            result = sync_ops.dry_run_sync(repo_path)
        else:
            return jsonify({'success': False, 'error': 'Unknown strategy'}), 400
        
        logger.info(f"Sync operation ({strategy}): {result.message}")
        return jsonify({
            'success': result.success,
            'message': result.message,
            'details': result.details
        })
    except Exception as e:
        logger.error(f"Error during sync: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@git_ops_bp.route('/branches', methods=['GET'])
def list_branches():
    """List branches."""
    try:
        repo_path = request.args.get('repo_path', default=str(Path.cwd()))
        repo_path = Path(repo_path)
        remote = request.args.get('remote', default=False, type=bool)
        
        branch_ops = GitBranch()
        branches = branch_ops.list_branches(repo_path, remote=remote)
        
        return jsonify({
            'success': True,
            'branches': [
                {
                    'name': b.name,
                    'is_current': b.is_current,
                    'tracking': b.tracking
                }
                for b in branches
            ]
        })
    except Exception as e:
        logger.error(f"Error listing branches: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@git_ops_bp.route('/branch/create', methods=['POST'])
def create_branch():
    """Create new branch."""
    try:
        data = request.get_json()
        repo_path = Path(data.get('repo_path', Path.cwd()))
        branch_name = data.get('branch_name')
        
        if not branch_name:
            return jsonify({'success': False, 'error': 'Branch name required'}), 400
        
        branch_ops = GitBranch()
        success, message = branch_ops.create_branch(repo_path, branch_name)
        
        return jsonify({'success': success, 'message': message})
    except Exception as e:
        logger.error(f"Error creating branch: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@git_ops_bp.route('/branch/switch', methods=['POST'])
def switch_branch():
    """Switch to branch."""
    try:
        data = request.get_json()
        repo_path = Path(data.get('repo_path', Path.cwd()))
        branch_name = data.get('branch_name')
        
        if not branch_name:
            return jsonify({'success': False, 'error': 'Branch name required'}), 400
        
        branch_ops = GitBranch()
        success, message = branch_ops.switch_branch(repo_path, branch_name)
        
        return jsonify({'success': success, 'message': message})
    except Exception as e:
        logger.error(f"Error switching branch: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@git_ops_bp.route('/stage/all', methods=['POST'])
def stage_all():
    """Stage all changes."""
    try:
        data = request.get_json()
        repo_path = Path(data.get('repo_path', Path.cwd()))
        
        stage_ops = GitStage()
        success, message = stage_ops.add_all(repo_path)
        
        return jsonify({'success': success, 'message': message})
    except Exception as e:
        logger.error(f"Error staging changes: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@git_ops_bp.route('/commit', methods=['POST'])
def commit():
    """Create commit."""
    try:
        data = request.get_json()
        repo_path = Path(data.get('repo_path', Path.cwd()))
        message = data.get('message')
        
        if not message:
            return jsonify({'success': False, 'error': 'Commit message required'}), 400
        
        commit_ops = GitCommit()
        success, msg = commit_ops.commit(repo_path, message)
        
        return jsonify({'success': success, 'message': msg})
    except Exception as e:
        logger.error(f"Error creating commit: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@git_ops_bp.route('/log', methods=['GET'])
def get_log():
    """Get commit log."""
    try:
        repo_path = request.args.get('repo_path', default=str(Path.cwd()))
        repo_path = Path(repo_path)
        max_count = request.args.get('max_count', default=10, type=int)
        
        commit_ops = GitCommit()
        logs = commit_ops.get_log(repo_path, max_count=max_count)
        
        return jsonify({
            'success': True,
            'logs': logs
        })
    except Exception as e:
        logger.error(f"Error getting log: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400
