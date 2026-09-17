# src/git_manager/web/routes/repositories.py
"""Repository routes."""

from flask import Blueprint, jsonify, request
from pathlib import Path
from ...core.database_manager import DatabaseManager

bp = Blueprint('repositories_api', __name__, url_prefix='/api/v1/repositories')

# Initialize database manager
database_manager = DatabaseManager()


@bp.route('/clone', methods=['POST'])
def clone_repository():
    """Clone a repository."""
    from ..app import git_operations, socketio
    
    data = request.json
    
    try:
        # Emit progress updates via WebSocket
        socketio.emit('clone_progress', {'status': 'starting', 'url': data['url']})
        
        repo = git_operations.clone(
            url=data['url'],
            account_name=data['account'],
            destination=Path(data.get('destination')) if data.get('destination') else None,
            branch=data.get('branch')
        )
        
        # Log to database
        database_manager.log_clone_operation(
            clone_url=data['url'],
            destination=str(repo.path),
            method='ssh',
            platform_id='github',
            status='success'
        )
        
        socketio.emit('clone_progress', {'status': 'complete', 'path': str(repo.path)})
        
        return jsonify({
            'name': repo.name,
            'path': str(repo.path),
            'url': repo.remote_url
        }), 201
    except Exception as e:
        # Log failed clone
        database_manager.log_clone_operation(
            clone_url=data.get('url', 'unknown'),
            destination=data.get('destination', 'unknown'),
            method='ssh',
            platform_id='github',
            status='failed',
            error_message=str(e)
        )
        socketio.emit('clone_progress', {'status': 'error', 'error': str(e)})
        return jsonify({'error': str(e)}), 400


@bp.route('/status', methods=['GET'])
def repository_status():
    """Get repository status."""
    from ..app import git_operations
    
    path = request.args.get('path')
    repo_path = Path(path) if path else None
    
    try:
        status = git_operations.check_status(repo_path)
        return jsonify({
            'branch': status.current_branch,
            'is_clean': status.is_clean,
            'is_synced': status.is_synced,
            'commits_ahead': status.commits_ahead,
            'commits_behind': status.commits_behind,
            'uncommitted_files': status.uncommitted_files
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 400


@bp.route('/pull', methods=['POST'])
def pull_repository():
    """Pull repository changes."""
    from ..app import git_operations
    
    data = request.json
    path = Path(data.get('path')) if data.get('path') else None
    rebase = data.get('rebase', False)
    
    try:
        success, message = git_operations.pull(path, rebase)
        return jsonify({
            'success': success,
            'message': message
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 400


@bp.route('/push', methods=['POST'])
def push_repository():
    """Push repository changes."""
    from ..app import git_operations
    
    data = request.json
    path = Path(data.get('path')) if data.get('path') else None
    set_upstream = data.get('set_upstream', False)
    branch = data.get('branch')
    
    try:
        success, message = git_operations.push(path, set_upstream, branch)
        return jsonify({
            'success': success,
            'message': message
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 400