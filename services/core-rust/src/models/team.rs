use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row representation of a team.
///
/// # Expected Behavior
///
/// Maps directly to the `core.teams` table. `join_code` is a unique
/// 8-character code for joining. `max_size` limits the number of members
/// (default 4).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub join_code: Option<String>,
    pub max_size: Option<i32>,
}

/// Database row representation of a team member.
///
/// # Expected Behavior
///
/// Maps directly to the `core.team_members` table. `role` defaults to
/// "member"; the creator is assigned "leader".
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamMember {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: Option<String>,
    pub joined_at: Option<DateTime<Utc>>,
}

/// Database row representation of a team invite.
///
/// # Expected Behavior
///
/// Maps directly to the `core.team_invites` table. `expires_at` controls
/// when the invite becomes invalid. `accepted` tracks whether it was used.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamInvite {
    pub id: Uuid,
    pub team_id: Uuid,
    pub email: Option<String>,
    pub invited_by: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub accepted: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Request body for creating a new team.
///
/// # Expected Behavior
///
/// `name` is required. `description` and `max_size` are optional.
/// The creator is automatically added as the team leader.
#[derive(Debug, Clone, Deserialize)]
pub struct TeamCreate {
    pub name: String,
    pub description: Option<String>,
    pub max_size: Option<i32>,
}

/// Request body for updating a team.
///
/// # Expected Behavior
///
/// All fields are optional; only provided fields will be updated.
#[derive(Debug, Clone, Deserialize)]
pub struct TeamUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_size: Option<i32>,
    pub status: Option<String>,
}

/// Response body for a single team with member count.
///
/// # Expected Behavior
///
/// Contains team fields plus `member_count` derived from counting
/// `team_members` rows.
#[derive(Debug, Clone, Serialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub join_code: Option<String>,
    pub max_size: i32,
    pub member_count: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Response body for a team member.
///
/// # Expected Behavior
///
/// Contains the member's `user_id`, role, and join timestamp.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct TeamMemberResponse {
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: Option<DateTime<Utc>>,
}

impl From<TeamMember> for TeamMemberResponse {
    fn from(m: TeamMember) -> Self {
        Self {
            user_id: m.user_id,
            role: m.role.unwrap_or_else(|| "member".to_string()),
            joined_at: m.joined_at,
        }
    }
}

/// Request body for inviting a member to a team.
///
/// # Expected Behavior
///
/// `email` is required. The invite will be created with an expiration
/// of 7 days from now.
#[derive(Debug, Clone, Deserialize)]
pub struct MemberInviteRequest {
    pub email: String,
}

/// Request body for accepting a team invite.
///
/// # Expected Behavior
///
/// `invite_id` is required. The invite must not be expired and must not
/// have been previously accepted.
#[derive(Debug, Clone, Deserialize)]
pub struct InviteAcceptRequest {
    pub invite_id: Uuid,
}
