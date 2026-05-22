#[cfg(test)]
mod tests {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use uuid::Uuid;

    use crate::config::Config;
    use crate::models::session::Claims;
    use crate::models::user::User;
    use crate::services::auth::{
        create_access_token, hash_password, validate_access_token, verify_password,
    };

    fn make_test_config() -> Config {
        Config {
            database_url: String::new(),
            redis_url: String::new(),
            port: 3001,
            jwt_secret: "test-secret-that-is-at-least-32-characters-long".to_string(),
            _jwt_expiry: "15m".to_string(),
            jwt_expiry_secs: 900,
            refresh_token_expiry_secs: 604800,
            rust_log: "info".to_string(),
            github_client_id: String::new(),
            github_client_secret: String::new(),
            google_client_id: String::new(),
            google_client_secret: String::new(),
            discord_client_id: String::new(),
            discord_client_secret: String::new(),
            frontend_url: "http://localhost:3000".to_string(),
        }
    }

    fn make_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: Some("irrelevant".to_string()),
            name: "Test User".to_string(),
            avatar_url: None,
            github_username: None,
            discord_id: None,
            email_verified: false,
            mfa_enabled: false,
            mfa_secret: None,
            sms_mfa_enabled: false,
            sms_phone_number: None,
            roles: vec!["participant".to_string()],
            created_at: None,
            updated_at: None,
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            email_verification_token: None,
            email_verification_token_expires_at: None,
            deletion_requested_at: None,
            deletion_scheduled_at: None,
        }
    }

    /// Test that Argon2 password hashing and verification work correctly.
    ///
    /// # Expected Behavior
    ///
    /// Hashes a plaintext password using `hash_password`, then verifies
    /// the same password with `verify_password` which should succeed.
    /// Verifies that a different (wrong) password fails verification.
    /// The hash output is in PHC string format containing algorithm
    /// identifier, version, parameters, salt, and hash.
    ///
    /// # Errors
    ///
    /// Panics if `hash_password` returns an error, if `verify_password`
    /// succeeds for a wrong password, or if `verify_password` fails for
    /// the correct password.
    ///
    /// # Side Effects
    ///
    /// - Calls the operating system's CSPRNG for salt generation during
    ///   `hash_password` (read from /dev/urandom or equivalent).
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_password_hash_verify() {
        let password = "correct-horse-battery-staple";
        let hash = hash_password(password).expect("hash_password should succeed");

        assert!(
            verify_password(password, &hash).is_ok(),
            "verify_password should succeed for the correct password"
        );

        assert!(
            verify_password("wrong-password", &hash).is_err(),
            "verify_password should fail for a wrong password"
        );
    }

    /// Test that JWT creation and validation produce matching claims.
    ///
    /// # Expected Behavior
    ///
    /// Creates a JWT access token for a test user using `create_access_token`,
    /// then decodes it with `validate_access_token`. Verifies that the decoded
    /// claims contain the same `sub` (user ID), `email`, and `roles` as the
    /// original user. The `exp` and `iat` fields are set by the current time
    /// and are not compared exactly.
    ///
    /// # Errors
    ///
    /// Panics if `create_access_token` or `validate_access_token` return an
    /// error, or if any claim field does not match the expected value.
    ///
    /// # Side Effects
    ///
    /// None. Pure computation with no I/O or state mutation.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_jwt_create_validate() {
        let config = make_test_config();
        let user = make_test_user();

        let token =
            create_access_token(&user, &config).expect("create_access_token should succeed");
        let claims = validate_access_token(&token, &config.jwt_secret)
            .expect("validate_access_token should succeed");

        assert_eq!(
            claims.sub,
            user.id.to_string(),
            "sub claim should match user ID"
        );
        assert_eq!(
            claims.email, user.email,
            "email claim should match user email"
        );
        assert_eq!(
            claims.roles, user.roles,
            "roles claim should match user roles"
        );
    }

    /// Test that an expired JWT token fails validation.
    ///
    /// # Expected Behavior
    ///
    /// Manually constructs a JWT with `exp` set to 1 (Unix epoch, far in the
    /// past) and encodes it with the test secret. Decoding this token with
    /// `validate_access_token` should fail because the `jsonwebtoken` library
    /// validates expiration by default.
    ///
    /// # Errors
    ///
    /// Panics if `validate_access_token` succeeds for an expired token.
    ///
    /// # Side Effects
    ///
    /// None. Pure computation with no I/O or state mutation.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_jwt_expired() {
        let config = make_test_config();

        let expired_claims = Claims {
            sub: "test-user-id".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["participant".to_string()],
            exp: 1,
            iat: 1,
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &expired_claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .expect("encoding should succeed");

        let result = validate_access_token(&token, &config.jwt_secret);
        assert!(
            result.is_err(),
            "validate_access_token should fail for an expired token"
        );
    }

    /// Test that a JWT signed with the wrong secret fails validation.
    ///
    /// # Expected Behavior
    ///
    /// Creates a JWT access token using a test user and config with one
    /// secret, then attempts to validate it with a different secret.
    /// `validate_access_token` should fail because the signature will
    /// not match.
    ///
    /// # Errors
    ///
    /// Panics if `validate_access_token` succeeds for a token signed with
    /// a different secret.
    ///
    /// # Side Effects
    ///
    /// None. Pure computation with no I/O or state mutation.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_jwt_wrong_secret() {
        let config = make_test_config();
        let user = make_test_user();

        let token =
            create_access_token(&user, &config).expect("create_access_token should succeed");

        let wrong_secret = "different-secret-that-is-at-least-32-characters";
        let result = validate_access_token(&token, wrong_secret);
        assert!(
            result.is_err(),
            "validate_access_token should fail for wrong secret"
        );
    }
}
