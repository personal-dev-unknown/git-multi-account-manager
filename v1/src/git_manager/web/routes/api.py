# src/git_manager/web/routes/api.py
"""API routes."""

from flask import Blueprint, jsonify, request
from ...core.exceptions import GitManagerError

bp = Blueprint('api', __name__, url_prefix='/api/v1')


@bp.route('/health')
def health():
    """Health check endpoint."""
    return jsonify({'status': 'healthy', 'version': '1.0.0'})


@bp.errorhandler(GitManagerError)
def handle_error(error):
    """Handle application errors."""
    return jsonify({'error': str(error)}), 400


@bp.errorhandler(404)
def not_found(error):
    """Handle 404 errors."""
    return jsonify({'error': 'Resource not found'}), 404


@bp.errorhandler(500)
def internal_error(error):
    """Handle 500 errors."""
    return jsonify({'error': 'Internal server error'}), 500