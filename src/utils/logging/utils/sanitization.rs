pub struct Sanitization;

impl Sanitization {
    pub fn sanitize_log_data(data: &str) -> String {
        let sensitive_patterns = [
            r"(?i)api[_-]?key[=:\s]*['\x22]?([a-zA-Z0-9\-_]+)['\x22]?",
            r"(?i)token[=:\s]*['\x22]?([a-zA-Z0-9\-_.]+)['\x22]?",
            r"(?i)password[=:\s]*['\x22]?([^\s'\x22]+)['\x22]?",
            r"(?i)secret[=:\s]*['\x22]?([^\s'\x22]+)['\x22]?",
        ];

        let mut sanitized = data.to_string();

        for pattern in &sensitive_patterns {
            let re = regex::Regex::new(pattern).unwrap();
            sanitized = re.replace_all(&sanitized, "***REDACTED***").to_string();
        }

        sanitized
    }

    pub fn mask_sensitive_data(input: &str) -> String {
        let sensitive_keys = [
            "api_key",
            "token",
            "password",
            "secret",
            "auth",
            "credential",
        ];

        let mut result = input.to_string();

        for key in &sensitive_keys {
            let patterns = [
                format!(r#""{}"\s*:\s*"([^"]+)""#, key),
                format!(r#"'{}'\s*:\s*'([^']+)'"#, key),
                format!(r#"{}[=:]\s*([^\s,}}\]]+)"#, key),
            ];

            for pattern in &patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    result = re
                        .replace_all(&result, |caps: &regex::Captures| {
                            let full_match = caps.get(0).unwrap().as_str();
                            let value = caps.get(1).unwrap().as_str();
                            let masked_value = if value.len() > 8 {
                                format!("{}***{}", &value[..2], &value[value.len() - 2..])
                            } else {
                                "***".to_string()
                            };
                            full_match.replace(value, &masked_value)
                        })
                        .to_string();
                }
            }
        }

        result
    }
}
