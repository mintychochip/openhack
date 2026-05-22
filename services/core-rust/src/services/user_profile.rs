use crate::errors::CoreError;
use crate::models::user_profile::{TeamSeekingListResponse, TeamSeekingUpdate, TeamSeekingUser};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct UserProfileService;

impl UserProfileService {
    pub async fn update_team_seeking(
        pool: &PgPool,
        user_id: Uuid,
        update: &TeamSeekingUpdate,
    ) -> Result<(), CoreError> {
        let skills_json = update.skills.as_ref().map(|s| {
            serde_json::to_value(s).unwrap_or(serde_json::Value::Null)
        });

        sqlx::query(
            r#"UPDATE auth.users 
               SET looking_for_team = $1,
                   skills = COALESCE($2, skills),
                   availability = COALESCE($3, availability),
                   timezone = COALESCE($4, timezone),
                   team_preferences = COALESCE($5, team_preferences)
               WHERE id = $6"#
        )
        .bind(update.looking_for_team)
        .bind(skills_json)
        .bind(&update.availability)
        .bind(&update.timezone)
        .bind(&update.team_preferences)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list_team_seekers(
        pool: &PgPool,
        skills: Option<&[String]>,
        limit: i64,
        offset: i64,
    ) -> Result<TeamSeekingListResponse, CoreError> {
        let (where_clause, bind_idx) = if let Some(skills) = skills {
            ("WHERE looking_for_team = TRUE AND skills && $1", 1)
        } else {
            ("WHERE looking_for_team = TRUE", 0)
        };

        let total: i64 = if skills.is_some() {
            sqlx::query_scalar(&format!("SELECT COUNT(*) FROM auth.users {}", where_clause))
                .bind(skills)
                .fetch_one(pool)
                .await
                .unwrap_or(0)
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM auth.users WHERE looking_for_team = TRUE")
                .fetch_one(pool)
                .await
                .unwrap_or(0)
        };

        let query = format!(
            r#"SELECT id, name, COALESCE(skills, '{{}}') as skills, availability, timezone
               FROM auth.users 
               {} 
               ORDER BY created_at DESC 
               LIMIT ${} OFFSET ${}"#,
            where_clause,
            bind_idx + 1,
            bind_idx + 2
        );

        let rows = if skills.is_some() {
            sqlx::query(&query).bind(skills.unwrap()).bind(limit).bind(offset).fetch_all(pool).await?
        } else {
            sqlx::query(&query).bind(limit).bind(offset).fetch_all(pool).await?
        };

        let users: Vec<TeamSeekingUser> = rows.iter().map(|row| TeamSeekingUser {
            id: row.get::<Uuid, _>(0),
            name: row.get::<String, _>(1),
            skills: row.get::<Vec<String>, _>(2),
            availability: row.get::<Option<String>, _>(3),
            timezone: row.get::<Option<String>, _>(4),
        }).collect();

        Ok(TeamSeekingListResponse { users, total })
    }

    pub async fn list_teams_seeking(
        pool: &PgPool,
        _skills: Option<&[String]>,
        limit: i64,
        offset: i64,
    ) -> Result<crate::models::team::TeamListResponse, CoreError> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM core.teams WHERE looking_for_members = TRUE")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let teams = sqlx::query_as::<_, crate::models::team::Team>(
            "SELECT id, name, description, created_at, updated_at, status, join_code, max_size
             FROM core.teams
             WHERE looking_for_members = TRUE
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let responses: Vec<crate::models::team::TeamResponse> = teams.into_iter().map(|t| {
            crate::models::team::TeamResponse {
                id: t.id,
                name: t.name,
                description: t.description,
                status: t.status.unwrap_or_else(|| "active".to_string()),
                join_code: t.join_code,
                max_size: t.max_size.unwrap_or(4),
                member_count: 0,
                created_at: t.created_at,
                updated_at: t.updated_at,
            }
        }).collect();

        Ok(crate::models::team::TeamListResponse {
            teams: responses,
            total,
        })
    }
}
