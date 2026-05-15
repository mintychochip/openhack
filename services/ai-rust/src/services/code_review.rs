use crate::config::Config;
use crate::errors::AiError;
use crate::services::llm::{self, LlmMessage};
use serde::{Deserialize, Serialize};

/// Request body for code review.
///
/// # Expected Behavior
///
/// `repo_url` and `commit_hash` identify the code to review. `focus_areas`
/// is an optional list of areas to focus on (e.g., "security", "performance").
/// `include_tests` indicates whether test coverage analysis should be included.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct CodeReviewRequest {
    pub repo_url: String,
    pub commit_hash: Option<String>,
    pub focus_areas: Option<Vec<String>>,
    pub include_tests: Option<bool>,
}

/// A single finding from the code review.
///
/// # Expected Behavior
///
/// Contains severity (critical/warning/info), the file path, optional line number,
/// description of the issue, and a suggested fix.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewFinding {
    pub severity: String,
    pub file: String,
    pub line: Option<i32>,
    pub description: String,
    pub suggestion: String,
}

/// Response body for the code review endpoint.
///
/// # Expected Behavior
///
/// Contains a summary, list of findings, and an overall score (0-100).
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewResponse {
    pub summary: String,
    pub findings: Vec<CodeReviewFinding>,
    pub overall_score: f64,
}

/// Review code using the LLM.
///
/// # Expected Behavior
///
/// Builds a prompt describing the repository URL, commit hash, focus areas,
/// and whether to include test analysis. Sends it to the configured LLM
/// provider. The system prompt instructs the LLM to return a JSON object
/// with summary, findings array, and `overall_score`. If the LLM cannot
/// access the repository directly, it provides general guidance based on
/// the URL and focus areas. The response is parsed as JSON; if parsing
/// fails, returns a default response with the raw LLM text as the summary.
///
/// # Errors
///
/// - `AiError::Llm` if the LLM API call fails.
/// - `AiError::Http` if the HTTP request fails.
///
/// # Side Effects
///
/// - Calls the configured LLM provider API (network I/O).
/// - Logs at WARN level if the LLM response cannot be parsed as JSON.
pub async fn review_code(
    config: &Config,
    http_client: &reqwest::Client,
    req: &CodeReviewRequest,
) -> Result<CodeReviewResponse, AiError> {
    let commit_str = req.commit_hash.as_deref().unwrap_or("latest");
    let focus_str = req
        .focus_areas
        .as_ref()
        .map_or_else(|| "general code quality".to_string(), |a| a.join(", "));
    let include_tests = req.include_tests.unwrap_or(false);

    let user_prompt = format!(
        "Repository: {}\nCommit: {}\nFocus areas: {}\nInclude test analysis: {}",
        req.repo_url, commit_str, focus_str, include_tests
    );

    let messages = vec![
        LlmMessage {
            role: "system".to_string(),
            content: "You are an expert code reviewer for hackathon projects. Review the code and provide findings. Return a JSON object with: summary (string), findings (array of objects with severity, file, line, description, suggestion), overall_score (float 0-100). Return ONLY the JSON object, no other text.".to_string(),
        },
        LlmMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    let response_text = llm::llm_chat(config, http_client, messages).await?;

    if let Ok(parsed) = serde_json::from_str(&response_text) {
        Ok(parsed)
    } else {
        let cleaned = response_text
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        match serde_json::from_str(cleaned) {
            Ok(parsed) => Ok(parsed),
            Err(e) => {
                log::warn!("Failed to parse LLM code review response as JSON: {e}");
                Ok(CodeReviewResponse {
                    summary: response_text,
                    findings: Vec::new(),
                    overall_score: 0.0,
                })
            }
        }
    }
}
