# src/git_manager/web/routes/accounts.py
"""Account routes."""

from flask import Blueprint, jsonify, request
from ...models.account import Platform
from ...core.exceptions import AccountError

bp = Blueprint('accounts_api', __name__, url_prefix='/api/v1/accounts')


@bp.route('', methods=['GET'])
def list_accounts():
    """List all accounts."""
    from ..app import account_manager
    
    platform = request.args.get('platform')
    platform_filter = Platform(platform) if platform else None
    
    accounts = account_manager.list_accounts(platform_filter)
    return jsonify([acc.to_dict() for acc in accounts])


@bp.route('', methods=['POST'])
def create_account():
    """Create new account."""
    from ..app import account_manager
    
    data = request.json
    
    try:
        account = account_manager.add_account(
            name=data['name'],
            platform=Platform(data['platform']),
            username=data['username'],
            email=data['email'],
            ssh_key_path=data['ssh_key_path'],
            host=data.get('host'),
            description=data.get('description')
        )
        return jsonify(account.to_dict()), 201
    except AccountError as e:
        return jsonify({'error': str(e)}), 400


@bp.route('/<name>', methods=['GET'])
def get_account(name):
    """Get account details."""
    from ..app import account_manager
    
    try:
        account = account_manager.get_account(name)
        return jsonify(account.to_dict())
    except AccountError as e:
        return jsonify({'error': str(e)}), 404


@bp.route('/<name>', methods=['DELETE'])
def delete_account(name):
    """Delete account."""
    from ..app import account_manager
    
    try:
        account_manager.remove_account(name)
        return jsonify({'message': f'Account {name} deleted'}), 200
    except AccountError as e:
        return jsonify({'error': str(e)}), 404


@bp.route('/<name>/test', methods=['POST'])
def test_account(name):
    """Test account SSH connection."""
    from ..app import account_manager, ssh_orchestrator
    
    try:
        account = account_manager.get_account(name)
        success, message = ssh_orchestrator.test_connection(
            account.host,
            account.ssh_key_path
        )
        return jsonify({
            'success': success,
            'message': message
        })
    except AccountError as e:
        return jsonify({'error': str(e)}), 404