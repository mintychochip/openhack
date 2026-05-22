use crate::middleware::auth::{get_auth_user, require_judge_or_admin};
use crate::models::score::{ScoreCreate, ScoreQuery, ScoreUpdate};
use crate::services::judging::JudgingService;
use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn submit_score(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<ScoreCreate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_judge_or_admin(&user) {
        return e.to_http_response();
    }

    match JudgingService::submit_score(pool.get_ref(), &body.into_inner()).await {
        Ok(s) => {
            openhack_common::metrics::inc_business_counter("judging_scores_submitted_total");
            HttpResponse::Created().json(s)
        }
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_my_scores(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<ScoreQuery>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    let judge_id = query.judge_id.unwrap_or(user.user_id);

    if judge_id != user.user_id && !user.is_admin_or_organizer() {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "Can only view your own scores"}));
    }

    match JudgingService::get_scores_by_judge(pool.get_ref(), judge_id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_project_scores(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let project_id = path.into_inner();
    match JudgingService::get_scores_for_project(pool.get_ref(), project_id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_score(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<ScoreUpdate>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_judge_or_admin(&user) {
        return e.to_http_response();
    }

    let score_id = path.into_inner();
    match JudgingService::update_score(pool.get_ref(), score_id, &body.into_inner()).await {
        Ok(s) => HttpResponse::Ok().json(s),
        Err(e) => e.to_http_response(),
    }
}
