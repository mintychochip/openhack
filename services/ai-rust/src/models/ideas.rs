use serde::{Deserialize, Serialize};

/// Request body for the idea generation endpoint.
///
/// # Expected Behavior
///
/// Theme and interests are required to generate relevant hackathon ideas.
/// `team_size`, difficulty, and `num_ideas` are optional hints for the LLM.
/// `num_ideas` defaults to 3 if not provided.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Deserialize)]
pub struct IdeaRequest {
    pub theme: String,
    pub interests: Vec<String>,
    pub team_size: Option<u32>,
    pub difficulty: Option<String>,
    pub num_ideas: Option<u32>,
}

/// A single generated hackathon idea from the LLM.
///
/// # Expected Behavior
///
/// Contains the idea title, detailed description, suggested technology stack,
/// difficulty level, and a feasibility score from 0.0 to 1.0.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedIdea {
    pub title: String,
    pub description: String,
    pub technologies: Vec<String>,
    pub difficulty: String,
    pub feasibility_score: f64,
}

/// Response body for the idea generation endpoint.
///
/// # Expected Behavior
///
/// Contains the list of generated ideas. May be empty if the LLM fails
/// to produce structured output.
///
/// # Side Effects
///
/// None. Pure data structure.
#[derive(Debug, Clone, Serialize)]
pub struct IdeaResponse {
    pub ideas: Vec<GeneratedIdea>,
}
