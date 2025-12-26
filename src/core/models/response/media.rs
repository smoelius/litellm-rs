//! Media response types (Image and Audio)

use serde::{Deserialize, Serialize};

/// Image generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Creation timestamp
    pub created: u64,
    /// Generated images
    pub data: Vec<ImageData>,
}

/// Image data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// Image URL
    pub url: Option<String>,
    /// Base64 encoded image
    pub b64_json: Option<String>,
    /// Revised prompt
    pub revised_prompt: Option<String>,
}

/// Audio transcription response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionResponse {
    /// Transcribed text
    pub text: String,
    /// Language (if detected)
    pub language: Option<String>,
    /// Duration
    pub duration: Option<f64>,
    /// Segments (if requested)
    pub segments: Option<Vec<TranscriptionSegment>>,
    /// Words (if requested)
    pub words: Option<Vec<TranscriptionWord>>,
}

/// Transcription segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    /// Segment ID
    pub id: u32,
    /// Seek offset
    pub seek: u32,
    /// Start time
    pub start: f64,
    /// End time
    pub end: f64,
    /// Segment text
    pub text: String,
    /// Tokens
    pub tokens: Vec<u32>,
    /// Temperature
    pub temperature: f64,
    /// Average log probability
    pub avg_logprob: f64,
    /// Compression ratio
    pub compression_ratio: f64,
    /// No speech probability
    pub no_speech_prob: f64,
}

/// Transcription word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionWord {
    /// Word text
    pub word: String,
    /// Start time
    pub start: f64,
    /// End time
    pub end: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ImageGenerationResponse Tests ====================

    #[test]
    fn test_image_generation_response_structure() {
        let response = ImageGenerationResponse {
            created: 1700000000,
            data: vec![],
        };

        assert_eq!(response.created, 1700000000);
        assert!(response.data.is_empty());
    }

    #[test]
    fn test_image_generation_response_with_images() {
        let response = ImageGenerationResponse {
            created: 1700000000,
            data: vec![
                ImageData {
                    url: Some("https://example.com/image1.png".to_string()),
                    b64_json: None,
                    revised_prompt: Some("A beautiful sunset".to_string()),
                },
                ImageData {
                    url: Some("https://example.com/image2.png".to_string()),
                    b64_json: None,
                    revised_prompt: None,
                },
            ],
        };

        assert_eq!(response.data.len(), 2);
        assert!(response.data[0].revised_prompt.is_some());
        assert!(response.data[1].revised_prompt.is_none());
    }

    #[test]
    fn test_image_generation_response_serialization() {
        let response = ImageGenerationResponse {
            created: 1700000000,
            data: vec![ImageData {
                url: Some("https://example.com/image.png".to_string()),
                b64_json: None,
                revised_prompt: None,
            }],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["created"], 1700000000);
        assert!(json["data"].is_array());
    }

    #[test]
    fn test_image_generation_response_clone() {
        let response = ImageGenerationResponse {
            created: 1700000000,
            data: vec![],
        };

        let cloned = response.clone();
        assert_eq!(response.created, cloned.created);
    }

    // ==================== ImageData Tests ====================

    #[test]
    fn test_image_data_url_variant() {
        let data = ImageData {
            url: Some("https://cdn.openai.com/image.png".to_string()),
            b64_json: None,
            revised_prompt: None,
        };

        assert!(data.url.is_some());
        assert!(data.b64_json.is_none());
    }

    #[test]
    fn test_image_data_b64_variant() {
        let data = ImageData {
            url: None,
            b64_json: Some("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJ...".to_string()),
            revised_prompt: None,
        };

        assert!(data.url.is_none());
        assert!(data.b64_json.is_some());
    }

    #[test]
    fn test_image_data_with_revised_prompt() {
        let data = ImageData {
            url: Some("https://example.com/image.png".to_string()),
            b64_json: None,
            revised_prompt: Some("A stunning mountain landscape at dawn".to_string()),
        };

        assert_eq!(
            data.revised_prompt,
            Some("A stunning mountain landscape at dawn".to_string())
        );
    }

    #[test]
    fn test_image_data_serialization() {
        let data = ImageData {
            url: Some("https://example.com/image.png".to_string()),
            b64_json: None,
            revised_prompt: Some("Revised prompt".to_string()),
        };

        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["url"], "https://example.com/image.png");
        assert!(json["b64_json"].is_null());
        assert_eq!(json["revised_prompt"], "Revised prompt");
    }

    #[test]
    fn test_image_data_deserialization() {
        let json = r#"{
            "url": "https://example.com/image.png",
            "b64_json": null,
            "revised_prompt": "Test prompt"
        }"#;

        let data: ImageData = serde_json::from_str(json).unwrap();
        assert_eq!(data.url, Some("https://example.com/image.png".to_string()));
        assert!(data.b64_json.is_none());
    }

    // ==================== AudioTranscriptionResponse Tests ====================

    #[test]
    fn test_audio_transcription_response_simple() {
        let response = AudioTranscriptionResponse {
            text: "Hello, world!".to_string(),
            language: None,
            duration: None,
            segments: None,
            words: None,
        };

        assert_eq!(response.text, "Hello, world!");
        assert!(response.language.is_none());
    }

    #[test]
    fn test_audio_transcription_response_full() {
        let response = AudioTranscriptionResponse {
            text: "Hello, world!".to_string(),
            language: Some("en".to_string()),
            duration: Some(5.5),
            segments: Some(vec![]),
            words: Some(vec![]),
        };

        assert_eq!(response.language, Some("en".to_string()));
        assert!((response.duration.unwrap() - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_audio_transcription_response_with_segments() {
        let segment = TranscriptionSegment {
            id: 0,
            seek: 0,
            start: 0.0,
            end: 2.5,
            text: "Hello".to_string(),
            tokens: vec![15339],
            temperature: 0.0,
            avg_logprob: -0.5,
            compression_ratio: 1.2,
            no_speech_prob: 0.01,
        };

        let response = AudioTranscriptionResponse {
            text: "Hello".to_string(),
            language: Some("en".to_string()),
            duration: Some(2.5),
            segments: Some(vec![segment]),
            words: None,
        };

        assert_eq!(response.segments.as_ref().unwrap().len(), 1);
        assert_eq!(response.segments.as_ref().unwrap()[0].text, "Hello");
    }

    #[test]
    fn test_audio_transcription_response_serialization() {
        let response = AudioTranscriptionResponse {
            text: "Test transcription".to_string(),
            language: Some("en".to_string()),
            duration: Some(10.0),
            segments: None,
            words: None,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["text"], "Test transcription");
        assert_eq!(json["language"], "en");
        assert_eq!(json["duration"], 10.0);
    }

    // ==================== TranscriptionSegment Tests ====================

    #[test]
    fn test_transcription_segment_structure() {
        let segment = TranscriptionSegment {
            id: 0,
            seek: 0,
            start: 0.0,
            end: 5.0,
            text: "First segment".to_string(),
            tokens: vec![1, 2, 3, 4],
            temperature: 0.0,
            avg_logprob: -0.25,
            compression_ratio: 1.5,
            no_speech_prob: 0.001,
        };

        assert_eq!(segment.id, 0);
        assert!((segment.start - 0.0).abs() < f64::EPSILON);
        assert!((segment.end - 5.0).abs() < f64::EPSILON);
        assert_eq!(segment.tokens.len(), 4);
    }

    #[test]
    fn test_transcription_segment_serialization() {
        let segment = TranscriptionSegment {
            id: 1,
            seek: 100,
            start: 5.0,
            end: 10.0,
            text: "Test".to_string(),
            tokens: vec![100, 200],
            temperature: 0.5,
            avg_logprob: -0.3,
            compression_ratio: 1.2,
            no_speech_prob: 0.05,
        };

        let json = serde_json::to_value(&segment).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["seek"], 100);
        assert_eq!(json["text"], "Test");
    }

    #[test]
    fn test_transcription_segment_clone() {
        let segment = TranscriptionSegment {
            id: 0,
            seek: 0,
            start: 0.0,
            end: 1.0,
            text: "Clone test".to_string(),
            tokens: vec![],
            temperature: 0.0,
            avg_logprob: 0.0,
            compression_ratio: 1.0,
            no_speech_prob: 0.0,
        };

        let cloned = segment.clone();
        assert_eq!(segment.text, cloned.text);
        assert_eq!(segment.id, cloned.id);
    }

    // ==================== TranscriptionWord Tests ====================

    #[test]
    fn test_transcription_word_structure() {
        let word = TranscriptionWord {
            word: "Hello".to_string(),
            start: 0.0,
            end: 0.5,
        };

        assert_eq!(word.word, "Hello");
        assert!((word.start - 0.0).abs() < f64::EPSILON);
        assert!((word.end - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transcription_word_serialization() {
        let word = TranscriptionWord {
            word: "world".to_string(),
            start: 0.5,
            end: 1.0,
        };

        let json = serde_json::to_value(&word).unwrap();
        assert_eq!(json["word"], "world");
        assert_eq!(json["start"], 0.5);
        assert_eq!(json["end"], 1.0);
    }

    #[test]
    fn test_transcription_word_clone() {
        let word = TranscriptionWord {
            word: "test".to_string(),
            start: 1.0,
            end: 1.5,
        };

        let cloned = word.clone();
        assert_eq!(word.word, cloned.word);
    }

    #[test]
    fn test_transcription_word_deserialization() {
        let json = r#"{"word": "example", "start": 2.0, "end": 2.5}"#;
        let word: TranscriptionWord = serde_json::from_str(json).unwrap();

        assert_eq!(word.word, "example");
        assert!((word.start - 2.0).abs() < f64::EPSILON);
        assert!((word.end - 2.5).abs() < f64::EPSILON);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_transcription_response() {
        let words = vec![
            TranscriptionWord {
                word: "Hello".to_string(),
                start: 0.0,
                end: 0.3,
            },
            TranscriptionWord {
                word: "world".to_string(),
                start: 0.4,
                end: 0.7,
            },
        ];

        let segments = vec![TranscriptionSegment {
            id: 0,
            seek: 0,
            start: 0.0,
            end: 1.0,
            text: "Hello world".to_string(),
            tokens: vec![15339, 995],
            temperature: 0.0,
            avg_logprob: -0.2,
            compression_ratio: 1.1,
            no_speech_prob: 0.005,
        }];

        let response = AudioTranscriptionResponse {
            text: "Hello world".to_string(),
            language: Some("en".to_string()),
            duration: Some(1.0),
            segments: Some(segments),
            words: Some(words),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: AudioTranscriptionResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.text, "Hello world");
        assert_eq!(deserialized.words.as_ref().unwrap().len(), 2);
        assert_eq!(deserialized.segments.as_ref().unwrap().len(), 1);
    }
}
