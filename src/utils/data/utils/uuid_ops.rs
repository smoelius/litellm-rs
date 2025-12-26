use uuid::Uuid;

pub struct UuidOps;

impl UuidOps {
    pub fn generate_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn generate_short_id() -> String {
        Uuid::new_v4().to_string()[..8].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== generate_uuid Tests ====================

    #[test]
    fn test_generate_uuid_format() {
        let uuid = UuidOps::generate_uuid();
        // UUID v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx (36 chars with hyphens)
        assert_eq!(uuid.len(), 36);
        assert_eq!(&uuid[8..9], "-");
        assert_eq!(&uuid[13..14], "-");
        assert_eq!(&uuid[18..19], "-");
        assert_eq!(&uuid[23..24], "-");
    }

    #[test]
    fn test_generate_uuid_version_4() {
        let uuid = UuidOps::generate_uuid();
        // UUID v4 has '4' at position 14
        assert_eq!(&uuid[14..15], "4");
    }

    #[test]
    fn test_generate_uuid_uniqueness() {
        let uuid1 = UuidOps::generate_uuid();
        let uuid2 = UuidOps::generate_uuid();
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_generate_uuid_hex_chars() {
        let uuid = UuidOps::generate_uuid();
        let chars_without_hyphens: String = uuid.chars().filter(|c| *c != '-').collect();
        assert!(chars_without_hyphens.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_uuid_parseable() {
        let uuid_str = UuidOps::generate_uuid();
        let parsed = Uuid::parse_str(&uuid_str);
        assert!(parsed.is_ok());
    }

    // ==================== generate_short_id Tests ====================

    #[test]
    fn test_generate_short_id_length() {
        let short_id = UuidOps::generate_short_id();
        assert_eq!(short_id.len(), 8);
    }

    #[test]
    fn test_generate_short_id_hex_chars() {
        let short_id = UuidOps::generate_short_id();
        assert!(short_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_short_id_uniqueness() {
        let id1 = UuidOps::generate_short_id();
        let id2 = UuidOps::generate_short_id();
        // Very unlikely to be the same (1 in 4 billion)
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_short_id_no_hyphens() {
        let short_id = UuidOps::generate_short_id();
        assert!(!short_id.contains('-'));
    }
}
