use crate::errors::MailError;
use crate::models::template::{
    MailTemplate, TemplateCreate, TemplateListResponse, TemplatePreviewRequest,
    TemplatePreviewResponse, TemplateResponse, TemplateUpdate,
};
use sqlx::PgPool;
use tera::Tera;
use uuid::Uuid;

pub struct TemplateService;

impl TemplateService {
    pub async fn create_template(
        pool: &PgPool,
        data: &TemplateCreate,
    ) -> Result<TemplateResponse, MailError> {
        let vars = serde_json::Value::Array(
            data.variables
                .iter()
                .map(|v| serde_json::Value::String(v.clone()))
                .collect(),
        );
        let row = sqlx::query_as::<_, MailTemplate>(
            "INSERT INTO mail.templates (name, subject_template, body_html_template, body_text_template, variables)
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(&data.name)
        .bind(&data.subject_template)
        .bind(&data.body_html_template)
        .bind(&data.body_text_template)
        .bind(&vars)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn list_templates(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<TemplateListResponse, MailError> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mail.templates")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let templates = sqlx::query_as::<_, MailTemplate>(
            "SELECT * FROM mail.templates ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(TemplateListResponse {
            templates: templates.into_iter().map(Into::into).collect(),
            total,
        })
    }

    pub async fn get_template(pool: &PgPool, id: Uuid) -> Result<TemplateResponse, MailError> {
        let row = sqlx::query_as::<_, MailTemplate>("SELECT * FROM mail.templates WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| MailError::TemplateNotFound(id.to_string()))?;

        Ok(row.into())
    }

    pub async fn update_template(
        pool: &PgPool,
        id: Uuid,
        data: &TemplateUpdate,
    ) -> Result<TemplateResponse, MailError> {
        let mut updates: Vec<String> = Vec::new();
        let mut param_idx = 1;
        let mut has_update = false;

        if data.name.is_some() {
            updates.push(format!("name = ${param_idx}"));
            param_idx += 1;
            has_update = true;
        }
        if data.subject_template.is_some() {
            updates.push(format!("subject_template = ${param_idx}"));
            param_idx += 1;
            has_update = true;
        }
        if data.body_html_template.is_some() {
            updates.push(format!("body_html_template = ${param_idx}"));
            param_idx += 1;
            has_update = true;
        }
        if data.body_text_template.is_some() {
            updates.push(format!("body_text_template = ${param_idx}"));
            param_idx += 1;
            has_update = true;
        }
        if data.variables.is_some() {
            updates.push(format!("variables = ${param_idx}"));
            param_idx += 1;
            has_update = true;
        }

        if !has_update {
            return Self::get_template(pool, id).await;
        }

        updates.push("updated_at = NOW()".to_string());

        let sql = format!(
            "UPDATE mail.templates SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_idx
        );

        let mut query = sqlx::query_as::<_, MailTemplate>(&sql);

        if let Some(ref v) = data.name {
            query = query.bind(v);
        }
        if let Some(ref v) = data.subject_template {
            query = query.bind(v);
        }
        if let Some(ref v) = data.body_html_template {
            query = query.bind(v);
        }
        if let Some(ref v) = data.body_text_template {
            query = query.bind(v);
        }
        if let Some(ref v) = data.variables {
            let vars = serde_json::Value::Array(
                v.iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            );
            query = query.bind(vars);
        }

        let row = query.bind(id).fetch_optional(pool).await?;
        let row = row.ok_or_else(|| MailError::TemplateNotFound(id.to_string()))?;

        Ok(row.into())
    }

    pub async fn delete_template(pool: &PgPool, id: Uuid) -> Result<bool, MailError> {
        let result = sqlx::query("DELETE FROM mail.templates WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn preview_template(
        pool: &PgPool,
        id: Uuid,
        req: &TemplatePreviewRequest,
    ) -> Result<TemplatePreviewResponse, MailError> {
        let tmpl = sqlx::query_as::<_, MailTemplate>("SELECT * FROM mail.templates WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| MailError::TemplateNotFound(id.to_string()))?;

        let context = tera::Context::from_serialize(&req.variables)
            .map_err(|e| MailError::TemplateRender(e.to_string()))?;

        let subject = render_string(&tmpl.subject_template, &context)?;
        let body_html = render_string(&tmpl.body_html_template, &context)?;
        let body_text = render_string(&tmpl.body_text_template, &context)?;

        Ok(TemplatePreviewResponse {
            subject,
            body_html,
            body_text,
        })
    }
}

fn render_string(template_str: &str, context: &tera::Context) -> Result<String, MailError> {
    let mut tera = Tera::default();
    tera.add_raw_template("render", template_str)
        .map_err(|e| MailError::TemplateRender(e.to_string()))?;
    tera.render("render", context)
        .map_err(|e| MailError::TemplateRender(e.to_string()))
}
