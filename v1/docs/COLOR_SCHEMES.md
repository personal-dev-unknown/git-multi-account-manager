# Color Schemes & Theme System

## Overview

The Git Multi-Account Manager now includes a comprehensive color scheme system with **32 pre-built themes** organized into three categories:

- **Light Themes** (9 themes) - Ideal for bright environments
- **Dark Themes** (7 themes) - Ideal for low-light environments
- **Colored Themes** (12 themes) - Vibrant, branded color palettes

## Available Themes

### Light Themes
Perfect for daytime use or bright environments:

1. **Pure White** - Clean, minimalist white background
2. **Soft Gray** - Gentle gray tones
3. **Silver** - Modern silver palette
4. **Ivory** - Warm ivory background
5. **Warm Beige** - Cozy beige tones
6. **Cream** - Soft cream background
7. **Light Blue** - Cool light blue palette
8. **Light Mint** - Fresh mint green tones
9. **Soft Yellow** - Warm yellow accents

### Dark Themes
Perfect for night mode or low-light environments:

1. **Jet Black** - Pure black background
2. **Graphite** - Deep gray tones
3. **Charcoal** - Dark charcoal palette
4. **Dark Navy** - Deep navy blue
5. **Deep Purple** - Rich purple tones
6. **Dark Forest Green** - Deep green palette
7. **Coffee Brown** - Warm brown tones

### Colored Themes
Vibrant, branded color palettes:

1. **Royal Blue** - Elegant blue theme
2. **Electric Blue** - Bright, energetic blue
3. **Teal** - Cool teal palette
4. **Emerald Green** - Rich emerald tones
5. **Leaf Green** - Fresh green palette
6. **Sunset Orange** - Warm orange tones
7. **Amber** - Golden amber palette
8. **Crimson Red** - Deep red theme
9. **Burgundy** - Rich burgundy tones
10. **Purple Orchid** - Elegant purple
11. **Magenta** - Vibrant magenta
12. **Rose Pink** - Soft pink tones

## Theme Management Commands

### List All Themes
```bash
python3 -m git_manager theme list
```

Output:
```
═══ Available Themes ═══

LIGHT THEMES:
  • cream
  • ivory
  • light_blue
  • light_mint
  • pure_white
  • silver
  • soft_gray
  • soft_yellow
  • warm_beige

DARK THEMES:
  • charcoal
  • coffee_brown
  • deep_purple
  • forest_green
  • graphite
  • jet_black
  • dark_navy

COLORED THEMES:
  • amber
  • burgundy
  • crimson_red
  • electric_blue
  • emerald_green
  • leaf_green
  • magenta
  • purple_orchid
  • rose_pink
  • royal_blue
  • sunset_orange
  • teal
```

### Set Current Theme
```bash
python3 -m git_manager theme set <theme_name>
```

Examples:
```bash
# Set to dark jet black theme
python3 -m git_manager theme set jet_black

# Set to emerald green theme
python3 -m git_manager theme set emerald_green

# Set to light blue theme
python3 -m git_manager theme set light_blue
```

### View Current Theme
```bash
python3 -m git_manager theme current
```

Output:
```
┏━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━┓
┃ Property       ┃ Color           ┃
┡━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━┩
│ Theme Name     │ Jet Black       │
│ Type           │ dark            │
│ Primary        │ bright_cyan     │
│ Secondary      │ cyan            │
│ Accent         │ bright_blue     │
│ Success        │ bright_green    │
│ Error          │ bright_red      │
│ Warning        │ bright_yellow   │
│ Info           │ bright_cyan     │
│ Background     │ black           │
│ Text           │ white           │
│ Border         │ bright_white    │
│ Highlight      │ bright_cyan     │
└────────────────┴─────────────────┘
```

### Preview Theme
```bash
# Preview a specific theme
python3 -m git_manager theme preview <theme_name>

# Preview current theme
python3 -m git_manager theme preview
```

Example:
```bash
python3 -m git_manager theme preview emerald_green
```

## Color Scheme Components

Each theme defines the following color properties:

| Property | Usage |
|----------|-------|
| **primary** | Main action colors (menu items, buttons) |
| **secondary** | Secondary action colors |
| **accent** | Highlight and section headers |
| **success** | Success messages and indicators |
| **error** | Error messages and alerts |
| **warning** | Warning messages |
| **info** | Information messages |
| **background** | Background color |
| **text** | Text color |
| **border** | Border and panel colors |
| **highlight** | Highlight and emphasis |

## Configuration

Theme preferences are stored in:
```
~/.config/git-manager/theme.json          # Linux/Unix (XDG)
~/Library/Application Support/git-manager/theme.json  # macOS
%APPDATA%\git-manager\theme.json          # Windows
```

Example `theme.json`:
```json
{
  "theme": "jet_black"
}
```

## Using Themes in Interactive Mode

When you start the interactive mode:
```bash
python3 -m git_manager --cli
```

The application automatically loads your saved theme preference and applies it to:
- Menu display
- Status messages
- Error/warning messages
- Account listings
- SSH testing output

## Programmatic Usage

### Get Current Theme
```python
from git_manager.cli.ui.theme_manager import ThemeManager

manager = ThemeManager()
current_theme = manager.get_current_theme()
print(f"Current theme: {current_theme.name}")
print(f"Primary color: {current_theme.primary}")
```

### Set Theme Programmatically
```python
from git_manager.cli.ui.theme_manager import ThemeManager

manager = ThemeManager()
success = manager.set_theme("emerald_green")
if success:
    print("Theme changed successfully")
```

### Get Specific Theme
```python
from git_manager.cli.ui.color_schemes import get_scheme

theme = get_scheme("royal_blue")
if theme:
    print(f"Theme: {theme.name}")
    print(f"Type: {theme.theme_type.value}")
```

### List Themes by Type
```python
from git_manager.cli.ui.color_schemes import list_schemes_by_type, ThemeType

dark_themes = list_schemes_by_type(ThemeType.DARK)
for name, scheme in dark_themes.items():
    print(f"{name}: {scheme.name}")
```

## Creating Custom Themes

To create a custom theme, add it to `color_schemes.py`:

```python
from git_manager.cli.ui.color_schemes import ColorScheme, ThemeType

MY_CUSTOM_THEME = ColorScheme(
    name="My Custom Theme",
    theme_type=ThemeType.DARK,
    primary="bright_blue",
    secondary="blue",
    accent="bright_cyan",
    success="bright_green",
    error="bright_red",
    warning="bright_yellow",
    info="bright_cyan",
    background="black",
    text="white",
    border="bright_white",
    highlight="bright_cyan"
)

# Add to ALL_SCHEMES dictionary
ALL_SCHEMES["my_custom_theme"] = MY_CUSTOM_THEME
```

Then use it:
```bash
python3 -m git_manager theme set my_custom_theme
```

## Rich Color Names Reference

The system uses Rich library color names. Common colors available:

**Basic Colors:**
- black, red, green, yellow, blue, magenta, cyan, white

**Bright Colors:**
- bright_black, bright_red, bright_green, bright_yellow, bright_blue, bright_magenta, bright_cyan, bright_white

**Grayscale:**
- grey0 through grey100 (in steps of 3)

**RGB Colors:**
- rgb(r,g,b) format for custom colors

**Named Colors:**
- navy_blue, dark_green, forest_green, teal, etc.

For a complete list, see [Rich Color Documentation](https://rich.readthedocs.io/en/latest/appendix/colors.html)

## Troubleshooting

### Theme not applying
1. Check theme name is correct: `python3 -m git_manager theme list`
2. Verify theme file exists: `cat ~/.config/git-manager/theme.json`
3. Try resetting to default: `python3 -m git_manager theme set jet_black`

### Colors look wrong
1. Ensure terminal supports 256 colors or true color
2. Try a different theme to test
3. Check terminal color settings

### Custom theme not working
1. Verify syntax in `color_schemes.py`
2. Ensure all color properties are defined
3. Use valid Rich color names
4. Restart the application

## Best Practices

1. **For Development:** Use `jet_black` or `dark_navy` for reduced eye strain
2. **For Presentations:** Use `royal_blue` or `emerald_green` for professional appearance
3. **For Accessibility:** Use `pure_white` or `soft_gray` with high contrast
4. **For Branding:** Create custom themes matching your organization colors

## Theme Statistics

- **Total Themes:** 32
- **Light Themes:** 9
- **Dark Themes:** 7
- **Colored Themes:** 12
- **Color Properties per Theme:** 12
- **Total Color Combinations:** 384+
