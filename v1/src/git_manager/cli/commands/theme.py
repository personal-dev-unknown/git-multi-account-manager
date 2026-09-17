# src/git_manager/cli/commands/theme.py
"""Theme management commands."""

import click
from rich.table import Table


@click.group()
def theme():
    """Theme management."""
    pass


@theme.command('list')
@click.pass_context
def list_themes(ctx):
    """List all available themes."""
    from ..ui.theme_manager import ThemeManager
    
    console = ctx.obj['console']
    manager = ThemeManager()
    themes = manager.list_available_themes()
    
    console.print("\n[cyan]═══ Available Themes ═══[/cyan]\n")
    
    for theme_type, theme_list in themes.items():
        console.print(f"[green]{theme_type.upper()} THEMES:[/green]")
        for theme_name in sorted(theme_list):
            console.print(f"  • {theme_name}")
        console.print()


@theme.command('set')
@click.argument('theme_name')
@click.pass_context
def set_theme(ctx, theme_name):
    """Set the current theme.
    
    Args:
        theme_name: Name of the theme to set
    """
    from ..ui.theme_manager import ThemeManager
    
    console = ctx.obj['console']
    manager = ThemeManager()
    
    if manager.set_theme(theme_name):
        console.print(f"[success]✓ Theme changed to: {theme_name}[/success]")
    else:
        console.print(f"[error]✗ Theme '{theme_name}' not found[/error]")
        console.print("[info]Use 'theme list' to see available themes[/info]")


@theme.command('current')
@click.pass_context
def current_theme(ctx):
    """Show the current theme."""
    from ..ui.theme_manager import ThemeManager
    
    console = ctx.obj['console']
    manager = ThemeManager()
    current = manager.get_current_theme()
    
    # Create a table showing current theme colors
    table = Table(title="Current Theme", border_style="cyan")
    table.add_column("Property", style="cyan")
    table.add_column("Color", style="green")
    
    table.add_row("Theme Name", current.name)
    table.add_row("Type", current.theme_type.value)
    table.add_row("Primary", current.primary)
    table.add_row("Secondary", current.secondary)
    table.add_row("Accent", current.accent)
    table.add_row("Success", current.success)
    table.add_row("Error", current.error)
    table.add_row("Warning", current.warning)
    table.add_row("Info", current.info)
    table.add_row("Background", current.background)
    table.add_row("Text", current.text)
    table.add_row("Border", current.border)
    table.add_row("Highlight", current.highlight)
    
    console.print(table)


@theme.command('preview')
@click.argument('theme_name', required=False)
@click.pass_context
def preview_theme(ctx, theme_name):
    """Preview a theme.
    
    Args:
        theme_name: Name of the theme to preview (uses current if not specified)
    """
    from ..ui.theme_manager import ThemeManager
    from ..ui.color_schemes import get_scheme
    
    console = ctx.obj['console']
    manager = ThemeManager()
    
    if theme_name:
        scheme = get_scheme(theme_name)
        if not scheme:
            console.print(f"[error]✗ Theme '{theme_name}' not found[/error]")
            return
    else:
        scheme = manager.get_current_theme()
    
    # Display preview
    console.print(f"\n[{scheme.accent}]═══ Theme Preview: {scheme.name} ═══[/{scheme.accent}]\n")
    
    console.print(f"[{scheme.primary}]Primary Color[/{scheme.primary}] - Used for main actions")
    console.print(f"[{scheme.secondary}]Secondary Color[/{scheme.secondary}] - Used for secondary actions")
    console.print(f"[{scheme.accent}]Accent Color[/{scheme.accent}] - Used for highlights")
    console.print(f"[{scheme.success}]Success Color[/{scheme.success}] - Used for success messages")
    console.print(f"[{scheme.error}]Error Color[/{scheme.error}] - Used for error messages")
    console.print(f"[{scheme.warning}]Warning Color[/{scheme.warning}] - Used for warnings")
    console.print(f"[{scheme.info}]Info Color[/{scheme.info}] - Used for information")
    console.print(f"[{scheme.highlight}]Highlight Color[/{scheme.highlight}] - Used for highlights\n")
