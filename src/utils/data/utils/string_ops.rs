use serde_json::Value;

pub struct StringOps;

impl StringOps {
    pub fn sanitize_for_json(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    pub fn extract_json_from_string(input: &str) -> Option<Value> {
        let trimmed = input.trim();

        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                let json_str = &trimmed[start..=end];
                return serde_json::from_str(json_str).ok();
            }
        }

        if let Some(start) = trimmed.find('[') {
            if let Some(end) = trimmed.rfind(']') {
                let json_str = &trimmed[start..=end];
                return serde_json::from_str(json_str).ok();
            }
        }

        serde_json::from_str(trimmed).ok()
    }

    pub fn truncate_string(input: &str, max_length: usize) -> String {
        if input.len() <= max_length {
            input.to_string()
        } else {
            let mut truncated = input
                .chars()
                .take(max_length.saturating_sub(3))
                .collect::<String>();
            truncated.push_str("...");
            truncated
        }
    }

    pub fn extract_urls_from_text(text: &str) -> Vec<String> {
        let url_pattern = regex::Regex::new(
            r"https?://(?:[-\w.])+(?::[0-9]+)?(?:/(?:[\w/_.])*(?:\?(?:[\w&=%.])*)?(?:#(?:[\w.])*)?)?")
            .unwrap();

        url_pattern
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    pub fn clean_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    pub fn word_count(text: &str) -> usize {
        text.split_whitespace().count()
    }

    pub fn character_count(text: &str) -> usize {
        text.chars().count()
    }
}
