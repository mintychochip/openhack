use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to update team-seeking status.
#[derive(Debug, Clone, Deserialize)]
pub struct TeamSeekingUpdate {
    pub looking_for_team: bool,
    pub skills: Option<Vec<String>>,
    pub availability: Option<String>,
    pub timezone: Option<String>,
    pub team_preferences: Option<serde_json::Value>,
}

/// Query for team seekers.
#[derive(Debug, Clone, Deserialize)]
pub struct TeamSeekerQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub skills: Option<String>,
}

/// Response for team-seeking users list.
#[derive(Debug, Clone, Serialize)]
pub struct TeamSeekingListResponse {
    pub users: Vec<TeamSeekingUser>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamSeekingUser {
    pub id: Uuid,
    pub name: String,
    pub skills: Vec<String>,
    pub availability: Option<String>,
    pub timezone: Option<String>,
}
