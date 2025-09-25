use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeConfig {
    pub appearance: AppearanceConfig,
    pub editor: EditorConfig,
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Background transparency level (0.0 = fully transparent, 1.0 = fully opaque)
    pub background_opacity: f32,
    /// Menu bar transparency level
    pub menu_opacity: f32,
    /// Theme selection
    pub theme: Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Font size for code editor
    pub font_size: f32,
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Tab width
    pub tab_width: usize,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Default window width
    pub default_width: f32,
    /// Default window height
    pub default_height: f32,
    /// Remember window position
    pub remember_position: bool,
    /// Always on top
    pub always_on_top: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    System,
}

impl Default for IdeConfig {
    fn default() -> Self {
        Self {
            appearance: AppearanceConfig {
                background_opacity: 0.0, // Fully transparent by default
                menu_opacity: 0.8,       // Slightly visible menu
                theme: Theme::Dark,
            },
            editor: EditorConfig {
                font_size: 14.0,
                show_line_numbers: true,
                tab_width: 4,
                auto_save_interval: 0, // Disabled
            },
            window: WindowConfig {
                default_width: 1400.0,
                default_height: 900.0,
                remember_position: false,
                always_on_top: false,
            },
        }
    }
}

impl IdeConfig {
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        path.push("zellij-ide");
        fs::create_dir_all(&path)?;
        path.push("config.toml");
        Ok(path)
    }

    /// Load config from file, or create default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: IdeConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// Get background color with configured opacity
    pub fn background_color(&self) -> egui::Color32 {
        if self.appearance.background_opacity == 0.0 {
            egui::Color32::TRANSPARENT
        } else {
            let alpha = (self.appearance.background_opacity * 255.0) as u8;
            egui::Color32::from_rgba_premultiplied(20, 20, 20, alpha)
        }
    }

    /// Get menu background color with configured opacity
    pub fn menu_background_color(&self) -> egui::Color32 {
        if self.appearance.menu_opacity == 0.0 {
            egui::Color32::TRANSPARENT
        } else {
            let alpha = (self.appearance.menu_opacity * 255.0) as u8;
            egui::Color32::from_rgba_premultiplied(40, 40, 40, alpha)
        }
    }

    /// Update background opacity and save
    pub fn set_background_opacity(&mut self, opacity: f32) -> Result<()> {
        self.appearance.background_opacity = opacity.clamp(0.0, 1.0);
        self.save()?;
        Ok(())
    }

    /// Update menu opacity and save
    pub fn set_menu_opacity(&mut self, opacity: f32) -> Result<()> {
        self.appearance.menu_opacity = opacity.clamp(0.0, 1.0);
        self.save()?;
        Ok(())
    }
}