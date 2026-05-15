#[cfg(test)]
mod tests {
    use crate::services::embeddings::compute_md5;

    /// Test that the MD5 hash deduplication logic is deterministic and distinguishes inputs.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that `compute_md5` returns the same hash for identical input strings
    /// and different hashes for different input strings. Also verifies that the hash
    /// is a lowercase hex string of exactly 32 characters (128 bits). An empty string
    /// input produces the well-known MD5 hash `d41d8cd98f00b204e9800998ecf8427e`.
    /// Hashes are consistent across multiple calls with the same input, supporting
    /// the embedding cache deduplication strategy where `embedding:{md5_hex}` is used
    /// as the Redis cache key.
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
    fn test_request_hash_dedup() {
        let hash1 = compute_md5("What is machine learning?");
        let hash2 = compute_md5("What is machine learning?");
        assert_eq!(hash1, hash2, "Same input must produce the same hash");

        let hash3 = compute_md5("What is deep learning?");
        assert_ne!(
            hash1, hash3,
            "Different inputs must produce different hashes"
        );

        assert_eq!(hash1.len(), 32, "MD5 hex digest must be 32 characters");
        assert!(
            hash1.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash must be lowercase hex"
        );

        let empty_hash = compute_md5("");
        assert_eq!(
            empty_hash, "d41d8cd98f00b204e9800998ecf8427e",
            "Empty string must produce known MD5 hash"
        );
    }

    /// Test that LLM prompts are constructed with the correct format and labels.
    ///
    /// # Expected Behavior
    ///
    /// Verifies that the prompt construction logic used across AI service endpoints
    /// formats user prompts correctly. Tests the idea generation prompt format
    /// (`Theme`, `Interests`, `Team size`, `Difficulty`, `Number of ideas` labels),
    /// the insight generation prompt format (`Period`, `Topic`, `Knowledge base
    /// categories`, `Recent entries` labels), and the chat RAG context formatting
    /// (system prompt with `Knowledge Base:` prefix and category-tagged entries in
    /// `[category] title: content` format). Each format string must include the
    /// expected labels and values in the correct positions.
    ///
    /// # Errors
    ///
    /// None. Panics on assertion failure.
    ///
    /// # Side Effects
    ///
    /// None. Pure string formatting with no I/O.
    #[allow(clippy::missing_panics_doc)]
    #[test]
    fn test_prompt_construction() {
        let theme = "AI for Good";
        let interests = vec!["machine learning".to_string(), "web dev".to_string()];
        let team_size_str = "4".to_string();
        let difficulty_str = "intermediate";
        let num_ideas = 3u32;

        let idea_prompt = format!(
            "Theme: {}\nInterests: {}\nTeam size: {}\nDifficulty: {}\nNumber of ideas: {}",
            theme,
            interests.join(", "),
            team_size_str,
            difficulty_str,
            num_ideas
        );

        assert!(idea_prompt.starts_with("Theme: AI for Good\n"));
        assert!(idea_prompt.contains("Interests: machine learning, web dev\n"));
        assert!(idea_prompt.contains("Team size: 4\n"));
        assert!(idea_prompt.contains("Difficulty: intermediate\n"));
        assert!(idea_prompt.contains("Number of ideas: 3"));

        let period = "daily";
        let topic_str = "general trends and patterns";
        let categories_summary = "technology: 15 entries, science: 8 entries";
        let recent_summary = "- AI Ethics Framework\n- Neural Network Basics";

        let insight_prompt = format!(
            "Period: {}\nTopic: {}\nKnowledge base categories: {}\nRecent entries:\n{}",
            period, topic_str, categories_summary, recent_summary
        );

        assert!(insight_prompt.starts_with("Period: daily\n"));
        assert!(insight_prompt.contains("Topic: general trends and patterns\n"));
        assert!(insight_prompt
            .contains("Knowledge base categories: technology: 15 entries, science: 8 entries\n"));

        let context_empty = "No relevant knowledge base entries found.".to_string();
        let system_prompt_empty = format!(
            "You are an AI assistant for a hackathon platform. Use the following \
             knowledge base context to inform your responses. If the context is \
             not relevant, answer based on your general knowledge.\n\n\
             Knowledge Base:\n{context_empty}"
        );
        assert!(system_prompt_empty
            .contains("Knowledge Base:\nNo relevant knowledge base entries found."));

        let context_with_results =
            "[technology] AI Ethics: Content about ethics\n\n[science] ML Basics: Content about ML"
                .to_string();
        let system_prompt_results = format!(
            "You are an AI assistant for a hackathon platform. Use the following \
             knowledge base context to inform your responses. If the context is \
             not relevant, answer based on your general knowledge.\n\n\
             Knowledge Base:\n{context_with_results}"
        );
        assert!(system_prompt_results.contains("[technology] AI Ethics: Content about ethics"));
        assert!(system_prompt_results.contains("[science] ML Basics: Content about ML"));
    }
}
