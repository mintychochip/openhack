use rand::Rng;
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::CoreError;
use crate::models::team::{Team, TeamCreate, TeamInvite, TeamMember, TeamResponse, TeamUpdate};
use crate::services::publisher;

pub struct TeamService;

impl TeamService {
    /// Generate a random 8-character alphanumeric join code.
    fn generate_join_code() -> String {
        let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
        let mut rng = rand::thread_rng();
        (0..8)
            .map(|_| chars[rng.gen_range(0..chars.len())])
            .collect()
    }

    /// Create a new team.
    ///
    /// # Expected Behavior
    ///
    /// Inserts a new row into `core.teams` with a unique `join_code`, then
    /// adds the creator as a team member with role "leader". Returns the
    /// created team. If Redis is available, publishes a `team.created` event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on insert failure.
    ///
    /// # Side Effects
    ///
    /// - Inserts a row into `core.teams` (database write).
    /// - Inserts a row into `core.team_members` (database write).
    /// - Publishes `team.created` event to Redis, if available.
    pub async fn create_team(
        pool: &PgPool,
        redis_conn: Option<&mut MultiplexedConnection>,
        user_id: Uuid,
        data: &TeamCreate,
    ) -> Result<TeamResponse, CoreError> {
        let join_code = Self::generate_join_code();
        let max_size = data.max_size.unwrap_or(4);

        let team = sqlx::query_as::<_, Team>(
            "INSERT INTO core.teams (name, description, join_code, max_size, status) VALUES ($1, $2, $3, $4, 'active') RETURNING id, name, description, created_at, updated_at, status, join_code, max_size",
        )
        .bind(&data.name)
        .bind(&data.description)
        .bind(&join_code)
        .bind(max_size)
        .fetch_one(pool)
        .await?;

        let team_id = team.id;

        sqlx::query(
            "INSERT INTO core.team_members (team_id, user_id, role) VALUES ($1, $2, 'leader')",
        )
        .bind(team_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        let member_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM core.team_members WHERE team_id = $1")
                .bind(team_id)
                .fetch_one(pool)
                .await
                .unwrap_or(1);

        publisher::team_created(redis_conn, &team_id.to_string(), &user_id.to_string()).await;
        log::info!("Created team {team_id} for user {user_id}");

        Ok(TeamResponse {
            id: team.id,
            name: team.name,
            description: team.description,
            status: team.status.unwrap_or_else(|| "active".to_string()),
            join_code: team.join_code,
            max_size: team.max_size.unwrap_or(4),
            member_count,
            created_at: team.created_at,
            updated_at: team.updated_at,
        })
    }

    /// Get a team by ID with member count.
    ///
    /// # Expected Behavior
    ///
    /// Fetches the team and counts its members. Returns `CoreError::NotFound`
    /// if the team does not exist.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if no team exists with the given ID.
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.teams` and `core.team_members` (database reads).
    pub async fn get_team(pool: &PgPool, id: Uuid) -> Result<TeamResponse, CoreError> {
        let team = sqlx::query_as::<_, Team>(
            "SELECT id, name, description, created_at, updated_at, status, join_code, max_size FROM core.teams WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Team".into(), id.to_string()))?;

        let member_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM core.team_members WHERE team_id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        Ok(TeamResponse {
            id: team.id,
            name: team.name,
            description: team.description,
            status: team.status.unwrap_or_else(|| "active".to_string()),
            join_code: team.join_code,
            max_size: team.max_size.unwrap_or(4),
            member_count,
            created_at: team.created_at,
            updated_at: team.updated_at,
        })
    }

    /// Update a team. Only the team leader can update.
    ///
    /// # Expected Behavior
    ///
    /// Verifies the user is a leader of the team, then updates only the
    /// provided fields. Returns the updated team. If Redis is available,
    /// publishes a `project.updated` event (as a general update notification).
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Forbidden` if the user is not a team leader.
    /// Returns `CoreError::NotFound` if the team does not exist.
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.team_members` to verify role (database read).
    /// - Updates row in `core.teams` (database write).
    /// - Publishes `project.updated` event to Redis, if available.
    pub async fn update_team(
        pool: &PgPool,
        redis_conn: Option<&mut MultiplexedConnection>,
        id: Uuid,
        user_id: Uuid,
        data: &TeamUpdate,
    ) -> Result<TeamResponse, CoreError> {
        let role: Option<String> = sqlx::query_scalar(
            "SELECT role FROM core.team_members WHERE team_id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .flatten();

        if role.as_deref() != Some("leader") {
            return Err(CoreError::Forbidden(
                "Only the team leader can update the team".into(),
            ));
        }

        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if data.name.is_some() {
            updates.push(format!("name = ${param_idx}"));
            param_idx += 1;
        }
        if data.description.is_some() {
            updates.push(format!("description = ${param_idx}"));
            param_idx += 1;
        }
        if data.max_size.is_some() {
            updates.push(format!("max_size = ${param_idx}"));
            param_idx += 1;
        }
        if data.status.is_some() {
            updates.push(format!("status = ${param_idx}"));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Self::get_team(pool, id).await;
        }

        let sql = format!(
            "UPDATE core.teams SET {} WHERE id = ${} RETURNING id, name, description, created_at, updated_at, status, join_code, max_size",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, Team>(&sql);

        if let Some(ref v) = data.name {
            query = query.bind(v);
        }
        if let Some(ref v) = data.description {
            query = query.bind(v);
        }
        if let Some(ref v) = data.max_size {
            query = query.bind(v);
        }
        if let Some(ref v) = data.status {
            query = query.bind(v);
        }

        let team = query
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| CoreError::NotFound("Team".into(), id.to_string()))?;

        publisher::project_updated(redis_conn, &id.to_string(), &user_id.to_string()).await;
        log::info!("Updated team {id}");

        let member_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM core.team_members WHERE team_id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        Ok(TeamResponse {
            id: team.id,
            name: team.name,
            description: team.description,
            status: team.status.unwrap_or_else(|| "active".to_string()),
            join_code: team.join_code,
            max_size: team.max_size.unwrap_or(4),
            member_count,
            created_at: team.created_at,
            updated_at: team.updated_at,
        })
    }

    /// Delete a team. Only the team leader can delete.
    ///
    /// # Expected Behavior
    ///
    /// Verifies the user is a leader of the team, then deletes the team.
    /// Cascade deletes will remove `team_members`.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Forbidden` if the user is not a team leader.
    /// Returns `CoreError::NotFound` if the team does not exist.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.team_members` to verify role (database read).
    /// - Deletes row from `core.teams` (database write, cascades).
    pub async fn delete_team(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), CoreError> {
        let role: Option<String> = sqlx::query_scalar(
            "SELECT role FROM core.team_members WHERE team_id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .flatten();

        if role.as_deref() != Some("leader") {
            return Err(CoreError::Forbidden(
                "Only the team leader can delete the team".into(),
            ));
        }

        let result = sqlx::query("DELETE FROM core.teams WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound("Team".into(), id.to_string()));
        }

        log::info!("Deleted team {id}");
        Ok(())
    }

    /// Join a team by ID.
    ///
    /// # Expected Behavior
    ///
    /// Checks that the team exists, is active, and has capacity. Adds the
    /// user as a member with role "member". If Redis is available, publishes
    /// a `team.joined` event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the team does not exist.
    /// Returns `CoreError::Conflict` if the user is already a member.
    /// Returns `CoreError::Validation` if the team is full.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.teams` and `core.team_members` (database reads).
    /// - Inserts into `core.team_members` (database write).
    /// - Publishes `team.joined` event to Redis, if available.
    pub async fn join_team(
        pool: &PgPool,
        redis_conn: Option<&mut MultiplexedConnection>,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<TeamResponse, CoreError> {
        let team = sqlx::query_as::<_, Team>(
            "SELECT id, name, description, created_at, updated_at, status, join_code, max_size FROM core.teams WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("Team".into(), id.to_string()))?;

        if team.status.as_deref() == Some("inactive") {
            return Err(CoreError::Validation("Team is not active".into()));
        }

        let member_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM core.team_members WHERE team_id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let max_size = i64::from(team.max_size.unwrap_or(4));
        if member_count >= max_size {
            return Err(CoreError::Validation("Team is full".into()));
        }

        let existing: Option<String> = sqlx::query_scalar(
            "SELECT role FROM core.team_members WHERE team_id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if existing.is_some() {
            return Err(CoreError::Conflict("Already a member of this team".into()));
        }

        sqlx::query(
            "INSERT INTO core.team_members (team_id, user_id, role) VALUES ($1, $2, 'member')",
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        publisher::team_joined(redis_conn, &id.to_string(), &user_id.to_string()).await;
        log::info!("User {user_id} joined team {id}");

        Self::get_team(pool, id).await
    }

    /// Leave a team.
    ///
    /// # Expected Behavior
    ///
    /// Removes the user from the team. If the user is the leader and there
    /// are other members, promotes the earliest member to leader. If the
    /// leader is the last member, the team is deleted. If Redis is available,
    /// publishes a `team.left` event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the user is not a member.
    ///
    /// # Side Effects
    ///
    /// - Deletes from `core.team_members` (database write).
    /// - May update a member's role to "leader" (database write).
    /// - May delete the team if leader leaves as last member (database write).
    /// - Publishes `team.left` event to Redis, if available.
    pub async fn leave_team(
        pool: &PgPool,
        redis_conn: Option<&mut MultiplexedConnection>,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), CoreError> {
        let member = sqlx::query_as::<_, TeamMember>(
            "SELECT id, team_id, user_id, role, joined_at FROM core.team_members WHERE team_id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("TeamMember".into(), user_id.to_string()))?;

        let is_leader = member.role.as_deref() == Some("leader");
        let member_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM core.team_members WHERE team_id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        if is_leader && member_count > 1 {
            sqlx::query(
                "UPDATE core.team_members SET role = 'leader' WHERE id = (
                    SELECT id FROM core.team_members WHERE team_id = $1 AND user_id != $2 ORDER BY joined_at ASC LIMIT 1
                )",
            )
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        }

        sqlx::query("DELETE FROM core.team_members WHERE team_id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if is_leader && member_count == 1 {
            sqlx::query("DELETE FROM core.teams WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
        }

        publisher::team_left(redis_conn, &id.to_string(), &user_id.to_string()).await;
        log::info!("User {user_id} left team {id}");
        Ok(())
    }

    /// Invite a member to a team by email.
    ///
    /// # Expected Behavior
    ///
    /// Creates a team invite with a 7-day expiration. Only team members
    /// can invite.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Forbidden` if the inviter is not a team member.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.team_members` to verify membership (database read).
    /// - Inserts into `core.team_invites` (database write).
    pub async fn invite_member(
        pool: &PgPool,
        team_id: Uuid,
        email: &str,
        invited_by: Uuid,
    ) -> Result<TeamInvite, CoreError> {
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM core.team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(team_id)
        .bind(invited_by)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if !is_member {
            return Err(CoreError::Forbidden("Only team members can invite".into()));
        }

        let invite = sqlx::query_as::<_, TeamInvite>(
            "INSERT INTO core.team_invites (team_id, email, invited_by, expires_at, accepted) VALUES ($1, $2, $3, NOW() + INTERVAL '7 days', FALSE) RETURNING id, team_id, email, invited_by, expires_at, accepted, created_at",
        )
        .bind(team_id)
        .bind(email)
        .bind(invited_by)
        .fetch_one(pool)
        .await?;

        log::info!("Invited {email} to team {team_id}");
        Ok(invite)
    }

    /// Kick a member from a team. Only the leader can kick.
    ///
    /// # Expected Behavior
    ///
    /// Verifies the kicker is a leader, then removes the target member.
    /// Cannot kick the leader.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Forbidden` if the kicker is not the leader.
    /// Returns `CoreError::NotFound` if the target is not a member.
    /// Returns `CoreError::Validation` if trying to kick the leader.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.team_members` to verify roles (database read).
    /// - Deletes from `core.team_members` (database write).
    pub async fn kick_member(
        pool: &PgPool,
        team_id: Uuid,
        user_id: Uuid,
        kicker_id: Uuid,
    ) -> Result<(), CoreError> {
        let kicker_role: Option<String> = sqlx::query_scalar(
            "SELECT role FROM core.team_members WHERE team_id = $1 AND user_id = $2",
        )
        .bind(team_id)
        .bind(kicker_id)
        .fetch_optional(pool)
        .await?;

        if kicker_role.as_deref() != Some("leader") {
            return Err(CoreError::Forbidden(
                "Only the team leader can kick members".into(),
            ));
        }

        let target_role: Option<String> = sqlx::query_scalar(
            "SELECT role FROM core.team_members WHERE team_id = $1 AND user_id = $2",
        )
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("TeamMember".into(), user_id.to_string()))?;

        if target_role == Some("leader".to_string()) {
            return Err(CoreError::Validation("Cannot kick the team leader".into()));
        }

        sqlx::query("DELETE FROM core.team_members WHERE team_id = $1 AND user_id = $2")
            .bind(team_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        log::info!("Kicked user {user_id} from team {team_id}");
        Ok(())
    }

    /// Accept a team invite.
    ///
    /// # Expected Behavior
    ///
    /// Marks the invite as accepted and adds the user to the team as a
    /// member. Validates that the invite exists, is not expired, and has
    /// not already been accepted. If Redis is available, publishes a
    /// `team.joined` event.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::NotFound` if the invite does not exist.
    /// Returns `CoreError::Validation` if the invite is expired or already accepted.
    /// Returns `CoreError::Conflict` if the user is already a member.
    ///
    /// # Side Effects
    ///
    /// - Updates `core.team_invites` (database write).
    /// - Inserts into `core.team_members` (database write).
    /// - Publishes `team.joined` event to Redis, if available.
    pub async fn accept_invite(
        pool: &PgPool,
        redis_conn: Option<&mut MultiplexedConnection>,
        invite_id: Uuid,
        user_id: Uuid,
    ) -> Result<TeamResponse, CoreError> {
        let invite = sqlx::query_as::<_, TeamInvite>(
            "SELECT id, team_id, email, invited_by, expires_at, accepted, created_at FROM core.team_invites WHERE id = $1",
        )
        .bind(invite_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| CoreError::NotFound("TeamInvite".into(), invite_id.to_string()))?;

        if invite.accepted == Some(true) {
            return Err(CoreError::Validation("Invite already accepted".into()));
        }

        if let Some(expires) = invite.expires_at {
            if expires < chrono::Utc::now() {
                return Err(CoreError::Validation("Invite has expired".into()));
            }
        }

        let existing: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM core.team_members WHERE team_id = $1 AND user_id = $2)",
        )
        .bind(invite.team_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

        if existing {
            return Err(CoreError::Conflict("Already a member of this team".into()));
        }

        sqlx::query(
            "INSERT INTO core.team_members (team_id, user_id, role) VALUES ($1, $2, 'member')",
        )
        .bind(invite.team_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        sqlx::query("UPDATE core.team_invites SET accepted = TRUE WHERE id = $1")
            .bind(invite_id)
            .execute(pool)
            .await?;

        publisher::team_joined(
            redis_conn,
            &invite.team_id.to_string(),
            &user_id.to_string(),
        )
        .await;
        log::info!(
            "User {} accepted invite to team {}",
            user_id,
            invite.team_id
        );

        Self::get_team(pool, invite.team_id).await
    }

    /// List all teams (admin endpoint).
    ///
    /// # Expected Behavior
    ///
    /// Returns all teams ordered by `created_at` descending with pagination.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::Database` on query failure.
    ///
    /// # Side Effects
    ///
    /// - Reads from `core.teams` (database read).
    pub async fn list_teams_admin(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TeamResponse>, CoreError> {
        let teams = sqlx::query_as::<_, Team>(
            "SELECT id, name, description, created_at, updated_at, status, join_code, max_size FROM core.teams ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let mut responses = Vec::new();
        for team in teams {
            let member_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM core.team_members WHERE team_id = $1")
                    .bind(team.id)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);

            responses.push(TeamResponse {
                id: team.id,
                name: team.name,
                description: team.description,
                status: team.status.unwrap_or_else(|| "active".to_string()),
                join_code: team.join_code,
                max_size: team.max_size.unwrap_or(4),
                member_count,
                created_at: team.created_at,
                updated_at: team.updated_at,
            });
        }

        Ok(responses)
    }
}
