use std::time::Duration;

use reqwest::Client;

use super::app::{ThemeConfig, ThemeConfigResponse, ThemeConfigUpdate};

fn client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| format!("HTTP client init error: {e}"))
}

/// Fetch the current hackathon config from the core API.
///
/// Expected Behavior:
///   Sends a GET request to `{api_url}/api/core/info` with a 3-second
///   timeout. On 200 OK, deserializes the JSON response into a
///   ThemeConfigResponse, then converts it to our internal ThemeConfig.
///   On any non-2xx status or network failure, returns an error string
///   describing the problem.
///
/// Raises:
///   Returns a String error on network failure, timeout, non-2xx status,
///   or JSON deserialization failure.
///
/// Side Effects:
///   - Makes one async HTTP GET request.
pub async fn fetch_config(api_url: &str) -> Result<ThemeConfig, String> {
    let client = client()?;
    let url = format!("{api_url}/api/core/info");
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let response: ThemeConfigResponse = resp
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;
    Ok(ThemeConfig::from(response))
}

/// Save the hackathon config to the core API.
///
/// Expected Behavior:
///   Sends a PUT request to `{api_url}/api/core/info` with a
///   ThemeConfigUpdate (only the fields we edit) serialized as JSON,
///   an optional `Authorization: Bearer <token>` header, and a
///   3-second timeout. On 200 OK, returns success. On non-2xx,
///   returns an error string with the status and any response body.
///
/// Raises:
///   Returns a String error on network failure, timeout, non-2xx status,
///   or JSON serialization failure.
///
/// Side Effects:
///   - Makes one async HTTP PUT request with JSON body.
pub async fn save_config(
    api_url: &str,
    config: &ThemeConfig,
    token: Option<&str>,
) -> Result<(), String> {
    let client = client()?;
    let url = format!("{api_url}/api/core/info");
    let update = ThemeConfigUpdate::from(config);
    let mut req = client.put(&url).json(&update);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    let resp = req.send().await.map_err(|e| format!("Network error: {e}"))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            Err("401 Unauthorized — set OPENHACK_ADMIN_TOKEN env var".to_string())
        } else {
            Err(format!("HTTP {status}: {body}"))
        }
    }
}
