# src/git_manager/cli/ui/colors.py
"""Color management for CLI."""

from dataclasses import dataclass
from typing import Dict
from rich.theme import Theme

from ...utils.constants import TERMINAL_COLORS


@dataclass
class ColorScheme:
    """Color scheme for terminal UI."""
    
    primary: str
    secondary: str
    accent: str
    text: str
    background: str
    success: str
    warning: str
    error: str
    info: str
    reset: str
    
    def get_rich_theme(self) -> Theme:
        """Get Rich theme from color scheme.
        
        Returns:
            Rich Theme object
        """
        return Theme({
            "primary": self.primary.replace('\033[', '').replace('m', ''),
            "secondary": self.secondary.replace('\033[', '').replace('m', ''),
            "accent": self.accent.replace('\033[', '').replace('m', ''),
            "success": self.success.replace('\033[', '').replace('m', ''),
            "warning": self.warning.replace('\033[', '').replace('m', ''),
            "error": self.error.replace('\033[', '').replace('m', ''),
            "info": self.info.replace('\033[', '').replace('m', ''),
        })


def get_color_scheme() -> ColorScheme:
    """Get the default color scheme.
    
    Returns:
        ColorScheme instance
    """
    return ColorScheme(**TERMINAL_COLORS)


def colorize(text: str, color: str) -> str:
    """Colorize text with ANSI codes.
    
    Args:
        text: Text to colorize
        color: Color name or ANSI code
        
    Returns:
        Colorized text
    """
    scheme = get_color_scheme()
    color_code = getattr(scheme, color, scheme.reset)
    return f"{color_code}{text}{scheme.reset}"


def print_colored(text: str, color: str = 'text') -> None:
    """Print colored text.
    
    Args:
        text: Text to print
        color: Color name
    """
    print(colorize(text, color))


def print_success(text: str) -> None:
    """Print success message."""
    print_colored(f"✓ {text}", 'success')


def print_error(text: str) -> None:
    """Print error message."""
    print_colored(f"✗ {text}", 'error')


def print_warning(text: str) -> None:
    """Print warning message."""
    print_colored(f"⚠ {text}", 'warning')


def print_info(text: str) -> None:
    """Print info message."""
    print_colored(f"ℹ {text}", 'info')


def print_header(text: str) -> None:
    """Print header with decoration.
    
    Args:
        text: Header text
    """
    scheme = get_color_scheme()
    width = len(text) + 4
    border = "═" * width
    
    print(f"{scheme.primary}╔{border}╗{scheme.reset}")
    print(f"{scheme.primary}║  {text}  ║{scheme.reset}")
    print(f"{scheme.primary}╚{border}╝{scheme.reset}")


def print_section(title: str) -> None:
    """Print section header.
    
    Args:
        title: Section title
    """
    scheme = get_color_scheme()
    print(f"\n{scheme.accent}═══ {title} ═══{scheme.reset}\n")