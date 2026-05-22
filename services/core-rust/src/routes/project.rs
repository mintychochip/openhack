use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures::StreamExt;
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::get_auth_user;
use crate::models::project::{ProjectCreate, ProjectListQuery, ProjectUpdate};
use crate::services::project::ProjectService;

/// POST /api/core/projects — Create a new project.
///
/// # Expected Behavior
///
/// Requires authentication. User must be a team member.
///
/// # Side Effects
///
/// - Inserts into `core.projects` (database write).
pub async fn create_project(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    body: web::Json<ProjectCreate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    match ProjectService::create_project(pool.get_ref(), user_id, &body.into_inner()).await {
        Ok(project) => HttpResponse::Created().json(project),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/projects — List projects with pagination and filters.
///
/// # Expected Behavior
///
/// Optional auth. Supports page, `page_size`, status, category query params.
///
/// # Side Effects
///
/// - Reads from `core.projects` (database read).
pub async fn list_projects(
    pool: web::Data<PgPool>,
    _req: actix_web::HttpRequest,
    query: web::Query<ProjectListQuery>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let status = query.status.as_deref();
    let category = query.category.as_deref();

    match ProjectService::list_projects(pool.get_ref(), page, page_size, status, category).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

/// GET /api/core/projects/{id} — Get a project by ID.
///
/// # Expected Behavior
///
/// Optional auth. Returns project details.
///
/// # Side Effects
///
/// - Reads from `core.projects` (database read).
pub async fn get_project(
    pool: web::Data<PgPool>,
    _req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    match ProjectService::get_project(pool.get_ref(), id).await {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => e.to_http_response(),
    }
}

/// PUT /api/core/projects/{id} — Update a project.
///
/// # Expected Behavior
///
/// Requires authentication. Only team members can update.
///
/// # Side Effects
///
/// - Updates `core.projects` (database write).
/// - Publishes `project.updated` event to Redis.
pub async fn update_project(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<ProjectUpdate>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let id = path.into_inner();
    let mut redis_local = redis_conn.get_ref().clone();
    match ProjectService::update_project(
        pool.get_ref(),
        redis_local.as_mut(),
        id,
        user_id,
        &body.into_inner(),
    )
    .await
    {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => e.to_http_response(),
    }
}

/// DELETE /api/core/projects/{id} — Delete a project.
///
/// # Expected Behavior
///
/// Requires authentication. Only team members can delete.
///
/// # Side Effects
///
/// - Deletes from `core.projects` (database write, cascades).
pub async fn delete_project(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let id = path.into_inner();
    match ProjectService::delete_project(pool.get_ref(), id, user_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/projects/{id}/submit — Submit a project.
///
/// # Expected Behavior
///
/// Requires authentication. Only team members can submit.
///
/// # Side Effects
///
/// - Updates `core.projects` status and submission fields (database write).
/// - Publishes `project.submitted` event to Redis.
pub async fn submit_project(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let id = path.into_inner();
    let mut redis_local = redis_conn.get_ref().clone();
    match ProjectService::submit_project(pool.get_ref(), redis_local.as_mut(), id, user_id).await {
        Ok(project) => {
            openhack_common::metrics::inc_business_counter("core_projects_submitted_total");
            HttpResponse::Ok().json(project)
        }
        Err(e) => e.to_http_response(),
    }
}

/// POST /api/core/projects/{id}/demo — Upload demo file.
///
/// # Expected Behavior
///
/// Requires authentication. Receives multipart upload and proxies to
/// media service. Only team members can upload demos.
///
/// # Side Effects
///
/// - Sends HTTP request to media service (network write).
/// - Updates `core.projects` demo fields (database write).
pub async fn upload_demo(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    mut payload: Multipart,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let id = path.into_inner();

    let media_url = req
        .app_data::<web::Data<crate::MediaServiceUrl>>()
        .map_or_else(|| "http://media-svc:3010".to_string(), |d| d.0.clone());

    let mut file_name = "demo".to_string();
    let mut file_data = Vec::new();
    let mut content_type = "application/octet-stream".to_string();

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                log::warn!("Multipart field error: {e}");
                continue;
            }
        };

        content_type = field.content_type().map_or_else(
            || "application/octet-stream".to_string(),
            std::string::ToString::to_string,
        );

        if let Some(name) = field.content_disposition().get_filename() {
            file_name = name.to_string();
        }

        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(data) => file_data.extend_from_slice(&data),
                Err(e) => {
                    log::warn!("Multipart chunk error: {e}");
                }
            }
        }
    }

    match ProjectService::upload_demo(
        pool.get_ref(),
        &media_url,
        id,
        user_id,
        &file_name,
        file_data,
        &content_type,
    )
    .await
    {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => e.to_http_response(),
    }
}

/// DELETE /api/core/projects/{id}/demo — Delete demo file.
///
/// # Expected Behavior
///
/// Requires authentication. Only team members can delete demos.
///
/// # Side Effects
///
/// - Updates `core.projects` demo fields to NULL (database write).
pub async fn delete_demo(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let user_id = auth_user.user_id;

    let id = path.into_inner();
    match ProjectService::delete_demo(pool.get_ref(), id, user_id).await {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => e.to_http_response(),
    }
}
