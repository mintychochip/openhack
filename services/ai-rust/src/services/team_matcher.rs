use crate::config::Config;
use crate::errors::AiError;
use crate::services::llm::{self, LlmMessage};
use serde::{Deserialize, Serialize};

/// Request body for team matching.
///
/// # Expected Behavior
///
/// Contains the user's ID, skills, what they're looking for in teammates,
/// preferred team size, and an optional project idea.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct TeamMatchRequest {
    pub _user_id: uuid::Uuid,
    pub user_skills: Vec<String>,
    pub looking_for: Vec<String>,
    pub preferred_team_size: Option<u32>,
    pub project_idea: Option<String>,
}

/// A single teammate match suggestion from the LLM.
///
/// # Expected Behavior
///
/// Contains a suggested user profile with skills, compatibility score (0-1),
/// and a reason explaining why they're a good match.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMatchResult {
    pub name: String,
    pub skills: Vec<String>,
    pub compatibility_score: f64,
    pub reason: String,
}

/// Response body for the team matcher endpoint.
///
/// # Expected Behavior
///
/// Contains a list of suggested teammate matches. May be empty if the LLM
/// fails to produce structured output.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct TeamMatchResponse {
    pub matches: Vec<TeamMatchResult>,
}

/// Generate teammate match suggestions using the LLM.
///
/// # Expected Behavior
///
/// Builds a prompt from the request parameters describing the user's skills,
/// what they're looking for, preferred team size, and project idea. Sends it
/// to the configured LLM provider. The system prompt instructs the LLM to
/// return a JSON array of match suggestions. The response is parsed as JSON;
/// if parsing fails, returns an empty matches list.
///
/// # Errors
///
/// - `AiError::Llm` if the LLM API call fails.
/// - `AiError::Http` if the HTTP request fails.
///
/// Returns an empty matches list (not an error) if the LLM response cannot
/// be parsed as JSON.
///
/// # Side Effects
///
/// - Calls the configured LLM provider API (network I/O).
/// - Logs at WARN level if the LLM response cannot be parsed as JSON.
pub async fn match_teammates(
    config: &Config,
    http_client: &reqwest::Client,
    req: &TeamMatchRequest,
) -> Result<TeamMatchResponse, AiError> {
    let team_size_str = req
        .preferred_team_size
        .map_or_else(|| "flexible".to_string(), |s| s.to_string());
    let project_idea_str = req
        .project_idea
        .as_deref()
        .unwrap_or("No specific idea yet");

    let user_prompt = format!(
        "User skills: {}\nLooking for: {}\nPreferred team size: {}\nProject idea: {}",
        req.user_skills.join(", "),
        req.looking_for.join(", "),
        team_size_str,
        project_idea_str
    );

    let messages = vec![
        LlmMessage {
            role: "system".to_string(),
            content: "You are a team matching assistant for a hackathon platform. Suggest ideal teammate profiles based on the user's skills and what they're looking for. Return a JSON array of objects, each with: name (string), skills (array of strings), compatibility_score (float 0-1), reason (string explaining why they're a good match). Return ONLY the JSON array, no other text.".to_string(),
        },
        LlmMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    let response_text = llm::llm_chat(config, http_client, messages).await?;

    let matches: Vec<TeamMatchResult> = if let Ok(parsed) = serde_json::from_str(&response_text) {
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
                log::warn!("Failed to parse LLM team match response as JSON: {e}");
                Vec::new()
            }
        }
    };

    Ok(TeamMatchResponse { matches })
}
