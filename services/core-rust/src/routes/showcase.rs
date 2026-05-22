use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::get_auth_user;
use crate::models::project::{FavoriteRequest, FeatureRequest, ShowcaseQuery};
use crate::services::showcase::ShowcaseService;

/// GET /api/core/showcase — Get featured projects for gallery.
pub async fn get_showcase(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    query: web::Query<ShowcaseQuery>,
) -> HttpResponse {
    let user_id = get_auth_user(&req).ok().map(|u| u.user_id);
    
    match ShowcaseService::get_showcase(pool.get_ref(), &query, user_id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "database_error"})),
    }
}

/// PUT /api/core/admin/projects/{id}/feature — Feature/unfeature a project (admin).
pub async fn set_featured(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<FeatureRequest>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if !auth_user.is_admin() {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "forbidden",
            "message": "Admin access required"
        }));
    }

    let project_id = path.into_inner();
    
    match ShowcaseService::set_featured(pool.get_ref(), project_id, &body).await {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "database_error"})),
    }
}

/// POST /api/core/projects/{id}/favorite — Toggle favorite.
pub async fn toggle_favorite(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<FavoriteRequest>,
) -> HttpResponse {
    let auth_user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let project_id = path.into_inner();
    
    match ShowcaseService::toggle_favorite(pool.get_ref(), project_id, auth_user.user_id, &body).await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "database_error"})),
    }
}

/// GET /api/core/projects/{id}/favorite — Get favorite info.
pub async fn get_favorite(
    pool: web::Data<PgPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let user_id = get_auth_user(&req).ok().map(|u| u.user_id);
    let project_id = path.into_inner();
    
    match ShowcaseService::get_favorite_info(pool.get_ref(), project_id, user_id).await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({"error": "database_error"})),
    }
}
