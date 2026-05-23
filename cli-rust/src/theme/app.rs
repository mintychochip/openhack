use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::presets;

/// Internal simplified theme configuration used by the TUI.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub tagline: String,
    pub logo_url: String,
    pub primary_color: String,
    pub daisyui_theme_preset: String,
    pub custom_css: String,
    pub font_config: FontConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FontConfig {
    pub display: FontEntry,
    pub heading: FontEntry,
    pub body: FontEntry,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FontEntry {
    pub url: String,
    pub family: String,
}

/// Backend GET /api/core/info response shape.
#[derive(Clone, Debug, Deserialize)]
pub struct ThemeConfigResponse {
    pub id: String,
    pub name: String,
    pub tagline: String,
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default)]
    pub custom_css: String,
    #[serde(default = "default_theme_preset")]
    pub daisyui_theme_preset: String,
    #[serde(default)]
    pub font_config: FontConfig,
}

fn default_theme_preset() -> String {
    "light".to_string()
}

impl From<ThemeConfigResponse> for ThemeConfig {
    fn from(r: ThemeConfigResponse) -> Self {
        Self {
            name: r.name,
            tagline: r.tagline,
            logo_url: r.logo_url.unwrap_or_default(),
            primary_color: String::new(),
            daisyui_theme_preset: r.daisyui_theme_preset,
            custom_css: r.custom_css,
            font_config: r.font_config,
        }
    }
}

/// Backend PUT /api/core/info request shape (all fields optional).
#[derive(Clone, Debug, Serialize)]
pub struct ThemeConfigUpdate {
    pub name: Option<String>,
    pub tagline: Option<String>,
    pub logo_url: Option<String>,
    pub daisyui_theme_preset: Option<String>,
    pub custom_css: Option<String>,
    pub font_config: Option<FontConfig>,
}

impl From<&ThemeConfig> for ThemeConfigUpdate {
    fn from(c: &ThemeConfig) -> Self {
        Self {
            name: Some(c.name.clone()),
            tagline: Some(c.tagline.clone()),
            logo_url: Some(c.logo_url.clone()),
            daisyui_theme_preset: Some(c.daisyui_theme_preset.clone()),
            custom_css: Some(c.custom_css.clone()),
            font_config: Some(c.font_config.clone()),
        }
    }
}

/// Active tab in the theme editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Branding,
    Colors,
    Typography,
    Preview,
    CustomCss,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Tab::Branding => "Branding",
            Tab::Colors => "Colors",
            Tab::Typography => "Typography",
            Tab::Preview => "Preview",
            Tab::CustomCss => "Custom CSS",
        }
    }

    pub fn all() -> &'static [Tab] {
        &[Tab::Branding, Tab::Colors, Tab::Typography, Tab::Preview, Tab::CustomCss]
    }
}

/// Application state for the theme editor TUI.
#[derive(Clone, Debug)]
pub struct ThemeApp {
    pub config: ThemeConfig,
    pub active_tab: Tab,
    pub selected_field: usize,
    pub editing: bool,
    pub input_buffer: String,
    pub status_message: Option<String>,
    pub status_time: Option<Instant>,
    pub api_url: String,
    pub api_connected: bool,
    pub auth_token: Option<String>,
}

impl ThemeApp {
    pub fn new() -> Self {
        Self {
            config: ThemeConfig::default(),
            active_tab: Tab::Branding,
            selected_field: 0,
            editing: false,
            input_buffer: String::new(),
            status_message: None,
            status_time: None,
            api_url: "http://localhost:8000".to_string(),
            api_connected: false,
            auth_token: std::env::var("OPENHACK_ADMIN_TOKEN").ok(),
        }
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_time = Some(Instant::now());
    }

    pub fn check_status_expiry(&mut self) {
        if let Some(t) = self.status_time {
            if t.elapsed() > Duration::from_secs(3) {
                self.status_message = None;
                self.status_time = None;
            }
        }
    }

    pub fn next_tab(&mut self) {
        let all = Tab::all();
        let idx = all.iter().position(|&t| t == self.active_tab).unwrap_or(0);
        self.active_tab = all[(idx + 1) % all.len()];
        self.selected_field = 0;
    }

    pub fn prev_tab(&mut self) {
        let all = Tab::all();
        let idx = all.iter().position(|&t| t == self.active_tab).unwrap_or(0);
        let new_idx = if idx == 0 { all.len() - 1 } else { idx - 1 };
        self.active_tab = all[new_idx];
        self.selected_field = 0;
    }

    pub fn field_count(&self) -> usize {
        match self.active_tab {
            Tab::Branding => 3,
            Tab::Colors => {
                if self.config.daisyui_theme_preset == "custom" {
                    1 + 1 // preset + primary_color placeholder
                } else {
                    1 // just preset
                }
            }
            Tab::Typography => {
                if self.font_preset_name() == "Custom" {
                    1 + 6 // preset + 3 url + 3 family
                } else {
                    1 // just preset
                }
            }
            Tab::Preview => 0,
            Tab::CustomCss => 1,
        }
    }

    pub fn next_field(&mut self) {
        let count = self.field_count();
        if count > 0 {
            self.selected_field = (self.selected_field + 1) % count;
        }
    }

    pub fn prev_field(&mut self) {
        let count = self.field_count();
        if count > 0 {
            self.selected_field = if self.selected_field == 0 {
                count - 1
            } else {
                self.selected_field - 1
            };
        }
    }

    /// Get the current value of the selected field as a string for editing.
    pub fn current_field_value(&self) -> String {
        match self.active_tab {
            Tab::Branding => match self.selected_field {
                0 => self.config.name.clone(),
                1 => self.config.tagline.clone(),
                2 => self.config.logo_url.clone(),
                _ => String::new(),
            },
            Tab::Colors => {
                if self.selected_field == 0 {
                    self.config.daisyui_theme_preset.clone()
                } else {
                    self.config.primary_color.clone()
                }
            }
            Tab::Typography => {
                if self.selected_field == 0 {
                    self.font_preset_name().to_string()
                } else {
                    let idx = self.selected_field - 1;
                    match idx {
                        0 => self.config.font_config.display.url.clone(),
                        1 => self.config.font_config.display.family.clone(),
                        2 => self.config.font_config.heading.url.clone(),
                        3 => self.config.font_config.heading.family.clone(),
                        4 => self.config.font_config.body.url.clone(),
                        5 => self.config.font_config.body.family.clone(),
                        _ => String::new(),
                    }
                }
            }
            Tab::CustomCss => self.config.custom_css.clone(),
            _ => String::new(),
        }
    }

    /// Apply the input buffer to the selected field.
    pub fn confirm_edit(&mut self) {
        let value = self.input_buffer.clone();
        match self.active_tab {
            Tab::Branding => match self.selected_field {
                0 => self.config.name = value,
                1 => self.config.tagline = value,
                2 => self.config.logo_url = value,
                _ => {}
            },
            Tab::Colors => {
                if self.selected_field == 0 {
                    let lower = value.to_lowercase();
                    if presets::DAISYUI_PRESETS.contains(&lower.as_str()) || lower == "custom" {
                        self.config.daisyui_theme_preset = lower.clone();
                        if lower != "custom" {
                            self.config.primary_color = String::new();
                        }
                    }
                } else {
                    self.config.primary_color = value;
                }
            }
            Tab::Typography => {
                if self.selected_field == 0 {
                    if let Some(preset) = presets::find_font_preset(&value) {
                        self.config.font_config = preset.config.clone();
                    }
                } else {
                    let idx = self.selected_field - 1;
                    match idx {
                        0 => self.config.font_config.display.url = value,
                        1 => self.config.font_config.display.family = value,
                        2 => self.config.font_config.heading.url = value,
                        3 => self.config.font_config.heading.family = value,
                        4 => self.config.font_config.body.url = value,
                        5 => self.config.font_config.body.family = value,
                        _ => {}
                    }
                }
            }
            Tab::CustomCss => self.config.custom_css = value,
            _ => {}
        }
    }

    /// Return the name of the currently selected font preset.
    pub fn font_preset_name(&self) -> &'static str {
        presets::font_preset_name(&self.config.font_config)
    }
}
