//! Profanity filtering
//!
//! Tools for detecting and filtering profane content.

/// Profanity filter
#[derive(Debug, Clone)]
pub struct ProfanityFilter {
    /// Blocked words list
    blocked_words: Vec<String>,
    /// Replacement character
    replacement_char: char,
    /// Whether to use fuzzy matching
    fuzzy_matching: bool,
}

impl Default for ProfanityFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfanityFilter {
    /// Create a new profanity filter
    pub fn new() -> Self {
        Self {
            blocked_words: vec![
                "badword1".to_string(),
                "badword2".to_string(),
                // Add more blocked words
            ],
            replacement_char: '*',
            fuzzy_matching: true,
        }
    }

    /// Check if text contains profanity
    pub fn contains_profanity(&self, text: &str) -> bool {
        let lower_text = text.to_lowercase();
        self.blocked_words
            .iter()
            .any(|word| lower_text.contains(word))
    }

    /// Filter profanity from text
    pub fn filter(&self, text: &str) -> String {
        let mut result = text.to_string();
        for word in &self.blocked_words {
            let replacement = self.replacement_char.to_string().repeat(word.len());
            result = result.replace(word, &replacement);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profanity_filter() {
        let filter = ProfanityFilter::new();
        assert!(filter.contains_profanity("This contains badword1"));

        let filtered = filter.filter("This contains badword1");
        assert!(!filtered.contains("badword1"));
    }
}
