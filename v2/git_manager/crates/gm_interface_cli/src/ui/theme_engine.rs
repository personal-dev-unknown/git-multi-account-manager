// crates/gm_interface_cli/src/ui/theme_engine.rs
//
// Colour-scheme engine: definitions, registry, persistence, and lookup.
//
// Ported from V1's 24+ colour schemes (color_schemes.py) with the same
// names and colour palettes, using owo-colors AnsiColors for runtime use.
// Theme persistence stores only the slug in ~/.config/git-manager/theme.json.

use owo_colors::AnsiColors;
use serde::{Deserialize, Serialize};

// ── Colour palette type ───────────────────────────────────────────────────────

/// The 8 semantic colour slots that every theme defines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColors {
    pub primary:   AnsiColors,
    pub secondary: AnsiColors,
    pub accent:    AnsiColors,
    pub success:   AnsiColors,
    pub error:     AnsiColors,
    pub warning:   AnsiColors,
    pub info:      AnsiColors,
    pub banner:    AnsiColors,
}

// ── Theme metadata ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeType {
    Light,
    Dark,
    Colored,
}

impl std::fmt::Display for ThemeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Light   => write!(f, "light"),
            Self::Dark    => write!(f, "dark"),
            Self::Colored => write!(f, "colored"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeDefinition {
    pub name:        String,
    pub slug:        String,
    pub theme_type:  ThemeType,
    pub description: String,
    pub colors:      ThemeColors,
}

// ── Colour palette default ─────────────────────────────────────────────────

impl ThemeColors {
    /// The default "Zyrix Green" palette.
    pub const fn zyrix() -> Self {
        Self {
            primary:   AnsiColors::BrightGreen,
            secondary: AnsiColors::Green,
            accent:    AnsiColors::Cyan,
            success:   AnsiColors::BrightGreen,
            error:     AnsiColors::BrightRed,
            warning:   AnsiColors::BrightYellow,
            info:      AnsiColors::Cyan,
            banner:    AnsiColors::BrightGreen,
        }
    }
}

// ── Persisted state ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThemeState {
    current_theme_slug: String,
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// Holds all built-in themes and the currently-selected one.
pub struct ThemeRegistry {
    themes:  Vec<ThemeDefinition>,
    current: usize,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let themes = Self::builtin_themes();
        Self { current: 0, themes }
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    pub fn all(&self) -> &[ThemeDefinition] { &self.themes }

    pub fn current(&self) -> &ThemeDefinition { &self.themes[self.current] }

    pub fn current_colors(&self) -> ThemeColors { self.current().colors }

    pub fn slug_index(&self, slug: &str) -> Option<usize> {
        self.themes.iter().position(|t| t.slug == slug)
    }

    pub fn set_by_slug(&mut self, slug: &str) -> Option<usize> {
        let idx = self.slug_index(slug)?;
        self.current = idx;
        Some(idx)
    }

    // ── Persistence ──────────────────────────────────────────────────────────

    fn state_path() -> std::path::PathBuf {
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .ok()
            .or_else(|| {
                std::env::var("HOME").ok()
                    .map(|h| std::path::PathBuf::from(h).join(".config"))
            })
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        base.join("git-zyrix").join("theme.json")
    }

    /// Load persisted theme selection. Silently ignores missing/corrupt files.
    pub fn load_state(&mut self) {
        let path = Self::state_path();
        if !path.exists() { return; }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let state: ThemeState = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => return,
        };
        if let Some(idx) = self.slug_index(&state.current_theme_slug) {
            self.current = idx;
        }
    }

    /// Persist the current theme selection. Silently ignores I/O errors.
    pub fn save_state(&self) {
        let path = Self::state_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let state = ThemeState {
            current_theme_slug: self.current().slug.clone(),
        };
        if let Ok(content) = serde_json::to_string_pretty(&state) {
            let _ = std::fs::write(&path, content);
        }
    }

    // ── Built-in themes (ported from V1 color_schemes.py) ────────────────────

    fn builtin_themes() -> Vec<ThemeDefinition> {
        macro_rules! theme {
            ($name:expr, $slug:expr, $type:ident, $desc:expr,
             $p:ident, $s:ident, $a:ident, $ok:ident, $err:ident, $warn:ident, $i:ident, $b:ident) => {
                ThemeDefinition {
                    name:        $name.to_string(),
                    slug:        $slug.to_string(),
                    theme_type:  ThemeType::$type,
                    description: $desc.to_string(),
                    colors: ThemeColors {
                        primary:   AnsiColors::$p,
                        secondary: AnsiColors::$s,
                        accent:    AnsiColors::$a,
                        success:   AnsiColors::$ok,
                        error:     AnsiColors::$err,
                        warning:   AnsiColors::$warn,
                        info:      AnsiColors::$i,
                        banner:    AnsiColors::$b,
                    },
                }
            };
        }

        vec![
            // ── Dark themes ──────────────────────────────────────────────────
            theme!("Zyrix Green",   "zyrix",       Dark, "Default – bright green on black",
                   BrightGreen, Green, Cyan, BrightGreen, BrightRed, BrightYellow, Cyan, BrightGreen),
            theme!("Jet Black",     "jet_black",   Dark, "High-contrast with cyan accents",
                   BrightCyan, Cyan, BrightBlue, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightCyan),
            theme!("Graphite",      "graphite",    Dark, "Soft dark with muted grey",
                   BrightCyan, Cyan, BrightBlue, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightCyan),
            theme!("Charcoal",      "charcoal",    Dark, "Deep charcoal with bright accents",
                   BrightCyan, Cyan, BrightBlue, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightWhite),
            theme!("Dark Navy",     "dark_navy",   Dark, "Navy blue with cyan highlights",
                   BrightCyan, BrightBlue, BrightBlue, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightCyan),
            theme!("Deep Purple",   "deep_purple", Dark, "Purple-tinted dark theme",
                   BrightMagenta, Magenta, BrightMagenta, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightMagenta),
            theme!("Forest Green",  "forest_green",Dark, "Green-tinted dark theme",
                   BrightGreen, Green, BrightGreen, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightGreen),
            theme!("Coffee Brown",  "coffee_brown",Dark, "Warm brown-tinted dark theme",
                   BrightYellow, Yellow, BrightYellow, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightYellow),
            // ── Light themes ─────────────────────────────────────────────────
            theme!("Pure White",    "pure_white",  Light, "Clean white-background light theme",
                   Blue, Cyan, BrightBlue, Green, Red, Yellow, Cyan, Blue),
            theme!("Soft Gray",     "soft_gray",   Light, "Gentle gray-background light theme",
                   Blue, Cyan, BrightBlue, Green, Red, Yellow, Cyan, BrightBlue),
            theme!("Silver",        "silver",      Light, "Silver-gray light theme",
                   BrightBlue, Cyan, BrightCyan, Green, Red, Yellow, BrightCyan, BrightWhite),
            theme!("Ivory",         "ivory",       Light, "Warm ivory light theme",
                   Blue, Cyan, BrightBlue, Green, Red, Yellow, Cyan, Blue),
            theme!("Warm Beige",    "warm_beige",  Light, "Warm beige light theme",
                   Blue, Cyan, BrightBlue, Green, Red, Yellow, Cyan, Blue),
            theme!("Cream",         "cream",       Light, "Cream-coloured light theme",
                   Blue, Cyan, BrightBlue, Green, Red, Yellow, Cyan, Blue),
            theme!("Light Blue",    "light_blue",  Light, "Blue-tinted light theme",
                   BrightBlue, Cyan, BrightCyan, Green, Red, Yellow, BrightCyan, BrightBlue),
            theme!("Light Mint",    "light_mint",  Light, "Mint-green light theme",
                   Cyan, BrightCyan, BrightGreen, Green, Red, Yellow, BrightCyan, BrightGreen),
            theme!("Soft Yellow",   "soft_yellow", Light, "Yellow-tinted light theme",
                   Blue, Cyan, BrightYellow, Green, Red, Yellow, Cyan, BrightYellow),
            // ── Colored themes ───────────────────────────────────────────────
            theme!("Royal Blue",    "royal_blue",      Colored, "Vibrant blue",
                   BrightBlue, Blue, BrightCyan, BrightGreen, BrightRed, BrightYellow, BrightBlue, BrightBlue),
            theme!("Electric Blue", "electric_blue",   Colored, "Bright electric blue",
                   BrightBlue, BrightCyan, BrightCyan, BrightGreen, BrightRed, BrightYellow, BrightBlue, BrightBlue),
            theme!("Teal",          "teal",            Colored, "Teal-tinted",
                   BrightCyan, Cyan, BrightGreen, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightCyan),
            theme!("Emerald Green", "emerald_green",   Colored, "Vibrant green",
                   BrightGreen, Green, BrightCyan, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightGreen),
            theme!("Leaf Green",    "leaf_green",      Colored, "Natural green leaf",
                   Green, BrightGreen, BrightCyan, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightGreen),
            theme!("Sunset Orange", "sunset_orange",   Colored, "Warm orange sunset",
                   BrightYellow, Yellow, BrightRed, BrightGreen, BrightRed, BrightYellow, BrightYellow, BrightYellow),
            theme!("Amber",         "amber",           Colored, "Warm amber",
                   BrightYellow, Yellow, BrightYellow, BrightGreen, BrightRed, BrightYellow, BrightYellow, BrightYellow),
            theme!("Crimson Red",   "crimson_red",     Colored, "Bold red",
                   BrightRed, Red, BrightYellow, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightRed),
            theme!("Burgundy",      "burgundy",        Colored, "Deep wine-red",
                   Red, BrightRed, BrightYellow, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightRed),
            theme!("Purple Orchid", "purple_orchid",   Colored, "Vibrant purple orchid",
                   BrightMagenta, Magenta, BrightMagenta, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightMagenta),
            theme!("Magenta",       "magenta",         Colored, "Bright magenta",
                   BrightMagenta, Magenta, BrightMagenta, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightMagenta),
            theme!("Rose Pink",     "rose_pink",       Colored, "Soft rose pink",
                   BrightMagenta, Magenta, BrightMagenta, BrightGreen, BrightRed, BrightYellow, BrightCyan, BrightMagenta),
        ]
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self { Self::new() }
}
