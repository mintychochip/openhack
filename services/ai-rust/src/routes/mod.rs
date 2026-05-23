pub mod chat;
pub mod code_review;
pub mod ideas;
pub mod insights;
pub mod knowledge;
pub mod team_matcher;
pub mod brand;

use actix_web::web;

/// Register all AI service routes on the given service config.
///
/// # Expected Behavior
///
/// Configures all API endpoints under the `/api/ai` prefix:
/// - POST `/chat` → send chat message with RAG
/// - GET `/conversations/{id}/history` → get conversation with messages
/// - DELETE `/conversations/{id}` → delete conversation
/// - POST `/knowledge` → create knowledge entry (auto-embed)
/// - GET `/knowledge` → list knowledge (with filters and pagination)
/// - GET `/knowledge/{id}` → get knowledge entry
/// - PUT `/knowledge/{id}` → update knowledge (re-embed if content changed)
/// - DELETE `/knowledge/{id}` → delete knowledge entry
/// - POST `/ideas` → generate hackathon ideas
/// - POST `/team-matcher` → match teammates
/// - POST `/code-review` → review code
/// - GET `/insights` → get cached insights
/// - POST `/insights/generate` → generate insights
/// - POST `/brand-normalize` → normalize scraped brand data into a theme
///
/// # Errors
///
/// None. Route registration is infallible.
///
/// # Side Effects
///
/// - Registers HTTP route handlers with the Actix-web service config.
///   No I/O or network calls occur during registration.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/ai")
            .route("/chat", web::post().to(chat::chat))
            .route(
                "/conversations/{id}/history",
                web::get().to(chat::get_history),
            )
            .route(
                "/conversations/{id}",
                web::delete().to(chat::delete_conversation),
            )
            .route("/knowledge", web::post().to(knowledge::create_knowledge))
            .route("/knowledge", web::get().to(knowledge::list_knowledge))
            .route("/knowledge/{id}", web::get().to(knowledge::get_knowledge))
            .route(
                "/knowledge/{id}",
                web::put().to(knowledge::update_knowledge),
            )
            .route(
                "/knowledge/{id}",
                web::delete().to(knowledge::delete_knowledge),
            )
            .route("/ideas", web::post().to(ideas::generate_ideas))
            .route(
                "/team-matcher",
                web::post().to(team_matcher::match_teammates),
            )
            .route("/code-review", web::post().to(code_review::review_code))
            .route("/insights", web::get().to(insights::get_insights))
            .route(
                "/insights/generate",
                web::post().to(insights::generate_insights),
            )
            .route(
                "/brand-normalize",
                web::post().to(brand::brand_normalize),
            ),
    );
}
