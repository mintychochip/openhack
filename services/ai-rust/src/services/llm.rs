use crate::config::Config;
use crate::errors::AiError;

/// A single message in the LLM conversation format.
///
/// # Expected Behavior
///
/// Role must be "system", "user", or "assistant". Content is the message text.
/// This struct is provider-agnostic and converted to provider-specific formats
/// when calling the LLM API.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

/// Send a chat completion request to the configured LLM provider.
///
/// # Expected Behavior
///
/// Routes the request to `OpenAI` or Anthropic based on `config.llm_provider`.
/// For `OpenAI`, sends all messages including system messages to the
/// `/v1/chat/completions` endpoint. For Anthropic, extracts system messages
/// into the separate `system` field and sends non-system messages to the
/// `/v1/messages` endpoint with `anthropic-version: 2023-06-01`. Returns
/// the assistant's response text on success.
///
/// # Errors
///
/// - `AiError::Llm` if the LLM provider is not "openai" or "anthropic".
/// - `AiError::Llm` if the API response does not contain expected content fields.
/// - `AiError::Http` if the HTTP request fails or returns a non-success status.
///
/// # Side Effects
///
/// - Makes an HTTP POST request to the configured LLM provider API (network I/O).
/// - Logs at ERROR level on unexpected provider or missing response content.
pub async fn llm_chat(
    config: &Config,
    http_client: &reqwest::Client,
    messages: Vec<LlmMessage>,
) -> Result<String, AiError> {
    match config.llm_provider.as_str() {
        "openai" => call_openai(config, http_client, messages).await,
        "anthropic" => call_anthropic(config, http_client, messages).await,
        other => {
            let msg = format!("Unknown LLM provider: {other}");
            log::error!("{msg}");
            Err(AiError::Llm(msg))
        }
    }
}

/// Call an OpenAI-compatible Chat Completions API.
///
/// # Expected Behavior
///
/// Sends a POST request to `{openai_base_url}/chat/completions` with
/// the configured model and messages. The `openai_base_url` defaults to
/// `https://api.openai.com/v1` but can be overridden via `OPENAI_BASE_URL`
/// to point at Ollama, LM Studio, vLLM, or any other OpenAI-compatible
/// endpoint. The Authorization header is only sent if `openai_api_key` is
/// non-empty (some local endpoints don't require one). Parses the response
/// and extracts the assistant's message content from
/// `choices[0].message.content`.
///
/// # Errors
///
/// - `AiError::Llm` if the response does not contain the expected content field.
/// - `AiError::Http` if the HTTP request fails.
///
/// # Side Effects
///
/// - Makes an HTTP POST request to the configured OpenAI-compatible endpoint
///   (network I/O).
async fn call_openai(
    config: &Config,
    http_client: &reqwest::Client,
    messages: Vec<LlmMessage>,
) -> Result<String, AiError> {
    let api_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content
            })
        })
        .collect();

    let body = serde_json::json!({
        "model": config.openai_model,
        "messages": api_messages,
    });

    let url = format!("{}/chat/completions", config.openai_base_url.trim_end_matches('/'));

    let mut request = http_client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body);

    if !config.openai_api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", config.openai_api_key));
    }

    let response = request.send().await?;

    let resp_body: serde_json::Value = response.json().await?;

    resp_body["choices"][0]["message"]["content"]
        .as_str()
        .map(std::string::ToString::to_string)
        .ok_or_else(|| AiError::Llm("No content in OpenAI response".to_string()))
}

/// Call the Anthropic Messages API.
///
/// # Expected Behavior
///
/// Sends a POST request to `https://api.anthropic.com/v1/messages` with
/// the configured model. System messages are extracted and sent in the
/// separate `system` field per Anthropic's API convention. Non-system
/// messages are sent in the `messages` array. Parses the response and
/// extracts the assistant's message content from `content[0].text`.
///
/// # Errors
///
/// - `AiError::Llm` if the response does not contain the expected content field.
/// - `AiError::Http` if the HTTP request fails.
///
/// # Side Effects
///
/// - Makes an HTTP POST request to the Anthropic API (network I/O).
async fn call_anthropic(
    config: &Config,
    http_client: &reqwest::Client,
    messages: Vec<LlmMessage>,
) -> Result<String, AiError> {
    let system_message: Option<String> = messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone());

    let api_messages: Vec<serde_json::Value> = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content
            })
        })
        .collect();

    let mut body = serde_json::json!({
        "model": config.anthropic_model,
        "max_tokens": 4096,
        "messages": api_messages,
    });

    if let Some(sys) = system_message {
        body["system"] = serde_json::json!(sys);
    }

    let response = http_client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &config.anthropic_api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let resp_body: serde_json::Value = response.json().await?;

    resp_body["content"][0]["text"]
        .as_str()
        .map(std::string::ToString::to_string)
        .ok_or_else(|| AiError::Llm("No content in Anthropic response".to_string()))
}
