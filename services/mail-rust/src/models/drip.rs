use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DripCampaign {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_event: String,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DripStep {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub step_order: i32,
    pub delay_hours: i32,
    pub template_id: Option<Uuid>,
    pub subject: String,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DripEnrollment {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub user_id: Uuid,
    pub trigger_event: String,
    pub trigger_data: Option<serde_json::Value>,
    pub current_step: i32,
    pub status: String,
    pub enrolled_at: DateTime<Utc>,
    pub next_step_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DripLog {
    pub id: Uuid,
    pub enrollment_id: Uuid,
    pub step_id: Option<Uuid>,
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DripCampaignCreate {
    pub name: String,
    pub description: Option<String>,
    pub trigger_event: String,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DripCampaignUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DripCampaignResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_event: String,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DripCampaign> for DripCampaignResponse {
    fn from(c: DripCampaign) -> Self {
        Self {
            id: c.id,
            name: c.name,
            description: c.description,
            trigger_event: c.trigger_event,
            status: c.status,
            created_by: c.created_by,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DripCampaignListResponse {
    pub campaigns: Vec<DripCampaignResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DripCampaignQuery {
    pub trigger_event: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DripStepCreate {
    pub step_order: i32,
    pub delay_hours: i32,
    pub template_id: Option<Uuid>,
    pub subject: String,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DripStepUpdate {
    pub step_order: Option<i32>,
    pub delay_hours: Option<i32>,
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DripStepResponse {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub step_order: i32,
    pub delay_hours: i32,
    pub template_id: Option<Uuid>,
    pub subject: String,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
}

impl From<DripStep> for DripStepResponse {
    fn from(s: DripStep) -> Self {
        Self {
            id: s.id,
            campaign_id: s.campaign_id,
            step_order: s.step_order,
            delay_hours: s.delay_hours,
            template_id: s.template_id,
            subject: s.subject,
            body_html: s.body_html,
            body_text: s.body_text,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DripStepListResponse {
    pub steps: Vec<DripStepResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DripEnrollRequest {
    pub user_id: Uuid,
    pub trigger_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DripEnrollmentResponse {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub user_id: Uuid,
    pub trigger_event: String,
    pub current_step: i32,
    pub status: String,
    pub enrolled_at: DateTime<Utc>,
    pub next_step_at: Option<DateTime<Utc>>,
}

impl From<DripEnrollment> for DripEnrollmentResponse {
    fn from(e: DripEnrollment) -> Self {
        Self {
            id: e.id,
            campaign_id: e.campaign_id,
            user_id: e.user_id,
            trigger_event: e.trigger_event,
            current_step: e.current_step,
            status: e.status,
            enrolled_at: e.enrolled_at,
            next_step_at: e.next_step_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DripEnrollmentListResponse {
    pub enrollments: Vec<DripEnrollmentResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DripLogResponse {
    pub id: Uuid,
    pub enrollment_id: Uuid,
    pub step_id: Option<Uuid>,
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl From<DripLog> for DripLogResponse {
    fn from(l: DripLog) -> Self {
        Self {
            id: l.id,
            enrollment_id: l.enrollment_id,
            step_id: l.step_id,
            status: l.status,
            sent_at: l.sent_at,
            error_message: l.error_message,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DripLogListResponse {
    pub logs: Vec<DripLogResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessPendingResponse {
    pub processed_count: i32,
    pub sent_count: i32,
    pub failed_count: i32,
    pub skipped_count: i32,
}
