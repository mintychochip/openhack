pub mod uploads;

use actix_web::web;

/// Register all media routes on the given service config.
///
/// # Expected Behavior
///
/// Configures all API endpoints under the `/api/media` prefix:
/// - POST `/upload` → upload a file (multipart/form-data)
/// - GET `/{file_id}` → download a file (streaming, Content-Disposition: attachment)
/// - DELETE `/{file_id}` → delete a file
/// - GET `/{file_id}/url` → get a presigned URL for a file
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
        web::scope("/api/media")
            .route("/upload", web::post().to(uploads::upload_file))
            .route("/{file_id}", web::get().to(uploads::download_file))
            .route("/{file_id}", web::delete().to(uploads::delete_file))
            .route("/{file_id}/url", web::get().to(uploads::get_signed_url)),
    );
}
