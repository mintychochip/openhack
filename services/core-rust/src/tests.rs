#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use actix_web::test::TestRequest;

    use crate::middleware::auth::check_lambda_internal_token;

    use super::ENV_LOCK;

    fn lock_env() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap()
    }

    /// Test that a valid Lambda internal token authenticates the request.
    ///
    /// # Expected Behavior
    ///
    /// Sets the `LAMBDA_INTERNAL_TOKEN` environment variable to a known value,
    /// creates a request with a matching `X-Lambda-Internal-Token` header,
    /// and calls `check_lambda_internal_token`. Returns `Some(AuthUser)` with
    /// `user_id` "lambda-internal", `_email` "internal@lambda", and
    /// `roles` containing "admin".
    ///
    /// # Errors
    ///
    /// Panics if `check_lambda_internal_token` returns `None` when the
    /// token matches, or if the returned `AuthUser` fields do not match
    /// the expected values.
    ///
    /// # Side Effects
    ///
    /// - Sets the `LAMBDA_INTERNAL_TOKEN` environment variable before the
    ///   test logic (write to process environment).
    /// - Removes the `LAMBDA_INTERNAL_TOKEN` environment variable after
    ///   the test logic (write to process environment).
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_lambda_internal_token_valid() {
        let _lock = lock_env();
        let test_token = "test-internal-token-value";
        std::env::set_var("LAMBDA_INTERNAL_TOKEN", test_token);

        let req = TestRequest::default()
            .insert_header(("X-Lambda-Internal-Token", test_token))
            .to_http_request();

        let result = check_lambda_internal_token(&req);

        assert!(
            result.is_some(),
            "check_lambda_internal_token should return Some for valid token"
        );
        let user = result.expect("already asserted Some");
        assert_eq!(user.user_id, uuid::Uuid::nil());
        assert_eq!(user.email, "internal@lambda");
        assert!(
            user.roles.iter().any(|r| r == "admin"),
            "roles should contain admin"
        );

        std::env::remove_var("LAMBDA_INTERNAL_TOKEN");
    }

    /// Test that an invalid Lambda internal token does not authenticate.
    ///
    /// # Expected Behavior
    ///
    /// Sets the `LAMBDA_INTERNAL_TOKEN` environment variable to a known value,
    /// creates a request with a different (wrong) `X-Lambda-Internal-Token`
    /// header value, and calls `check_lambda_internal_token`. Returns `None`
    /// because the provided token does not match the expected value.
    ///
    /// # Errors
    ///
    /// Panics if `check_lambda_internal_token` returns `Some` for a mismatched
    /// token.
    ///
    /// # Side Effects
    ///
    /// - Sets the `LAMBDA_INTERNAL_TOKEN` environment variable before the
    ///   test logic (write to process environment).
    /// - Removes the `LAMBDA_INTERNAL_TOKEN` environment variable after
    ///   the test logic (write to process environment).
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_lambda_internal_token_invalid() {
        let _lock = lock_env();
        let expected_token = "expected-internal-token";
        std::env::set_var("LAMBDA_INTERNAL_TOKEN", expected_token);

        let req = TestRequest::default()
            .insert_header(("X-Lambda-Internal-Token", "wrong-token-value"))
            .to_http_request();

        let result = check_lambda_internal_token(&req);

        assert!(
            result.is_none(),
            "check_lambda_internal_token should return None for invalid token"
        );

        std::env::remove_var("LAMBDA_INTERNAL_TOKEN");
    }

    /// Test that when LAMBDA_INTERNAL_TOKEN env var is not set, internal
    /// auth bypass is unavailable.
    ///
    /// # Expected Behavior
    ///
    /// Ensures the `LAMBDA_INTERNAL_TOKEN` environment variable is not set,
    /// creates a request with an `X-Lambda-Internal-Token` header, and calls
    /// `check_lambda_internal_token`. Returns `None` because without the env
    /// var set, there is no expected token to match against. This means
    /// normal JWT authentication is required.
    ///
    /// # Errors
    ///
    /// Panics if `check_lambda_internal_token` returns `Some` when the env
    /// var is not set.
    ///
    /// # Side Effects
    ///
    /// - Removes the `LAMBDA_INTERNAL_TOKEN` environment variable before
    ///   the test logic (write to process environment).
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_lambda_internal_token_not_set() {
        let _lock = lock_env();
        std::env::remove_var("LAMBDA_INTERNAL_TOKEN");

        let req = TestRequest::default()
            .insert_header(("X-Lambda-Internal-Token", "any-token-value"))
            .to_http_request();

        let result = check_lambda_internal_token(&req);

        assert!(
            result.is_none(),
            "check_lambda_internal_token should return None when env var is not set"
        );
    }
}
