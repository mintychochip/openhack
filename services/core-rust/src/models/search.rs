use serde::{Deserialize, Serialize};

/// Unified search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SearchResult {
    Project(ProjectResult),
    Team(TeamResult),
    User(UserResult),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResult {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub team_name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub featured: bool,
    pub favorite_count: i32,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamResult {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i32,
    pub looking_for_members: bool,
    pub seeking_skills: Vec<String>,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResult {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub skills: Vec<String>,
    pub looking_for_team: bool,
    pub rank: f64,
}

/// Search query parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub types: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
