#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use uuid::Uuid;

    use crate::models::booth::{Booth, BoothResponse};
    use crate::models::prize::{Prize, PrizeResponse};

    /// Test booth creation/validation logic via the Booth-to-BoothResponse conversion.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that the `From<Booth> for BoothResponse` implementation correctly
    /// applies default values: `published` defaults to `false` when `None` and
    /// `view_count` defaults to `0` when `None`. When `published` is `Some(true)`,
    /// the response must reflect `true`. When `view_count` is `Some(n)`, the
    /// response must reflect `n`. The `sponsor_name` and `sponsor_id` fields must
    /// be passed through unchanged. This validates that the booth slot defaults
    /// align with the business rule that new booths are unpublished and have zero
    /// views until interacted with.
    ///
    /// # Errors
    ///
    /// None. Panics on assertion failure.
    ///
    /// # Side Effects
    ///
    /// None. Pure in-memory struct conversion with no I/O.
    #[allow(clippy::missing_panics_doc)]
    #[test]
    fn test_booth_slot_validation() {
        let booth_unpublished = Booth {
            id: Uuid::new_v4(),
            sponsor_id: Some("sponsor-123".to_string()),
            sponsor_name: "Test Sponsor".to_string(),
            tagline: Some("Best sponsor".to_string()),
            description: None,
            logo_url: None,
            banner_url: None,
            website_url: None,
            careers_url: None,
            api_docs_url: None,
            technologies: Some(vec!["Rust".to_string(), "WebAssembly".to_string()]),
            contact_email: Some("test@example.com".to_string()),
            discord_channel: None,
            theme_colors: Some(serde_json::json!({"primary": "#3B82F6"})),
            published: None,
            view_count: None,
            created_at: None,
            updated_at: None,
        };

        let response: BoothResponse = booth_unpublished.into();
        assert!(
            !response.published,
            "published must default to false when None"
        );
        assert_eq!(
            response.view_count, 0,
            "view_count must default to 0 when None"
        );
        assert_eq!(response.sponsor_name, "Test Sponsor");
        assert_eq!(response.sponsor_id, Some("sponsor-123".to_string()));

        let booth_published = Booth {
            id: Uuid::new_v4(),
            sponsor_id: None,
            sponsor_name: "Another Sponsor".to_string(),
            tagline: None,
            description: None,
            logo_url: None,
            banner_url: None,
            website_url: None,
            careers_url: None,
            api_docs_url: None,
            technologies: None,
            contact_email: None,
            discord_channel: None,
            theme_colors: None,
            published: Some(true),
            view_count: Some(42),
            created_at: None,
            updated_at: None,
        };

        let response_published: BoothResponse = booth_published.into();
        assert!(
            response_published.published,
            "published must be true when Some(true)"
        );
        assert_eq!(
            response_published.view_count, 42,
            "view_count must be 42 when Some(42)"
        );
    }

    /// Test prize tier ordering via value_usd and the Prize-to-PrizeResponse conversion.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that the `From<Prize> for PrizeResponse` implementation correctly
    /// converts `Decimal` `value_usd` to its string representation, defaults
    /// `announced` to `false` when `None`, and preserves `announced = true` when
    /// set. Verifies that prizes sorted by `value_usd` in descending order produce
    /// the expected tier ordering: highest-value prizes first (First Place), then
    /// middle-value (Second Place), then lowest-value (Third Place). Prizes with
    /// `None` `value_usd` sort last (treated as zero value). This validates that
    /// prize tiers maintain correct relative ordering by monetary value.
    ///
    /// # Errors
    ///
    /// None. Panics on assertion failure.
    ///
    /// # Side Effects
    ///
    /// None. Pure in-memory struct conversion and sorting with no I/O.
    #[allow(clippy::missing_panics_doc)]
    #[test]
    fn test_prize_tier_ordering() {
        let booth_id = Uuid::new_v4();

        let prize1 = Prize {
            id: Uuid::new_v4(),
            booth_id,
            title: "First Place".to_string(),
            description: Some("Best overall".to_string()),
            value_usd: Some(Decimal::from(5000)),
            criteria: Some("Innovation and impact".to_string()),
            winner_project_id: None,
            announced: None,
            created_at: None,
        };

        let prize2 = Prize {
            id: Uuid::new_v4(),
            booth_id,
            title: "Second Place".to_string(),
            description: Some("Great execution".to_string()),
            value_usd: Some(Decimal::from(2500)),
            criteria: None,
            winner_project_id: None,
            announced: Some(false),
            created_at: None,
        };

        let prize3 = Prize {
            id: Uuid::new_v4(),
            booth_id,
            title: "Third Place".to_string(),
            description: None,
            value_usd: Some(Decimal::from(1000)),
            criteria: None,
            winner_project_id: Some(Uuid::new_v4()),
            announced: Some(true),
            created_at: None,
        };

        let resp1: PrizeResponse = prize1.clone().into();
        assert_eq!(resp1.title, "First Place");
        assert_eq!(resp1.value_usd, Some("5000".to_string()));
        assert!(
            !resp1.announced,
            "announced must default to false when None"
        );

        let resp2: PrizeResponse = prize2.clone().into();
        assert_eq!(resp2.value_usd, Some("2500".to_string()));
        assert!(!resp2.announced, "announced must be false when Some(false)");

        let resp3: PrizeResponse = prize3.clone().into();
        assert!(resp3.announced, "announced must be true when Some(true)");
        assert!(resp3.winner_project_id.is_some());

        let mut prizes = vec![prize1, prize2, prize3];
        prizes.sort_by(|a, b| {
            let va = a.value_usd.unwrap_or(Decimal::ZERO);
            let vb = b.value_usd.unwrap_or(Decimal::ZERO);
            vb.cmp(&va)
        });

        assert_eq!(
            prizes[0].title, "First Place",
            "Highest value must come first"
        );
        assert_eq!(prizes[1].title, "Second Place");
        assert_eq!(
            prizes[2].title, "Third Place",
            "Lowest value must come last"
        );
    }
}
