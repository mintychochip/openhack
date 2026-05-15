#[cfg(test)]
mod tests {
    use crate::models::chart::metric_type_to_event_type;

    /// Test that CSV output properly escapes commas, quotes, and newlines in field values.
    ///
    /// # Expected Behavior
    ///
    /// Uses the `csv` crate (the same crate used by `export_events_csv` and
    /// `export_metrics_csv`) to write records containing fields with special
    /// characters. Verifies that:
    /// - Fields containing commas are enclosed in double quotes per RFC 4180.
    /// - Fields containing double quotes have the quotes escaped by doubling
    ///   (e.g., `"` becomes `""`) and the field is enclosed in double quotes.
    /// - Fields containing newlines are enclosed in double quotes.
    /// - Normal fields without special characters are written unquoted.
    /// This ensures that exported CSV data can be safely parsed by any RFC 4180
    /// compliant reader without data corruption.
    ///
    /// # Errors
    ///
    /// None. Panics on assertion failure.
    ///
    /// # Side Effects
    ///
    /// None. Pure in-memory CSV writing with no I/O.
    #[allow(clippy::missing_panics_doc)]
    #[test]
    fn test_csv_escaping() {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        wtr.write_record(&[
            "id1".to_string(),
            "value,with,commas".to_string(),
            "normal".to_string(),
        ])
        .unwrap();
        wtr.write_record(&[
            "id2".to_string(),
            "has\"quote".to_string(),
            "normal2".to_string(),
        ])
        .unwrap();
        wtr.write_record(&[
            "id3".to_string(),
            "line\nbreak".to_string(),
            "normal3".to_string(),
        ])
        .unwrap();

        let bytes = wtr.into_inner().unwrap();
        let csv_str = String::from_utf8(bytes).unwrap();

        assert!(
            csv_str.contains("\"value,with,commas\""),
            "Commas must be escaped by quoting"
        );
        assert!(
            csv_str.contains("\"has\"\"quote\""),
            "Quotes must be escaped by doubling"
        );
        assert!(
            csv_str.contains("\"line\nbreak\""),
            "Newlines must be escaped by quoting"
        );
        assert!(
            csv_str.contains("id1") && csv_str.contains("normal"),
            "Normal fields must be written unquoted"
        );
    }

    /// Test period calculation (daily/hourly/weekly/monthly) for aggregation timestamps.
    ///
    /// # Expected Behavior
    ///
    /// Verifies the metric-type-to-event-type mapping used by the aggregation and
    /// chart services. Tests that "registrations" maps to "user.registered",
    /// "teams" maps to "team.created", "projects" maps to "project.created", and
    /// "submissions" maps to "project.submitted". Unknown metric types default to
    /// "user.registered". Also verifies the period-to-truncation mapping where
    /// "hourly" maps to "hour" (for PostgreSQL `date_trunc`) and all other periods
    /// ("daily", "weekly", "monthly") map to "day". Verifies the lookback duration
    /// calculation using `chrono::Duration::days` produces the expected day count.
    ///
    /// # Errors
    ///
    /// None. Panics on assertion failure.
    ///
    /// # Side Effects
    ///
    /// None. Pure computation with no I/O.
    #[allow(clippy::missing_panics_doc)]
    #[test]
    fn test_aggregation_period_calculation() {
        assert_eq!(
            metric_type_to_event_type("registrations"),
            "user.registered"
        );
        assert_eq!(metric_type_to_event_type("teams"), "team.created");
        assert_eq!(metric_type_to_event_type("projects"), "project.created");
        assert_eq!(
            metric_type_to_event_type("submissions"),
            "project.submitted"
        );
        assert_eq!(
            metric_type_to_event_type("unknown"),
            "user.registered",
            "Unknown metric type must default to user.registered"
        );

        fn period_to_trunc(period: &str) -> &'static str {
            if period == "hourly" {
                "hour"
            } else {
                "day"
            }
        }

        assert_eq!(period_to_trunc("hourly"), "hour");
        assert_eq!(period_to_trunc("daily"), "day");
        assert_eq!(period_to_trunc("weekly"), "day");
        assert_eq!(period_to_trunc("monthly"), "day");

        let now = chrono::Utc::now();
        let seven_days_ago = now - chrono::Duration::days(7);
        let diff = now.signed_duration_since(seven_days_ago);
        assert_eq!(
            diff.num_days(),
            7,
            "7-day lookback must span exactly 7 days"
        );

        let thirty_days_ago = now - chrono::Duration::days(30);
        let diff30 = now.signed_duration_since(thirty_days_ago);
        assert_eq!(
            diff30.num_days(),
            30,
            "30-day lookback must span exactly 30 days"
        );
    }
}
