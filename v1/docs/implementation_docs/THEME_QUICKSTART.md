# Theme System Quick Start Guide

## What's New?

The Git Multi-Account Manager now includes a **comprehensive color theme system** with 32 pre-built themes! You can customize the appearance of the CLI to match your preferences.

## Quick Start

### 1. See Available Themes
```bash
python3 -m git_manager theme list
```

### 2. Choose Your Theme
Pick from:
- **9 Light Themes** - For bright environments
- **7 Dark Themes** - For low-light environments  
- **12 Colored Themes** - For vibrant, branded looks

### 3. Set Your Theme
```bash
python3 -m git_manager theme set <theme_name>
```

Examples:
```bash
# Dark themes (recommended for terminal)
python3 -m git_manager theme set jet_black
python3 -m git_manager theme set dark_navy
python3 -m git_manager theme set forest_green

# Colored themes
python3 -m git_manager theme set emerald_green
python3 -m git_manager theme set royal_blue
python3 -m git_manager theme set crimson_red

# Light themes
python3 -m git_manager theme set pure_white
python3 -m git_manager theme set light_blue
```

### 4. Verify Your Theme
```bash
python3 -m git_manager theme current
```

### 5. Preview Before Setting
```bash
python3 -m git_manager theme preview emerald_green
```

## Popular Theme Combinations

### For Development (Reduced Eye Strain)
```bash
python3 -m git_manager theme set jet_black
```

### For Presentations
```bash
python3 -m git_manager theme set royal_blue
```

### For Accessibility
```bash
python3 -m git_manager theme set pure_white
```

### For Fun/Branding
```bash
python3 -m git_manager theme set emerald_green
python3 -m git_manager theme set sunset_orange
python3 -m git_manager theme set purple_orchid
```

## All Available Themes

### Light Themes (9)
- pure_white
- soft_gray
- silver
- ivory
- warm_beige
- cream
- light_blue
- light_mint
- soft_yellow

### Dark Themes (7)
- jet_black ⭐ (default)
- graphite
- charcoal
- dark_navy
- deep_purple
- forest_green
- coffee_brown

### Colored Themes (12)
- royal_blue
- electric_blue
- teal
- emerald_green
- leaf_green
- sunset_orange
- amber
- crimson_red
- burgundy
- purple_orchid
- magenta
- rose_pink

## Theme Persistence

Your theme choice is saved automatically and will be used every time you start the application.

Location: `~/.config/git-manager/theme.json`

## Troubleshooting

**Theme not applying?**
```bash
# Check current theme
python3 -m git_manager theme current

# Reset to default
python3 -m git_manager theme set jet_black
```

**Colors look wrong?**
- Ensure your terminal supports 256 colors or true color
- Try a different theme to test
- Check your terminal color settings

## Next Steps

- Use `python3 -m git_manager --cli` to start interactive mode with your theme
- Create custom themes by editing `color_schemes.py`
- Share your favorite theme combinations!

## Need Help?

For detailed documentation, see: `docs/COLOR_SCHEMES.md`
