//! Audio transcription response types

use serde::{Deserialize, Serialize};

/// Audio transcription response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionResponse {
    /// Transcription text
    pub text: String,

    /// Language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Duration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// Word details (when enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<WordInfo>>,

    /// Segment information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<SegmentInfo>>,
}

/// Word information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordInfo {
    /// Word text
    pub word: String,

    /// Start time
    pub start: f64,

    /// End time
    pub end: f64,
}

/// Segment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInfo {
    /// Segment ID
    pub id: u32,

    /// Start time
    pub start: f64,

    /// End time
    pub end: f64,

    /// Text content
    pub text: String,

    /// Temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Average log probability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_logprob: Option<f64>,

    /// Compression ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<f64>,

    /// No speech probability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_speech_prob: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== WordInfo Tests ====================

    #[test]
    fn test_word_info_structure() {
        let word = WordInfo {
            word: "hello".to_string(),
            start: 0.0,
            end: 0.5,
        };
        assert_eq!(word.word, "hello");
        assert!((word.start - 0.0).abs() < f64::EPSILON);
        assert!((word.end - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_word_info_serialization() {
        let word = WordInfo {
            word: "world".to_string(),
            start: 1.0,
            end: 1.5,
        };
        let json = serde_json::to_value(&word).unwrap();
        assert_eq!(json["word"], "world");
        assert_eq!(json["start"], 1.0);
        assert_eq!(json["end"], 1.5);
    }

    #[test]
    fn test_word_info_deserialization() {
        let json = r#"{"word": "test", "start": 2.0, "end": 2.5}"#;
        let word: WordInfo = serde_json::from_str(json).unwrap();
        assert_eq!(word.word, "test");
        assert!((word.start - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_word_info_clone() {
        let word = WordInfo {
            word: "clone".to_string(),
            start: 0.0,
            end: 0.3,
        };
        let cloned = word.clone();
        assert_eq!(word.word, cloned.word);
        assert_eq!(word.start, cloned.start);
    }

    // ==================== SegmentInfo Tests ====================

    #[test]
    fn test_segment_info_structure() {
        let segment = SegmentInfo {
            id: 0,
            start: 0.0,
            end: 5.0,
            text: "This is a segment.".to_string(),
            temperature: None,
            avg_logprob: None,
            compression_ratio: None,
            no_speech_prob: None,
        };
        assert_eq!(segment.id, 0);
        assert_eq!(segment.text, "This is a segment.");
    }

    #[test]
    fn test_segment_info_with_metadata() {
        let segment = SegmentInfo {
            id: 1,
            start: 5.0,
            end: 10.0,
            text: "Another segment".to_string(),
            temperature: Some(0.5),
            avg_logprob: Some(-0.3),
            compression_ratio: Some(1.5),
            no_speech_prob: Some(0.02),
        };
        assert!((segment.temperature.unwrap() - 0.5).abs() < f32::EPSILON);
        assert!((segment.avg_logprob.unwrap() - (-0.3)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_segment_info_serialization() {
        let segment = SegmentInfo {
            id: 2,
            start: 10.0,
            end: 15.0,
            text: "Segment text".to_string(),
            temperature: Some(0.7),
            avg_logprob: None,
            compression_ratio: None,
            no_speech_prob: None,
        };
        let json = serde_json::to_value(&segment).unwrap();
        assert_eq!(json["id"], 2);
        assert_eq!(json["text"], "Segment text");
        assert!((json["temperature"].as_f64().unwrap() - 0.7).abs() < 0.001);
        assert!(json.get("avg_logprob").is_none());
    }

    #[test]
    fn test_segment_info_skip_none() {
        let segment = SegmentInfo {
            id: 0,
            start: 0.0,
            end: 1.0,
            text: "test".to_string(),
            temperature: None,
            avg_logprob: None,
            compression_ratio: None,
            no_speech_prob: None,
        };
        let json = serde_json::to_value(&segment).unwrap();
        let obj = json.as_object().unwrap();
        assert!(!obj.contains_key("temperature"));
        assert!(!obj.contains_key("avg_logprob"));
        assert!(!obj.contains_key("compression_ratio"));
        assert!(!obj.contains_key("no_speech_prob"));
    }

    #[test]
    fn test_segment_info_deserialization() {
        let json = r#"{
            "id": 3,
            "start": 20.0,
            "end": 25.0,
            "text": "Deserialized segment",
            "temperature": 0.8
        }"#;
        let segment: SegmentInfo = serde_json::from_str(json).unwrap();
        assert_eq!(segment.id, 3);
        assert!((segment.temperature.unwrap() - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_segment_info_clone() {
        let segment = SegmentInfo {
            id: 0,
            start: 0.0,
            end: 5.0,
            text: "clone test".to_string(),
            temperature: Some(0.6),
            avg_logprob: Some(-0.5),
            compression_ratio: None,
            no_speech_prob: None,
        };
        let cloned = segment.clone();
        assert_eq!(segment.id, cloned.id);
        assert_eq!(segment.text, cloned.text);
    }

    // ==================== AudioTranscriptionResponse Tests ====================

    #[test]
    fn test_audio_transcription_response_minimal() {
        let response = AudioTranscriptionResponse {
            text: "Hello world".to_string(),
            language: None,
            duration: None,
            words: None,
            segments: None,
        };
        assert_eq!(response.text, "Hello world");
        assert!(response.language.is_none());
    }

    #[test]
    fn test_audio_transcription_response_full() {
        let word = WordInfo {
            word: "hello".to_string(),
            start: 0.0,
            end: 0.5,
        };
        let segment = SegmentInfo {
            id: 0,
            start: 0.0,
            end: 1.0,
            text: "hello world".to_string(),
            temperature: Some(0.5),
            avg_logprob: Some(-0.2),
            compression_ratio: Some(1.2),
            no_speech_prob: Some(0.01),
        };

        let response = AudioTranscriptionResponse {
            text: "hello world".to_string(),
            language: Some("en".to_string()),
            duration: Some(1.0),
            words: Some(vec![word]),
            segments: Some(vec![segment]),
        };
        assert_eq!(response.language, Some("en".to_string()));
        assert!((response.duration.unwrap() - 1.0).abs() < f64::EPSILON);
        assert_eq!(response.words.as_ref().unwrap().len(), 1);
        assert_eq!(response.segments.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_audio_transcription_response_serialization() {
        let response = AudioTranscriptionResponse {
            text: "Test transcription".to_string(),
            language: Some("en".to_string()),
            duration: Some(5.5),
            words: None,
            segments: None,
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["text"], "Test transcription");
        assert_eq!(json["language"], "en");
        assert_eq!(json["duration"], 5.5);
    }

    #[test]
    fn test_audio_transcription_response_skip_none() {
        let response = AudioTranscriptionResponse {
            text: "Only text".to_string(),
            language: None,
            duration: None,
            words: None,
            segments: None,
        };
        let json = serde_json::to_value(&response).unwrap();
        let obj = json.as_object().unwrap();
        assert!(!obj.contains_key("language"));
        assert!(!obj.contains_key("duration"));
        assert!(!obj.contains_key("words"));
        assert!(!obj.contains_key("segments"));
    }

    #[test]
    fn test_audio_transcription_response_deserialization() {
        let json = r#"{
            "text": "Deserialized text",
            "language": "fr",
            "duration": 10.5
        }"#;
        let response: AudioTranscriptionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.text, "Deserialized text");
        assert_eq!(response.language, Some("fr".to_string()));
    }

    #[test]
    fn test_audio_transcription_response_clone() {
        let response = AudioTranscriptionResponse {
            text: "Clone test".to_string(),
            language: Some("en".to_string()),
            duration: Some(2.0),
            words: None,
            segments: None,
        };
        let cloned = response.clone();
        assert_eq!(response.text, cloned.text);
        assert_eq!(response.language, cloned.language);
    }

    #[test]
    fn test_audio_transcription_with_multiple_words() {
        let words = vec![
            WordInfo { word: "hello".to_string(), start: 0.0, end: 0.5 },
            WordInfo { word: "world".to_string(), start: 0.6, end: 1.0 },
        ];
        let response = AudioTranscriptionResponse {
            text: "hello world".to_string(),
            language: Some("en".to_string()),
            duration: Some(1.0),
            words: Some(words),
            segments: None,
        };
        assert_eq!(response.words.as_ref().unwrap().len(), 2);
        assert_eq!(response.words.as_ref().unwrap()[0].word, "hello");
        assert_eq!(response.words.as_ref().unwrap()[1].word, "world");
    }

    #[test]
    fn test_audio_transcription_with_multiple_segments() {
        let segments = vec![
            SegmentInfo {
                id: 0,
                start: 0.0,
                end: 5.0,
                text: "First segment".to_string(),
                temperature: None,
                avg_logprob: None,
                compression_ratio: None,
                no_speech_prob: None,
            },
            SegmentInfo {
                id: 1,
                start: 5.0,
                end: 10.0,
                text: "Second segment".to_string(),
                temperature: None,
                avg_logprob: None,
                compression_ratio: None,
                no_speech_prob: None,
            },
        ];
        let response = AudioTranscriptionResponse {
            text: "First segment Second segment".to_string(),
            language: None,
            duration: Some(10.0),
            words: None,
            segments: Some(segments),
        };
        assert_eq!(response.segments.as_ref().unwrap().len(), 2);
    }
}
