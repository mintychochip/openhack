#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use lettre::message::Mailbox;

    use crate::config::Config;

    /// Test parsing of various email address formats.
    ///
    /// # Expected Behavior
    ///
    /// Parses valid email addresses in both bare (`user@example.com`) and
    /// named (`Sender Name <user@example.com>`) formats using
    /// `lettre::message::Mailbox::from_str`. Verifies that valid addresses
    /// parse successfully and invalid addresses (empty string, missing `@`,
    /// missing domain) fail to parse.
    ///
    /// # Errors
    ///
    /// Panics if a valid email address fails to parse, or if an invalid
    /// email address successfully parses.
    ///
    /// # Side Effects
    ///
    /// None. Pure computation with no I/O or state mutation.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_email_address_parsing() {
        let valid_addresses = [
            "user@example.com",
            "test.user@domain.org",
            "admin@openhack.local",
            "Sender Name <sender@example.com>",
        ];

        for addr in &valid_addresses {
            assert!(
                Mailbox::from_str(addr).is_ok(),
                "Valid address should parse: {addr}"
            );
        }

        let invalid_addresses = ["", "no-at-sign", "@nodomain.com"];

        for addr in &invalid_addresses {
            assert!(
                Mailbox::from_str(addr).is_err(),
                "Invalid address should not parse: {addr}"
            );
        }
    }

    /// Test that the default MAIL_FROM value is used when the env var is not set.
    ///
    /// # Expected Behavior
    ///
    /// Removes the `MAIL_FROM` environment variable, sets `DATABASE_URL`
    /// to a dummy value (required by `Config::from_env`), then calls
    /// `Config::from_env()`. Verifies that `config.mail_from` equals
    /// the default value "noreply@openhack.local". Restores environment
    /// variables after the test.
    ///
    /// # Errors
    ///
    /// Panics if `Config::from_env()` returns a `mail_from` value that
    /// does not equal "noreply@openhack.local".
    ///
    /// # Side Effects
    ///
    /// - Sets the `DATABASE_URL` environment variable before the test
    ///   logic (write to process environment).
    /// - Removes the `MAIL_FROM` environment variable before the test
    ///   logic (write to process environment).
    /// - Restores original `DATABASE_URL` and `MAIL_FROM` values after
    ///   the test logic (write to process environment).
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_mail_from_env_default() {
        let original_db = std::env::var("DATABASE_URL").ok();
        let original_mail_from = std::env::var("MAIL_FROM").ok();

        std::env::set_var("DATABASE_URL", "postgresql://dummy");
        std::env::remove_var("MAIL_FROM");

        let config = Config::from_env();
        assert_eq!(
            config.mail_from, "noreply@openhack.local",
            "mail_from should default to noreply@openhack.local"
        );

        if let Some(db) = original_db {
            std::env::set_var("DATABASE_URL", db);
        } else {
            std::env::remove_var("DATABASE_URL");
        }

        if let Some(mail_from) = original_mail_from {
            std::env::set_var("MAIL_FROM", mail_from);
        } else {
            std::env::remove_var("MAIL_FROM");
        }
    }
}
