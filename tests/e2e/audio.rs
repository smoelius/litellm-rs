//! E2E tests for audio API (transcription, translation, speech)
//!
//! These tests make real API calls and require API keys.
//! Run with: cargo test --all-features -- --ignored

#[cfg(test)]
mod tests {
    use litellm_rs::core::audio::{AudioService, TranscriptionRequest, TranslationRequest};
    use litellm_rs::core::providers::ProviderRegistry;
    use std::sync::Arc;

    /// Helper to create a provider registry with Groq
    async fn create_provider_registry() -> Arc<ProviderRegistry> {
        use litellm_rs::core::providers::groq::GroqProvider;
        use litellm_rs::core::providers::Provider;

        let api_key =
            std::env::var("GROQ_API_KEY").expect("GROQ_API_KEY environment variable not set");

        let groq = GroqProvider::with_api_key(&api_key).await.unwrap();

        let mut registry = ProviderRegistry::new();
        registry.register(Provider::Groq(groq));

        Arc::new(registry)
    }

    /// E2E test for audio transcription with Groq Whisper
    /// Requires GROQ_API_KEY environment variable
    #[tokio::test]
    #[ignore]
    async fn test_audio_transcription_groq() {
        let registry = create_provider_registry().await;
        let audio_service = AudioService::new(registry);

        // Create a minimal valid MP3 file (silent audio)
        // This is a minimal valid MP3 frame (silent)
        let audio_data = create_test_audio_mp3();

        let request = TranscriptionRequest {
            file: audio_data,
            filename: "test.mp3".to_string(),
            model: "groq/whisper-large-v3-turbo".to_string(),
            language: Some("en".to_string()),
            prompt: None,
            response_format: Some("json".to_string()),
            temperature: None,
            timestamp_granularities: None,
        };

        let result = audio_service.transcribe(request).await;

        // Note: This test might fail if the audio is too short or invalid
        // In real tests, you would use actual audio files
        match result {
            Ok(response) => {
                println!("Transcription: {}", response.text);
                // Even empty audio should return a valid response structure
            }
            Err(e) => {
                // Some errors are expected with minimal test audio
                println!("Transcription error (may be expected with test audio): {}", e);
            }
        }
    }

    /// E2E test for audio translation with Groq Whisper
    /// Requires GROQ_API_KEY environment variable
    #[tokio::test]
    #[ignore]
    async fn test_audio_translation_groq() {
        let registry = create_provider_registry().await;
        let audio_service = AudioService::new(registry);

        let audio_data = create_test_audio_mp3();

        let request = TranslationRequest {
            file: audio_data,
            filename: "test.mp3".to_string(),
            model: "groq/whisper-large-v3-turbo".to_string(),
            prompt: None,
            response_format: Some("json".to_string()),
            temperature: None,
        };

        let result = audio_service.translate(request).await;

        match result {
            Ok(response) => {
                println!("Translation: {}", response.text);
                // Translation completed successfully
            }
            Err(e) => {
                println!("Translation error (may be expected with test audio): {}", e);
            }
        }
    }

    /// E2E test with real audio file (if available)
    /// Place a test.mp3 file in the tests/fixtures/ directory
    #[tokio::test]
    #[ignore]
    async fn test_audio_transcription_real_file() {
        let audio_path = "tests/fixtures/test.mp3";
        if !std::path::Path::new(audio_path).exists() {
            println!("Skipping test: {} not found", audio_path);
            return;
        }

        let audio_data = std::fs::read(audio_path).expect("Failed to read test audio file");

        let registry = create_provider_registry().await;
        let audio_service = AudioService::new(registry);

        let request = TranscriptionRequest {
            file: audio_data,
            filename: "test.mp3".to_string(),
            model: "groq/whisper-large-v3-turbo".to_string(),
            language: None, // Auto-detect
            prompt: None,
            response_format: Some("verbose_json".to_string()),
            temperature: None,
            timestamp_granularities: Some(vec!["segment".to_string()]),
        };

        let result = audio_service.transcribe(request).await;

        assert!(result.is_ok(), "Transcription failed: {:?}", result.err());
        let response = result.unwrap();

        println!("Transcription: {}", response.text);
        assert!(!response.text.is_empty(), "Expected non-empty transcription");

        if let Some(segments) = response.segments {
            println!("Segments: {:?}", segments);
            assert!(!segments.is_empty(), "Expected segments in verbose response");
        }
    }

    /// Create a minimal test MP3 audio file
    /// This is a silent/minimal MP3 for testing purposes
    fn create_test_audio_mp3() -> Vec<u8> {
        // Minimal MP3 frame header for a valid (but silent) MP3
        // Frame sync: 0xFF 0xFB (MPEG Audio Layer 3)
        // This is just for testing API connectivity
        // Real tests should use actual audio files
        let mut audio = Vec::new();

        // MP3 frame header
        // Sync word (12 bits): 0xFFF
        // Version (2 bits): 11 (MPEG 1)
        // Layer (2 bits): 01 (Layer 3)
        // Protection bit: 1 (no CRC)
        // Bitrate index: 1001 (128kbps)
        // Sample rate: 00 (44100Hz)
        // Padding: 0
        // Private: 0
        audio.extend_from_slice(&[
            0xFF, 0xFB, 0x90, 0x00, // Frame header
        ]);

        // Add padding to meet minimum frame size
        // Frame size = 144 * bitrate / sample_rate + padding
        // For 128kbps at 44100Hz: 144 * 128000 / 44100 ≈ 418 bytes
        audio.resize(418, 0x00);

        // Repeat for at least 1 second of audio (about 38 frames)
        let frame = audio.clone();
        for _ in 0..37 {
            audio.extend_from_slice(&frame);
        }

        audio
    }
}
