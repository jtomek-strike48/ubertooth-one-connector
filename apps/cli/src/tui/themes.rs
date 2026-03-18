//! Theme system for TUI application
//!
//! Provides built-in themes and support for custom user themes.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name
    pub name: String,

    /// Theme description
    pub description: String,

    /// Color palette
    pub colors: ColorPalette,
}

/// Color palette for theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    // UI Elements
    pub background: ColorDef,
    pub foreground: ColorDef,
    pub border: ColorDef,
    pub border_focused: ColorDef,
    pub title: ColorDef,

    // Status colors
    pub success: ColorDef,
    pub error: ColorDef,
    pub warning: ColorDef,
    pub info: ColorDef,

    // Interactive elements
    pub selected: ColorDef,
    pub selected_bg: ColorDef,
    pub highlight: ColorDef,
    pub dimmed: ColorDef,

    // Syntax/data colors
    pub primary: ColorDef,
    pub secondary: ColorDef,
    pub accent: ColorDef,

    // Specific elements
    pub device_connected: ColorDef,
    pub device_disconnected: ColorDef,
    pub packet_header: ColorDef,
    pub packet_data: ColorDef,
    pub shortcut: ColorDef,
}

/// Color definition (can be named color or RGB)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorDef {
    /// Named color (e.g., "red", "blue")
    Named(String),
    /// RGB color
    Rgb { r: u8, g: u8, b: u8 },
}

impl ColorDef {
    /// Convert to ratatui Color
    pub fn to_color(&self) -> Color {
        match self {
            ColorDef::Named(name) => match name.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "gray" | "grey" => Color::Gray,
                "darkgray" | "darkgrey" => Color::DarkGray,
                "lightred" => Color::LightRed,
                "lightgreen" => Color::LightGreen,
                "lightyellow" => Color::LightYellow,
                "lightblue" => Color::LightBlue,
                "lightmagenta" => Color::LightMagenta,
                "lightcyan" => Color::LightCyan,
                "white" => Color::White,
                _ => Color::White, // Default fallback
            },
            ColorDef::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
        }
    }

    /// Create from named color
    pub fn named(name: &str) -> Self {
        ColorDef::Named(name.to_string())
    }

    /// Create from RGB values
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        ColorDef::Rgb { r, g, b }
    }
}

impl Theme {
    /// Get built-in dark theme (default)
    pub fn dark() -> Self {
        Theme {
            name: "Dark".to_string(),
            description: "Classic dark theme with cyan accents".to_string(),
            colors: ColorPalette {
                background: ColorDef::named("black"),
                foreground: ColorDef::named("white"),
                border: ColorDef::named("white"),
                border_focused: ColorDef::named("cyan"),
                title: ColorDef::named("cyan"),

                success: ColorDef::named("green"),
                error: ColorDef::named("red"),
                warning: ColorDef::named("yellow"),
                info: ColorDef::named("blue"),

                selected: ColorDef::named("black"),
                selected_bg: ColorDef::named("cyan"),
                highlight: ColorDef::named("cyan"),
                dimmed: ColorDef::named("darkgray"),

                primary: ColorDef::named("cyan"),
                secondary: ColorDef::named("magenta"),
                accent: ColorDef::named("yellow"),

                device_connected: ColorDef::named("green"),
                device_disconnected: ColorDef::named("red"),
                packet_header: ColorDef::named("cyan"),
                packet_data: ColorDef::named("white"),
                shortcut: ColorDef::named("green"),
            },
        }
    }

    /// Get built-in light theme
    pub fn light() -> Self {
        Theme {
            name: "Light".to_string(),
            description: "Light theme for bright environments".to_string(),
            colors: ColorPalette {
                background: ColorDef::named("white"),
                foreground: ColorDef::named("black"),
                border: ColorDef::named("black"),
                border_focused: ColorDef::named("blue"),
                title: ColorDef::named("blue"),

                success: ColorDef::named("green"),
                error: ColorDef::named("red"),
                warning: ColorDef::rgb(184, 134, 11), // Dark orange
                info: ColorDef::named("blue"),

                selected: ColorDef::named("white"),
                selected_bg: ColorDef::named("blue"),
                highlight: ColorDef::named("blue"),
                dimmed: ColorDef::named("gray"),

                primary: ColorDef::named("blue"),
                secondary: ColorDef::named("magenta"),
                accent: ColorDef::rgb(184, 134, 11), // Dark orange

                device_connected: ColorDef::named("green"),
                device_disconnected: ColorDef::named("red"),
                packet_header: ColorDef::named("blue"),
                packet_data: ColorDef::named("black"),
                shortcut: ColorDef::named("green"),
            },
        }
    }

    /// Get built-in cyberpunk theme
    pub fn cyberpunk() -> Self {
        Theme {
            name: "Cyberpunk".to_string(),
            description: "Neon colors inspired by cyberpunk aesthetics".to_string(),
            colors: ColorPalette {
                background: ColorDef::rgb(10, 0, 20), // Very dark purple
                foreground: ColorDef::rgb(0, 255, 255), // Bright cyan
                border: ColorDef::rgb(255, 0, 255), // Magenta
                border_focused: ColorDef::rgb(0, 255, 255), // Bright cyan
                title: ColorDef::rgb(255, 0, 255), // Magenta

                success: ColorDef::rgb(0, 255, 128), // Neon green
                error: ColorDef::rgb(255, 0, 128), // Hot pink
                warning: ColorDef::rgb(255, 255, 0), // Bright yellow
                info: ColorDef::rgb(0, 128, 255), // Electric blue

                selected: ColorDef::rgb(0, 0, 0),
                selected_bg: ColorDef::rgb(255, 0, 255), // Magenta
                highlight: ColorDef::rgb(0, 255, 255), // Bright cyan
                dimmed: ColorDef::rgb(80, 0, 80), // Dark purple

                primary: ColorDef::rgb(255, 0, 255), // Magenta
                secondary: ColorDef::rgb(0, 255, 255), // Cyan
                accent: ColorDef::rgb(255, 255, 0), // Yellow

                device_connected: ColorDef::rgb(0, 255, 128), // Neon green
                device_disconnected: ColorDef::rgb(255, 0, 128), // Hot pink
                packet_header: ColorDef::rgb(255, 0, 255), // Magenta
                packet_data: ColorDef::rgb(0, 255, 255), // Cyan
                shortcut: ColorDef::rgb(0, 255, 128), // Neon green
            },
        }
    }

    /// Get built-in solarized dark theme
    pub fn solarized_dark() -> Self {
        Theme {
            name: "Solarized Dark".to_string(),
            description: "Solarized dark color scheme".to_string(),
            colors: ColorPalette {
                background: ColorDef::rgb(0, 43, 54), // base03
                foreground: ColorDef::rgb(131, 148, 150), // base0
                border: ColorDef::rgb(88, 110, 117), // base01
                border_focused: ColorDef::rgb(38, 139, 210), // blue
                title: ColorDef::rgb(38, 139, 210), // blue

                success: ColorDef::rgb(133, 153, 0), // green
                error: ColorDef::rgb(220, 50, 47), // red
                warning: ColorDef::rgb(181, 137, 0), // yellow
                info: ColorDef::rgb(38, 139, 210), // blue

                selected: ColorDef::rgb(0, 43, 54), // base03
                selected_bg: ColorDef::rgb(38, 139, 210), // blue
                highlight: ColorDef::rgb(42, 161, 152), // cyan
                dimmed: ColorDef::rgb(88, 110, 117), // base01

                primary: ColorDef::rgb(38, 139, 210), // blue
                secondary: ColorDef::rgb(211, 54, 130), // magenta
                accent: ColorDef::rgb(203, 75, 22), // orange

                device_connected: ColorDef::rgb(133, 153, 0), // green
                device_disconnected: ColorDef::rgb(220, 50, 47), // red
                packet_header: ColorDef::rgb(42, 161, 152), // cyan
                packet_data: ColorDef::rgb(131, 148, 150), // base0
                shortcut: ColorDef::rgb(133, 153, 0), // green
            },
        }
    }

    /// Get built-in matrix theme
    pub fn matrix() -> Self {
        Theme {
            name: "Matrix".to_string(),
            description: "Green on black inspired by The Matrix".to_string(),
            colors: ColorPalette {
                background: ColorDef::named("black"),
                foreground: ColorDef::rgb(0, 255, 0), // Bright green
                border: ColorDef::rgb(0, 200, 0), // Green
                border_focused: ColorDef::rgb(0, 255, 0), // Bright green
                title: ColorDef::rgb(0, 255, 0), // Bright green

                success: ColorDef::rgb(0, 255, 0), // Bright green
                error: ColorDef::rgb(255, 0, 0), // Red
                warning: ColorDef::rgb(255, 255, 0), // Yellow
                info: ColorDef::rgb(0, 200, 0), // Green

                selected: ColorDef::named("black"),
                selected_bg: ColorDef::rgb(0, 255, 0), // Bright green
                highlight: ColorDef::rgb(0, 255, 128), // Light green
                dimmed: ColorDef::rgb(0, 100, 0), // Dark green

                primary: ColorDef::rgb(0, 255, 0), // Bright green
                secondary: ColorDef::rgb(0, 200, 0), // Green
                accent: ColorDef::rgb(0, 255, 128), // Light green

                device_connected: ColorDef::rgb(0, 255, 0), // Bright green
                device_disconnected: ColorDef::rgb(255, 0, 0), // Red
                packet_header: ColorDef::rgb(0, 255, 128), // Light green
                packet_data: ColorDef::rgb(0, 255, 0), // Bright green
                shortcut: ColorDef::rgb(0, 255, 0), // Bright green
            },
        }
    }

    /// Get all built-in themes
    pub fn all_built_in() -> Vec<Theme> {
        vec![
            Self::dark(),
            Self::light(),
            Self::cyberpunk(),
            Self::solarized_dark(),
            Self::matrix(),
        ]
    }

    /// Load theme from file
    pub fn load_from_file(path: PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let theme: Theme = toml::from_str(&content)?;
        Ok(theme)
    }

    /// Save theme to file
    pub fn save_to_file(&self, path: PathBuf) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get theme config directory
    pub fn config_dir() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let config_dir = home.join(".config").join("ubertooth");
        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

    /// Get path to current theme file
    pub fn current_theme_path() -> anyhow::Result<PathBuf> {
        Ok(Self::config_dir()?.join("theme.toml"))
    }

    /// Load current theme (or default if not found)
    pub fn load_current() -> Self {
        Self::current_theme_path()
            .ok()
            .and_then(|path| {
                if path.exists() {
                    Self::load_from_file(path).ok()
                } else {
                    None
                }
            })
            .unwrap_or_else(Self::dark)
    }

    /// Save as current theme
    pub fn save_as_current(&self) -> anyhow::Result<()> {
        let path = Self::current_theme_path()?;
        self.save_to_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_themes_can_be_created() {
        let themes = Theme::all_built_in();
        assert_eq!(themes.len(), 5);
        assert_eq!(themes[0].name, "Dark");
        assert_eq!(themes[1].name, "Light");
        assert_eq!(themes[2].name, "Cyberpunk");
        assert_eq!(themes[3].name, "Solarized Dark");
        assert_eq!(themes[4].name, "Matrix");
    }

    #[test]
    fn test_color_def_to_color() {
        let red = ColorDef::named("red");
        assert_eq!(red.to_color(), Color::Red);

        let custom = ColorDef::rgb(255, 128, 0);
        assert_eq!(custom.to_color(), Color::Rgb(255, 128, 0));
    }
}
