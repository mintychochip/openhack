use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Certificate {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub certificate_type: String,
    pub issued_at: DateTime<Utc>,
    pub verification_code: String,
    pub revoked_at: Option<DateTime<Utc>>,
    pub pdf_file_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateMetadata {
    pub team_name: Option<String>,
    pub project_name: Option<String>,
    pub placement: Option<String>,
    pub hackathon_name: String,
    pub participant_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CertificateCreateRequest {
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub certificate_type: String,
    pub metadata: CertificateMetadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CertificateBulkCreateRequest {
    pub event_id: Uuid,
    pub certificate_type: String,
    pub certificates: Vec<BulkCertificateEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkCertificateEntry {
    pub user_id: Uuid,
    pub metadata: CertificateMetadata,
}

#[derive(Debug, Clone, Serialize)]
pub struct CertificateResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub certificate_type: String,
    pub issued_at: DateTime<Utc>,
    pub verification_code: String,
    pub is_revoked: bool,
    pub pdf_file_id: Option<Uuid>,
    pub metadata: Option<CertificateMetadata>,
}

impl From<Certificate> for CertificateResponse {
    fn from(c: Certificate) -> Self {
        Self {
            id: c.id,
            user_id: c.user_id,
            event_id: c.event_id,
            certificate_type: c.certificate_type,
            issued_at: c.issued_at,
            verification_code: c.verification_code,
            is_revoked: c.revoked_at.is_some(),
            pdf_file_id: c.pdf_file_id,
            metadata: c.metadata.and_then(|m| serde_json::from_value(m).ok()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CertificateListResponse {
    pub certificates: Vec<CertificateResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CertificateQuery {
    pub user_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub certificate_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationResponse {
    pub is_valid: bool,
    pub certificate: Option<CertificateResponse>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CertificateRevokeRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CertificatePdfRequest {
    pub template_id: Option<String>,
    pub custom_data: Option<serde_json::Value>,
}
