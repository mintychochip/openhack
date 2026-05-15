use crate::models::score::{ScoreCreate, ScoreQuery, ScoreUpdate};
use crate::services::judging::JudgingService;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn submit_score(pool: web::Data<PgPool>, body: web::Json<ScoreCreate>) -> HttpResponse {
    match JudgingService::submit_score(pool.get_ref(), &body.into_inner()).await {
        Ok(s) => HttpResponse::Created().json(s),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_my_scores(pool: web::Data<PgPool>, query: web::Query<ScoreQuery>) -> HttpResponse {
    let Some(judge_id) = query.judge_id else {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "judge_id is required"}));
    };
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
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<ScoreUpdate>,
) -> HttpResponse {
    let score_id = path.into_inner();
    match JudgingService::update_score(pool.get_ref(), score_id, &body.into_inner()).await {
        Ok(s) => HttpResponse::Ok().json(s),
        Err(e) => e.to_http_response(),
    }
}
