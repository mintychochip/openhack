use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MailTemplate {
    pub id: Uuid,
    pub name: String,
    pub subject_template: String,
    pub body_html_template: String,
    pub body_text_template: String,
    pub variables: serde_json::Value,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub name: String,
    pub subject_template: String,
    pub body_html_template: String,
    pub body_text_template: String,
    pub variables: Vec<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<MailTemplate> for TemplateResponse {
    fn from(t: MailTemplate) -> Self {
        let vars: Vec<String> = t
            .variables
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            id: t.id,
            name: t.name,
            subject_template: t.subject_template,
            body_html_template: t.body_html_template,
            body_text_template: t.body_text_template,
            variables: vars,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateCreate {
    pub name: String,
    pub subject_template: String,
    pub body_html_template: String,
    #[serde(default)]
    pub body_text_template: String,
    #[serde(default)]
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateUpdate {
    pub name: Option<String>,
    pub subject_template: Option<String>,
    pub body_html_template: Option<String>,
    pub body_text_template: Option<String>,
    pub variables: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateListResponse {
    pub templates: Vec<TemplateResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplatePreviewRequest {
    #[serde(default)]
    pub variables: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplatePreviewResponse {
    pub subject: String,
    pub body_html: String,
    pub body_text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
