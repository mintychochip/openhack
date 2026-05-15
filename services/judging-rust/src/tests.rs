#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use std::collections::HashMap;

    use crate::models::rubric::CriterionInput;

    /// Verify score values maintain decimal precision (e.g., 8.5, 9.25).
    ///
    /// # Expected Behavior
    ///
    /// Converts f64 score values (8.5, 9.25) to `rust_decimal::Decimal`
    /// using `from_f64_retain`, the same method used by
    /// `JudgingService::submit_score` and `JudgingService::update_score`.
    /// Asserts that the resulting `Decimal` values exactly match their
    /// expected string representations ("8.5", "9.25") to confirm no
    /// precision is lost during conversion from f64 to Decimal. Also
    /// verifies that summing scores into a total f64 and converting to
    /// Decimal preserves the expected value (24.75).
    ///
    /// # Errors
    ///
    /// Panics if any `Decimal::from_f64_retain` conversion yields a value
    /// whose string representation does not exactly match the expected
    /// string, or if the summed total does not equal 24.75.
    ///
    /// # Side Effects
    ///
    /// None. Pure in-memory computation with no I/O.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_score_decimal_precision() {
        let scores: HashMap<String, f64> = HashMap::from([
            ("innovation".to_string(), 8.5),
            ("technical".to_string(), 9.25),
            ("creativity".to_string(), 7.0),
        ]);

        let d_8_5 = Decimal::from_f64_retain(8.5).unwrap_or(Decimal::ZERO);
        let d_9_25 = Decimal::from_f64_retain(9.25).unwrap_or(Decimal::ZERO);

        assert_eq!(d_8_5.to_string(), "8.5");
        assert_eq!(d_9_25.to_string(), "9.25");

        let total: f64 = scores.values().sum();
        assert_eq!(total, 24.75);

        let total_decimal = Decimal::from_f64_retain(total).unwrap_or(Decimal::ZERO);
        assert_eq!(total_decimal.to_string(), "24.75");
    }

    /// Verify that rubric weights sum to 100 or validate correctly.
    ///
    /// # Expected Behavior
    ///
    /// Creates two sets of `CriterionInput` values. The first set has three
    /// criteria with weights 30.0, 40.0, and 30.0, summing to exactly 100.0.
    /// The second set has two criteria with weights 30.0 and 40.0, summing to
    /// 70.0 (not 100.0). Asserts that the weight totals are computed correctly
    /// and that the first set sums to 100.0 while the second does not. This
    /// mirrors the weight accumulation logic in `JudgingService::validate_score`
    /// where `total_weight` is computed by iterating over criteria and summing
    /// their `weight` fields.
    ///
    /// # Errors
    ///
    /// Panics if the computed weight sums do not match expected values.
    ///
    /// # Side Effects
    ///
    /// None. Pure in-memory computation with no I/O.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_rubric_weight_sum() {
        let criteria = vec![
            CriterionInput {
                name: "innovation".into(),
                description: None,
                max_score: 10.0,
                weight: 30.0,
                order: None,
                required: true,
            },
            CriterionInput {
                name: "technical".into(),
                description: None,
                max_score: 10.0,
                weight: 40.0,
                order: None,
                required: true,
            },
            CriterionInput {
                name: "creativity".into(),
                description: None,
                max_score: 10.0,
                weight: 30.0,
                order: None,
                required: true,
            },
        ];

        let total_weight: f64 = criteria.iter().map(|c| c.weight).sum();
        assert_eq!(total_weight, 100.0);

        let bad_criteria = vec![
            CriterionInput {
                name: "innovation".into(),
                description: None,
                max_score: 10.0,
                weight: 30.0,
                order: None,
                required: true,
            },
            CriterionInput {
                name: "technical".into(),
                description: None,
                max_score: 10.0,
                weight: 40.0,
                order: None,
                required: true,
            },
        ];

        let bad_total: f64 = bad_criteria.iter().map(|c| c.weight).sum();
        assert_ne!(bad_total, 100.0);
        assert_eq!(bad_total, 70.0);
    }
}
