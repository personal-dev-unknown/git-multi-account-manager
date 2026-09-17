# src/git_manager/web/routes/main.py
"""Main routes."""

from flask import Blueprint, render_template

bp = Blueprint('main', __name__)


@bp.route('/')
def index():
    """Main dashboard."""
    return render_template('index.html')