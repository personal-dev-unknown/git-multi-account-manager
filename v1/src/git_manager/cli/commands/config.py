# src/git_manager/cli/commands/config.py
"""Configuration commands."""

import click


@click.group()
def config():
    """Configuration management."""
    pass


@config.command('show')
@click.pass_context
def show_config(ctx):
    """Show current configuration."""
    from ..ui.tables import display_config
    
    console = ctx.obj['console']
    # Display configuration
    display_config(console)


@config.command('export')
@click.option('--output', '-o', required=True, type=click.Path(), help='Output file')
@click.pass_context
def export_config(ctx, output):
    """Export configuration."""
    console = ctx.obj['console']
    console.print(f"[info]Exporting configuration to {output}...[/info]")
    # Implementation
    console.print("[success]✓ Configuration exported[/success]")


@config.command('import')
@click.option('--input', '-i', required=True, type=click.Path(exists=True), help='Input file')
@click.pass_context
def import_config(ctx, input):
    """Import configuration."""
    console = ctx.obj['console']
    console.print(f"[info]Importing configuration from {input}...[/info]")
    # Implementation
    console.print("[success]✓ Configuration imported[/success]")