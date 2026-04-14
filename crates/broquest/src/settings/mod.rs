mod view;

pub use view::SettingsView;

use crate::app_settings::AppSettings;
use gpui::{App, SharedString};
use gpui_component::Theme;
use serde::{Deserialize, Serialize};

/// Apply user font settings on top of the current theme.
pub fn apply_font_settings(cx: &mut App) {
    let appearance = AppSettings::global(cx).settings.appearance.clone();
    if !appearance.font_family.is_empty() {
        Theme::global_mut(cx).font_family = SharedString::from(appearance.font_family);
    }
    if !appearance.mono_font_family.is_empty() {
        Theme::global_mut(cx).mono_font_family = SharedString::from(appearance.mono_font_family);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub general: GeneralSettings,
    pub connection: ConnectionSettings,
    pub editor: EditorSettings,
    pub appearance: AppearanceSettings,
}

impl Settings {
    /// Load settings from key-value pairs
    pub fn from_key_values(values: &[(String, String)]) -> Self {
        let mut settings = Settings::default();

        for (key, value) in values {
            match key.as_str() {
                "general.check_for_updates" => {
                    settings.general.check_for_updates = value.parse().unwrap_or(true);
                }
                "connection.request_timeout_seconds" => {
                    settings.connection.request_timeout_seconds = value.parse().unwrap_or(300);
                }
                "editor.show_whitespace" => {
                    settings.editor.show_whitespace = value.parse().unwrap_or(false);
                }
                "editor.soft_wrap" => {
                    settings.editor.soft_wrap = value.parse().unwrap_or(false);
                }
                "editor.folding" => {
                    settings.editor.folding = value.parse().unwrap_or(false);
                }
                "editor.layout" => {
                    settings.editor.layout = EditorLayout::from_str(value);
                }
                "appearance.theme" => {
                    settings.appearance.theme = value.clone();
                }
                "appearance.font_family" => {
                    settings.appearance.font_family = value.clone();
                }
                "appearance.mono_font_family" => {
                    settings.appearance.mono_font_family = value.clone();
                }
                _ => {}
            }
        }

        settings
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub check_for_updates: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            check_for_updates: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSettings {
    pub request_timeout_seconds: u32,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            request_timeout_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorSettings {
    pub show_whitespace: bool,
    pub soft_wrap: bool,
    pub folding: bool,
    pub layout: EditorLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EditorLayout {
    #[default]
    Vertical,
    Horizontal,
}

impl EditorLayout {
    pub fn as_str(&self) -> &'static str {
        match self {
            EditorLayout::Vertical => "vertical",
            EditorLayout::Horizontal => "horizontal",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "horizontal" => EditorLayout::Horizontal,
            _ => EditorLayout::Vertical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub theme: String,
    pub font_family: String,
    pub mono_font_family: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "Catppuccin Macchiato".to_string(),
            font_family: String::new(),
            mono_font_family: String::new(),
        }
    }
}
