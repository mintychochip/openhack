use crate::config::Config;
use crate::errors::AiError;
use crate::models::brand::{
    BrandNormalizeRequest, BrandNormalizeResponse, FontConfigRecommendation,
    FontRecommendation, NormalizedExtractedData, SemanticColors,
};
use crate::services::llm::{self, LlmMessage};

/// Normalize scraped brand data into a DaisyUI-compatible theme using the LLM.
///
/// # Expected Behavior
///
/// Accepts raw extracted colors, fonts, and an optional vibe description.
/// Constructs a strict system prompt that instructs the LLM to return a
/// JSON object with `semantic_colors` (mapped to DaisyUI roles),
/// `suggested_preset` (one of 32 built-in DaisyUI presets), and
/// `font_config` (Google Fonts CDN URLs and families for display,
/// heading, and body). The response is parsed as JSON; if parsing fails,
/// returns an `AiError::Llm` with the raw response text included.
///
/// # Errors
///
/// - `AiError::Llm` if the LLM API call fails or the response cannot be
///   parsed into the expected JSON shape.
/// - `AiError::Http` if the HTTP request to the LLM provider fails.
///
/// # Side Effects
///
/// - Calls the configured LLM provider API (network I/O).
/// - Logs at WARN level if the LLM response cannot be parsed as JSON.
pub async fn normalize_brand(
    config: &Config,
    http_client: &reqwest::Client,
    req: &BrandNormalizeRequest,
) -> Result<BrandNormalizeResponse, AiError> {
    let daisyui_presets = [
        "light", "dark", "cupcake", "bumblebee", "emerald", "corporate",
        "synthwave", "retro", "cyberpunk", "valentine", "halloween",
        "garden", "forest", "aqua", "lofi", "pastel", "fantasy",
        "wireframe", "black", "luxury", "dracula", "cmyk", "autumn",
        "business", "acid", "lemonade", "night", "coffee", "winter",
        "dim", "nord", "sunset",
    ];

    let context_block = if let Some(ref ctx) = req.color_context {
        let dominant = ctx.dominant_colors.iter()
            .take(5)
            .map(|c| format!("{} ({} pixels)", c.hex, c.count))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Color context:\n- Hero/hero section colors: {:?}\n- CTA button colors: {:?}\n- Heading text colors: {:?}\n- Dominant screenshot colors: {}\n",
            ctx.hero_colors,
            ctx.cta_colors,
            ctx.heading_colors,
            dominant
        )
    } else {
        String::new()
    };

    let user_prompt = format!(
        "Extracted website colors: {:?}\nExtracted fonts: {:?}\nWebsite vibe: {}\n{}\nPlease normalize these into a DaisyUI theme.",
        req.raw_colors,
        req.raw_fonts,
        req.website_vibe.as_deref().unwrap_or("modern tech"),
        context_block
    );

    let system_prompt = format!(
        "You are a design-system assistant that maps scraped website brand data into structured DaisyUI themes.\n\nReturn STRICT JSON only — no markdown, no prose. The JSON must match this exact shape:\n\n{{\n  \"semantic_colors\": {{\n    \"primary\": \"#RRGGBB\",\n    \"secondary\": \"#RRGGBB\",\n    \"accent\": \"#RRGGBB\",\n    \"neutral\": \"#RRGGBB\",\n    \"base_100\": \"#RRGGBB\",\n    \"info\": \"#RRGGBB\",\n    \"success\": \"#RRGGBB\",\n    \"warning\": \"#RRGGBB\",\n    \"error\": \"#RRGGBB\"\n  }},\n  \"suggested_preset\": \"PRESET_NAME\",\n  \"font_config\": {{\n    \"display\": {{\"url\": \"https://fonts.googleapis.com/...\", \"family\": \"'Font Name', fallback\"}},\n    \"heading\": {{\"url\": \"https://fonts.googleapis.com/...\", \"family\": \"'Font Name', fallback\"}},\n    \"body\": {{\"url\": \"https://fonts.googleapis.com/...\", \"family\": \"'Font Name', fallback\"}}\n  }}\n}}\n\nColor mapping rules:\n- PRIMARY should be the main brand color (usually from CTA buttons or the hero section's dominant color).\n- SECONDARY should be a complementary color (often from headings or a secondary accent).\n- ACCENT should be a bright highlight color used sparingly (often from small UI elements or links).\n- NEUTRAL should be a gray or muted tone from body text or borders.\n- BASE_100 should be the main page background color (usually light; if the site has a dark theme, use a dark value like #1a1a1a).\n- INFO, SUCCESS, WARNING, ERROR should be standard semantic colors ONLY if the brand clearly uses custom ones; otherwise use null and let DaisyUI defaults handle them.\n\nRules:\n- Choose the closest DaisyUI preset from this list: {:?}\n- All color values must be 6-digit hex (e.g., #635BFF).\n- Do NOT use neon/bright colors unless the brand genuinely uses them (e.g., #00FFFF, #FF00FF are almost never correct).\n- Do NOT use pure white (#FFFFFF) or pure black (#000000) for primary/secondary/accent.\n- If a color looks like a default browser color (e.g., blue links, gray borders), ignore it.\n- Font URLs must be valid Google Fonts CSS2 URLs with wght axes and display=swap.\n- If a role cannot be confidently inferred, omit it (null) rather than guess.\n- The suggested_preset should be the single best match from the list above.",
        daisyui_presets
    );

    let messages = vec![
        LlmMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        LlmMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    let response_text = llm::llm_chat(config, http_client, messages).await?;

    let cleaned = response_text
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let parsed: serde_json::Value = match serde_json::from_str(cleaned) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to parse LLM brand normalization response as JSON: {e}. Raw: {response_text}");
            return Err(AiError::Llm(format!(
                "Invalid JSON from LLM: {e}. Raw: {response_text}"
            )));
        }
    };

    let semantic_colors = parse_semantic_colors(&parsed["semantic_colors"]);
    let suggested_preset = parsed["suggested_preset"]
        .as_str()
        .unwrap_or("light")
        .to_string();
    let font_config = parse_font_config(&parsed["font_config"]);

    Ok(BrandNormalizeResponse {
        semantic_colors,
        suggested_preset,
        font_config,
        extracted: NormalizedExtractedData {
            raw_colors: req.raw_colors.clone(),
            raw_fonts: req.raw_fonts.clone(),
            website_vibe: req.website_vibe.clone(),
        },
    })
}

fn parse_semantic_colors(value: &serde_json::Value) -> SemanticColors {
    SemanticColors {
        primary: value["primary"].as_str().map(String::from),
        secondary: value["secondary"].as_str().map(String::from),
        accent: value["accent"].as_str().map(String::from),
        neutral: value["neutral"].as_str().map(String::from),
        base_100: value["base_100"].as_str().map(String::from),
        info: value["info"].as_str().map(String::from),
        success: value["success"].as_str().map(String::from),
        warning: value["warning"].as_str().map(String::from),
        error: value["error"].as_str().map(String::from),
    }
}

fn parse_font_config(value: &serde_json::Value) -> FontConfigRecommendation {
    FontConfigRecommendation {
        display: parse_font_rec(&value["display"]),
        heading: parse_font_rec(&value["heading"]),
        body: parse_font_rec(&value["body"]),
    }
}

fn parse_font_rec(value: &serde_json::Value) -> Option<FontRecommendation> {
    let url = value["url"].as_str()?;
    let family = value["family"].as_str()?;
    Some(FontRecommendation {
        url: url.to_string(),
        family: family.to_string(),
    })
}
