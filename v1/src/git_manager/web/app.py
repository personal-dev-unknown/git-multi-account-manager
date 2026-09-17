
# src/git_manager/web/app.py
"""Flask web application."""

from flask import Flask, render_template, jsonify, request
from flask_socketio import SocketIO, emit
from pathlib import Path
import os

from ..core.account_manager import AccountManager
from ..core.ssh import SSHWorkflowOrchestrator
from ..core.git_operations import GitOperations
from ..core.database_manager import DatabaseManager
from ..core.config_manager import ConfigManager
from ..utils.logger import get_logger
from ..utils.log_config import initialize_logging, LogLevel, LogCategory, get_logger as get_advanced_logger
from ..utils.constants import APP_NAME, APP_VERSION
from ..cli.ui.theme_manager import ThemeManager
from .routes.git_operations import git_ops_bp

# Initialize advanced logging for web
initialize_logging(
    log_level=LogLevel.INFO,
    use_json=True,  # Use JSON for structured logging in web
    enable_console=False,  # Disable console for web server
)

logger = get_advanced_logger(__name__, category=LogCategory.ACTIVITY)

# Initialize Flask app
app = Flask(__name__)
app.config['SECRET_KEY'] = os.environ.get('SECRET_KEY', 'dev-secret-key-change-in-production')

# Initialize SocketIO with automatic async mode detection
# Tries gevent first (production), falls back to threading (development)
try:
    import gevent
    from gevent import pywsgi
    from geventwebsocket.handler import WebSocketHandler
    async_mode = 'gevent'
    logger.info("Using gevent async mode for WebSocket support")
except ImportError:
    async_mode = 'threading'
    logger.info("gevent not available, using threading mode (polling fallback)")

socketio = SocketIO(
    app,
    cors_allowed_origins="*",
    async_mode=async_mode,
    ping_timeout=60,
    ping_interval=25,
    engineio_logger=False,  # Disable engineio debug logs
    socketio_logger=False   # Disable socketio debug logs
)

# Initialize managers
database_manager = DatabaseManager()
account_manager = AccountManager()
ssh_orchestrator = SSHWorkflowOrchestrator()
git_operations = GitOperations(account_manager)
config_manager = ConfigManager()
theme_manager = ThemeManager()
current_theme = theme_manager.get_current_theme()

# Store managers in app config for routes
app.config['database_manager'] = database_manager
app.config['account_manager'] = account_manager
app.config['config_manager'] = config_manager
app.config['ssh_orchestrator'] = ssh_orchestrator

# Register blueprints
app.register_blueprint(git_ops_bp)

logger.info(f"Web application initialized with theme: {current_theme.name}")


@app.route('/')
def index():
    """Main dashboard."""
    return render_template('index.html', 
                         app_name=APP_NAME, 
                         version=APP_VERSION,
                         theme=current_theme)


@app.route('/accounts')
def accounts_page():
    """Accounts management page."""
    return render_template('accounts/list.html')


@app.route('/repositories')
def repositories_page():
    """Repositories management page."""
    return render_template('repositories/list.html')


@app.route('/theme')
def theme_page():
    """Theme settings page."""
    return render_template('theme.html')


@app.route('/ssh')
def ssh_page():
    """SSH keys management page."""
    return render_template('ssh/keys.html')


@app.route('/clone')
def clone_page():
    """Clone repository page."""
    return render_template('repositories/clone.html')


# Register blueprints
from .routes import api, accounts, repositories, ssh_routes, clone_routes, interactive

app.register_blueprint(api.bp)
app.register_blueprint(accounts.bp)
app.register_blueprint(repositories.bp)
app.register_blueprint(ssh_routes.ssh_bp)
app.register_blueprint(clone_routes.clone_bp)
app.register_blueprint(interactive.interactive_bp)


# WebSocket events
@socketio.on('connect')
def handle_connect():
    """Handle client connection."""
    logger.info('WebSocket client connected')
    emit('status', {'message': 'Connected to Git Manager'})


@socketio.on('disconnect')
def handle_disconnect():
    """Handle client disconnection."""
    logger.info('WebSocket client disconnected')


# Theme API endpoints
@app.route('/api/v1/theme/current', methods=['GET'])
def get_current_theme():
    """Get current theme."""
    theme = theme_manager.get_current_theme()
    return jsonify({
        'name': theme.name,
        'type': theme.theme_type.value,
        'colors': {
            'primary': theme.primary,
            'secondary': theme.secondary,
            'accent': theme.accent,
            'success': theme.success,
            'error': theme.error,
            'warning': theme.warning,
            'info': theme.info,
            'background': theme.background,
            'text': theme.text,
            'border': theme.border,
            'highlight': theme.highlight,
        }
    })


@app.route('/api/v1/theme/list', methods=['GET'])
def list_themes():
    """List all available themes."""
    themes = theme_manager.list_available_themes()
    return jsonify(themes)


@app.route('/api/v1/theme/set', methods=['POST'])
def set_theme():
    """Set current theme."""
    data = request.json
    theme_name = data.get('theme')
    
    if not theme_name:
        return jsonify({'error': 'Theme name required'}), 400
    
    if theme_manager.set_theme(theme_name):
        global current_theme
        current_theme = theme_manager.get_current_theme()
        logger.info(f"Theme changed to: {theme_name}")
        return jsonify({'success': True, 'message': f'Theme changed to {theme_name}'})
    else:
        return jsonify({'error': f'Theme {theme_name} not found'}), 404


def create_app(config=None):
    """Application factory."""
    if config:
        app.config.update(config)
    return app


def run_web_server(host='0.0.0.0', port=5000, debug=False):
    """Run the web server with appropriate async mode."""
    try:
        logger.info(f"Starting web server on {host}:{port}")
        logger.info(f"{APP_NAME} v{APP_VERSION} (Web)")
        logger.info(f"Using async mode: {async_mode}")
        logger.info("Press CTRL+C to shutdown")
        
        if async_mode == 'gevent':
            # Production: Use gevent with WebSocket support
            logger.info("Running with gevent-websocket support")
            try:
                socketio.run(app, host=host, port=port, debug=debug)
            except KeyboardInterrupt:
                logger.info("Shutting down server...")
                logger.info("Server stopped gracefully")
        else:
            # Development: Use threading with polling fallback
            logger.info("Running with threading mode (polling fallback)")
            try:
                socketio.run(app, host=host, port=port, debug=debug, use_reloader=debug)
            except KeyboardInterrupt:
                logger.info("Shutting down server...")
                logger.info("Server stopped gracefully")
    except KeyboardInterrupt:
        logger.info("Server interrupted by user")
    except Exception as e:
        logger.error(f"Web server error: {str(e)}", exc_info=True)
        raise


def run_production_server(host='0.0.0.0', port=5000):
    """Run production server with gevent-websocket.
    
    Usage:
        gunicorn --worker-class geventwebsocket.gunicorn.workers.GeventWebSocketWorker \\
                 --bind 0.0.0.0:5000 \\
                 src.git_manager.web.app:app
    
    Or use this function directly:
        from src.git_manager.web.app import run_production_server
        run_production_server()
    """
    if async_mode != 'gevent':
        logger.warning("gevent not available. Install with: pip install gevent gevent-websocket")
        logger.warning("Falling back to threading mode")
    
    try:
        logger.info(f"Starting production server on {host}:{port}")
        logger.info(f"{APP_NAME} v{APP_VERSION} (Production Web)")
        logger.info(f"Using async mode: {async_mode}")
        
        if async_mode == 'gevent':
            # Use gevent's pywsgi server with WebSocket support
            from gevent import pywsgi
            from geventwebsocket.handler import WebSocketHandler
            
            server = pywsgi.WSGIServer(
                (host, port),
                app,
                handler_class=WebSocketHandler
            )
            logger.info("WebSocket support enabled")
            logger.info("Press CTRL+C to shutdown")
            
            try:
                server.serve_forever()
            except KeyboardInterrupt:
                logger.info("Shutting down server...")
                server.stop()
                logger.info("Server stopped gracefully")
        else:
            # Fallback to socketio.run with threading
            try:
                socketio.run(app, host=host, port=port, debug=False)
            except KeyboardInterrupt:
                logger.info("Shutting down server...")
                logger.info("Server stopped gracefully")
    except KeyboardInterrupt:
        logger.info("Server interrupted by user")
    except Exception as e:
        logger.error(f"Production server error: {str(e)}", exc_info=True)
        raise


if __name__ == '__main__':
    import sys
    
    # Parse command line arguments
    host = os.environ.get('GIT_MANAGER_WEB_HOST', '0.0.0.0')
    port = int(os.environ.get('GIT_MANAGER_WEB_PORT', 5000))
    debug = '--debug' in sys.argv or os.environ.get('FLASK_DEBUG') == '1'
    production = '--production' in sys.argv or os.environ.get('PRODUCTION') == '1'
    
    if production:
        run_production_server(host, port)
    else:
        run_web_server(host, port, debug=debug)