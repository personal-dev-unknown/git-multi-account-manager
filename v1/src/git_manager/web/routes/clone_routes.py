"""Clone routes for web API."""

from flask import Blueprint, request, jsonify
from ...core.clone.api import CloneAPI, RepositoryAPI, PlatformAPI
from ...core.account_manager import AccountManager
from ...core.config_manager import ConfigManager
from ...core.database_manager import DatabaseManager

clone_bp = Blueprint('clone', __name__, url_prefix='/api/v1/clone')

# Initialize managers
account_manager = AccountManager()
config_manager = ConfigManager()
database_manager = DatabaseManager()

# Initialize APIs
clone_api = CloneAPI(account_manager, config_manager, database_manager)
repo_api = RepositoryAPI(account_manager, config_manager)
platform_api = PlatformAPI(account_manager, config_manager)


@clone_bp.route('/external', methods=['POST'])
def clone_external():
    """Clone external repository."""
    try:
        data = request.get_json()
        
        result = clone_api.clone_external_repository(
            repo_url=data.get('repo_url'),
            account_name=data.get('account_name'),
            auth_method=data.get('auth_method', 'ssh'),
            destination=data.get('destination'),
            recursive=data.get('recursive', False),
            shallow=data.get('shallow', False),
            fork=data.get('fork', False)
        )
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/personal', methods=['POST'])
def clone_personal():
    """Clone personal repository."""
    try:
        data = request.get_json()
        
        result = clone_api.clone_personal_repository(
            platform=data.get('platform'),
            account_name=data.get('account_name'),
            repo_name=data.get('repo_name'),
            auth_method=data.get('auth_method', 'ssh'),
            destination=data.get('destination'),
            recursive=data.get('recursive', False),
            shallow=data.get('shallow', False)
        )
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/status/<path:destination>', methods=['GET'])
def get_clone_status(destination):
    """Get clone status."""
    try:
        result = clone_api.get_clone_status(destination)
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/repositories', methods=['GET'])
def list_repositories():
    """List personal repositories."""
    try:
        platform = request.args.get('platform')
        account = request.args.get('account')
        refresh = request.args.get('refresh', 'false').lower() == 'true'
        
        result = repo_api.list_personal_repositories(
            platform=platform,
            account_name=account,
            force_refresh=refresh
        )
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/repositories/search', methods=['GET'])
def search_repositories():
    """Search repositories."""
    try:
        platform = request.args.get('platform')
        account = request.args.get('account')
        query = request.args.get('q', '')
        
        result = repo_api.search_repositories(
            platform=platform,
            account_name=account,
            query=query
        )
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/repositories/filter', methods=['GET'])
def filter_repositories():
    """Filter repositories."""
    try:
        platform = request.args.get('platform')
        account = request.args.get('account')
        visibility = request.args.get('visibility')
        language = request.args.get('language')
        is_fork = request.args.get('is_fork')
        
        result = repo_api.filter_repositories(
            platform=platform,
            account_name=account,
            visibility=visibility,
            language=language,
            is_fork=is_fork == 'true' if is_fork else None
        )
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/info', methods=['POST'])
def get_repository_info():
    """Get repository information."""
    try:
        data = request.get_json()
        
        result = repo_api.get_repository_info(
            repo_url=data.get('repo_url')
        )
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/platforms', methods=['GET'])
def get_platforms():
    """Get supported platforms."""
    try:
        result = platform_api.get_supported_platforms()
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/platforms/<platform>', methods=['GET'])
def get_platform_info(platform):
    """Get platform information."""
    try:
        result = platform_api.get_platform_info(platform)
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/platforms/<platform>/test', methods=['POST'])
def test_platform(platform):
    """Test platform connection."""
    try:
        data = request.get_json()
        
        result = platform_api.test_platform_connection(
            platform=platform,
            account_name=data.get('account_name')
        )
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/platforms/<platform>/accounts', methods=['GET'])
def get_platform_accounts(platform):
    """Get accounts for platform."""
    try:
        result = platform_api.get_platform_accounts(platform)
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400


@clone_bp.route('/platforms/<platform>/auth-methods', methods=['GET'])
def get_auth_methods(platform):
    """Get authentication methods for platform."""
    try:
        result = platform_api.get_authentication_methods(platform)
        return jsonify(result)
    
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)}), 400
