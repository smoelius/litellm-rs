//! OpenAI Capabilities and Feature Detection
//!
//! Comprehensive capability detection and feature management

use std::collections::HashMap;

use super::{
    config::{OpenAIConfig, OpenAIFeature},
    models::{OpenAIModelFamily, OpenAIModelFeature},
};

/// OpenAI capability detection and management
pub struct OpenAICapabilities {
    config: OpenAIConfig,
}

impl OpenAICapabilities {
    /// Create new capabilities manager
    pub fn new(config: OpenAIConfig) -> Self {
        Self { config }
    }

    /// Check if a capability is available
    pub fn is_available(&self, capability: OpenAICapability) -> bool {
        match capability {
            OpenAICapability::ChatCompletion => true, // Always available
            OpenAICapability::Streaming => true,      // Always available
            OpenAICapability::FunctionCalling => true, // Always available
            OpenAICapability::VisionSupport => true,  // Available for supported models
            OpenAICapability::ImageGeneration => self
                .config
                .is_feature_enabled(OpenAIFeature::ImageGeneration),
            OpenAICapability::AudioTranscription => self
                .config
                .is_feature_enabled(OpenAIFeature::AudioTranscription),
            OpenAICapability::AudioOutput => {
                self.config.is_feature_enabled(OpenAIFeature::AudioModels)
            }
            OpenAICapability::Embeddings => true, // Always available
            OpenAICapability::FineTuning => {
                self.config.is_feature_enabled(OpenAIFeature::FineTuning)
            }
            OpenAICapability::VectorStores => {
                self.config.is_feature_enabled(OpenAIFeature::VectorStores)
            }
            OpenAICapability::RealtimeAudio => {
                self.config.is_feature_enabled(OpenAIFeature::RealtimeAudio)
            }
            OpenAICapability::OSeriesOptimizations => self
                .config
                .is_feature_enabled(OpenAIFeature::OSeriesOptimizations),
            OpenAICapability::GPT5Features => {
                self.config.is_feature_enabled(OpenAIFeature::GPT5Features)
            }
        }
    }

    /// Get all available capabilities
    pub fn get_available_capabilities(&self) -> Vec<OpenAICapability> {
        OpenAICapability::all()
            .into_iter()
            .filter(|cap| self.is_available(*cap))
            .collect()
    }

    /// Get capability matrix for all models
    pub fn get_model_capability_matrix(&self) -> HashMap<String, Vec<OpenAIModelFeature>> {
        use super::models::get_openai_registry;

        let registry = get_openai_registry();
        let mut matrix = HashMap::new();

        for model_info in registry.get_all_models() {
            if let Some(spec) = registry.get_model_spec(&model_info.id) {
                matrix.insert(model_info.id.clone(), spec.features.clone());
            }
        }

        matrix
    }

    /// Get recommended models for specific capabilities
    pub fn get_models_for_capability(&self, capability: OpenAICapability) -> Vec<String> {
        use super::models::{OpenAIModelFeature, get_openai_registry};

        let registry = get_openai_registry();
        let feature = match capability {
            OpenAICapability::ChatCompletion => OpenAIModelFeature::ChatCompletion,
            OpenAICapability::Streaming => OpenAIModelFeature::StreamingSupport,
            OpenAICapability::FunctionCalling => OpenAIModelFeature::FunctionCalling,
            OpenAICapability::VisionSupport => OpenAIModelFeature::VisionSupport,
            OpenAICapability::ImageGeneration => OpenAIModelFeature::ImageGeneration,
            OpenAICapability::AudioTranscription => OpenAIModelFeature::AudioTranscription,
            OpenAICapability::AudioOutput => OpenAIModelFeature::AudioOutput,
            OpenAICapability::Embeddings => OpenAIModelFeature::Embeddings,
            OpenAICapability::FineTuning => OpenAIModelFeature::FineTuning,
            OpenAICapability::RealtimeAudio => OpenAIModelFeature::RealtimeAudio,
            OpenAICapability::OSeriesOptimizations => OpenAIModelFeature::ReasoningMode,
            _ => return Vec::new(),
        };

        registry.get_models_with_feature(&feature)
    }

    /// Check model family capabilities
    pub fn get_family_capabilities(&self, family: OpenAIModelFamily) -> Vec<OpenAICapability> {
        match family {
            OpenAIModelFamily::GPT4 | OpenAIModelFamily::GPT4Turbo | OpenAIModelFamily::GPT4O => {
                vec![
                    OpenAICapability::ChatCompletion,
                    OpenAICapability::Streaming,
                    OpenAICapability::FunctionCalling,
                    OpenAICapability::VisionSupport,
                ]
            }
            OpenAIModelFamily::GPT35 => vec![
                OpenAICapability::ChatCompletion,
                OpenAICapability::Streaming,
                OpenAICapability::FunctionCalling,
            ],
            OpenAIModelFamily::O1 => vec![
                OpenAICapability::ChatCompletion,
                OpenAICapability::Streaming,
                OpenAICapability::OSeriesOptimizations,
            ],
            OpenAIModelFamily::DALLE2 | OpenAIModelFamily::DALLE3 => {
                vec![OpenAICapability::ImageGeneration]
            }
            OpenAIModelFamily::Whisper => vec![OpenAICapability::AudioTranscription],
            OpenAIModelFamily::TTS => vec![OpenAICapability::AudioOutput],
            OpenAIModelFamily::Embedding => vec![OpenAICapability::Embeddings],
            _ => Vec::new(),
        }
    }

    /// Get capability requirements
    pub fn get_capability_requirements(
        &self,
        capability: OpenAICapability,
    ) -> CapabilityRequirements {
        match capability {
            OpenAICapability::ChatCompletion => CapabilityRequirements {
                min_api_version: None,
                required_features: vec![],
                enterprise_only: false,
                beta_feature: false,
            },
            OpenAICapability::ImageGeneration => CapabilityRequirements {
                min_api_version: None,
                required_features: vec![OpenAIFeature::ImageGeneration],
                enterprise_only: false,
                beta_feature: false,
            },
            OpenAICapability::FineTuning => CapabilityRequirements {
                min_api_version: None,
                required_features: vec![OpenAIFeature::FineTuning],
                enterprise_only: true,
                beta_feature: false,
            },
            OpenAICapability::RealtimeAudio => CapabilityRequirements {
                min_api_version: Some("2024-10-01".to_string()),
                required_features: vec![OpenAIFeature::RealtimeAudio],
                enterprise_only: false,
                beta_feature: true,
            },
            OpenAICapability::GPT5Features => CapabilityRequirements {
                min_api_version: Some("2024-12-01".to_string()),
                required_features: vec![OpenAIFeature::GPT5Features],
                enterprise_only: true,
                beta_feature: true,
            },
            _ => CapabilityRequirements::default(),
        }
    }
}

/// OpenAI capabilities enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAICapability {
    /// Basic chat completion
    ChatCompletion,
    /// Streaming responses
    Streaming,
    /// Function/tool calling
    FunctionCalling,
    /// Vision/multimodal support
    VisionSupport,
    /// Image generation (DALL-E)
    ImageGeneration,
    /// Audio transcription (Whisper)
    AudioTranscription,
    /// Audio output (TTS)
    AudioOutput,
    /// Text embeddings
    Embeddings,
    /// Model fine-tuning
    FineTuning,
    /// Vector store integration
    VectorStores,
    /// Real-time audio processing
    RealtimeAudio,
    /// O-series model optimizations
    OSeriesOptimizations,
    /// GPT-5 specific features
    GPT5Features,
}

impl OpenAICapability {
    /// Get all capabilities
    pub fn all() -> Vec<Self> {
        vec![
            Self::ChatCompletion,
            Self::Streaming,
            Self::FunctionCalling,
            Self::VisionSupport,
            Self::ImageGeneration,
            Self::AudioTranscription,
            Self::AudioOutput,
            Self::Embeddings,
            Self::FineTuning,
            Self::VectorStores,
            Self::RealtimeAudio,
            Self::OSeriesOptimizations,
            Self::GPT5Features,
        ]
    }

    /// Get capability name
    pub fn name(&self) -> &'static str {
        match self {
            Self::ChatCompletion => "Chat Completion",
            Self::Streaming => "Streaming Responses",
            Self::FunctionCalling => "Function Calling",
            Self::VisionSupport => "Vision Support",
            Self::ImageGeneration => "Image Generation",
            Self::AudioTranscription => "Audio Transcription",
            Self::AudioOutput => "Audio Output",
            Self::Embeddings => "Text Embeddings",
            Self::FineTuning => "Model Fine-tuning",
            Self::VectorStores => "Vector Stores",
            Self::RealtimeAudio => "Real-time Audio",
            Self::OSeriesOptimizations => "O-series Optimizations",
            Self::GPT5Features => "GPT-5 Features",
        }
    }

    /// Get capability description
    pub fn description(&self) -> &'static str {
        match self {
            Self::ChatCompletion => "Basic text generation and conversation",
            Self::Streaming => "Real-time streaming of response tokens",
            Self::FunctionCalling => "Structured function and tool calling",
            Self::VisionSupport => "Image and multimodal input processing",
            Self::ImageGeneration => "AI-powered image creation with DALL-E",
            Self::AudioTranscription => "Speech-to-text conversion with Whisper",
            Self::AudioOutput => "Text-to-speech synthesis",
            Self::Embeddings => "Vector representations for semantic search",
            Self::FineTuning => "Custom model training and optimization",
            Self::VectorStores => "Integrated vector database operations",
            Self::RealtimeAudio => "Low-latency audio processing",
            Self::OSeriesOptimizations => "Advanced reasoning optimizations for O1 models",
            Self::GPT5Features => "Next-generation GPT-5 capabilities",
        }
    }
}

/// Capability requirements structure
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct CapabilityRequirements {
    /// Minimum API version required
    pub min_api_version: Option<String>,
    /// Required configuration features
    pub required_features: Vec<OpenAIFeature>,
    /// Whether this is an enterprise-only feature
    pub enterprise_only: bool,
    /// Whether this is a beta feature
    pub beta_feature: bool,
}


/// Capability validation result
#[derive(Debug)]
pub struct CapabilityValidation {
    pub available: bool,
    pub reasons: Vec<String>,
}

impl OpenAICapabilities {
    /// Validate if a capability can be used
    pub fn validate_capability(&self, capability: OpenAICapability) -> CapabilityValidation {
        let mut reasons = Vec::new();
        let requirements = self.get_capability_requirements(capability);

        // Check if feature is enabled in config
        if !self.is_available(capability) {
            reasons.push(format!(
                "Feature {} is disabled in configuration",
                capability.name()
            ));
        }

        // Check required features
        for required_feature in &requirements.required_features {
            if !self.config.is_feature_enabled(required_feature.clone()) {
                reasons.push(format!(
                    "Required feature {:?} is not enabled",
                    required_feature
                ));
            }
        }

        // Check API key for enterprise features
        if requirements.enterprise_only && self.config.base.api_key.is_none() {
            reasons.push("Enterprise feature requires valid API key".to_string());
        }

        // Warn about beta features
        if requirements.beta_feature {
            reasons.push(format!(
                "{} is a beta feature and may change",
                capability.name()
            ));
        }

        CapabilityValidation {
            available: reasons.is_empty(),
            reasons,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_detection() {
        let config = OpenAIConfig::default();
        let capabilities = OpenAICapabilities::new(config);

        // Basic capabilities should always be available
        assert!(capabilities.is_available(OpenAICapability::ChatCompletion));
        assert!(capabilities.is_available(OpenAICapability::Streaming));
        assert!(capabilities.is_available(OpenAICapability::FunctionCalling));

        // Feature-gated capabilities depend on config
        assert!(capabilities.is_available(OpenAICapability::ImageGeneration)); // Default enabled
        assert!(!capabilities.is_available(OpenAICapability::FineTuning)); // Default disabled
    }

    #[test]
    fn test_capability_validation() {
        let config = OpenAIConfig::default();
        let capabilities = OpenAICapabilities::new(config);

        let chat_validation = capabilities.validate_capability(OpenAICapability::ChatCompletion);
        assert!(chat_validation.available);
        assert!(chat_validation.reasons.is_empty());

        let fine_tuning_validation = capabilities.validate_capability(OpenAICapability::FineTuning);
        assert!(!fine_tuning_validation.available);
        assert!(!fine_tuning_validation.reasons.is_empty());
    }

    #[test]
    fn test_model_recommendations() {
        let config = OpenAIConfig::default();
        let capabilities = OpenAICapabilities::new(config);

        let chat_models = capabilities.get_models_for_capability(OpenAICapability::ChatCompletion);
        assert!(!chat_models.is_empty());

        let vision_models = capabilities.get_models_for_capability(OpenAICapability::VisionSupport);
        assert!(!vision_models.is_empty());
        assert!(vision_models.contains(&"gpt-4o".to_string()));
    }

    #[test]
    fn test_capability_names() {
        assert_eq!(OpenAICapability::ChatCompletion.name(), "Chat Completion");
        assert_eq!(OpenAICapability::ImageGeneration.name(), "Image Generation");

        assert!(!OpenAICapability::ChatCompletion.description().is_empty());
        assert!(!OpenAICapability::VisionSupport.description().is_empty());
    }
}
