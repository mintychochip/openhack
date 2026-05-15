pub mod admin;
pub mod login;
pub mod logout;
pub mod me;
pub mod mfa;
pub mod oauth;
pub mod password;
pub mod refresh;
pub mod register;

use actix_web::web;

/// Register all auth routes on the given service config.
///
/// # Expected Behavior
///
/// Configures all API endpoints under the `/api/auth` prefix:
/// - POST `/register` → user registration (rate limited 10/min)
/// - POST `/login` → user login (rate limited 10/min)
/// - POST `/refresh` → refresh access token
/// - POST `/logout` → logout and revoke sessions (auth required)
/// - GET `/me` → get current user profile (auth required)
/// - PUT `/me` → update current user profile (auth required)
/// - POST `/forgot-password` → request password reset (rate limited 100/min)
/// - POST `/reset-password` → reset password with token (rate limited 100/min)
/// - POST `/mfa/enable` → enable MFA (auth required)
/// - POST `/mfa/verify` → verify MFA setup (auth required)
/// - POST `/mfa/disable` → disable MFA (auth required)
/// - GET `/oauth/{provider}` → redirect to OAuth provider
/// - GET `/oauth/{provider}/callback` → handle OAuth callback
/// - GET `/users` → paginated user list (admin only)
/// - DELETE `/users/{id}` → delete user (admin only)
/// - POST `/users/{id}/role` → update user role (admin only)
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
        web::scope("/api/auth")
            .route("/register", web::post().to(register::register))
            .route("/login", web::post().to(login::login))
            .route("/refresh", web::post().to(refresh::refresh))
            .route("/logout", web::post().to(logout::logout))
            .route("/me", web::get().to(me::get_me))
            .route("/me", web::put().to(me::update_me))
            .route(
                "/forgot-password",
                web::post().to(password::forgot_password),
            )
            .route("/reset-password", web::post().to(password::reset_password))
            .route("/mfa/enable", web::post().to(mfa::enable_mfa))
            .route("/mfa/verify", web::post().to(mfa::verify_mfa))
            .route("/mfa/disable", web::post().to(mfa::disable_mfa))
            .route("/oauth/{provider}", web::get().to(oauth::oauth_redirect))
            .route(
                "/oauth/{provider}/callback",
                web::get().to(oauth::oauth_callback),
            )
            .route("/users", web::get().to(admin::list_users))
            .route("/users/{id}", web::delete().to(admin::delete_user))
            .route("/users/{id}/role", web::post().to(admin::update_role)),
    );
}
