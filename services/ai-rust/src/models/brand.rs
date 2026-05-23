use serde::{Deserialize, Serialize};

/// Request body for the brand normalization endpoint.
///
/// # Expected Behavior
///
/// Contains raw scraped data from a target website: extracted CSS colors,
/// font-family stacks, and an optional website description/vibe. The AI
/// service normalizes this into a DaisyUI-compatible theme with semantic
/// color roles, a suggested built-in preset, and curated Google Font
/// recommendations.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ColorContext {
    pub hero_colors: Vec<String>,
    pub cta_colors: Vec<String>,
    pub heading_colors: Vec<String>,
    pub dominant_colors: Vec<DominantColor>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DominantColor {
    pub hex: String,
    pub count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrandNormalizeRequest {
    /// Raw hex colors extracted from the target website.
    pub raw_colors: Vec<String>,
    /// Font-family strings extracted from heading and body elements.
    pub raw_fonts: Vec<String>,
    /// Optional human-readable description of the website's visual vibe.
    pub website_vibe: Option<String>,
    /// Context about where colors were found on the page.
    pub color_context: Option<ColorContext>,
}

/// A normalized DaisyUI semantic color map.
///
/// # Expected Behavior
///
/// Each field is a hex color string mapped to the corresponding DaisyUI
/// semantic role. All fields are optional; the LLM may omit roles it cannot
/// confidently infer.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticColors {
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub accent: Option<String>,
    pub neutral: Option<String>,
    pub base_100: Option<String>,
    pub info: Option<String>,
    pub success: Option<String>,
    pub warning: Option<String>,
    pub error: Option<String>,
}

/// A font configuration recommendation.
///
/// # Expected Behavior
///
/// Contains Google Fonts CDN URL and CSS font-family string for display,
/// heading, and body text roles. All fields are optional.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontRecommendation {
    pub url: String,
    pub family: String,
}

/// Response body for the brand normalization endpoint.
///
/// # Expected Behavior
///
/// Returns the normalized semantic colors, the closest matching DaisyUI
/// preset name, and recommended font configurations. The `extracted`
/// block echoes the raw inputs for debugging.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct BrandNormalizeResponse {
    pub semantic_colors: SemanticColors,
    pub suggested_preset: String,
    pub font_config: FontConfigRecommendation,
    pub extracted: NormalizedExtractedData,
}

/// Font config recommendation wrapper.
///
/// # Expected Behavior
///
/// Maps display, heading, and body text to optional font recommendations.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfigRecommendation {
    pub display: Option<FontRecommendation>,
    pub heading: Option<FontRecommendation>,
    pub body: Option<FontRecommendation>,
}

/// Echo of the raw extracted data for transparency/debugging.
///
/// # Expected Behavior
///
/// Mirrors the inputs that were sent to the LLM so the caller can see
/// what was actually used for normalization.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct NormalizedExtractedData {
    pub raw_colors: Vec<String>,
    pub raw_fonts: Vec<String>,
    pub website_vibe: Option<String>,
}
