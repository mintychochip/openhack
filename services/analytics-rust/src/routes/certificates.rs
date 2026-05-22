use crate::middleware::auth::{get_auth_user, require_admin};
use crate::models::certificate::{
    CertificateBulkCreateRequest, CertificateCreateRequest, CertificatePdfRequest,
    CertificateQuery, CertificateRevokeRequest,
};
use crate::services::certificates::CertificateService;
use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_certificate(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CertificateCreateRequest>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    match CertificateService::create(pool.get_ref(), &body.into_inner()).await {
        Ok(cert) => HttpResponse::Created().json(cert),
        Err(e) => e.to_http_response(),
    }
}

pub async fn bulk_create_certificates(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CertificateBulkCreateRequest>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let data = body.into_inner();
    match CertificateService::bulk_create(
        pool.get_ref(),
        data.event_id,
        &data.certificate_type,
        &data.certificates,
    )
    .await
    {
        Ok(certs) => HttpResponse::Created().json(certs),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_certificate(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    match CertificateService::get_by_id(pool.get_ref(), id).await {
        Ok(cert) => HttpResponse::Ok().json(cert),
        Err(e) => e.to_http_response(),
    }
}

pub async fn list_certificates(
    pool: web::Data<PgPool>,
    query: web::Query<CertificateQuery>,
) -> HttpResponse {
    match CertificateService::list(pool.get_ref(), &query.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => e.to_http_response(),
    }
}

pub async fn verify_certificate(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let code = path.into_inner();
    match CertificateService::verify(pool.get_ref(), &code).await {
        Ok(result) => {
            if result.is_valid {
                HttpResponse::Ok().json(result)
            } else {
                HttpResponse::BadRequest().json(result)
            }
        }
        Err(e) => e.to_http_response(),
    }
}

pub async fn revoke_certificate(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<CertificateRevokeRequest>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let id = path.into_inner();
    match CertificateService::revoke(pool.get_ref(), id, &body.into_inner()).await {
        Ok(cert) => HttpResponse::Ok().json(cert),
        Err(e) => e.to_http_response(),
    }
}

pub async fn get_certificate_pdf(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    query: web::Query<CertificatePdfRequest>,
) -> HttpResponse {
    let id = path.into_inner();

    let cert = match CertificateService::get_by_id(pool.get_ref(), id).await {
        Ok(c) => c,
        Err(e) => return e.to_http_response(),
    };

    if cert.is_revoked {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Certificate has been revoked"
        }));
    }

    match CertificateService::generate_pdf(&cert).await {
        Ok(pdf_data) => HttpResponse::Ok()
            .content_type("application/pdf")
            .append_header((
                "Content-Disposition",
                format!("attachment; filename=\"certificate_{}.pdf\"", cert.verification_code),
            ))
            .body(pdf_data),
        Err(e) => e.to_http_response(),
    }
}

pub async fn update_certificate_pdf(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let user = match get_auth_user(&req) {
        Ok(u) => u,
        Err(e) => return e.to_http_response(),
    };

    if let Err(e) = require_admin(&user) {
        return e.to_http_response();
    }

    let certificate_id = path.into_inner();
    let pdf_file_id = body.get("pdf_file_id").and_then(|v| v.as_str()).and_then(|s| {
        Uuid::parse_str(s).ok()
    });

    let Some(pdf_file_id) = pdf_file_id else {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "pdf_file_id is required"
        }));
    };

    match CertificateService::update_pdf_reference(pool.get_ref(), certificate_id, pdf_file_id).await {
        Ok(cert) => HttpResponse::Ok().json(cert),
        Err(e) => e.to_http_response(),
    }
}
