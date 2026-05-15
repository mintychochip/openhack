use crate::config::Config;
use crate::errors::AiError;
use crate::models::ideas::{GeneratedIdea, IdeaRequest, IdeaResponse};
use crate::services::llm::{self, LlmMessage};

/// Generate hackathon project ideas using the LLM.
///
/// # Expected Behavior
///
/// Builds a detailed prompt from the request parameters (theme, interests,
/// team size, difficulty, number of ideas) and sends it to the configured
/// LLM provider. The system prompt instructs the LLM to return a JSON array
/// of ideas. The response is parsed as JSON; if parsing fails, returns an
/// empty ideas list. Each idea includes title, description, technologies,
/// difficulty, and `feasibility_score`.
///
/// # Errors
///
/// - `AiError::Llm` if the LLM API call fails.
/// - `AiError::Http` if the HTTP request fails.
///
/// Returns an empty ideas list (not an error) if the LLM response cannot
/// be parsed as JSON.
///
/// # Side Effects
///
/// - Calls the configured LLM provider API (network I/O).
/// - Logs at WARN level if the LLM response cannot be parsed as JSON.
pub async fn generate_ideas(
    config: &Config,
    http_client: &reqwest::Client,
    req: &IdeaRequest,
) -> Result<IdeaResponse, AiError> {
    let num_ideas = req.num_ideas.unwrap_or(3);
    let team_size_str = req
        .team_size
        .map_or_else(|| "flexible".to_string(), |s| s.to_string());
    let difficulty_str = req.difficulty.as_deref().unwrap_or("intermediate");

    let user_prompt = format!(
        "Theme: {}\nInterests: {}\nTeam size: {}\nDifficulty: {}\nNumber of ideas: {}",
        req.theme,
        req.interests.join(", "),
        team_size_str,
        difficulty_str,
        num_ideas
    );

    let messages = vec![
        LlmMessage {
            role: "system".to_string(),
            content: "You are a creative hackathon idea generator. Generate innovative hackathon project ideas based on the given theme and interests. Return a JSON array of objects, each with: title (string), description (string), technologies (array of strings), difficulty (string: beginner/intermediate/advanced), feasibility_score (float 0-1). Return ONLY the JSON array, no other text.".to_string(),
        },
        LlmMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    let response_text = llm::llm_chat(config, http_client, messages).await?;

    let ideas: Vec<GeneratedIdea> = if let Ok(parsed) = serde_json::from_str(&response_text) {
        parsed
    } else {
        let cleaned = response_text
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        match serde_json::from_str(cleaned) {
            Ok(parsed) => parsed,
            Err(e) => {
                log::warn!("Failed to parse LLM idea response as JSON: {e}");
                Vec::new()
            }
        }
    };

    Ok(IdeaResponse { ideas })
}
