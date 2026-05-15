use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::AuthError;
use crate::models::oauth::{OAuthAccount, OAuthProvider, ProviderUserInfo};

/// Generate the OAuth authorization URL for a provider.
///
/// # Expected Behavior
///
/// Constructs the full authorization URL with `client_id`, `redirect_uri`,
/// state, scope, and `response_type=code` parameters. The `redirect_uri`
/// is constructed from the service's base URL and the provider name.
/// Returns the authorization URL and the CSRF state token.
///
/// # Errors
///
/// Returns `AuthError::OAuth` if the provider has no configured
/// client credentials (`client_id` or `client_secret` empty).
///
/// # Side Effects
///
/// None. Pure URL construction.
pub fn get_authorization_url(
    provider: &OAuthProvider,
    config: &Config,
    base_url: &str,
) -> Result<(String, String), AuthError> {
    let (client_id, _client_secret) = match provider {
        OAuthProvider::GitHub => {
            if config.github_client_id.is_empty() || config.github_client_secret.is_empty() {
                return Err(AuthError::OAuth("GitHub OAuth not configured".to_string()));
            }
            (&config.github_client_id, &config.github_client_secret)
        }
        OAuthProvider::Google => {
            if config.google_client_id.is_empty() || config.google_client_secret.is_empty() {
                return Err(AuthError::OAuth("Google OAuth not configured".to_string()));
            }
            (&config.google_client_id, &config.google_client_secret)
        }
        OAuthProvider::Discord => {
            if config.discord_client_id.is_empty() || config.discord_client_secret.is_empty() {
                return Err(AuthError::OAuth("Discord OAuth not configured".to_string()));
            }
            (&config.discord_client_id, &config.discord_client_secret)
        }
    };

    let redirect_uri = format!("{}/api/auth/oauth/{}/callback", base_url, provider.as_str());
    let state = Uuid::new_v4().to_string();

    let mut url = format!(
        "{}?client_id={}&redirect_uri={}&state={}&scope={}&response_type=code",
        provider.auth_url(),
        urlencoding::encode(client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&state),
        urlencoding::encode(provider.scope()),
    );

    if *provider == OAuthProvider::Discord {
        url.push_str("&prompt=consent");
    }

    Ok((url, state))
}

/// Exchange an OAuth authorization code for access tokens and user info.
///
/// # Expected Behavior
///
/// Sends a POST request to the provider's token endpoint with the
/// authorization code, `client_id`, `client_secret`, and `redirect_uri`.
/// Parses the response to extract the `access_token`. Then fetches
/// the user profile from the provider's user info endpoint.
/// Returns the `ProviderUserInfo` with `provider_user_id`, email, name,
/// and `avatar_url`.
///
/// # Errors
///
/// Returns `AuthError::OAuth` if the token exchange fails, the
/// provider returns an error, or the user info cannot be fetched.
///
/// # Side Effects
///
/// - Makes HTTP POST to the provider's token endpoint (network).
/// - Makes HTTP GET to the provider's user info endpoint (network).
pub async fn exchange_code_and_get_user_info(
    provider: &OAuthProvider,
    code: &str,
    config: &Config,
    base_url: &str,
) -> Result<(ProviderUserInfo, String), AuthError> {
    let (client_id, client_secret) = match provider {
        OAuthProvider::GitHub => (&config.github_client_id, &config.github_client_secret),
        OAuthProvider::Google => (&config.google_client_id, &config.google_client_secret),
        OAuthProvider::Discord => (&config.discord_client_id, &config.discord_client_secret),
    };

    let redirect_uri = format!("{}/api/auth/oauth/{}/callback", base_url, provider.as_str());

    let client = reqwest::Client::new();
    let mut token_body = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", code.to_string()),
        ("client_id", client_id.clone()),
        ("client_secret", client_secret.clone()),
        ("redirect_uri", redirect_uri.clone()),
    ];

    if *provider == OAuthProvider::Discord {
        token_body.push(("scope", provider.scope().to_string()));
    }

    let response = client
        .post(provider.token_url())
        .header("Accept", "application/json")
        .form(&token_body)
        .send()
        .await
        .map_err(|e| AuthError::OAuth(format!("Token exchange request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AuthError::OAuth(format!(
            "Token exchange failed with status {status}: {body}"
        )));
    }

    let token_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AuthError::OAuth(format!("Failed to parse token response: {e}")))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| AuthError::OAuth("No access_token in response".to_string()))?
        .to_string();

    let user_info = fetch_provider_user_info(provider, &access_token).await?;

    Ok((user_info, access_token))
}

/// Fetch user profile from an OAuth provider using an access token.
///
/// # Expected Behavior
///
/// Sends a GET request to the provider's user info endpoint with the
/// Bearer access token. Parses the response to extract `provider_user_id`,
/// email, name, and `avatar_url`. Field names vary by provider:
/// - GitHub: id (number), email, name/login, `avatar_url`
/// - Google: id, email, name, picture
/// - Discord: id, email, username, avatar (combined with CDN URL)
///
/// # Errors
///
/// Returns `AuthError::OAuth` if the request fails or the response
/// cannot be parsed.
///
/// # Side Effects
///
/// - Makes HTTP GET to the provider's user info endpoint (network).
async fn fetch_provider_user_info(
    provider: &OAuthProvider,
    access_token: &str,
) -> Result<ProviderUserInfo, AuthError> {
    let client = reqwest::Client::new();

    let mut request = client
        .get(provider.user_info_url())
        .header("Authorization", format!("Bearer {access_token}"));

    if *provider == OAuthProvider::GitHub {
        request = request.header("User-Agent", "OpenHack-Auth-Service");
    }

    let response = request
        .send()
        .await
        .map_err(|e| AuthError::OAuth(format!("User info request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AuthError::OAuth(format!(
            "User info fetch failed with status {status}: {body}"
        )));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AuthError::OAuth(format!("Failed to parse user info: {e}")))?;

    let info = match provider {
        OAuthProvider::GitHub => {
            let provider_user_id = data["id"]
                .as_i64()
                .map(|i| i.to_string())
                .or_else(|| data["id"].as_str().map(std::string::ToString::to_string))
                .unwrap_or_default();
            let email = data["email"].as_str().map(std::string::ToString::to_string);
            let name = data["name"]
                .as_str()
                .map(std::string::ToString::to_string)
                .or_else(|| data["login"].as_str().map(std::string::ToString::to_string));
            let avatar_url = data["avatar_url"]
                .as_str()
                .map(std::string::ToString::to_string);
            ProviderUserInfo {
                provider_user_id,
                email,
                name,
                avatar_url,
            }
        }
        OAuthProvider::Google => {
            let provider_user_id = data["id"].as_str().unwrap_or_default().to_string();
            let email = data["email"].as_str().map(std::string::ToString::to_string);
            let name = data["name"].as_str().map(std::string::ToString::to_string);
            let avatar_url = data["picture"]
                .as_str()
                .map(std::string::ToString::to_string);
            ProviderUserInfo {
                provider_user_id,
                email,
                name,
                avatar_url,
            }
        }
        OAuthProvider::Discord => {
            let provider_user_id = data["id"].as_str().unwrap_or_default().to_string();
            let email = data["email"].as_str().map(std::string::ToString::to_string);
            let name = data["username"]
                .as_str()
                .map(std::string::ToString::to_string);
            let avatar_url = data["avatar"].as_str().map(|hash| {
                format!("https://cdn.discordapp.com/avatars/{provider_user_id}/{hash}.png")
            });
            ProviderUserInfo {
                provider_user_id,
                email,
                name,
                avatar_url,
            }
        }
    };

    Ok(info)
}

/// Find an existing OAuth account link or create a new user.
///
/// # Expected Behavior
///
/// Looks up `auth.oauth_accounts` by (provider, `provider_user_id`). If found,
/// returns the associated `user_id`. If not found and an email is provided,
/// attempts to find an existing user by email. If found, creates an
/// `auth.oauth_accounts` link. If not found, creates a new user in
/// `auth.users` with a random password (since OAuth users don't need one),
/// then creates the `auth.oauth_accounts` link. If no email is available
/// from the provider, generates a placeholder.
///
/// # Errors
///
/// Returns `AuthError::DatabaseError` on database failures.
///
/// # Side Effects
///
/// - Reads from `auth.oauth_accounts` and `auth.users` tables.
/// - May write to `auth.oauth_accounts` and `auth.users` tables (new user/link).
pub async fn find_or_create_user(
    pool: &PgPool,
    provider: &OAuthProvider,
    user_info: &ProviderUserInfo,
    access_token: &str,
) -> Result<Uuid, AuthError> {
    let existing_oauth: Option<OAuthAccount> = sqlx::query_as::<_, OAuthAccount>(
        "SELECT id, user_id, provider, provider_user_id, access_token, refresh_token, expires_at, created_at, updated_at FROM auth.oauth_accounts WHERE provider = $1 AND provider_user_id = $2",
    )
    .bind(provider.as_str())
    .bind(&user_info.provider_user_id)
    .fetch_optional(pool)
    .await
    .map_err(AuthError::DatabaseError)?;

    if let Some(oauth) = existing_oauth {
        sqlx::query(
            "UPDATE auth.oauth_accounts SET access_token = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(access_token)
        .bind(oauth.id)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;
        return Ok(oauth.user_id);
    }

    let user_id = if let Some(email) = &user_info.email {
        let existing_user: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM auth.users WHERE email = $1")
                .bind(email)
                .fetch_optional(pool)
                .await
                .map_err(AuthError::DatabaseError)?;

        if let Some((id,)) = existing_user {
            id
        } else {
            let new_id = Uuid::new_v4();
            let random_password = Uuid::new_v4().to_string();
            let password_hash = crate::services::auth::hash_password(&random_password)?;
            let name = user_info
                .name
                .clone()
                .unwrap_or_else(|| "OAuth User".to_string());

            sqlx::query(
                "INSERT INTO auth.users (id, email, password_hash, name, avatar_url, email_verified, mfa_enabled, roles, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, true, false, ARRAY['participant'], NOW(), NOW())",
            )
            .bind(new_id)
            .bind(email)
            .bind(&password_hash)
            .bind(&name)
            .bind(&user_info.avatar_url)
            .execute(pool)
            .await
            .map_err(AuthError::DatabaseError)?;

            new_id
        }
    } else {
        let new_id = Uuid::new_v4();
        let random_password = Uuid::new_v4().to_string();
        let password_hash = crate::services::auth::hash_password(&random_password)?;
        let placeholder_email = format!(
            "oauth_{}@placeholder.openhack.dev",
            Uuid::new_v4().as_simple()
        );
        let name = user_info
            .name
            .clone()
            .unwrap_or_else(|| "OAuth User".to_string());

        sqlx::query(
            "INSERT INTO auth.users (id, email, password_hash, name, avatar_url, email_verified, mfa_enabled, roles, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, false, false, ARRAY['participant'], NOW(), NOW())",
        )
        .bind(new_id)
        .bind(&placeholder_email)
        .bind(&password_hash)
        .bind(&name)
        .bind(&user_info.avatar_url)
        .execute(pool)
        .await
        .map_err(AuthError::DatabaseError)?;

        new_id
    };

    if let Some(avatar_url) = &user_info.avatar_url {
        sqlx::query("UPDATE auth.users SET avatar_url = $1 WHERE id = $2 AND avatar_url IS NULL")
            .bind(avatar_url)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(AuthError::DatabaseError)?;
    }

    let oauth_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO auth.oauth_accounts (id, user_id, provider, provider_user_id, access_token, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
    )
    .bind(oauth_id)
    .bind(user_id)
    .bind(provider.as_str())
    .bind(&user_info.provider_user_id)
    .bind(access_token)
    .execute(pool)
    .await
    .map_err(AuthError::DatabaseError)?;

    Ok(user_id)
}
