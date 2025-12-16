#[cfg(test)]
mod tests {
    use crate::utils::data::utils::DataUtils;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn test_base64_operations() {
        let original = "Hello, World!";
        let encoded = DataUtils::encode_base64(original);
        assert!(DataUtils::is_base64_encoded(&encoded));

        let decoded = DataUtils::decode_base64(&encoded).unwrap();
        assert_eq!(decoded, original);

        assert!(!DataUtils::is_base64_encoded("not base64!"));
    }

    #[test]
    fn test_json_operations() {
        let data = json!({
            "name": "test",
            "value": 42,
            "nested": {
                "inner": "data"
            }
        });

        let dict = DataUtils::convert_to_dict(&data).unwrap();
        assert!(dict.contains_key("name"));
        assert!(dict.contains_key("nested"));
    }

    #[test]
    fn test_uuid_generation() {
        let uuid1 = DataUtils::generate_uuid();
        let uuid2 = DataUtils::generate_uuid();
        assert_ne!(uuid1, uuid2);
        assert!(Uuid::parse_str(&uuid1).is_ok());

        let short_id = DataUtils::generate_short_id();
        assert_eq!(short_id.len(), 8);
    }

    #[test]
    fn test_json_merging() {
        let mut base = json!({
            "a": 1,
            "b": {
                "c": 2
            }
        });

        let overlay = json!({
            "b": {
                "d": 3
            },
            "e": 4
        });

        DataUtils::merge_json_objects(&mut base, &overlay).unwrap();

        assert_eq!(base["a"], json!(1));
        assert_eq!(base["b"]["c"], json!(2));
        assert_eq!(base["b"]["d"], json!(3));
        assert_eq!(base["e"], json!(4));
    }

    #[test]
    fn test_nested_value_extraction() {
        let data = json!({
            "level1": {
                "level2": {
                    "value": "found"
                }
            },
            "array": [1, 2, {"key": "value"}]
        });

        let value = DataUtils::extract_nested_value(&data, &["level1", "level2", "value"]);
        assert_eq!(value, Some(&json!("found")));

        let array_value = DataUtils::extract_nested_value(&data, &["array", "2", "key"]);
        assert_eq!(array_value, Some(&json!("value")));

        let missing = DataUtils::extract_nested_value(&data, &["missing", "path"]);
        assert_eq!(missing, None);
    }

    #[test]
    fn test_json_flattening() {
        let data = json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": {
                    "e": 3
                }
            },
            "f": [1, 2, 3]
        });

        let flattened = DataUtils::flatten_json(&data, None);
        assert_eq!(flattened.get("a"), Some(&json!(1)));
        assert_eq!(flattened.get("b.c"), Some(&json!(2)));
        assert_eq!(flattened.get("b.d.e"), Some(&json!(3)));
        assert_eq!(flattened.get("f.0"), Some(&json!(1)));
    }

    #[test]
    fn test_json_schema_validation() {
        let data = json!({
            "name": "test",
            "age": 25
        });

        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        });

        assert!(DataUtils::validate_json_schema(&data, &schema).is_ok());

        let invalid_data = json!({
            "age": "not a number"
        });

        assert!(DataUtils::validate_json_schema(&invalid_data, &schema).is_err());
    }

    #[test]
    fn test_string_utilities() {
        assert_eq!(
            DataUtils::truncate_string("Hello, World!", 10),
            "Hello, ..."
        );
        assert_eq!(DataUtils::truncate_string("Short", 10), "Short");

        assert_eq!(
            DataUtils::clean_whitespace("  Hello   world  "),
            "Hello world"
        );

        assert_eq!(DataUtils::word_count("Hello world test"), 3);
        assert_eq!(DataUtils::character_count("Hello 🌍"), 7);
    }

    #[test]
    fn test_url_extraction() {
        let text = "Check out https://example.com and http://test.org/path?query=1";
        let urls = DataUtils::extract_urls_from_text(text);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://example.com".to_string()));
        assert!(urls.contains(&"http://test.org/path?query=1".to_string()));
    }

    #[test]
    fn test_json_extraction_from_string() {
        let text = "Here is some JSON: {\"key\": \"value\"} and more text";
        let extracted = DataUtils::extract_json_from_string(text);
        assert_eq!(extracted, Some(json!({"key": "value"})));

        let no_json = "This has no JSON content";
        let no_extracted = DataUtils::extract_json_from_string(no_json);
        assert_eq!(no_extracted, None);
    }

    #[test]
    fn test_json_utilities() {
        let data = json!({"test": "value"});

        let pretty = DataUtils::pretty_print_json(&data).unwrap();
        assert!(pretty.contains("  "));

        let compact = DataUtils::compact_json(&data).unwrap();
        assert!(!compact.contains("  "));

        let hash = DataUtils::hash_json(&data).unwrap();
        assert_eq!(hash.len(), 64); // SHA-256 hex string length

        let size = DataUtils::json_size_bytes(&data);
        assert!(size > 0);
    }
}
