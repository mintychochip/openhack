use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Typed HTTP client for calling OpenHack internal services via the gateway.
///
/// # Expected Behavior
///
/// All requests include the `X-Lambda-Internal-Token` header for
/// service-to-service authentication. Uses a 15-second timeout.
/// Methods return typed responses or `BotError::OpenHackError` on failure.
#[derive(Debug, Clone)]
pub struct OpenHackClient {
    base_url: String,
    internal_token: String,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct HackathonStatus {
    pub name: String,
    pub current_phase: Option<String>,
    pub participant_count: i64,
    pub team_count: i64,
    pub project_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct TeamInfo {
    pub id: String,
    pub name: String,
    pub members: Vec<TeamMember>,
    pub project: Option<ProjectInfo>,
    pub invite_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TeamMember {
    #[allow(dead_code)]
    pub user_id: String,
    pub display_name: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectInfo {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
    #[allow(dead_code)]
    pub repo_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub team_name: String,
    pub score: f64,
    pub project_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleEvent {
    pub name: String,
    pub event_type: String,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitProjectRequest {
    pub name: String,
    pub description: String,
    pub repo_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinTeamRequest {
    pub invite_code: String,
}

impl OpenHackClient {
    /// Create a new OpenHack client with the given gateway URL and internal token.
    ///
    /// # Expected Behavior
    ///
    /// Stores the base URL and internal token for use in all subsequent requests.
    /// Creates a reqwest client with a 15-second timeout.
    ///
    /// # Errors
    ///
    /// None. Always succeeds.
    ///
    /// # Side Effects
    ///
    /// None.
    pub fn new(base_url: &str, internal_token: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            internal_token: internal_token.to_string(),
            http,
        }
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, crate::errors::BotError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .header("X-Lambda-Internal-Token", &self.internal_token)
            .send()
            .await
            .map_err(|e| crate::errors::BotError::OpenHackError(format!("Request failed: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(crate::errors::BotError::OpenHackError(format!(
                "OpenHack API returned {status}: {body}"
            )));
        }

        resp.json::<T>().await.map_err(|e| {
            crate::errors::BotError::OpenHackError(format!("Failed to parse response: {e}"))
        })
    }

    async fn post<T: Serialize, R: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R, crate::errors::BotError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .header("X-Lambda-Internal-Token", &self.internal_token)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| crate::errors::BotError::OpenHackError(format!("Request failed: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(crate::errors::BotError::OpenHackError(format!(
                "OpenHack API returned {status}: {body}"
            )));
        }

        resp.json::<R>().await.map_err(|e| {
            crate::errors::BotError::OpenHackError(format!("Failed to parse response: {e}"))
        })
    }

    /// Get current hackathon status.
    ///
    /// # Expected Behavior
    ///
    /// Calls `GET /api/core/hackathon/status` on the OpenHack gateway.
    /// Returns hackathon name, current phase, and counts.
    ///
    /// # Errors
    ///
    /// Returns `BotError::OpenHackError` if the request fails or returns non-2xx.
    ///
    /// # Side Effects
    ///
    /// - Makes an HTTP GET request to the OpenHack gateway.
    pub async fn get_hackathon_status(&self) -> Result<HackathonStatus, crate::errors::BotError> {
        self.get("/api/core/hackathon/status").await
    }

    /// Get a user's team by their OpenHack user ID.
    ///
    /// # Expected Behavior
    ///
    /// Calls `GET /api/core/users/{user_id}/team` on the OpenHack gateway.
    /// Returns team info including members and project.
    ///
    /// # Errors
    ///
    /// Returns `BotError::OpenHackError` if the request fails.
    ///
    /// # Side Effects
    ///
    /// - Makes an HTTP GET request to the OpenHack gateway.
    pub async fn get_team(&self, user_id: &str) -> Result<TeamInfo, crate::errors::BotError> {
        self.get(&format!("/api/core/users/{user_id}/team")).await
    }

    /// Create a new team.
    ///
    /// # Expected Behavior
    ///
    /// Calls `POST /api/core/teams` with the team name. Returns the new team info.
    ///
    /// # Errors
    ///
    /// Returns `BotError::OpenHackError` if the request fails.
    ///
    /// # Side Effects
    ///
    /// - Makes an HTTP POST request to the OpenHack gateway.
    /// - Creates a new team in the OpenHack database.
    pub async fn create_team(
        &self,
        user_id: &str,
        name: &str,
    ) -> Result<TeamInfo, crate::errors::BotError> {
        self.post(
            &format!("/api/core/users/{user_id}/teams"),
            &CreateTeamRequest {
                name: name.to_string(),
            },
        )
        .await
    }

    /// Join a team by invite code.
    ///
    /// # Expected Behavior
    ///
    /// Calls `POST /api/core/teams/join` with the invite code.
    /// Returns the team info after joining.
    ///
    /// # Errors
    ///
    /// Returns `BotError::OpenHackError` if the request fails
    /// (e.g., invalid code, team full).
    ///
    /// # Side Effects
    ///
    /// - Makes an HTTP POST request to the OpenHack gateway.
    /// - Adds the user to the team in the OpenHack database.
    pub async fn join_team(
        &self,
        user_id: &str,
        invite_code: &str,
    ) -> Result<TeamInfo, crate::errors::BotError> {
        self.post(
            &format!("/api/core/users/{user_id}/teams/join"),
            &JoinTeamRequest {
                invite_code: invite_code.to_string(),
            },
        )
        .await
    }

    /// Submit a project.
    ///
    /// # Expected Behavior
    ///
    /// Calls `POST /api/core/projects` with project details.
    /// Returns the project info after submission.
    ///
    /// # Errors
    ///
    /// Returns `BotError::OpenHackError` if the request fails.
    ///
    /// # Side Effects
    ///
    /// - Makes an HTTP POST request to the OpenHack gateway.
    /// - Creates a project submission in the OpenHack database.
    pub async fn submit_project(
        &self,
        team_id: &str,
        name: &str,
        description: &str,
        repo_url: Option<&str>,
    ) -> Result<ProjectInfo, crate::errors::BotError> {
        self.post(
            &format!("/api/core/teams/{team_id}/projects"),
            &SubmitProjectRequest {
                name: name.to_string(),
                description: description.to_string(),
                repo_url: repo_url.map(String::from),
            },
        )
        .await
    }

    /// Get leaderboard entries.
    ///
    /// # Expected Behavior
    ///
    /// Calls `GET /api/leaderboard/rankings?limit={top}` on the OpenHack gateway.
    /// Returns a list of leaderboard entries.
    ///
    /// # Errors
    ///
    /// Returns `BotError::OpenHackError` if the request fails.
    ///
    /// # Side Effects
    ///
    /// - Makes an HTTP GET request to the OpenHack gateway.
    pub async fn get_leaderboard(
        &self,
        top: u32,
    ) -> Result<Vec<LeaderboardEntry>, crate::errors::BotError> {
        self.get(&format!("/api/leaderboard/rankings?limit={top}"))
            .await
    }

    /// Get schedule events.
    ///
    /// # Expected Behavior
    ///
    /// Calls `GET /api/core/events` on the OpenHack gateway.
    /// Returns a list of scheduled events.
    ///
    /// # Errors
    ///
    /// Returns `BotError::OpenHackError` if the request fails.
    ///
    /// # Side Effects
    ///
    /// - Makes an HTTP GET request to the OpenHack gateway.
    pub async fn get_schedule(&self) -> Result<Vec<ScheduleEvent>, crate::errors::BotError> {
        self.get("/api/core/events").await
    }

    /// Check in a user to the hackathon.
    ///
    /// # Expected Behavior
    ///
    /// Calls `POST /api/core/checkins` with the user ID.
    /// Returns success/failure.
    ///
    /// # Errors
    ///
    /// Returns `BotError::OpenHackError` if the request fails.
    ///
    /// # Side Effects
    ///
    /// - Makes an HTTP POST request to the OpenHack gateway.
    /// - Records a check-in in the OpenHack database.
    pub async fn check_in(
        &self,
        user_id: &str,
    ) -> Result<serde_json::Value, crate::errors::BotError> {
        self.post(
            &format!("/api/core/users/{user_id}/checkin"),
            &serde_json::json!({}),
        )
        .await
    }

    /// Send a message to a Discord channel using the bot.
    ///
    /// # Expected Behavior
    ///
    /// Makes a POST to the Discord API `POST /channels/{channel_id}/messages`
    /// with the bot token. Returns the message response.
    ///
    /// # Errors
    ///
    /// Returns `BotError::DiscordError` if the request fails.
    ///
    /// # Side Effects
    ///
    /// - Makes an HTTP POST request to the Discord API.
    /// - Posts a message to the specified Discord channel.
    pub async fn send_channel_message(
        &self,
        bot_token: &str,
        channel_id: &str,
        content: &str,
        embeds: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, crate::errors::BotError> {
        let mut body = serde_json::json!({
            "content": content,
        });
        if let Some(e) = embeds {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("embeds".to_string(), e.clone());
            }
        }

        let resp = self
            .http
            .post(format!(
                "https://discord.com/api/v10/channels/{channel_id}/messages"
            ))
            .header("Authorization", format!("Bot {bot_token}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::errors::BotError::DiscordError(format!("Request failed: {e}")))?;

        let status = resp.status();
        let resp_body = resp.text().await.unwrap_or_default();

        if status.is_success() {
            Ok(serde_json::json!({"status": "sent"}))
        } else {
            Err(crate::errors::BotError::DiscordError(format!(
                "Discord API returned {status}: {resp_body}"
            )))
        }
    }

    /// Send a DM to a Discord user.
    ///
    /// # Expected Behavior
    ///
    /// Creates a DM channel via `POST /users/@me/channels` with the recipient
    /// user ID, then posts a message to that channel. Returns success/failure.
    ///
    /// # Errors
    ///
    /// Returns `BotError::DiscordError` if either request fails.
    ///
    /// # Side Effects
    ///
    /// - Makes HTTP POST requests to the Discord API.
    /// - Creates a DM channel and sends a message.
    pub async fn send_dm(
        &self,
        bot_token: &str,
        discord_user_id: &str,
        content: &str,
        embeds: Option<&serde_json::Value>,
    ) -> Result<(), crate::errors::BotError> {
        let dm_channel: serde_json::Value = self
            .http
            .post("https://discord.com/api/v10/users/@me/channels")
            .header("Authorization", format!("Bot {bot_token}"))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({"recipient_id": discord_user_id}))
            .send()
            .await
            .map_err(|e| {
                crate::errors::BotError::DiscordError(format!("DM channel creation failed: {e}"))
            })?
            .json()
            .await
            .map_err(|e| {
                crate::errors::BotError::DiscordError(format!("DM channel parse failed: {e}"))
            })?;

        let channel_id = dm_channel
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::errors::BotError::DiscordError("No DM channel ID".into()))?;

        self.send_channel_message(bot_token, channel_id, content, embeds)
            .await?;

        Ok(())
    }
}
