"""Interactive routes for web API - uses shared interactive service."""

from flask import Blueprint, request, jsonify

from ...core.interactive_service import InteractiveService
from ...utils.log_config import get_logger as get_advanced_logger, LogCategory

interactive_bp = Blueprint('interactive', __name__, url_prefix='/api/v1/interactive')

# Initialize shared service
service = InteractiveService()
logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)


# ============ ACCOUNT MANAGEMENT ============

@interactive_bp.route('/accounts', methods=['GET'])
def get_all_accounts():
    """Get all configured accounts with platform info."""
    try:
        result = service.list_all_accounts()
        return jsonify({'success': True, 'accounts': result})
    except Exception as e:
        logger.error(f"Error listing accounts: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@interactive_bp.route('/accounts/<account_name>/test-ssh', methods=['POST'])
def test_ssh_connection(account_name):
    """Test SSH connection for an account."""
    try:
        result = service.test_ssh_connection(account_name)
        
        if result['success']:
            return jsonify(result)
        else:
            return jsonify(result), 400
    except Exception as e:
        logger.error(f"SSH test failed: {e}")
        return jsonify({'success': False, 'message': str(e)}), 400


# ============ CLONE OPERATIONS ============

@interactive_bp.route('/clone/analyze-url', methods=['POST'])
def analyze_repository_url():
    """Analyze a repository URL."""
    try:
        data = request.get_json()
        url = data.get('url')
        
        if not url:
            return jsonify({'success': False, 'error': 'URL is required'}), 400
        
        result = service.analyze_repository_url(url)
        return jsonify(result)
    except Exception as e:
        logger.error(f"URL analysis failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@interactive_bp.route('/clone/personal-repositories', methods=['GET'])
def get_personal_repositories():
    """Fetch personal repositories for an account."""
    try:
        platform = request.args.get('platform')
        account_name = request.args.get('account')
        
        if not platform or not account_name:
            return jsonify({
                'success': False,
                'error': 'platform and account parameters are required'
            }), 400
        
        result = service.get_personal_repositories(platform, account_name)
        return jsonify(result)
    except Exception as e:
        logger.error(f"Repository fetch failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@interactive_bp.route('/clone/repository', methods=['POST'])
def clone_repository():
    """Clone a repository."""
    try:
        data = request.get_json()
        
        repo_url = data.get('repo_url')
        account_name = data.get('account_name')
        destination = data.get('destination')
        auth_method = data.get('auth_method', 'ssh')
        recursive = data.get('recursive', False)
        shallow = data.get('shallow', False)
        
        if not repo_url or not account_name or not destination:
            return jsonify({
                'success': False,
                'error': 'repo_url, account_name, and destination are required'
            }), 400
        
        result = service.clone_repository(
            repo_url,
            account_name,
            destination,
            auth_method,
            recursive,
            shallow
        )
        
        return jsonify(result)
    except Exception as e:
        logger.error(f"Clone failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


# ============ GIT OPERATIONS ============

@interactive_bp.route('/git/status', methods=['GET'])
def check_repository_status():
    """Check repository status."""
    try:
        repo_path = request.args.get('path', '.')
        result = service.check_repository_status(repo_path)
        return jsonify(result)
    except Exception as e:
        logger.error(f"Status check failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@interactive_bp.route('/git/push', methods=['POST'])
def git_push():
    """Push changes to remote."""
    try:
        data = request.get_json()
        repo_path = data.get('path', '.')
        
        result = service.git_push(repo_path)
        return jsonify(result)
    except Exception as e:
        logger.error(f"Push failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@interactive_bp.route('/git/pull', methods=['POST'])
def git_pull():
    """Pull changes from remote."""
    try:
        data = request.get_json()
        repo_path = data.get('path', '.')
        
        result = service.git_pull(repo_path)
        return jsonify(result)
    except Exception as e:
        logger.error(f"Pull failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@interactive_bp.route('/git/sync', methods=['POST'])
def git_sync():
    """Sync repository (pull then push)."""
    try:
        data = request.get_json()
        repo_path = data.get('path', '.')
        
        result = service.git_sync(repo_path)
        return jsonify(result)
    except Exception as e:
        logger.error(f"Sync failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


# ============ REPOSITORY SETUP ============

@interactive_bp.route('/repository/setup', methods=['POST'])
def setup_new_repository():
    """Setup a new local repository."""
    try:
        data = request.get_json()
        
        repo_path = data.get('path')
        account_name = data.get('account_name')
        repo_name = data.get('repo_name')
        description = data.get('description', '')
        branch = data.get('branch', 'main')
        
        if not repo_path or not account_name or not repo_name:
            return jsonify({
                'success': False,
                'error': 'path, account_name, and repo_name are required'
            }), 400
        
        result = service.setup_new_repository(
            repo_path,
            account_name,
            repo_name,
            description,
            branch
        )
        
        return jsonify(result)
    except Exception as e:
        logger.error(f"Repository setup failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


# ============ SSH KEY MANAGEMENT ============

@interactive_bp.route('/ssh/generate-key', methods=['POST'])
def generate_ssh_key():
    """Generate SSH key for account."""
    try:
        data = request.get_json()
        
        account_name = data.get('account_name')
        email = data.get('email')
        platform = data.get('platform')
        account_type = data.get('account_type', 'personal')
        key_type = data.get('key_type', 'ed25519')
        passphrase = data.get('passphrase')
        
        if not account_name or not email or not platform:
            return jsonify({
                'success': False,
                'error': 'account_name, email, and platform are required'
            }), 400
        
        result = service.generate_ssh_key(
            account_name,
            email,
            platform,
            account_type,
            key_type,
            passphrase
        )
        
        return jsonify(result)
    except Exception as e:
        logger.error(f"Key generation failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@interactive_bp.route('/ssh/test-pat', methods=['POST'])
def test_pat_token():
    """Test if a PAT token is valid."""
    try:
        data = request.get_json()
        platform = data.get('platform')
        pat_token = data.get('pat_token')
        
        if not platform or not pat_token:
            return jsonify({
                'success': False,
                'error': 'platform and pat_token are required'
            }), 400
        
        result = service.test_pat_token(platform, pat_token)
        return jsonify(result)
    except Exception as e:
        logger.error(f"PAT test failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


# ============ PLATFORM INFO ============

@interactive_bp.route('/platforms', methods=['GET'])
def get_all_platforms():
    """Get all available platforms."""
    try:
        result = service.get_all_platforms()
        return jsonify(result)
    except Exception as e:
        logger.error(f"Platform fetch failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400


@interactive_bp.route('/platforms/<platform_name>', methods=['GET'])
def get_platform_info(platform_name):
    """Get platform information."""
    try:
        result = service.get_platform_info(platform_name)
        
        if not result['success']:
            return jsonify(result), 404
        
        return jsonify(result)
    except Exception as e:
        logger.error(f"Platform info fetch failed: {e}")
        return jsonify({'success': False, 'error': str(e)}), 400
