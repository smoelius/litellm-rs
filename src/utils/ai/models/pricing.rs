use super::utils::ModelUtils;

impl ModelUtils {
    pub fn get_model_pricing(model: &str) -> Option<(f64, f64)> {
        let model_lower = model.to_lowercase();

        match model_lower.as_str() {
            m if m.starts_with("gpt-4-turbo") => Some((0.01, 0.03)),
            m if m.starts_with("gpt-4") => Some((0.03, 0.06)),
            m if m.starts_with("gpt-3.5-turbo") => Some((0.0015, 0.002)),
            m if m.contains("claude-3-opus") => Some((0.015, 0.075)),
            m if m.contains("claude-3-sonnet") => Some((0.003, 0.015)),
            m if m.contains("claude-3-haiku") => Some((0.00025, 0.00125)),
            m if m.starts_with("gemini-pro") => Some((0.0005, 0.0015)),
            _ => None,
        }
    }

    pub fn get_model_aliases(model: &str) -> Vec<String> {
        let model_lower = model.to_lowercase();
        let mut aliases = vec![];

        match model_lower.as_str() {
            "gpt-4" => {
                aliases.extend_from_slice(&[
                    "openai/gpt-4".to_string(),
                    "gpt-4-0314".to_string(),
                    "gpt-4-0613".to_string(),
                ]);
            }
            "claude-3-opus" => {
                aliases.extend_from_slice(&[
                    "anthropic/claude-3-opus".to_string(),
                    "claude-3-opus-20240229".to_string(),
                ]);
            }
            "gemini-pro" => {
                aliases.extend_from_slice(&[
                    "google/gemini-pro".to_string(),
                    "gemini-1.0-pro".to_string(),
                ]);
            }
            _ => {}
        }

        aliases
    }

    pub fn is_chat_model(model: &str) -> bool {
        let model_lower = model.to_lowercase();

        let chat_patterns = ["gpt-", "claude-", "gemini-", "command", "llama", "mistral"];

        chat_patterns
            .iter()
            .any(|pattern| model_lower.contains(pattern))
    }

    pub fn is_completion_model(model: &str) -> bool {
        let model_lower = model.to_lowercase();

        let completion_patterns = [
            "text-davinci",
            "text-curie",
            "text-babbage",
            "text-ada",
            "davinci",
            "curie",
        ];

        completion_patterns
            .iter()
            .any(|pattern| model_lower.contains(pattern))
    }

    pub fn get_recommended_temperature(model: &str) -> f32 {
        match Self::get_model_family(model).as_str() {
            "gpt" => 0.7,
            "claude" => 0.9,
            "gemini" => 0.8,
            "command" => 0.8,
            _ => 0.7,
        }
    }
}
