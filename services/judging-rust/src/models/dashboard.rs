use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DashboardResponse {
    pub total_projects: i64,
    pub total_assigned: i64,
    pub total_completed: i64,
    pub total_pending: i64,
    pub avg_score: Option<f64>,
}
