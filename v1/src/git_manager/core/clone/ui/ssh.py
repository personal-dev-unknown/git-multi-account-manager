# src/git_manager/web/routes/ssh.py
"""SSH routes."""

from flask import Blueprint, jsonify, request
from ...models.ssh_key import SSHKeyType

bp = Blueprint('ssh_api', __name__, url_prefix='/api/v1/ssh')


@bp.route('/generate', methods=['POST'])
def generate_key():
    """Generate new SSH key."""
    from ..app import ssh_manager
    
    data = request.json
    
    try:
        key = ssh_manager.generate_key(
            email=data['email'],
            key_name=data['name'],
            key_type=SSHKeyType(data.get('type', 'ed25519')),
            passphrase=data.get('passphrase')
        )
        
        return jsonify({
            'name': key.name,
            'public_key': key.public_key,
            'private_key_path': str(key.private_key_path),
            'public_key_path': str(key.public_key_path)
        }), 201
    except Exception as e:
        return jsonify({'error': str(e)}), 400


@bp.route('/add-to-agent', methods=['POST'])
def add_to_agent():
    """Add an existing SSH key to the SSH agent."""
    from ..app import ssh_manager
    from pathlib import Path
    
    data = request.json
    
    try:
        key_path = Path(data['key_path'])
        passphrase = data.get('passphrase')
        
        # Check if key exists
        if not key_path.exists():
            return jsonify({'error': f'SSH key not found: {key_path}'}), 400
            
        # Add key to agent
        ssh_manager.add_to_agent(key_path=key_path, passphrase=passphrase)
        
        return jsonify({
            'success': True,
            'message': f'Successfully added key to SSH agent: {key_path}'
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 400


@bp.route('/test', methods=['POST'])
def test_connection():
    """Test SSH connection."""
    from ..app import ssh_manager
    from pathlib import Path
    
    data = request.json
    
    try:
        success, message = ssh_manager.test_connection(
            host=data['host'],
            key_path=Path(data['key_path'])
        )
        
        return jsonify({
            'success': success,
            'message': message
        })
    except Exception as e:
        return jsonify({'error': str(e)}), 400