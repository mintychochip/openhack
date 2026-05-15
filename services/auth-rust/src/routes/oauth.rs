use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;

use crate::config::Config;
use crate::errors::AuthError;
use crate::events::publisher;
use crate::models::oauth::OAuthProvider;
use crate::services::{auth, oauth as oauth_service, session};

/// GET /api/auth/oauth/{provider} — Redirect to OAuth provider for authentication.
///
/// # Expected Behavior
///
/// Validates that the provider is supported (github, google, discord).
/// Generates the OAuth authorization URL with `client_id`, `redirect_uri`,
/// state (CSRF token), scope, and `response_type=code`. Sets the state
/// in an `HttpOnly`, Secure, SameSite=Lax cookie named `oauth_state_{provider}`
/// with a 10-minute Max-Age. Redirects (302) the user to the provider's
/// authorization page.
///
/// # Errors
///
/// Returns 400 if the provider is not supported or not configured.
/// Returns 500 on internal errors.
///
/// # Side Effects
///
/// - Sets a cookie on the response (`oauth_state`_{provider}).
/// - No database or Redis writes.
pub async fn oauth_redirect(
    _pool: web::Data<PgPool>,
    _redis_conn: web::Data<Option<MultiplexedConnection>>,
    config: web::Data<Config>,
    req: HttpRequest,
    provider: web::Path<String>,
) -> HttpResponse {
    let provider_str = provider.into_inner();
    let Some(provider) = OAuthProvider::from_str_loose(&provider_str) else {
        return AuthError::BadRequest(format!(
            "Unsupported OAuth provider: {provider_str}. Supported: github, google, discord"
        ))
        .error_response();
    };

    let base_url = extract_base_url(&req);

    let (auth_url, state) =
        match oauth_service::get_authorization_url(&provider, config.get_ref(), &base_url) {
            Ok(result) => result,
            Err(e) => return e.error_response(),
        };

    let cookie_name = format!("oauth_state_{}", provider.as_str());
    let cookie = actix_web::cookie::Cookie::build(&cookie_name, &state)
        .path("/api/auth/oauth")
        .http_only(true)
        .secure(false)
        .same_site(actix_web::cookie::SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::minutes(10))
        .finish();

    HttpResponse::TemporaryRedirect()
        .append_header(("Location", auth_url))
        .cookie(cookie)
        .finish()
}

/// GET /api/auth/oauth/{provider}/callback — Handle OAuth provider callback.
///
/// # Expected Behavior
///
/// The OAuth provider redirects here after the user authorizes (or denies)
/// the application. Validates the `state` query parameter against the
/// `oauth_state_{provider}` cookie to prevent CSRF attacks. Exchanges the
/// authorization `code` for access tokens via the provider's token endpoint.
/// Fetches the user profile from the provider. Finds or creates a local user
/// in `auth.users` and links the OAuth account in `auth.oauth_accounts`.
/// Creates a new session with refresh token. Generates a JWT access token.
/// Redirects the user to the frontend with tokens in query parameters:
/// `{FRONTEND_URL}/auth/callback?accessToken=xxx&refreshToken=yyy&expiresIn=900`.
///
/// # Errors
///
/// Returns 400 if the state does not match the cookie, the provider is
/// unsupported, or the code exchange fails.
/// Returns 500 on database or internal errors.
///
/// # Side Effects
///
/// - Reads and deletes the `oauth_state_{provider}` cookie.
/// - Makes HTTP POST to the provider's token endpoint (network).
/// - Makes HTTP GET to the provider's user info endpoint (network).
/// - Reads/writes to `auth.users` and `auth.oauth_accounts` tables.
/// - Writes to `auth.sessions` table (insert).
/// - Writes to Redis (session cache).
/// - Publishes `user.logged_in` event.
/// - Sets a response cookie to clear the state cookie.
#[allow(clippy::too_many_lines)]
pub async fn oauth_callback(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<Option<MultiplexedConnection>>,
    config: web::Data<Config>,
    req: HttpRequest,
    provider: web::Path<String>,
    query: web::Query<serde_json::Value>,
) -> HttpResponse {
    let provider_str = provider.into_inner();
    let Some(provider) = OAuthProvider::from_str_loose(&provider_str) else {
        return AuthError::BadRequest(format!("Unsupported OAuth provider: {provider_str}"))
            .error_response();
    };

    let state = query["state"].as_str().unwrap_or("").to_string();
    let code = query["code"].as_str().unwrap_or("").to_string();

    if state.is_empty() || code.is_empty() {
        let redirect_url = format!(
            "{}/auth/callback?error=oauth_failed&reason=missing_params",
            config.frontend_url
        );
        return HttpResponse::TemporaryRedirect()
            .append_header(("Location", redirect_url))
            .finish();
    }

    let cookie_name = format!("oauth_state_{}", provider.as_str());
    let cookie_state = req
        .cookie(&cookie_name)
        .map(|c| c.value().to_string())
        .unwrap_or_default();

    if cookie_state.is_empty() || cookie_state != state {
        let redirect_url = format!(
            "{}/auth/callback?error=oauth_failed&reason=invalid_state",
            config.frontend_url
        );
        return HttpResponse::TemporaryRedirect()
            .append_header(("Location", redirect_url))
            .finish();
    }

    let base_url = extract_base_url(&req);

    let (user_info, access_token) = match oauth_service::exchange_code_and_get_user_info(
        &provider,
        &code,
        config.get_ref(),
        &base_url,
    )
    .await
    {
        Ok(result) => result,
        Err(_e) => {
            let redirect_url = format!(
                "{}/auth/callback?error=oauth_failed&reason=user_not_found",
                config.frontend_url
            );
            return HttpResponse::TemporaryRedirect()
                .append_header(("Location", redirect_url))
                .finish();
        }
    };

    let user_id = match oauth_service::find_or_create_user(
        pool.get_ref(),
        &provider,
        &user_info,
        &access_token,
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            let redirect_url = format!(
                "{}/auth/callback?error=oauth_failed&reason={}",
                config.frontend_url,
                urlencoding::encode(&e.to_string())
            );
            return HttpResponse::TemporaryRedirect()
                .append_header(("Location", redirect_url))
                .finish();
        }
    };

    let Ok(Some(user)) = auth::find_user_by_id(pool.get_ref(), user_id).await else {
        let redirect_url = format!(
            "{}/auth/callback?error=oauth_failed&reason=user_not_found",
            config.frontend_url
        );
        return HttpResponse::TemporaryRedirect()
            .append_header(("Location", redirect_url))
            .finish();
    };

    let refresh_token = match session::create_session(
        pool.get_ref(),
        redis_conn.get_ref().as_ref(),
        user.id,
        None,
        None,
        config.get_ref(),
    )
    .await
    {
        Ok(token) => token,
        Err(_e) => {
            let redirect_url = format!(
                "{}/auth/callback?error=oauth_failed&reason=invalid_state",
                config.frontend_url
            );
            return HttpResponse::TemporaryRedirect()
                .append_header(("Location", redirect_url))
                .finish();
        }
    };

    let jwt_token = match auth::create_access_token(&user, config.get_ref()) {
        Ok(token) => token,
        Err(_e) => {
            let redirect_url = format!(
                "{}/auth/callback?error=oauth_failed&reason=token_error",
                config.frontend_url
            );
            return HttpResponse::TemporaryRedirect()
                .append_header(("Location", redirect_url))
                .finish();
        }
    };

    if let Err(e) = auth::update_last_login(pool.get_ref(), user.id).await {
        log::warn!("Failed to update last_login_at: {e}");
    }

    publisher::user_logged_in(
        redis_conn.get_ref().as_ref(),
        &user.id.to_string(),
        &user.email,
    )
    .await;

    let clear_cookie = actix_web::cookie::Cookie::build(&cookie_name, "")
        .path("/api/auth/oauth")
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .finish();

    let redirect_url = format!(
        "{}/auth/callback?accessToken={}&refreshToken={}&expiresIn={}",
        config.frontend_url,
        urlencoding::encode(&jwt_token),
        urlencoding::encode(&refresh_token),
        config.jwt_expiry_secs,
    );

    HttpResponse::TemporaryRedirect()
        .append_header(("Location", redirect_url))
        .cookie(clear_cookie)
        .finish()
}

/// Extract the base URL (scheme + host) from the request for constructing
/// the OAuth `redirect_uri`.
///
/// # Expected Behavior
///
/// Reads the scheme and host from the request's connection info. Defaults
/// to "<http://localhost:3001>" if the host cannot be determined.
///
/// # Errors
///
/// None. Returns a default URL if extraction fails.
///
/// # Side Effects
///
/// None. Reads request metadata only.
fn extract_base_url(req: &HttpRequest) -> String {
    let conn_info = req.connection_info();
    let scheme = conn_info.scheme();
    let host = conn_info.host();
    format!("{scheme}://{host}")
}
