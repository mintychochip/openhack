#[cfg(test)]
mod tests {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    /// Test HMAC signature generation and verification for webhook payloads.
    ///
    /// # Expected Behavior
    ///
    /// Calls `compute_hmac` with a known secret and payload, then manually
    /// computes the expected HMAC-SHA256 using the `hmac` and `sha2` crates.
    /// Asserts that the function output matches the manually computed
    /// hex-encoded digest. Also verifies that different payloads produce
    /// different signatures, that an empty secret returns an empty string,
    /// and that the same inputs produce the same signature (idempotency).
    ///
    /// # Errors
    ///
    /// Panics if any assertion fails (signature mismatch, empty secret
    /// behavior, or idempotency violation).
    ///
    /// # Side Effects
    ///
    /// None. Pure in-memory computation with no I/O.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_hmac_signature() {
        let secret = "my-webhook-secret";
        let payload = b"{\"event\":\"test\"}";

        let signature = crate::services::webhook::compute_hmac(secret, payload);

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let expected = hex::encode(mac.finalize().into_bytes());

        assert_eq!(signature, expected);
        assert!(!signature.is_empty());

        let payload2 = b"{\"event\":\"different\"}";
        let signature2 = crate::services::webhook::compute_hmac(secret, payload2);
        assert_ne!(signature, signature2);

        let empty_sig = crate::services::webhook::compute_hmac("", payload);
        assert!(empty_sig.is_empty());

        let sig_again = crate::services::webhook::compute_hmac(secret, payload);
        assert_eq!(signature, sig_again);
    }

    /// Test the Lambda internal token bypass in auth.rs.
    ///
    /// # Expected Behavior
    ///
    /// Sets `LAMBDA_INTERNAL_TOKEN` environment variable to a known value,
    /// then calls `validate_lambda_internal_token` with matching and
    /// non-matching tokens. When the provided token matches the environment
    /// variable, returns `AuthUser` with nil UUID user_id and admin role.
    /// When the provided token does not match, returns `None`. When no
    /// token is provided (`None`), returns `None`. When the env var is unset
    /// or empty, returns `None`. Cleans up the environment variable after
    /// the test.
    ///
    /// # Errors
    ///
    /// Panics if the token validation does not produce the expected results.
    ///
    /// # Side Effects
    ///
    /// - Sets and removes `LAMBDA_INTERNAL_TOKEN` environment variable
    ///   (mutation of process environment).
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_lambda_internal_token_bypass() {
        std::env::set_var("LAMBDA_INTERNAL_TOKEN", "notify-test-token");

        let matching =
            crate::middleware::auth::validate_lambda_internal_token(Some("notify-test-token"));
        assert!(matching.is_some());
        let user = matching.unwrap();
        assert!(user.roles.iter().any(|r| r == "admin"));

        let wrong = crate::middleware::auth::validate_lambda_internal_token(Some("wrong"));
        assert!(wrong.is_none());

        let no_token = crate::middleware::auth::validate_lambda_internal_token(None);
        assert!(no_token.is_none());

        std::env::remove_var("LAMBDA_INTERNAL_TOKEN");

        let missing_env =
            crate::middleware::auth::validate_lambda_internal_token(Some("notify-test-token"));
        assert!(missing_env.is_none());

        std::env::set_var("LAMBDA_INTERNAL_TOKEN", "");
        let empty_env =
            crate::middleware::auth::validate_lambda_internal_token(Some("notify-test-token"));
        assert!(empty_env.is_none());
        std::env::remove_var("LAMBDA_INTERNAL_TOKEN");
    }
}
