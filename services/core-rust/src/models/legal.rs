use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HackathonRule {
    pub id: Uuid,
    pub event_id: Uuid,
    pub title: String,
    pub rules_text: String,
    pub version: i32,
    pub published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HackathonRuleCreate {
    pub event_id: Uuid,
    pub title: String,
    pub rules_text: String,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HackathonRuleUpdate {
    pub title: Option<String>,
    pub rules_text: Option<String>,
    pub published: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HackathonRuleResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub title: String,
    pub rules_text: String,
    pub version: i32,
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<HackathonRule> for HackathonRuleResponse {
    fn from(r: HackathonRule) -> Self {
        Self {
            id: r.id,
            event_id: r.event_id,
            title: r.title,
            rules_text: r.rules_text,
            version: r.version,
            is_published: r.published,
            published_at: r.published_at,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HackathonRuleListResponse {
    pub rules: Vec<HackathonRuleResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HackathonRuleQuery {
    pub event_id: Option<Uuid>,
    pub published_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PrizeFulfillment {
    pub id: Uuid,
    pub prize_id: Uuid,
    pub sponsor_id: Option<Uuid>,
    pub winner_project_id: Uuid,
    pub winner_user_id: Uuid,
    pub status: String,
    pub shipping_address: Option<serde_json::Value>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub tracking_number: Option<String>,
    pub carrier: Option<String>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingAddress {
    pub name: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrizeFulfillmentCreate {
    pub prize_id: Uuid,
    pub sponsor_id: Option<Uuid>,
    pub winner_project_id: Uuid,
    pub winner_user_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrizeFulfillmentUpdate {
    pub status: Option<String>,
    pub shipping_address: Option<ShippingAddress>,
    pub tracking_number: Option<String>,
    pub carrier: Option<String>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrizeFulfillmentResponse {
    pub id: Uuid,
    pub prize_id: Uuid,
    pub sponsor_id: Option<Uuid>,
    pub winner_project_id: Uuid,
    pub winner_user_id: Uuid,
    pub status: String,
    pub shipping_address: Option<ShippingAddress>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub tracking_number: Option<String>,
    pub carrier: Option<String>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

impl From<PrizeFulfillment> for PrizeFulfillmentResponse {
    fn from(f: PrizeFulfillment) -> Self {
        Self {
            id: f.id,
            prize_id: f.prize_id,
            sponsor_id: f.sponsor_id,
            winner_project_id: f.winner_project_id,
            winner_user_id: f.winner_user_id,
            status: f.status,
            shipping_address: f.shipping_address.and_then(|addr| serde_json::from_value(addr).ok()),
            claimed_at: f.claimed_at,
            shipped_at: f.shipped_at,
            tracking_number: f.tracking_number,
            carrier: f.carrier,
            estimated_delivery: f.estimated_delivery,
            delivered_at: f.delivered_at,
            notes: f.notes,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PrizeFulfillmentListResponse {
    pub fulfillments: Vec<PrizeFulfillmentResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrizeFulfillmentQuery {
    pub prize_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
