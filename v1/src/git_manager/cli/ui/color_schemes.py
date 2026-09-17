# src/git_manager/cli/ui/color_schemes.py
"""Color schemes for the CLI interface."""

from dataclasses import dataclass
from typing import Dict, Optional
from enum import Enum


class ThemeType(Enum):
    """Available theme types."""
    LIGHT = "light"
    DARK = "dark"
    COLORED = "colored"


@dataclass
class ColorScheme:
    """Represents a color scheme with Rich color names."""
    name: str
    theme_type: ThemeType
    primary: str          # Main action color
    secondary: str        # Secondary action color
    accent: str          # Accent/highlight color
    success: str         # Success messages
    error: str           # Error messages
    warning: str         # Warning messages
    info: str            # Info messages
    background: str      # Background color
    text: str            # Text color
    border: str          # Border color
    highlight: str       # Highlight color


# Light Themes
LIGHT_PURE_WHITE = ColorScheme(
    name="Pure White",
    theme_type=ThemeType.LIGHT,
    primary="blue",
    secondary="cyan",
    accent="bright_blue",
    success="green",
    error="red",
    warning="yellow",
    info="cyan",
    background="white",
    text="black",
    border="bright_black",
    highlight="bright_cyan"
)

LIGHT_SOFT_GRAY = ColorScheme(
    name="Soft Gray",
    theme_type=ThemeType.LIGHT,
    primary="blue",
    secondary="cyan",
    accent="bright_blue",
    success="green",
    error="red",
    warning="yellow",
    info="cyan",
    background="grey93",
    text="grey11",
    border="grey50",
    highlight="bright_cyan"
)

LIGHT_SILVER = ColorScheme(
    name="Silver",
    theme_type=ThemeType.LIGHT,
    primary="bright_blue",
    secondary="cyan",
    accent="bright_cyan",
    success="green",
    error="red",
    warning="yellow",
    info="bright_cyan",
    background="grey89",
    text="grey15",
    border="grey60",
    highlight="bright_white"
)

LIGHT_IVORY = ColorScheme(
    name="Ivory",
    theme_type=ThemeType.LIGHT,
    primary="blue",
    secondary="cyan",
    accent="bright_blue",
    success="green",
    error="red",
    warning="yellow",
    info="cyan",
    background="rgb(255,255,240)",
    text="grey11",
    border="grey70",
    highlight="bright_cyan"
)

LIGHT_WARM_BEIGE = ColorScheme(
    name="Warm Beige",
    theme_type=ThemeType.LIGHT,
    primary="blue",
    secondary="cyan",
    accent="bright_blue",
    success="green",
    error="red",
    warning="yellow",
    info="cyan",
    background="rgb(245,245,220)",
    text="grey11",
    border="grey70",
    highlight="bright_cyan"
)

LIGHT_CREAM = ColorScheme(
    name="Cream",
    theme_type=ThemeType.LIGHT,
    primary="blue",
    secondary="cyan",
    accent="bright_blue",
    success="green",
    error="red",
    warning="yellow",
    info="cyan",
    background="rgb(255,253,208)",
    text="grey11",
    border="grey70",
    highlight="bright_cyan"
)

LIGHT_BLUE = ColorScheme(
    name="Light Blue",
    theme_type=ThemeType.LIGHT,
    primary="bright_blue",
    secondary="cyan",
    accent="bright_cyan",
    success="green",
    error="red",
    warning="yellow",
    info="bright_cyan",
    background="rgb(230,245,255)",
    text="grey11",
    border="bright_blue",
    highlight="bright_cyan"
)

LIGHT_MINT = ColorScheme(
    name="Light Mint",
    theme_type=ThemeType.LIGHT,
    primary="cyan",
    secondary="bright_cyan",
    accent="bright_green",
    success="green",
    error="red",
    warning="yellow",
    info="bright_cyan",
    background="rgb(240,255,240)",
    text="grey11",
    border="cyan",
    highlight="bright_green"
)

LIGHT_SOFT_YELLOW = ColorScheme(
    name="Soft Yellow",
    theme_type=ThemeType.LIGHT,
    primary="blue",
    secondary="cyan",
    accent="bright_yellow",
    success="green",
    error="red",
    warning="yellow",
    info="cyan",
    background="rgb(255,255,240)",
    text="grey11",
    border="yellow",
    highlight="bright_yellow"
)

# Dark Themes
DARK_JET_BLACK = ColorScheme(
    name="Jet Black",
    theme_type=ThemeType.DARK,
    primary="bright_cyan",
    secondary="cyan",
    accent="bright_blue",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="black",
    text="white",
    border="bright_white",
    highlight="bright_cyan"
)

DARK_GRAPHITE = ColorScheme(
    name="Graphite",
    theme_type=ThemeType.DARK,
    primary="bright_cyan",
    secondary="cyan",
    accent="bright_blue",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="grey23",
    text="grey93",
    border="grey60",
    highlight="bright_cyan"
)

DARK_CHARCOAL = ColorScheme(
    name="Charcoal",
    theme_type=ThemeType.DARK,
    primary="bright_cyan",
    secondary="cyan",
    accent="bright_blue",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="grey19",
    text="grey89",
    border="grey50",
    highlight="bright_cyan"
)

DARK_NAVY = ColorScheme(
    name="Dark Navy",
    theme_type=ThemeType.DARK,
    primary="bright_cyan",
    secondary="bright_blue",
    accent="bright_blue",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(25,25,112)",
    text="bright_white",
    border="bright_cyan",
    highlight="bright_cyan"
)

DARK_DEEP_PURPLE = ColorScheme(
    name="Deep Purple",
    theme_type=ThemeType.DARK,
    primary="bright_magenta",
    secondary="magenta",
    accent="bright_magenta",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(75,0,130)",
    text="bright_white",
    border="bright_magenta",
    highlight="bright_magenta"
)

DARK_FOREST_GREEN = ColorScheme(
    name="Dark Forest Green",
    theme_type=ThemeType.DARK,
    primary="bright_green",
    secondary="green",
    accent="bright_green",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(34,139,34)",
    text="bright_white",
    border="bright_green",
    highlight="bright_green"
)

DARK_COFFEE_BROWN = ColorScheme(
    name="Coffee Brown",
    theme_type=ThemeType.DARK,
    primary="bright_yellow",
    secondary="yellow",
    accent="bright_yellow",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(101,67,33)",
    text="bright_white",
    border="bright_yellow",
    highlight="bright_yellow"
)

# Colored Themes
COLORED_ROYAL_BLUE = ColorScheme(
    name="Royal Blue",
    theme_type=ThemeType.COLORED,
    primary="bright_blue",
    secondary="blue",
    accent="bright_cyan",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_blue",
    background="rgb(65,105,225)",
    text="bright_white",
    border="bright_blue",
    highlight="bright_cyan"
)

COLORED_ELECTRIC_BLUE = ColorScheme(
    name="Electric Blue",
    theme_type=ThemeType.COLORED,
    primary="bright_blue",
    secondary="bright_cyan",
    accent="bright_cyan",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_blue",
    background="rgb(0,102,204)",
    text="bright_white",
    border="bright_blue",
    highlight="bright_cyan"
)

COLORED_TEAL = ColorScheme(
    name="Teal",
    theme_type=ThemeType.COLORED,
    primary="bright_cyan",
    secondary="cyan",
    accent="bright_green",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(0,128,128)",
    text="bright_white",
    border="bright_cyan",
    highlight="bright_green"
)

COLORED_EMERALD_GREEN = ColorScheme(
    name="Emerald Green",
    theme_type=ThemeType.COLORED,
    primary="bright_green",
    secondary="green",
    accent="bright_cyan",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(80,200,120)",
    text="bright_white",
    border="bright_green",
    highlight="bright_cyan"
)

COLORED_LEAF_GREEN = ColorScheme(
    name="Leaf Green",
    theme_type=ThemeType.COLORED,
    primary="green",
    secondary="bright_green",
    accent="bright_cyan",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(34,177,76)",
    text="bright_white",
    border="bright_green",
    highlight="bright_cyan"
)

COLORED_SUNSET_ORANGE = ColorScheme(
    name="Sunset Orange",
    theme_type=ThemeType.COLORED,
    primary="bright_yellow",
    secondary="yellow",
    accent="bright_red",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_yellow",
    background="rgb(255,140,0)",
    text="bright_white",
    border="bright_yellow",
    highlight="bright_red"
)

COLORED_AMBER = ColorScheme(
    name="Amber",
    theme_type=ThemeType.COLORED,
    primary="bright_yellow",
    secondary="yellow",
    accent="bright_yellow",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_yellow",
    background="rgb(255,191,0)",
    text="bright_white",
    border="bright_yellow",
    highlight="bright_yellow"
)

COLORED_CRIMSON_RED = ColorScheme(
    name="Crimson Red",
    theme_type=ThemeType.COLORED,
    primary="bright_red",
    secondary="red",
    accent="bright_yellow",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(220,20,60)",
    text="bright_white",
    border="bright_red",
    highlight="bright_yellow"
)

COLORED_BURGUNDY = ColorScheme(
    name="Burgundy",
    theme_type=ThemeType.COLORED,
    primary="red",
    secondary="bright_red",
    accent="bright_yellow",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(128,0,32)",
    text="bright_white",
    border="bright_red",
    highlight="bright_yellow"
)

COLORED_PURPLE_ORCHID = ColorScheme(
    name="Purple Orchid",
    theme_type=ThemeType.COLORED,
    primary="bright_magenta",
    secondary="magenta",
    accent="bright_magenta",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(218,112,214)",
    text="bright_white",
    border="bright_magenta",
    highlight="bright_magenta"
)

COLORED_MAGENTA = ColorScheme(
    name="Magenta",
    theme_type=ThemeType.COLORED,
    primary="bright_magenta",
    secondary="magenta",
    accent="bright_magenta",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(255,0,255)",
    text="bright_white",
    border="bright_magenta",
    highlight="bright_magenta"
)

COLORED_ROSE_PINK = ColorScheme(
    name="Rose Pink",
    theme_type=ThemeType.COLORED,
    primary="bright_magenta",
    secondary="magenta",
    accent="bright_magenta",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="rgb(255,102,178)",
    text="bright_white",
    border="bright_magenta",
    highlight="bright_magenta"
)

# All available color schemes
ALL_SCHEMES: Dict[str, ColorScheme] = {
    # Light themes
    "pure_white": LIGHT_PURE_WHITE,
    "soft_gray": LIGHT_SOFT_GRAY,
    "silver": LIGHT_SILVER,
    "ivory": LIGHT_IVORY,
    "warm_beige": LIGHT_WARM_BEIGE,
    "cream": LIGHT_CREAM,
    "light_blue": LIGHT_BLUE,
    "light_mint": LIGHT_MINT,
    "soft_yellow": LIGHT_SOFT_YELLOW,
    # Dark themes
    "jet_black": DARK_JET_BLACK,
    "graphite": DARK_GRAPHITE,
    "charcoal": DARK_CHARCOAL,
    "dark_navy": DARK_NAVY,
    "deep_purple": DARK_DEEP_PURPLE,
    "forest_green": DARK_FOREST_GREEN,
    "coffee_brown": DARK_COFFEE_BROWN,
    # Colored themes
    "royal_blue": COLORED_ROYAL_BLUE,
    "electric_blue": COLORED_ELECTRIC_BLUE,
    "teal": COLORED_TEAL,
    "emerald_green": COLORED_EMERALD_GREEN,
    "leaf_green": COLORED_LEAF_GREEN,
    "sunset_orange": COLORED_SUNSET_ORANGE,
    "amber": COLORED_AMBER,
    "crimson_red": COLORED_CRIMSON_RED,
    "burgundy": COLORED_BURGUNDY,
    "purple_orchid": COLORED_PURPLE_ORCHID,
    "magenta": COLORED_MAGENTA,
    "rose_pink": COLORED_ROSE_PINK,
}

# Default scheme - Simple, readable dark theme
DEFAULT_SCHEME = DARK_GRAPHITE


def get_scheme(name: str) -> Optional[ColorScheme]:
    """Get a color scheme by name.
    
    Args:
        name: Scheme name (key from ALL_SCHEMES)
        
    Returns:
        ColorScheme object or None if not found
    """
    return ALL_SCHEMES.get(name.lower())


def list_schemes_by_type(theme_type: ThemeType) -> Dict[str, ColorScheme]:
    """Get all schemes of a specific type.
    
    Args:
        theme_type: ThemeType to filter by
        
    Returns:
        Dictionary of schemes matching the type
    """
    return {
        name: scheme
        for name, scheme in ALL_SCHEMES.items()
        if scheme.theme_type == theme_type
    }


def get_all_scheme_names() -> list:
    """Get all available scheme names.
    
    Returns:
        List of scheme names
    """
    return list(ALL_SCHEMES.keys())
