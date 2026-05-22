use crate::errors::AuthError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use utoipa::ToSchema;
use uuid::Uuid;

/// GDPR data export response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GdprExport {
    pub export_date: String,
    pub user_id: String,
    pub profile: UserProfileData,
    pub sessions: Vec<SessionData>,
    pub oauth_accounts: Vec<OAuthAccountData>,
    pub audit_logs: Vec<AuditLogData>,
    pub submissions: Vec<SubmissionData>,
    pub scores: Vec<ScoreData>,
    pub team_memberships: Vec<TeamMembershipData>,
    pub event_participations: Vec<EventParticipationData>,
    pub data_retention: DataRetentionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileData {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub github_username: Option<String>,
    pub discord_id: Option<String>,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub sms_mfa_enabled: bool,
    pub roles: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub last_login_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionData {
    pub id: String,
    pub user_id: String,
    pub created_at: String,
    pub expires_at: String,
    pub last_used_at: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OAuthAccountData {
    pub provider: String,
    pub provider_user_id: String,
    pub linked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogData {
    pub timestamp: String,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub outcome: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionData {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScoreData {
    pub id: String,
    pub submission_id: String,
    pub score: i32,
    pub comment: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TeamMembershipData {
    pub team_id: String,
    pub team_name: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventParticipationData {
    pub event_id: String,
    pub event_name: String,
    pub role: String,
    pub registered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DataRetentionInfo {
    pub deletion_requested: bool,
    pub deletion_scheduled_at: Option<String>,
    pub data_retention_days: i64,
}

pub struct GdprService;

impl GdprService {
    pub async fn export_user_data(pool: &PgPool, user_id: Uuid) -> Result<GdprExport, AuthError> {
        let user = sqlx::query_as::<_, crate::models::user::User>(
            "SELECT * FROM auth.users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AuthError::DatabaseError)?
        .ok_or(AuthError::NotFound("User not found".to_string()))?;

        let profile = UserProfileData {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
            avatar_url: user.avatar_url,
            github_username: user.github_username,
            discord_id: user.discord_id,
            email_verified: user.email_verified,
            mfa_enabled: user.mfa_enabled,
            sms_mfa_enabled: user.sms_mfa_enabled,
            roles: user.roles,
            created_at: user.created_at.map(|dt| dt.to_string()),
            updated_at: user.updated_at.map(|dt| dt.to_string()),
            last_login_at: user.last_login_at.map(|dt| dt.to_string()),
        };

        let sessions = sqlx::query(
            "SELECT id, user_id, created_at, expires_at, last_used_at, ip_address, user_agent 
             FROM auth.sessions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 100",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| SessionData {
            id: row.get::<Uuid, _>(0).to_string(),
            user_id: row.get::<Uuid, _>(1).to_string(),
            created_at: row.get::<chrono::NaiveDateTime, _>(2).to_string(),
            expires_at: row.get::<chrono::NaiveDateTime, _>(3).to_string(),
            last_used_at: row.get::<Option<chrono::NaiveDateTime>, _>(4).map(|dt| dt.to_string()),
            ip_address: row.get::<Option<String>, _>(5),
            user_agent: row.get::<Option<String>, _>(6),
        })
        .collect();

        let oauth_accounts = sqlx::query(
            "SELECT provider, provider_user_id, created_at FROM auth.oauth_accounts WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| OAuthAccountData {
            provider: row.get::<String, _>(0),
            provider_user_id: row.get::<String, _>(1),
            linked_at: row.get::<Option<chrono::NaiveDateTime>, _>(2).map(|dt| dt.to_string()),
        })
        .collect();

        let audit_logs = sqlx::query(
            "SELECT timestamp, action, resource_type, resource_id, outcome, ip_address 
             FROM auth.audit_log WHERE actor = $1 ORDER BY timestamp DESC LIMIT 1000",
        )
        .bind(user.id.to_string())
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| AuditLogData {
            timestamp: row.get::<chrono::NaiveDateTime, _>(0).to_string(),
            action: row.get::<String, _>(1),
            resource_type: row.get::<Option<String>, _>(2),
            resource_id: row.get::<Option<String>, _>(3),
            outcome: row.get::<String, _>(4),
            ip_address: row.get::<Option<String>, _>(5),
        })
        .collect();

        let submissions = Self::fetch_user_submissions(pool, user_id).await;
        let scores = Self::fetch_user_scores(pool, user_id).await;
        let team_memberships = Self::fetch_team_memberships(pool, user_id).await;
        let event_participations = Self::fetch_event_participations(pool, user_id).await;

        Ok(GdprExport {
            export_date: Utc::now().to_rfc3339(),
            user_id: user.id.to_string(),
            profile,
            sessions,
            oauth_accounts,
            audit_logs,
            submissions,
            scores,
            team_memberships,
            event_participations,
            data_retention: DataRetentionInfo {
                deletion_requested: user.deletion_requested_at.is_some(),
                deletion_scheduled_at: user.deletion_scheduled_at.map(|dt| dt.to_string()),
                data_retention_days: 2555,
            },
        })
    }

    pub async fn request_deletion(pool: &PgPool, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            "UPDATE auth.users 
             SET deletion_requested_at = $1,
                 deletion_scheduled_at = $2
             WHERE id = $3",
        )
        .bind(Utc::now().naive_utc())
        .bind((Utc::now() + chrono::Duration::days(30)).naive_utc())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

        Ok(())
    }

    async fn fetch_user_submissions(pool: &PgPool, user_id: Uuid) -> Vec<SubmissionData> {
        sqlx::query(
            "SELECT id, title, description, created_at, updated_at, status 
             FROM core.submissions WHERE creator_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| SubmissionData {
            id: row.get::<Uuid, _>(0).to_string(),
            title: row.get::<String, _>(1),
            description: row.get::<Option<String>, _>(2),
            created_at: row.get::<chrono::NaiveDateTime, _>(3).to_string(),
            updated_at: row.get::<chrono::NaiveDateTime, _>(4).to_string(),
            status: row.get::<String, _>(5),
        })
        .collect()
    }

    async fn fetch_user_scores(pool: &PgPool, user_id: Uuid) -> Vec<ScoreData> {
        sqlx::query(
            "SELECT id, submission_id, score, comment, created_at 
             FROM judging.scores WHERE judge_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| ScoreData {
            id: row.get::<Uuid, _>(0).to_string(),
            submission_id: row.get::<Uuid, _>(1).to_string(),
            score: row.get::<i32, _>(2),
            comment: row.get::<Option<String>, _>(3),
            created_at: row.get::<chrono::NaiveDateTime, _>(4).to_string(),
        })
        .collect()
    }

    async fn fetch_team_memberships(pool: &PgPool, user_id: Uuid) -> Vec<TeamMembershipData> {
        sqlx::query(
            "SELECT t.id, t.name, m.role, m.joined_at 
             FROM core.team_members m
             JOIN core.teams t ON t.id = m.team_id
             WHERE m.user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| TeamMembershipData {
            team_id: row.get::<Uuid, _>(0).to_string(),
            team_name: row.get::<String, _>(1),
            role: row.get::<String, _>(2),
            joined_at: row.get::<chrono::NaiveDateTime, _>(3).to_string(),
        })
        .collect()
    }

    async fn fetch_event_participations(pool: &PgPool, user_id: Uuid) -> Vec<EventParticipationData> {
        sqlx::query(
            "SELECT e.id, e.name, p.role, p.registered_at 
             FROM core.event_participations p
             JOIN core.events e ON e.id = p.event_id
             WHERE p.user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| EventParticipationData {
            event_id: row.get::<Uuid, _>(0).to_string(),
            event_name: row.get::<String, _>(1),
            role: row.get::<String, _>(2),
            registered_at: row.get::<chrono::NaiveDateTime, _>(3).to_string(),
        })
        .collect()
    }
}
