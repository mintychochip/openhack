use std::sync::OnceLock;

use super::app::{FontConfig, FontEntry};

pub const DAISYUI_PRESETS: &[&str] = &[
    "light", "dark", "cupcake", "bumblebee", "emerald", "corporate",
    "synthwave", "retro", "cyberpunk", "valentine", "halloween", "garden",
    "forest", "aqua", "lofi", "pastel", "fantasy", "wireframe", "black",
    "luxury", "dracula", "cmyk", "autumn", "business", "acid", "lemonade",
    "night", "coffee", "winter", "dim", "nord", "sunset", "custom",
];

pub struct FontPreset {
    pub name: &'static str,
    pub config: FontConfig,
}

fn build_font_presets() -> Vec<FontPreset> {
    vec![
        FontPreset {
            name: "Modern Editorial",
            config: FontConfig {
                display: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Playfair+Display:wght@400;700&display=swap".to_string(),
                    family: "'Playfair Display', serif".to_string(),
                },
                heading: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Poppins:wght@400;500;600;700&display=swap".to_string(),
                    family: "'Poppins', sans-serif".to_string(),
                },
                body: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap".to_string(),
                    family: "'Inter', sans-serif".to_string(),
                },
            },
        },
        FontPreset {
            name: "Clean Tech",
            config: FontConfig {
                display: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&display=swap".to_string(),
                    family: "'Space Grotesk', sans-serif".to_string(),
                },
                heading: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap".to_string(),
                    family: "'Inter', sans-serif".to_string(),
                },
                body: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap".to_string(),
                    family: "'Inter', sans-serif".to_string(),
                },
            },
        },
        FontPreset {
            name: "Academic Classic",
            config: FontConfig {
                display: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Crimson+Text:wght@400;600;700&display=swap".to_string(),
                    family: "'Crimson Text', serif".to_string(),
                },
                heading: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Libre+Baskerville:wght@400;700&display=swap".to_string(),
                    family: "'Libre Baskerville', serif".to_string(),
                },
                body: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Source+Sans+3:wght@400;500;600;700&display=swap".to_string(),
                    family: "'Source Sans 3', sans-serif".to_string(),
                },
            },
        },
        FontPreset {
            name: "Bold Startup",
            config: FontConfig {
                display: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Outfit:wght@400;500;600;700;800&display=swap".to_string(),
                    family: "'Outfit', sans-serif".to_string(),
                },
                heading: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap".to_string(),
                    family: "'Plus Jakarta Sans', sans-serif".to_string(),
                },
                body: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap".to_string(),
                    family: "'Inter', sans-serif".to_string(),
                },
            },
        },
        FontPreset {
            name: "Creative Studio",
            config: FontConfig {
                display: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Poppins:wght@400;500;600;700;800&display=swap".to_string(),
                    family: "'Poppins', sans-serif".to_string(),
                },
                heading: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Work+Sans:wght@400;500;600;700&display=swap".to_string(),
                    family: "'Work Sans', sans-serif".to_string(),
                },
                body: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap".to_string(),
                    family: "'Inter', sans-serif".to_string(),
                },
            },
        },
        FontPreset {
            name: "Corporate",
            config: FontConfig {
                display: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;700&display=swap".to_string(),
                    family: "'Roboto', sans-serif".to_string(),
                },
                heading: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;700&display=swap".to_string(),
                    family: "'Roboto', sans-serif".to_string(),
                },
                body: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Open+Sans:wght@400;500;600;700&display=swap".to_string(),
                    family: "'Open Sans', sans-serif".to_string(),
                },
            },
        },
        FontPreset {
            name: "Retro Terminal",
            config: FontConfig {
                display: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=VT323&display=swap".to_string(),
                    family: "'VT323', monospace".to_string(),
                },
                heading: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Space+Mono:wght@400;700&display=swap".to_string(),
                    family: "'Space Mono', monospace".to_string(),
                },
                body: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Space+Mono:wght@400;700&display=swap".to_string(),
                    family: "'Space Mono', monospace".to_string(),
                },
            },
        },
        FontPreset {
            name: "Friendly Community",
            config: FontConfig {
                display: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Fredoka:wght@400;500;600;700&display=swap".to_string(),
                    family: "'Fredoka', sans-serif".to_string(),
                },
                heading: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Nunito:wght@400;500;600;700;800&display=swap".to_string(),
                    family: "'Nunito', sans-serif".to_string(),
                },
                body: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Nunito:wght@400;500;600;700;800&display=swap".to_string(),
                    family: "'Nunito', sans-serif".to_string(),
                },
            },
        },
        FontPreset {
            name: "Luxury",
            config: FontConfig {
                display: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Cormorant+Garamond:wght@400;500;600;700&display=swap".to_string(),
                    family: "'Cormorant Garamond', serif".to_string(),
                },
                heading: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Montserrat:wght@400;500;600;700&display=swap".to_string(),
                    family: "'Montserrat', sans-serif".to_string(),
                },
                body: FontEntry {
                    url: "https://fonts.googleapis.com/css2?family=Montserrat:wght@400;500;600;700&display=swap".to_string(),
                    family: "'Montserrat', sans-serif".to_string(),
                },
            },
        },
    ]
}

static FONT_PRESETS_LOCK: OnceLock<Vec<FontPreset>> = OnceLock::new();

pub fn font_presets() -> &'static [FontPreset] {
    FONT_PRESETS_LOCK.get_or_init(build_font_presets)
}

pub fn find_font_preset(name: &str) -> Option<&'static FontPreset> {
    font_presets().iter().find(|p| p.name.eq_ignore_ascii_case(name))
}

pub fn font_preset_name(config: &FontConfig) -> &'static str {
    font_presets()
        .iter()
        .find(|p| p.config == *config)
        .map(|p| p.name)
        .unwrap_or("Custom")
}
