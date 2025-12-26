//! Content part types for multimodal messages

use serde::{Deserialize, Serialize};

/// Content part (multimodal support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },

    /// Image URL
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },

    /// Audio data
    #[serde(rename = "audio")]
    Audio { audio: AudioData },

    /// Base64 encoded image
    #[serde(rename = "image")]
    Image {
        /// Base64 encoded image data
        source: ImageSource,
        /// Image detail level
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
        /// Image URL (compatibility field)
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<ImageUrl>,
    },

    /// Document content (PDF etc)
    #[serde(rename = "document")]
    Document {
        /// Document source data
        source: DocumentSource,
        /// Cache control (Anthropic specific)
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },

    /// Tool result
    #[serde(rename = "tool_result")]
    ToolResult {
        /// Tool usage ID
        tool_use_id: String,
        /// Result content
        content: serde_json::Value,
        /// Error flag
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    /// Tool usage
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Tool usage ID
        id: String,
        /// Tool name
        name: String,
        /// Tool input
        input: serde_json::Value,
    },
}

/// Image URL structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    /// Image URL
    pub url: String,
    /// Detail level ("auto", "low", "high")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Image source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    /// Media type
    pub media_type: String,
    /// Base64 encoded data
    pub data: String,
}

/// Audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    /// Base64 encoded audio data
    pub data: String,
    /// Audio format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Input audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudio {
    /// Base64 encoded audio data
    pub data: String,
    /// Audio format
    pub format: String,
}

/// Document source data (Anthropic PDF support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSource {
    /// Media type (application/pdf)
    pub media_type: String,
    /// Base64 encoded data
    pub data: String,
}

/// Cache control (Anthropic Cache Control)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    /// Cache type ("ephemeral", "persistent")
    #[serde(rename = "type")]
    pub cache_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ContentPart Tests ====================

    #[test]
    fn test_content_part_text_serialization() {
        let part = ContentPart::Text {
            text: "Hello world".to_string(),
        };
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello world");
    }

    #[test]
    fn test_content_part_text_deserialization() {
        let json = r#"{"type": "text", "text": "Test message"}"#;
        let part: ContentPart = serde_json::from_str(json).unwrap();
        match part {
            ContentPart::Text { text } => assert_eq!(text, "Test message"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_content_part_image_url_serialization() {
        let part = ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: "https://example.com/image.png".to_string(),
                detail: Some("high".to_string()),
            },
        };
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["type"], "image_url");
        assert_eq!(json["image_url"]["url"], "https://example.com/image.png");
    }

    #[test]
    fn test_content_part_audio_serialization() {
        let part = ContentPart::Audio {
            audio: AudioData {
                data: "base64data".to_string(),
                format: Some("mp3".to_string()),
            },
        };
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["type"], "audio");
        assert_eq!(json["audio"]["data"], "base64data");
    }

    #[test]
    fn test_content_part_image_serialization() {
        let part = ContentPart::Image {
            source: ImageSource {
                media_type: "image/png".to_string(),
                data: "iVBORw0...".to_string(),
            },
            detail: Some("auto".to_string()),
            image_url: None,
        };
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["source"]["media_type"], "image/png");
    }

    #[test]
    fn test_content_part_document_serialization() {
        let part = ContentPart::Document {
            source: DocumentSource {
                media_type: "application/pdf".to_string(),
                data: "JVBERi0...".to_string(),
            },
            cache_control: Some(CacheControl {
                cache_type: "ephemeral".to_string(),
            }),
        };
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["type"], "document");
        assert_eq!(json["source"]["media_type"], "application/pdf");
    }

    #[test]
    fn test_content_part_tool_result_serialization() {
        let part = ContentPart::ToolResult {
            tool_use_id: "tool-123".to_string(),
            content: serde_json::json!({"result": "success"}),
            is_error: Some(false),
        };
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["type"], "tool_result");
        assert_eq!(json["tool_use_id"], "tool-123");
    }

    #[test]
    fn test_content_part_tool_use_serialization() {
        let part = ContentPart::ToolUse {
            id: "use-456".to_string(),
            name: "get_weather".to_string(),
            input: serde_json::json!({"city": "Tokyo"}),
        };
        let json = serde_json::to_value(&part).unwrap();
        assert_eq!(json["type"], "tool_use");
        assert_eq!(json["name"], "get_weather");
    }

    #[test]
    fn test_content_part_clone() {
        let part = ContentPart::Text {
            text: "clone test".to_string(),
        };
        let cloned = part.clone();
        match (part, cloned) {
            (ContentPart::Text { text: a }, ContentPart::Text { text: b }) => assert_eq!(a, b),
            _ => panic!("Clone mismatch"),
        }
    }

    // ==================== ImageUrl Tests ====================

    #[test]
    fn test_image_url_structure() {
        let url = ImageUrl {
            url: "https://example.com/img.jpg".to_string(),
            detail: None,
        };
        assert_eq!(url.url, "https://example.com/img.jpg");
        assert!(url.detail.is_none());
    }

    #[test]
    fn test_image_url_with_detail() {
        let url = ImageUrl {
            url: "https://example.com/img.jpg".to_string(),
            detail: Some("low".to_string()),
        };
        assert_eq!(url.detail, Some("low".to_string()));
    }

    #[test]
    fn test_image_url_serialization_skip_none() {
        let url = ImageUrl {
            url: "https://example.com/img.jpg".to_string(),
            detail: None,
        };
        let json = serde_json::to_value(&url).unwrap();
        assert!(!json.as_object().unwrap().contains_key("detail"));
    }

    #[test]
    fn test_image_url_clone() {
        let url = ImageUrl {
            url: "https://example.com/img.jpg".to_string(),
            detail: Some("high".to_string()),
        };
        let cloned = url.clone();
        assert_eq!(url.url, cloned.url);
        assert_eq!(url.detail, cloned.detail);
    }

    // ==================== ImageSource Tests ====================

    #[test]
    fn test_image_source_structure() {
        let source = ImageSource {
            media_type: "image/jpeg".to_string(),
            data: "base64data".to_string(),
        };
        assert_eq!(source.media_type, "image/jpeg");
        assert_eq!(source.data, "base64data");
    }

    #[test]
    fn test_image_source_serialization() {
        let source = ImageSource {
            media_type: "image/png".to_string(),
            data: "iVBORw0KGgo=".to_string(),
        };
        let json = serde_json::to_value(&source).unwrap();
        assert_eq!(json["media_type"], "image/png");
        assert_eq!(json["data"], "iVBORw0KGgo=");
    }

    #[test]
    fn test_image_source_clone() {
        let source = ImageSource {
            media_type: "image/gif".to_string(),
            data: "R0lGODlh".to_string(),
        };
        let cloned = source.clone();
        assert_eq!(source.media_type, cloned.media_type);
    }

    // ==================== AudioData Tests ====================

    #[test]
    fn test_audio_data_structure() {
        let audio = AudioData {
            data: "audio_base64".to_string(),
            format: Some("wav".to_string()),
        };
        assert_eq!(audio.data, "audio_base64");
        assert_eq!(audio.format, Some("wav".to_string()));
    }

    #[test]
    fn test_audio_data_no_format() {
        let audio = AudioData {
            data: "audio_base64".to_string(),
            format: None,
        };
        assert!(audio.format.is_none());
    }

    #[test]
    fn test_audio_data_serialization() {
        let audio = AudioData {
            data: "audio".to_string(),
            format: Some("mp3".to_string()),
        };
        let json = serde_json::to_value(&audio).unwrap();
        assert_eq!(json["data"], "audio");
        assert_eq!(json["format"], "mp3");
    }

    #[test]
    fn test_audio_data_clone() {
        let audio = AudioData {
            data: "clone".to_string(),
            format: Some("ogg".to_string()),
        };
        let cloned = audio.clone();
        assert_eq!(audio.data, cloned.data);
    }

    // ==================== InputAudio Tests ====================

    #[test]
    fn test_input_audio_structure() {
        let audio = InputAudio {
            data: "input_audio".to_string(),
            format: "pcm16".to_string(),
        };
        assert_eq!(audio.data, "input_audio");
        assert_eq!(audio.format, "pcm16");
    }

    #[test]
    fn test_input_audio_serialization() {
        let audio = InputAudio {
            data: "data".to_string(),
            format: "mp3".to_string(),
        };
        let json = serde_json::to_value(&audio).unwrap();
        assert_eq!(json["data"], "data");
        assert_eq!(json["format"], "mp3");
    }

    #[test]
    fn test_input_audio_clone() {
        let audio = InputAudio {
            data: "test".to_string(),
            format: "wav".to_string(),
        };
        let cloned = audio.clone();
        assert_eq!(audio.format, cloned.format);
    }

    // ==================== DocumentSource Tests ====================

    #[test]
    fn test_document_source_structure() {
        let doc = DocumentSource {
            media_type: "application/pdf".to_string(),
            data: "pdf_base64".to_string(),
        };
        assert_eq!(doc.media_type, "application/pdf");
        assert_eq!(doc.data, "pdf_base64");
    }

    #[test]
    fn test_document_source_serialization() {
        let doc = DocumentSource {
            media_type: "application/pdf".to_string(),
            data: "JVBERi0=".to_string(),
        };
        let json = serde_json::to_value(&doc).unwrap();
        assert_eq!(json["media_type"], "application/pdf");
    }

    #[test]
    fn test_document_source_clone() {
        let doc = DocumentSource {
            media_type: "application/pdf".to_string(),
            data: "data".to_string(),
        };
        let cloned = doc.clone();
        assert_eq!(doc.media_type, cloned.media_type);
    }

    // ==================== CacheControl Tests ====================

    #[test]
    fn test_cache_control_structure() {
        let cache = CacheControl {
            cache_type: "ephemeral".to_string(),
        };
        assert_eq!(cache.cache_type, "ephemeral");
    }

    #[test]
    fn test_cache_control_serialization() {
        let cache = CacheControl {
            cache_type: "persistent".to_string(),
        };
        let json = serde_json::to_value(&cache).unwrap();
        assert_eq!(json["type"], "persistent");
    }

    #[test]
    fn test_cache_control_deserialization() {
        let json = r#"{"type": "ephemeral"}"#;
        let cache: CacheControl = serde_json::from_str(json).unwrap();
        assert_eq!(cache.cache_type, "ephemeral");
    }

    #[test]
    fn test_cache_control_clone() {
        let cache = CacheControl {
            cache_type: "ephemeral".to_string(),
        };
        let cloned = cache.clone();
        assert_eq!(cache.cache_type, cloned.cache_type);
    }
}
