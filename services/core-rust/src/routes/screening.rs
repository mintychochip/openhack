use actix_web::{web, HttpResponse};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

use crate::middleware::auth::{get_auth_user, AuthUser};
use crate::models::project::ScreenRequest;
use crate::models::user_profile::TeamSeekingUpdate;
use crate::services::user_profile::UserProfileService;

fn check_admin_or_judge(user: &AuthUser) -> bool {
    user.is_admin()
}

/// GET /api/core/admin/projects/screening — List projects pending screening.
pub async fn list_screening_queue(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(_) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "unauthorized"})),
    };

    if !check_admin_or_judge(&auth_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden"
        }));
    }

    match list_pending_projects(pool.get_ref()).await {
        Ok(projects) => HttpResponse::Ok().json(serde_json::json!({
            "projects": projects,
            "total": projects.len()
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "database_error"})),
    }
}

async fn list_pending_projects(pool: &PgPool) -> Result<Vec<crate::models::project::ProjectResponse>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT * FROM core.projects WHERE screening_status = 'pending' ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(pool)
    .await?;

    let projects: Vec<crate::models::project::ProjectResponse> = rows.iter().filter_map(|row| {
        let project: Result<crate::models::project::Project, _> = FromRow::from_row(row);
        project.ok().map(Into::into)
    }).collect();

    Ok(projects)
}

/// POST /api/core/admin/projects/{id}/screen — Screen a project (approve/reject).
pub async fn screen_project(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<ScreenRequest>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(_) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "unauthorized"})),
    };

    if !check_admin_or_judge(&auth_user) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden"
        }));
    }

    let project_id = path.into_inner();
    
    let result = sqlx::query(
        "UPDATE core.projects 
         SET screening_status = $1, 
             screening_notes = COALESCE($2, screening_notes),
             screened_by = $3,
             screened_at = NOW()
         WHERE id = $4 RETURNING *"
    )
    .bind(&body.status)
    .bind(&body.notes)
    .bind(auth_user.user_id)
    .bind(project_id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            if let Ok(project) = crate::models::project::Project::from_row(&row) {
                let response: crate::models::project::ProjectResponse = project.into();
                HttpResponse::Ok().json(response)
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({"error": "parse_error"}))
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "database_error"})),
    }
}

/// PUT /api/core/profile/team-seeking — Update team-seeking status.
pub async fn update_team_seeking(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    body: web::Json<TeamSeekingUpdate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(_) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "unauthorized"})),
    };

    match UserProfileService::update_team_seeking(pool.get_ref(), auth_user.user_id, &body).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "updated"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "database_error"})),
    }
}

/// GET /api/core/users/looking-for-team — List users looking for teams.
pub async fn list_team_seekers(
    pool: web::Data<PgPool>,
    query: web::Query<crate::models::user_profile::TeamSeekerQuery>,
) -> HttpResponse {
    let skills = query.skills.as_ref().map(|s| s.split(',').map(String::from).collect::<Vec<_>>());
    let skills_slice = skills.as_ref().map(|s| s.as_slice()).unwrap_or(&[]);
    
    match UserProfileService::list_team_seekers(
        pool.get_ref(),
        if skills_slice.is_empty() { None } else { Some(skills_slice) },
        query.limit.unwrap_or(20),
        (query.page.unwrap_or(1) - 1) * query.limit.unwrap_or(20),
    ).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "database_error"})),
    }
}

/// GET /api/core/teams/looking-for-members — List teams seeking members.
pub async fn list_teams_seeking(
    pool: web::Data<PgPool>,
    query: web::Query<crate::models::team::TeamSeekingQuery>,
) -> HttpResponse {
    let skills = query.skills.as_ref().map(|s| s.split(',').map(String::from).collect::<Vec<_>>());
    let skills_slice = skills.as_ref().map(|s| s.as_slice()).unwrap_or(&[]);
    
    match UserProfileService::list_teams_seeking(
        pool.get_ref(),
        if skills_slice.is_empty() { None } else { Some(skills_slice) },
        query.limit.unwrap_or(20),
        (query.page.unwrap_or(1) - 1) * query.limit.unwrap_or(20),
    ).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "database_error"})),
    }
}
