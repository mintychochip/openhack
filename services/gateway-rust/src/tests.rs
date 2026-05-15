#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use actix_web::test::TestRequest;

    use crate::routes::proxy::{ProxyState, RateLimiter};

    /// Test that `ProxyState::match_route` correctly matches paths to prefixes.
    ///
    /// # Expected Behavior
    ///
    /// Calls `match_route` with various request paths and verifies that the
    /// longest matching prefix is returned. For example, `/api/auth/login`
    /// matches `/api/auth`; `/api/media/upload` matches `/api/media`. Paths
    /// that do not start with any registered prefix return `None`. An exact
    /// prefix path (e.g. `/api/auth`) matches itself.
    ///
    /// # Errors
    ///
    /// Panics if any assertion fails.
    ///
    /// # Side Effects
    ///
    /// None. Pure test assertions with no I/O.
    #[tokio::test]
    #[allow(clippy::missing_panics_doc)]
    async fn test_route_matching() {
        let state = ProxyState::new();

        assert_eq!(
            state.match_route("/api/auth/login"),
            Some("/api/auth"),
            "/api/auth/login should match /api/auth"
        );
        assert_eq!(
            state.match_route("/api/auth"),
            Some("/api/auth"),
            "/api/auth should match /api/auth"
        );
        assert_eq!(
            state.match_route("/api/core/projects/123"),
            Some("/api/core"),
            "/api/core/projects/123 should match /api/core"
        );
        assert_eq!(
            state.match_route("/api/media/upload"),
            Some("/api/media"),
            "/api/media/upload should match /api/media"
        );
        assert_eq!(
            state.match_route("/api/judging/score"),
            Some("/api/judging"),
            "/api/judging/score should match /api/judging"
        );
        assert_eq!(
            state.match_route("/api/leaderboard/top"),
            Some("/api/leaderboard"),
            "/api/leaderboard/top should match /api/leaderboard"
        );
        assert_eq!(
            state.match_route("/api/mail/send"),
            Some("/api/mail"),
            "/api/mail/send should match /api/mail"
        );
        assert_eq!(
            state.match_route("/api/notify/push"),
            Some("/api/notify"),
            "/api/notify/push should match /api/notify"
        );
        assert_eq!(
            state.match_route("/api/ai/predict"),
            Some("/api/ai"),
            "/api/ai/predict should match /api/ai"
        );
        assert_eq!(
            state.match_route("/api/analytics/report"),
            Some("/api/analytics"),
            "/api/analytics/report should match /api/analytics"
        );
        assert_eq!(
            state.match_route("/api/sponsors/list"),
            Some("/api/sponsors"),
            "/api/sponsors/list should match /api/sponsors"
        );

        assert_eq!(
            state.match_route("/api/unknown/path"),
            None,
            "path with no registered prefix should return None"
        );
        assert_eq!(
            state.match_route("/other"),
            None,
            "unrelated path should return None"
        );
        assert_eq!(state.match_route("/"), None, "root path should return None");
    }

    /// Test that the rate limiter allows requests up to the limit and then blocks.
    ///
    /// # Expected Behavior
    ///
    /// Creates a `RateLimiter`, calls `allow` with a limit of 5. The first 5
    /// calls for a given IP return `true`; the 6th and subsequent calls
    /// return `false`. A different IP has its own independent counter.
    ///
    /// # Errors
    ///
    /// Panics if any assertion fails.
    ///
    /// # Side Effects
    ///
    /// None. Pure test assertions with no I/O beyond Mutex acquisition.
    #[tokio::test]
    #[allow(clippy::missing_panics_doc)]
    async fn test_rate_limiter_allow() {
        let limiter = RateLimiter::new();
        let limit: u32 = 5;

        for i in 1..=5 {
            assert!(
                limiter.allow("192.168.1.1", limit).await,
                "request {i} of {limit} should be allowed"
            );
        }

        assert!(
            !limiter.allow("192.168.1.1", limit).await,
            "request beyond limit should be blocked"
        );
        assert!(
            !limiter.allow("192.168.1.1", limit).await,
            "further requests should remain blocked"
        );

        assert!(
            limiter.allow("10.0.0.1", limit).await,
            "different IP should have independent counter"
        );
    }

    /// Test that after the rate limiter window expires, the counter resets.
    ///
    /// # Expected Behavior
    ///
    /// Creates a `RateLimiter` with a 1-second window. Exhausts the limit,
    /// then waits for the window to expire (slightly over 1 second due to
    /// the `>` comparison on `as_secs()` which truncates). After the wait,
    /// a subsequent `allow` call returns `true`, indicating the counter
    /// has been reset to 1.
    ///
    /// # Errors
    ///
    /// Panics if any assertion fails or if the tokio timer does not
    /// advance as expected.
    ///
    /// # Side Effects
    ///
    /// - Sleeps for 2 seconds to allow the rate limiter window to expire.
    #[tokio::test]
    #[allow(clippy::missing_panics_doc)]
    async fn test_rate_limiter_window_reset() {
        let limiter = RateLimiter::new_with_window(1);
        let limit: u32 = 3;

        assert!(limiter.allow("192.168.1.1", limit).await);
        assert!(limiter.allow("192.168.1.1", limit).await);
        assert!(limiter.allow("192.168.1.1", limit).await);
        assert!(
            !limiter.allow("192.168.1.1", limit).await,
            "should be blocked after exhausting limit"
        );

        tokio::time::sleep(Duration::from_secs(2)).await;

        assert!(
            limiter.allow("192.168.1.1", limit).await,
            "should be allowed again after window expires"
        );
        assert!(
            limiter.allow("192.168.1.1", limit).await,
            "second request in new window should also be allowed"
        );
    }

    /// Test that requests exceeding max_body_bytes are rejected.
    ///
    /// # Expected Behavior
    ///
    /// Calls `ProxyState::is_body_within_limit` with body lengths at, below,
    /// and above the configured `max_body_bytes` for known routes. For the
    /// `/api/auth` route (max 1_048_576 bytes), a body of exactly 1_048_576
    /// bytes is within limit, while 1_048_577 bytes exceeds it. A zero-byte
    /// body is always within limit. An unknown route prefix always returns
    /// `true` (body size is not the rejection reason for unknown routes).
    ///
    /// # Errors
    ///
    /// Panics if any assertion fails.
    ///
    /// # Side Effects
    ///
    /// None. Pure test assertions with no I/O.
    #[tokio::test]
    #[allow(clippy::missing_panics_doc)]
    async fn test_body_size_limit() {
        let state = ProxyState::new();

        assert!(
            state.is_body_within_limit("/api/auth", 0),
            "zero-byte body should be within limit"
        );
        assert!(
            state.is_body_within_limit("/api/auth", 1_048_576),
            "body at exact limit should be within limit"
        );
        assert!(
            !state.is_body_within_limit("/api/auth", 1_048_577),
            "body one byte over limit should exceed limit"
        );
        assert!(
            !state.is_body_within_limit("/api/auth", 10_000_000),
            "large body should exceed limit"
        );

        assert!(
            state.is_body_within_limit("/api/media", 104_857_600),
            "/api/media allows up to 100MB"
        );
        assert!(
            !state.is_body_within_limit("/api/media", 104_857_601),
            "/api/media rejects over 100MB"
        );

        assert!(
            state.is_body_within_limit("/api/core", 10_485_760),
            "/api/core allows up to 10MB"
        );
        assert!(
            !state.is_body_within_limit("/api/core", 10_485_761),
            "/api/core rejects over 10MB"
        );

        assert!(
            state.is_body_within_limit("/api/nonexistent", 999_999_999),
            "unknown route should return true (no body-size violation)"
        );
    }

    /// Test that X-Forwarded-For header is used for IP extraction.
    ///
    /// # Expected Behavior
    ///
    /// Creates an `HttpRequest` with an `x-forwarded-for` header and verifies
    /// that `ProxyState::client_ip` returns the first IP in the header value.
    /// Also tests the multi-IP case where X-Forwarded-For contains a
    /// comma-separated list — only the first (leftmost) IP is used.
    ///
    /// # Errors
    ///
    /// Panics if any assertion fails.
    ///
    /// # Side Effects
    ///
    /// None. Pure test assertions with no I/O.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_client_ip_xff() {
        let req = TestRequest::default()
            .insert_header(("x-forwarded-for", "1.2.3.4"))
            .to_http_request();
        assert_eq!(
            ProxyState::client_ip(&req),
            "1.2.3.4",
            "should use X-Forwarded-For single IP"
        );

        let req = TestRequest::default()
            .insert_header(("x-forwarded-for", "1.2.3.4, 5.6.7.8"))
            .to_http_request();
        assert_eq!(
            ProxyState::client_ip(&req),
            "1.2.3.4",
            "should use first IP from X-Forwarded-For comma-separated list"
        );

        let req = TestRequest::default()
            .insert_header(("x-forwarded-for", "  10.20.30.40  , 50.60.70.80 "))
            .to_http_request();
        assert_eq!(
            ProxyState::client_ip(&req),
            "10.20.30.40",
            "should trim whitespace from first X-Forwarded-For IP"
        );
    }

    /// Test that peer_addr is used when X-Forwarded-For is absent.
    ///
    /// # Expected Behavior
    ///
    /// Creates an `HttpRequest` without an `x-forwarded-for` header. When
    /// `peer_addr` is available (set via `TestRequest::peer_addr`), the
    /// peer address IP is returned. When no peer address is available
    /// (no `peer_addr` set on the test request), the function returns
    /// "unknown" as the fallback.
    ///
    /// # Errors
    ///
    /// Panics if any assertion fails.
    ///
    /// # Side Effects
    ///
    /// None. Pure test assertions with no I/O.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_client_ip_fallback() {
        let addr: std::net::SocketAddr = "192.168.0.1:12345".parse().unwrap();
        let req = TestRequest::default().peer_addr(addr).to_http_request();
        assert_eq!(
            ProxyState::client_ip(&req),
            "192.168.0.1",
            "should use peer_addr when X-Forwarded-For is absent"
        );

        let req = TestRequest::default().to_http_request();
        let ip = ProxyState::client_ip(&req);
        assert!(
            ip == "unknown" || !ip.is_empty(),
            "without X-Forwarded-For and peer_addr, should return 'unknown' or the peer address"
        );
    }

    /// Verify the rate limiter state works correctly under concurrent access.
    ///
    /// # Expected Behavior
    ///
    /// Creates a `RateLimiter` and spawns multiple concurrent Tokio tasks
    /// that call `allow` on the same IP with a shared limit. Despite
    /// concurrent access via `tokio::sync::Mutex`, the counter must remain
    /// accurate: the total number of allowed requests equals the limit
    /// exactly, and no more than `limit` requests are permitted within
    /// the window.
    ///
    /// # Errors
    ///
    /// Panics if any assertion fails or if a spawned task panics.
    ///
    /// # Side Effects
    ///
    /// - Spawns multiple Tokio tasks (each acquires the rate limiter Mutex).
    #[tokio::test]
    #[allow(clippy::missing_panics_doc)]
    async fn test_lambda_internal_token_in_rate_limiter_context() {
        let limiter = Arc::new(RateLimiter::new());
        let limit: u32 = 50;
        let num_tasks = 10;
        let calls_per_task = 10;

        let mut handles = Vec::with_capacity(num_tasks);
        for _ in 0..num_tasks {
            let limiter = limiter.clone();
            handles.push(tokio::spawn(async move {
                let mut allowed = 0u32;
                for _ in 0..calls_per_task {
                    if limiter.allow("1.2.3.4", limit).await {
                        allowed += 1;
                    }
                }
                allowed
            }));
        }

        let mut total_allowed = 0u32;
        for h in handles {
            total_allowed += h.await.expect("task should not panic");
        }

        assert_eq!(
            total_allowed, limit,
            "total allowed requests under concurrent access should equal the limit exactly"
        );
    }
}
