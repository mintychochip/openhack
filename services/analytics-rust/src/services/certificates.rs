use crate::errors::AnalyticsError;
use crate::models::certificate::{
    BulkCertificateEntry, Certificate, CertificateCreateRequest, CertificateListResponse,
    CertificateQuery, CertificateResponse, CertificateRevokeRequest, VerificationResponse,
};
use chrono::Utc;
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CertificateService;

impl CertificateService {
    fn generate_verification_code() -> String {
        let mut rng = rand::thread_rng();
        let bytes: [u8; 16] = rng.gen();
        hex::encode(bytes)
    }

    pub async fn create(
        pool: &PgPool,
        data: &CertificateCreateRequest,
    ) -> Result<CertificateResponse, AnalyticsError> {
        let verification_code = Self::generate_verification_code();

        let metadata_json = serde_json::to_value(&data.metadata)
            .map_err(|e| AnalyticsError::Validation(format!("Invalid metadata: {}", e)))?;

        let cert = sqlx::query_as::<_, Certificate>(
            "INSERT INTO analytics.certificates (user_id, event_id, certificate_type, verification_code, metadata)
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(data.user_id)
        .bind(data.event_id)
        .bind(&data.certificate_type)
        .bind(&verification_code)
        .bind(metadata_json)
        .fetch_one(pool)
        .await?;

        Ok(cert.into())
    }

    pub async fn bulk_create(
        pool: &PgPool,
        event_id: Uuid,
        certificate_type: &str,
        entries: &[BulkCertificateEntry],
    ) -> Result<Vec<CertificateResponse>, AnalyticsError> {
        let mut results = Vec::new();

        for entry in entries {
            let verification_code = Self::generate_verification_code();
            let metadata_json = serde_json::to_value(&entry.metadata)
                .map_err(|e| AnalyticsError::Validation(format!("Invalid metadata: {}", e)))?;

            let cert = sqlx::query_as::<_, Certificate>(
                "INSERT INTO analytics.certificates (user_id, event_id, certificate_type, verification_code, metadata)
                 VALUES ($1, $2, $3, $4, $5) RETURNING *",
            )
            .bind(entry.user_id)
            .bind(event_id)
            .bind(certificate_type)
            .bind(&verification_code)
            .bind(metadata_json)
            .fetch_one(pool)
            .await?;

            results.push(cert.into());
        }

        Ok(results)
    }

    pub async fn get_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<CertificateResponse, AnalyticsError> {
        let cert = sqlx::query_as::<_, Certificate>(
            "SELECT * FROM analytics.certificates WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AnalyticsError::NotFound("Certificate".into(), id.to_string()))?;

        Ok(cert.into())
    }

    pub async fn list(
        pool: &PgPool,
        query: &CertificateQuery,
    ) -> Result<CertificateListResponse, AnalyticsError> {
        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        let mut where_clauses = Vec::new();
        let mut param_idx = 1;

        if let Some(user_id) = query.user_id {
            where_clauses.push(format!("user_id = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(event_id) = query.event_id {
            where_clauses.push(format!("event_id = ${param_idx}"));
            param_idx += 1;
        }
        if let Some(ref cert_type) = query.certificate_type {
            where_clauses.push(format!("certificate_type = ${param_idx}"));
            param_idx += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_query = format!(
            "SELECT COUNT(*) FROM analytics.certificates {}",
            where_clause
        );

        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let data_query = format!(
            "SELECT * FROM analytics.certificates {} ORDER BY issued_at DESC LIMIT ${} OFFSET ${}",
            where_clause,
            param_idx,
            param_idx + 1
        );

        let mut db_query = sqlx::query_as::<_, Certificate>(&data_query);

        if let Some(user_id) = query.user_id {
            db_query = db_query.bind(user_id);
        }
        if let Some(event_id) = query.event_id {
            db_query = db_query.bind(event_id);
        }
        if let Some(ref cert_type) = query.certificate_type {
            db_query = db_query.bind(cert_type);
        }

        let certs = db_query
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        Ok(CertificateListResponse {
            certificates: certs.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn verify(
        pool: &PgPool,
        verification_code: &str,
    ) -> Result<VerificationResponse, AnalyticsError> {
        let cert = sqlx::query_as::<_, Certificate>(
            "SELECT * FROM analytics.certificates WHERE verification_code = $1",
        )
        .bind(verification_code)
        .fetch_optional(pool)
        .await?;

        match cert {
            Some(c) if c.revoked_at.is_none() => Ok(VerificationResponse {
                is_valid: true,
                certificate: Some(c.into()),
                message: "Certificate is valid".to_string(),
            }),
            Some(c) => Ok(VerificationResponse {
                is_valid: false,
                certificate: Some(c.into()),
                message: "Certificate has been revoked".to_string(),
            }),
            None => Ok(VerificationResponse {
                is_valid: false,
                certificate: None,
                message: "Certificate not found".to_string(),
            }),
        }
    }

    pub async fn revoke(
        pool: &PgPool,
        id: Uuid,
        _request: &CertificateRevokeRequest,
    ) -> Result<CertificateResponse, AnalyticsError> {
        let cert = sqlx::query_as::<_, Certificate>(
            "UPDATE analytics.certificates SET revoked_at = NOW(), updated_at = NOW()
             WHERE id = $1 AND revoked_at IS NULL RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AnalyticsError::NotFound("Certificate".into(), id.to_string()))?;

        Ok(cert.into())
    }

    pub async fn update_pdf_reference(
        pool: &PgPool,
        certificate_id: Uuid,
        pdf_file_id: Uuid,
    ) -> Result<CertificateResponse, AnalyticsError> {
        let cert = sqlx::query_as::<_, Certificate>(
            "UPDATE analytics.certificates SET pdf_file_id = $1, updated_at = NOW()
             WHERE id = $2 RETURNING *",
        )
        .bind(pdf_file_id)
        .bind(certificate_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AnalyticsError::NotFound("Certificate".into(), certificate_id.to_string()))?;

        Ok(cert.into())
    }

    pub async fn generate_pdf(
        certificate: &CertificateResponse,
    ) -> Result<Vec<u8>, AnalyticsError> {
        let participant_name = certificate
            .metadata
            .as_ref()
            .map(|m| m.participant_name.as_str())
            .unwrap_or("Participant");

        let hackathon_name = certificate
            .metadata
            .as_ref()
            .map(|m| m.hackathon_name.as_str())
            .unwrap_or("the hackathon");

        let pdf_content = format!(
            r#"<?xml version="1.0"?>
<pdf>
  <title>Certificate of Participation</title>
  <body>
    <h1 align="center">Certificate</h1>
    <p align="center">This certifies that</p>
    <h2 align="center">{}</h2>
    <p align="center">has participated as a {}</p>
    <p align="center">for participation in {}</p>
    <p align="center">Issued: {}</p>
    <p align="center">Verification Code: {}</p>
  </body>
</pdf>"#,
            participant_name,
            certificate.certificate_type.to_uppercase(),
            hackathon_name,
            certificate.issued_at.format("%B %d, %Y"),
            certificate.verification_code
        );

        Ok(pdf_content.into_bytes())
    }
}
